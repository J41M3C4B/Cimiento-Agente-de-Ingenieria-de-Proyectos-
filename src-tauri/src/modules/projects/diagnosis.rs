//! Use cases of the stages after the conversation: summary, needs, stage moves. The conversation itself is in
//! `conversation`. Code keeps the flow and the numbers. No AI: what it wrote (the conversation, the summary, the
//! proposed needs) comes back as capabilities of the platform.

use crate::modules::projects::conversation::{conversation_view, ConversationView};
use crate::modules::projects::domain::priority::{self, Scores, Weights};
use crate::modules::projects::domain::stage::{self, Missing, Stage, StageError};
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::core::api::guard_texts;
use crate::modules::projects::ProjectsError;
use crate::modules::projects::storage::projects::{self as store, NeedRow, ProjectRow};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use crate::storage::SharedDb;

pub(crate) fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectsError> {
    db.lock().map_err(|_| ProjectsError::Internal("database lock poisoned".into()))
}


#[cfg(test)]
pub fn profile_summary_for_tests(conn: &Connection) -> Result<String, ProjectsError> {
    Ok(crate::core::api::institution_sheet(conn)?)
}

// ------------------------------------------------------------------ projects and stages

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CreateProjectOutcome {
    Created { project: ProjectRow },
    Quarantine { report: QuarantineReport },
}

pub fn create_project(db: &SharedDb, initial_request: &str, decision: Option<Decision>) -> Result<CreateProjectOutcome, ProjectsError> {
    if initial_request.trim().is_empty() {
        return Err(ProjectsError::EmptyText);
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
    let project = store::get_project(&conn, &project.id)?.ok_or(ProjectsError::NotFound)?;
    Ok(CreateProjectOutcome::Created { project })
}

pub fn advance(db: &SharedDb, project_id: &str) -> Result<ProjectRow, ProjectsError> {
    let mut conn = lock(db)?;
    let project = store::get_project(&conn, project_id)?.ok_or(ProjectsError::NotFound)?;
    let mut facts = store::facts(&conn, project_id)?;
    // these two need the plan of sections and the review of the whole project, which the database alone cannot say
    match project.stage {
        Stage::Drafting => facts.sections_confirmed = crate::modules::projects::drafting::sections_confirmed(&conn, project_id)?,
        Stage::Review => facts.checklist_clean = crate::modules::projects::review::review(&conn, project_id)?.report.clean(),
        _ => {}
    }
    let next = stage::advance(project.stage, &facts)?;
    store::set_stage(&mut conn, project_id, next, false)?;
    Ok(store::get_project(&conn, project_id)?.ok_or(ProjectsError::NotFound)?)
}

pub fn go_back(db: &SharedDb, project_id: &str, target: Stage) -> Result<ProjectRow, ProjectsError> {
    let mut conn = lock(db)?;
    let project = store::get_project(&conn, project_id)?.ok_or(ProjectsError::NotFound)?;
    let out = stage::go_back(project.stage, target)?;
    store::set_stage(&mut conn, project_id, out.new_stage, !out.needs_review.is_empty())?;
    Ok(store::get_project(&conn, project_id)?.ok_or(ProjectsError::NotFound)?)
}

// ------------------------------------------------------------------ summary

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

pub fn edit_summary(db: &SharedDb, project_id: &str, edit: SummaryEdit, decision: Option<Decision>) -> Result<EditOutcome, ProjectsError> {
    let conn = lock(db)?;
    let current = store::get_summary(&conn, project_id)?.ok_or(ProjectsError::NotFound)?;
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

pub fn confirm_summary(db: &SharedDb, project_id: &str) -> Result<ConversationView, ProjectsError> {
    let conn = lock(db)?;
    if !store::confirm_summary(&conn, project_id)? {
        return Err(ProjectsError::NotFound);
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

pub fn needs_view(conn: &Connection, project_id: &str) -> Result<NeedsView, ProjectsError> {
    let project = store::get_project(conn, project_id)?.ok_or(ProjectsError::NotFound)?;
    let needs = store::list_needs(conn, project_id)?;
    let weights = Weights::default();
    let rated: Vec<(usize, Scores)> = needs.iter().enumerate().filter_map(|(i, n)| n.scores.map(|s| (i, s))).collect();
    let ranked = priority::rank(&rated.iter().map(|(_, s)| *s).collect::<Vec<_>>(), &weights)
        .map_err(|e| ProjectsError::Priority(format!("{e:?}")))?;
    let ranking = ranked.iter().map(|r| needs[rated[r.index].0].id.clone()).collect();
    let affected = store::get_summary(conn, project_id)?
        .and_then(|s| s.summary["affected"]["count"].as_u64())
        .map(|c| c as u32);
    let population = crate::core::api::people_served(conn)?;
    let beneficiaries_suggestion = match (affected, population) {
        (Some(a), Some(p)) => priority::suggest_beneficiaries_score(a, p),
        _ => None,
    };
    Ok(NeedsView { project, needs, ranking, beneficiaries_suggestion, weights })
}

pub fn get_needs(db: &SharedDb, project_id: &str) -> Result<NeedsView, ProjectsError> {
    let conn = lock(db)?;
    needs_view(&conn, project_id)
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AddNeedOutcome {
    Saved { view: NeedsView },
    Quarantine { report: QuarantineReport },
}

pub fn add_need(db: &SharedDb, project_id: &str, title: &str, description: &str, decision: Option<Decision>) -> Result<AddNeedOutcome, ProjectsError> {
    if title.trim().is_empty() {
        return Err(ProjectsError::EmptyText);
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

pub fn rate_need(db: &SharedDb, project_id: &str, need_id: &str, scores: Scores) -> Result<NeedsView, ProjectsError> {
    let conn = lock(db)?;
    let total = priority::total_score(&scores, &Weights::default()).map_err(|e| ProjectsError::Priority(format!("{e:?}")))?;
    if !store::set_need_scores(&conn, need_id, &scores, total)? {
        return Err(ProjectsError::NotFound);
    }
    needs_view(&conn, project_id)
}

pub fn select_need(db: &SharedDb, project_id: &str, need_id: &str) -> Result<NeedsView, ProjectsError> {
    let mut conn = lock(db)?;
    if !store::select_need(&mut conn, project_id, need_id)? {
        return Err(ProjectsError::NotFound);
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
#[path = "diagnosis_tests.rs"]
mod tests;
