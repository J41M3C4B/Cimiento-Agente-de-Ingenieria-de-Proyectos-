//! Use cases of the stages after the conversation: summary, needs, stage moves. The conversation itself is in
//! `conversation_service`. Code keeps the flow and the numbers; the AI only words text.

use crate::ai::pipeline::{self, AiCall, SqliteLedger};
use crate::ai::{prompts, AiError, AiProvider, AiTask};
use crate::audit::{self, AuditKind};
use crate::conversation_service::{call_context, conversation_view, figure_sources, person_words, transcript, ConversationView};
use crate::domain::conversation::Phase;
use crate::domain::figures;
use crate::domain::priority::{self, Scores, Weights};
use crate::domain::stage::{self, Missing, Stage, StageError};
use crate::scanner::guard::{guard_fields, Decision, GuardOutcome, QuarantineReport};
use crate::service::ServiceError;
use crate::storage::profile as profile_store;
use crate::storage::projects::{self as store, NeedRow, ProjectRow};
use crate::scanner::RegexScanner;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub type SharedDb = Arc<Mutex<Connection>>;

/// How the AI part went. The UI uses it to say "seguimos sin ayuda automática".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    Used,
    NotConfigured,
    Offline,
    Busy,
    BudgetExhausted,
    /// The provider's daily allowance is used up (a free plan has few calls per day).
    QuotaReached,
    KeyRejected,
    Unavailable,
    /// The AI was not needed (follow-up limit reached, manual summary...).
    Skipped,
}

impl From<&AiError> for AiStatus {
    fn from(e: &AiError) -> Self {
        match e {
            AiError::NoApiKey => AiStatus::NotConfigured,
            AiError::Offline => AiStatus::Offline,
            AiError::RateLimited | AiError::Timeout => AiStatus::Busy,
            AiError::BudgetExhausted => AiStatus::BudgetExhausted,
            AiError::QuotaReached => AiStatus::QuotaReached,
            AiError::Auth => AiStatus::KeyRejected,
            _ => AiStatus::Unavailable,
        }
    }
}

pub(crate) fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("database lock poisoned".into()))
}

fn scanner_for(conn: &Connection) -> Result<RegexScanner, ServiceError> {
    Ok(RegexScanner::new(profile_store::scanner_config(conn)?))
}

fn counts_json(counts: &BTreeMap<&'static str, usize>, decision: &str) -> Value {
    json!({ "findings": counts, "decision": decision })
}

/// Scans free texts. `Ok(Err(report))` means "quarantine: show it, save nothing".
/// `Ok(Ok(texts))` are the texts to save (covered if the person chose so).
pub(crate) fn guard_texts(
    conn: &Connection,
    entity: &str,
    fields: &[(String, String)],
    decision: Option<Decision>,
) -> Result<Result<Vec<String>, QuarantineReport>, ServiceError> {
    let scanner = scanner_for(conn)?;
    match guard_fields(&scanner, fields, decision)? {
        GuardOutcome::Clean => Ok(Ok(fields.iter().map(|(_, t)| t.clone()).collect())),
        GuardOutcome::Quarantine(r) => Ok(Err(r)),
        GuardOutcome::Redacted { texts, counts } => {
            audit::record(conn, AuditKind::ScannerQuarantine, Some(entity), None, counts_json(&counts, "redacted"))?;
            Ok(Ok(texts))
        }
        GuardOutcome::Overridden { counts } => {
            audit::record(conn, AuditKind::ScannerOverride, Some(entity), None, counts_json(&counts, "not_personal"))?;
            Ok(Ok(fields.iter().map(|(_, t)| t.clone()).collect()))
        }
    }
}

// ------------------------------------------------------------------ context for the AI

#[cfg(test)]
pub fn profile_summary_for_tests(conn: &Connection) -> Result<String, ServiceError> {
    profile_context(conn)
}

/// What the AI reads about the institution: the sheet built in `institution_context` (aggregates only).
pub use crate::institution_context::profile_context;

// ------------------------------------------------------------------ views

fn collect_strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| collect_strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| collect_strings(x, out)),
        Value::Number(n) => out.push(n.to_string()),
        _ => {}
    }
}

pub(crate) fn check_summary_figures(summary: &Value, sources: &[String]) -> Vec<String> {
    let mut outs = Vec::new();
    collect_strings(summary, &mut outs);
    let o: Vec<&str> = outs.iter().map(String::as_str).collect();
    let s: Vec<&str> = sources.iter().map(String::as_str).collect();
    figures::unsupported_figures(&o, &s)
}

// ------------------------------------------------------------------ projects and stages

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CreateProjectOutcome {
    Created { project: ProjectRow },
    Quarantine { report: QuarantineReport },
}

pub fn create_project(db: &SharedDb, initial_request: &str, decision: Option<Decision>) -> Result<CreateProjectOutcome, ServiceError> {
    if initial_request.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    let mut conn = lock(db)?;
    let facts = store::facts(&conn, "")?;
    stage::advance(Stage::Profile, &facts)?; // profile confirmed and recent, else an error before creating anything
    let fields = vec![("initial_request".to_string(), initial_request.to_string())];
    let text = match guard_texts(&conn, "project", &fields, decision)? {
        Ok(mut t) => t.remove(0),
        Err(report) => return Ok(CreateProjectOutcome::Quarantine { report }),
    };
    let title: String = text.trim().chars().take(80).collect();
    let project = store::create_project(&mut conn, &title, Some(&text))?;
    store::set_stage(&mut conn, &project.id, Stage::Diagnosis, false)?;
    let project = store::get_project(&conn, &project.id)?.ok_or(ServiceError::NotFound)?;
    Ok(CreateProjectOutcome::Created { project })
}

pub fn advance(db: &SharedDb, project_id: &str) -> Result<ProjectRow, ServiceError> {
    let mut conn = lock(db)?;
    let project = store::get_project(&conn, project_id)?.ok_or(ServiceError::NotFound)?;
    let mut facts = store::facts(&conn, project_id)?;
    // these two need the plan of sections and the review of the whole project, which the database alone cannot say
    match project.stage {
        Stage::Drafting => facts.sections_confirmed = crate::drafting_service::sections_confirmed(&conn, project_id)?,
        Stage::Review => facts.checklist_clean = crate::review_service::review(&conn, project_id)?.report.clean(),
        _ => {}
    }
    let next = stage::advance(project.stage, &facts)?;
    store::set_stage(&mut conn, project_id, next, false)?;
    Ok(store::get_project(&conn, project_id)?.ok_or(ServiceError::NotFound)?)
}

pub fn go_back(db: &SharedDb, project_id: &str, target: Stage) -> Result<ProjectRow, ServiceError> {
    let mut conn = lock(db)?;
    let project = store::get_project(&conn, project_id)?.ok_or(ServiceError::NotFound)?;
    let out = stage::go_back(project.stage, target)?;
    store::set_stage(&mut conn, project_id, out.new_stage, !out.needs_review.is_empty())?;
    Ok(store::get_project(&conn, project_id)?.ok_or(ServiceError::NotFound)?)
}

// ------------------------------------------------------------------ the AI

pub(crate) async fn ask_ai(
    db: &SharedDb,
    provider: Option<&dyn AiProvider>,
    task: AiTask,
    project_id: &str,
    context: Vec<String>,
    user: String,
) -> Result<Value, AiError> {
    let provider = provider.ok_or(AiError::NoApiKey)?;
    let scanner = {
        let conn = db.lock().map_err(|_| AiError::Internal("lock".into()))?;
        let cfg = profile_store::scanner_config(&conn).map_err(|e| AiError::Internal(e.to_string()))?;
        RegexScanner::new(cfg)
    };
    let ledger = SqliteLedger(db.clone());
    pipeline::run(provider, &scanner, &ledger, AiCall { task, context, user, project_id: Some(project_id.to_string()) }).await
}

// ------------------------------------------------------------------ summary

#[derive(Debug, Serialize)]
pub struct SummaryOutcome {
    pub view: ConversationView,
    pub ai: AiStatus,
}

/// Writes the summary of the diagnosis from the conversation, once the person confirmed the root cause. The AI
/// must not invent numbers: if it did, it is asked once more. If it fails nothing is saved and the person can
/// try again; there is no summary made without the AI.
pub async fn generate_summary(
    db: &SharedDb,
    provider: Option<&dyn AiProvider>,
    project_id: &str,
) -> Result<SummaryOutcome, ServiceError> {
    let (context, sources) = {
        let conn = lock(db)?;
        let view = conversation_view(&conn, project_id)?;
        if view.phase != Phase::Closed {
            return Err(ServiceError::WrongStage);
        }
        // what the person confirmed or corrected is theirs: a late or repeated request never writes over it
        if view.summary.as_ref().is_some_and(|s| s.confirmed_at.is_some() || s.origin != "ai_assumption") {
            return Ok(SummaryOutcome { view, ai: AiStatus::Skipped });
        }
        let rows = store::turns(&conn, project_id)?;
        let root = view.root.as_ref().map(|r| r.text.clone()).unwrap_or_default();
        let sources = figure_sources(&conn, &view.project, &rows)?;
        let context = vec![
            format!("Perfil de la institución:\n{}", profile_context(&conn)?),
            format!("Convocatoria que la persona confirmó:\n{}", call_context(&conn, &view.project)?),
            format!("Conversación con la persona:\n{}", transcript(&rows)),
            format!("Causa de fondo que la persona confirmó:\n{root}"),
        ];
        (context, sources)
    };
    let user = "Escriba el resumen del diagnóstico.".to_string();

    let mut result = ask_ai(db, provider, AiTask::DiagnosisSummary, project_id, context.clone(), user.clone()).await;
    // The AI must not invent numbers: ask once more if it did.
    if let Ok(v) = &result {
        let bad = check_summary_figures(v, &sources);
        if !bad.is_empty() {
            let again = format!(
                "{user}\n\nEstas cifras no las dijo la persona: {}. No las use; use solo las cifras que ella dio.",
                bad.join(", ")
            );
            if let Ok(v2) = ask_ai(db, provider, AiTask::DiagnosisSummary, project_id, context, again).await {
                result = Ok(v2);
            }
        }
    }

    let conn = lock(db)?;
    let ai = match result {
        Ok(v) => {
            store::save_summary(&conn, project_id, &v, "ai_assumption", Some(&json!({ "prompt": prompts::SUMMARY_VERSION })))?;
            AiStatus::Used
        }
        Err(e) => AiStatus::from(&e),
    };
    Ok(SummaryOutcome { view: conversation_view(&conn, project_id)?, ai })
}

/// Fields the person can correct in the summary. Alternatives stay as they are.
#[derive(Debug, Clone, Deserialize)]
pub struct SummaryEdit {
    pub problem_statement: String,
    pub reframed_need: String,
    pub affected_description: String,
    pub current_consequences: Vec<String>,
    pub root_causes: Vec<String>,
    pub suggested_indicators: Vec<String>,
    pub open_questions: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum EditOutcome {
    Saved { view: ConversationView },
    Quarantine { report: QuarantineReport },
}

pub fn edit_summary(db: &SharedDb, project_id: &str, edit: SummaryEdit, decision: Option<Decision>) -> Result<EditOutcome, ServiceError> {
    let conn = lock(db)?;
    let current = store::get_summary(&conn, project_id)?.ok_or(ServiceError::NotFound)?;
    let mut fields: Vec<(String, String)> = vec![
        ("problem_statement".into(), edit.problem_statement.clone()),
        ("reframed_need".into(), edit.reframed_need.clone()),
        ("affected_description".into(), edit.affected_description.clone()),
    ];
    let lists = [
        ("current_consequences", &edit.current_consequences),
        ("root_causes", &edit.root_causes),
        ("suggested_indicators", &edit.suggested_indicators),
        ("open_questions", &edit.open_questions),
    ];
    for (name, items) in lists {
        for (i, t) in items.iter().enumerate() {
            fields.push((format!("{name}[{i}]"), t.clone()));
        }
    }
    let texts = match guard_texts(&conn, "diagnosis_summary", &fields, decision)? {
        Ok(t) => t,
        Err(report) => return Ok(EditOutcome::Quarantine { report }),
    };
    let mut it = texts.into_iter();
    let mut s = current.summary.clone();
    s["problem_statement"] = json!(it.next().unwrap());
    s["reframed_need"] = json!(it.next().unwrap());
    s["affected"]["description"] = json!(it.next().unwrap());
    for (name, items) in [
        ("current_consequences", edit.current_consequences.len()),
        ("root_causes", edit.root_causes.len()),
        ("suggested_indicators", edit.suggested_indicators.len()),
        ("open_questions", edit.open_questions.len()),
    ] {
        s[name] = json!((0..items).map(|_| it.next().unwrap()).filter(|t| !t.trim().is_empty()).collect::<Vec<_>>());
    }
    store::save_summary(&conn, project_id, &s, "user", None)?;
    Ok(EditOutcome::Saved { view: conversation_view(&conn, project_id)? })
}

pub fn confirm_summary(db: &SharedDb, project_id: &str) -> Result<ConversationView, ServiceError> {
    let conn = lock(db)?;
    if !store::confirm_summary(&conn, project_id)? {
        return Err(ServiceError::NotFound);
    }
    conversation_view(&conn, project_id)
}

// ------------------------------------------------------------------ needs

#[derive(Debug, Clone, Serialize)]
pub struct NeedsView {
    pub project: ProjectRow,
    pub needs: Vec<NeedRow>,
    /// Need ids from best to worst, only those already rated (order computed by code).
    pub ranking: Vec<String>,
    /// Suggested rating for "people benefited" from the share of the population.
    pub beneficiaries_suggestion: Option<u8>,
    pub weights: Weights,
}

pub fn needs_view(conn: &Connection, project_id: &str) -> Result<NeedsView, ServiceError> {
    let project = store::get_project(conn, project_id)?.ok_or(ServiceError::NotFound)?;
    let needs = store::list_needs(conn, project_id)?;
    let weights = Weights::default();
    let rated: Vec<(usize, Scores)> = needs.iter().enumerate().filter_map(|(i, n)| n.scores.map(|s| (i, s))).collect();
    let ranked = priority::rank(&rated.iter().map(|(_, s)| *s).collect::<Vec<_>>(), &weights)
        .map_err(|e| ServiceError::Priority(format!("{e:?}")))?;
    let ranking = ranked.iter().map(|r| needs[rated[r.index].0].id.clone()).collect();
    let affected = store::get_summary(conn, project_id)?
        .and_then(|s| s.summary["affected"]["count"].as_u64())
        .map(|c| c as u32);
    let population = profile_store::load_current(conn)?.map(|p| p.input.totals(p.as_of_year).population.max(0) as u32);
    let beneficiaries_suggestion = match (affected, population) {
        (Some(a), Some(p)) => priority::suggest_beneficiaries_score(a, p),
        _ => None,
    };
    Ok(NeedsView { project, needs, ranking, beneficiaries_suggestion, weights })
}

pub fn get_needs(db: &SharedDb, project_id: &str) -> Result<NeedsView, ServiceError> {
    let conn = lock(db)?;
    needs_view(&conn, project_id)
}

pub async fn propose_needs(db: &SharedDb, provider: Option<&dyn AiProvider>, project_id: &str) -> Result<(NeedsView, AiStatus), ServiceError> {
    let context = {
        let conn = lock(db)?;
        let summary = store::get_summary(&conn, project_id)?.ok_or(ServiceError::NotFound)?;
        if summary.confirmed_at.is_none() {
            return Err(ServiceError::WrongStage);
        }
        let project = store::get_project(&conn, project_id)?.ok_or(ServiceError::NotFound)?;
        let root = store::get_root(&conn, project_id)?.map(|r| r.text).unwrap_or_default();
        let said = person_words(&store::turns(&conn, project_id)?);
        // each block says how far to trust it: data captured in Mi institución and what the person said are facts;
        // the summary was written by the AI and only accepted by the person
        vec![
            format!("Perfil de la institución:\n{}", profile_context(&conn)?),
            format!("Convocatoria que la persona confirmó:\n{}", call_context(&conn, &project)?),
            format!("Lo que la persona dijo, con sus palabras:\n{}", if said.is_empty() { "(no hay mensajes)".to_string() } else { said }),
            format!("Causa de fondo que la persona confirmó:\n{root}"),
            format!("Borrador del diagnóstico (lo redactó la IA y la persona lo aceptó), en JSON:\n{}", summary.summary),
        ]
    };
    let result = ask_ai(db, provider, AiTask::PrioritizationProposeNeeds, project_id, context, "Proponga las necesidades.".into()).await;
    let conn = lock(db)?;
    let ai = match result {
        Ok(v) => {
            // replace earlier suggestions that nobody has chosen; the model's order (most important first) is the order they are stored in
            conn.execute(
                "DELETE FROM need WHERE project_id=?1 AND origin='ai_assumption' AND scores_json IS NULL AND selected=0",
                [project_id],
            ).map_err(ServiceError::from)?;
            for n in v["needs"].as_array().into_iter().flatten().take(3) {
                store::add_need(
                    &conn,
                    project_id,
                    n["title"].as_str().unwrap_or_default(),
                    n["description"].as_str(),
                    "ai_assumption",
                    Some(&json!({ "prompt": prompts::PROPOSE_NEEDS_VERSION })),
                )?;
            }
            AiStatus::Used
        }
        Err(e) => AiStatus::from(&e),
    };
    Ok((needs_view(&conn, project_id)?, ai))
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AddNeedOutcome {
    Saved { view: NeedsView },
    Quarantine { report: QuarantineReport },
}

pub fn add_need(db: &SharedDb, project_id: &str, title: &str, description: &str, decision: Option<Decision>) -> Result<AddNeedOutcome, ServiceError> {
    if title.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    let conn = lock(db)?;
    let fields = vec![("title".to_string(), title.to_string()), ("description".to_string(), description.to_string())];
    let texts = match guard_texts(&conn, "need", &fields, decision)? {
        Ok(t) => t,
        Err(report) => return Ok(AddNeedOutcome::Quarantine { report }),
    };
    let desc = (!texts[1].trim().is_empty()).then_some(texts[1].as_str());
    store::add_need(&conn, project_id, &texts[0], desc, "user", None)?;
    Ok(AddNeedOutcome::Saved { view: needs_view(&conn, project_id)? })
}

pub fn rate_need(db: &SharedDb, project_id: &str, need_id: &str, scores: Scores) -> Result<NeedsView, ServiceError> {
    let conn = lock(db)?;
    let total = priority::total_score(&scores, &Weights::default()).map_err(|e| ServiceError::Priority(format!("{e:?}")))?;
    if !store::set_need_scores(&conn, need_id, &scores, total)? {
        return Err(ServiceError::NotFound);
    }
    needs_view(&conn, project_id)
}

pub fn select_need(db: &SharedDb, project_id: &str, need_id: &str) -> Result<NeedsView, ServiceError> {
    let mut conn = lock(db)?;
    if !store::select_need(&mut conn, project_id, need_id)? {
        return Err(ServiceError::NotFound);
    }
    needs_view(&conn, project_id)
}

pub fn missing_code(m: Missing) -> &'static str {
    match m {
        Missing::ProfileNotConfirmed => "profile_not_confirmed",
        Missing::ProfileTooOld => "profile_too_old",
        Missing::DiagnosisIncomplete => "diagnosis_incomplete",
        Missing::SummaryNotConfirmed => "summary_not_confirmed",
        Missing::NoNeedSelected => "no_need_selected",
        Missing::CallNotReady => "call_not_ready",
        Missing::SectionsNotConfirmed => "sections_not_confirmed",
        Missing::ChecklistHasErrors => "checklist_has_errors",
    }
}

pub fn stage_error_code(e: &StageError) -> &'static str {
    match e {
        StageError::NotReady(m) => missing_code(*m),
        StageError::AlreadyFinal => "already_final",
        StageError::NotEarlier => "not_earlier",
    }
}

#[cfg(test)]
#[path = "diagnosis_service_tests.rs"]
mod tests;
