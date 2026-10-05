//! Use cases of the drafting stage (ADR-018): the plan of sections, the text of each one (the AI writes, the person
//! edits and confirms), the budget and the schedule (the person captures, the code adds up and validates).
//!
//! The AI never calculates and never writes a file: every figure it may use comes from what the person said, the
//! profile, the call, the budget or the schedule, and the code checks it.

use crate::ai::{prompts, AiProvider, AiTask};
use crate::conversation_service::{call_context, figure_sources};
use crate::diagnosis_service::{ask_ai, guard_texts, lock, profile_context, AiStatus, SharedDb};
use crate::documents::canonical::requirements::call_requirements;
use crate::domain::budget::{self, format_mxn, Funder, Item, LineTotal, Totals};
use crate::domain::requirements::CallRequirements;
use crate::domain::schedule;
use crate::domain::sections::{plan_sections, SectionKind, SectionSpec, KEY_BUDGET, KEY_SCHEDULE};
use crate::domain::{figures, stage::Stage};
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::service::ServiceError;
use crate::storage::calls;
use crate::storage::drafting::{self as store, ActivityRow, BudgetInput, BudgetRow, SectionRow};
use crate::storage::projects::{self as projects, ProjectRow};
use crate::storage::StorageError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// How long what the person writes in a section may be.
const MAX_SECTION_BYTES: usize = 12_000;
const MAX_OPEN_POINTS: usize = 6;
/// What the assistant may propose when the drafting begins (ADR-021).
const MAX_PROPOSED_LINES: usize = 12;
const MAX_PROPOSED_ACTIVITIES: usize = 10;
/// A project with no maximum duration in its call is planned within this many months.
const DEFAULT_MAX_MONTHS: u32 = 24;

// ------------------------------------------------------------------ the view

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionStatus {
    Empty,
    /// Written by the AI, waiting for the person.
    DraftAi,
    /// Written or edited by the person, not confirmed yet.
    DraftUser,
    Confirmed,
    /// Confirmed or written before something it depends on changed.
    NeedsReview,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionView {
    #[serde(flatten)]
    pub spec: SectionSpec,
    pub content: String,
    pub status: SectionStatus,
    /// What the AI says is missing in its text.
    pub open_points: Vec<String>,
    /// Numbers in the AI's text that nobody gave (it was asked again once). Empty once the person edits it.
    pub unsupported_figures: Vec<String>,
    /// What the section asks, in plain words, as the assistant explained it when the drafting began.
    pub plain: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BudgetItemView {
    #[serde(flatten)]
    pub row: BudgetRow,
    pub line: LineTotal,
}

#[derive(Debug, Clone, Serialize)]
pub struct BudgetView {
    pub items: Vec<BudgetItemView>,
    pub totals: Totals,
    pub confirmed: bool,
    /// Lines that still have no cost: the budget cannot be confirmed until they do.
    pub missing_prices: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleView {
    pub activities: Vec<ActivityRow>,
    pub duration_months: u32,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DraftingView {
    pub project: ProjectRow,
    /// Whether the proposal follows what the call asks for (what the person confirmed, or what the code proposes).
    pub asks_for_proposal: bool,
    /// The person already answered it.
    pub asks_confirmed: bool,
    pub requirements: CallRequirements,
    pub sections: Vec<SectionView>,
    pub budget: BudgetView,
    pub schedule: ScheduleView,
    /// The goal the person chose, as the sections talk about it.
    pub objective: Option<String>,
    /// The assistant already prepared the draft (explanations, budget lines and schedule), or tried and was not needed.
    pub plan_ready: bool,
}

pub(crate) fn requirements_of(conn: &Connection, project: &ProjectRow) -> Result<CallRequirements, ServiceError> {
    let Some(reading) = &project.call_reading_id else { return Ok(CallRequirements::default()) };
    Ok(calls::result(conn, reading)?.map(|(doc, _)| call_requirements(&doc)).unwrap_or_default())
}

/// The plan of sections with what the person answered about the proposal.
pub(crate) fn plan_of(conn: &Connection, project: &ProjectRow, req: &CallRequirements) -> Result<(bool, bool, Vec<SectionSpec>), ServiceError> {
    let answered = store::asks_for_proposal(conn, &project.id)?;
    let asks = answered.unwrap_or_else(|| req.asks_for_proposal());
    Ok((asks, answered.is_some(), plan_sections(req, asks)))
}

pub(crate) fn budget_items(rows: &[BudgetRow]) -> Vec<Item> {
    rows.iter().map(|r| Item { quantity: r.quantity, unit_price: r.unit_price_mxn, vat_included: r.vat_included, funded_by: r.funded_by, administrative: r.administrative }).collect()
}

fn funder_words(f: Funder) -> &'static str {
    match f {
        Funder::Requested => "lo pide a la convocatoria",
        Funder::Institution => "lo aporta la institución",
        Funder::Other => "lo aporta otra fuente",
    }
}

/// The budget as words, for the AI and for the guide: every figure here was added up by the code.
pub(crate) fn budget_text(rows: &[BudgetRow], t: &Totals) -> String {
    if rows.is_empty() {
        return "Todavía no hay partidas.".into();
    }
    let missing = rows.iter().filter(|r| r.unit_price_mxn <= 0.0).count();
    let mut s = String::new();
    for r in rows {
        if r.unit_price_mxn <= 0.0 {
            let unit = r.unit.as_deref().map(|u| format!(" {u}")).unwrap_or_default();
            s.push_str(&format!("- {} ({}): {}{unit} — costo por definir ({})\n", r.description, r.category, r.quantity, funder_words(r.funded_by)));
            continue;
        }
        let l = budget::line_total(&Item { quantity: r.quantity, unit_price: r.unit_price_mxn, vat_included: r.vat_included, funded_by: r.funded_by, administrative: r.administrative });
        let unit = r.unit.as_deref().map(|u| format!(" {u}")).unwrap_or_default();
        s.push_str(&format!("- {} ({}): {}{unit} a {} = {} ({})\n", r.description, r.category, r.quantity, format_mxn(r.unit_price_mxn), format_mxn(l.total), funder_words(r.funded_by)));
    }
    if missing > 0 {
        s.push_str(&format!("Falta el costo de {missing} partida(s): todavía no hay total del proyecto.\n"));
    } else {
        s.push_str(&format!(
            "Total del proyecto: {}. Lo que se pide: {}. Lo que aporta la institución: {}. Otras fuentes: {}. Contrapartida: {} %.\n",
            format_mxn(t.total), format_mxn(t.requested), format_mxn(t.institution), format_mxn(t.other), t.counterpart_percent
        ));
    }
    s
}

pub(crate) fn schedule_text(rows: &[ActivityRow]) -> String {
    if rows.is_empty() {
        return "Todavía no hay actividades.".into();
    }
    let mut s = String::new();
    for a in rows {
        s.push_str(&format!("- {}: del mes {} al mes {}\n", a.title, a.start_month, a.end_month));
    }
    let months: Vec<(u32, u32)> = rows.iter().map(|a| (a.start_month, a.end_month)).collect();
    s.push_str(&format!("Duración del proyecto: {} meses.\n", schedule::duration_months(&months)));
    s
}

/// The goal the person chose: its title and what it says.
pub(crate) fn objective_of(conn: &Connection, project_id: &str) -> Result<Option<String>, ServiceError> {
    Ok(projects::list_needs(conn, project_id)?.into_iter().find(|n| n.selected).map(|n| match n.description {
        Some(d) if !d.trim().is_empty() => format!("{}. {}", n.title.trim_end_matches('.'), d),
        _ => n.title,
    }))
}

fn status_of(row: Option<&SectionRow>) -> SectionStatus {
    match row {
        None => SectionStatus::Empty,
        Some(r) if r.content.trim().is_empty() => SectionStatus::Empty,
        Some(r) if r.needs_review => SectionStatus::NeedsReview,
        Some(r) if r.confirmed_at.is_some() => SectionStatus::Confirmed,
        Some(r) if r.origin == "ai_assumption" => SectionStatus::DraftAi,
        Some(_) => SectionStatus::DraftUser,
    }
}

fn strings_of(row: &SectionRow, key: &str) -> Vec<String> {
    row.source_ref.as_ref().and_then(|s| s[key].as_array()).map(|a| a.iter().filter_map(|p| p.as_str().map(String::from)).collect()).unwrap_or_default()
}

pub fn drafting_view(conn: &Connection, project_id: &str) -> Result<DraftingView, ServiceError> {
    let project = projects::get_project(conn, project_id)?.ok_or(ServiceError::NotFound)?;
    let requirements = requirements_of(conn, &project)?;
    let (asks_for_proposal, asks_confirmed, plan) = plan_of(conn, &project, &requirements)?;
    let rows = store::sections(conn, project_id)?;
    let find = |key: &str| rows.iter().find(|r| r.key == key);
    let prepared = store::plan(conn, project_id)?;
    let sections = plan
        .into_iter()
        .map(|mut spec| {
            let row = find(&spec.key);
            let note = prepared.as_ref().and_then(|p| p.get(&spec.key));
            let words = |field: &str| note.and_then(|n| n[field].as_str()).map(str::trim).filter(|w| !w.is_empty()).map(String::from);
            // the title the assistant gave replaces the first words of the call's sentence, which read badly
            if let Some(title) = words("title") {
                spec.title = title;
            }
            SectionView {
                status: status_of(row),
                content: row.map(|r| r.content.clone()).unwrap_or_default(),
                open_points: row.map(|r| strings_of(r, "open_points")).unwrap_or_default(),
                unsupported_figures: row.map(|r| strings_of(r, "unsupported_figures")).unwrap_or_default(),
                plain: words("plain"),
                spec,
            }
        })
        .collect();
    let items = store::budget(conn, project_id)?;
    let totals = budget::totals(&budget_items(&items));
    let activities = store::schedule(conn, project_id)?;
    let months: Vec<(u32, u32)> = activities.iter().map(|a| (a.start_month, a.end_month)).collect();
    let confirmed = |key: &str| find(key).is_some_and(|r| r.confirmed_at.is_some() && !r.needs_review);
    Ok(DraftingView {
        asks_for_proposal,
        asks_confirmed,
        requirements,
        sections,
        budget: BudgetView {
            items: items.iter().map(|r| BudgetItemView { line: budget::line_total(&Item { quantity: r.quantity, unit_price: r.unit_price_mxn, vat_included: r.vat_included, funded_by: r.funded_by, administrative: r.administrative }), row: r.clone() }).collect(),
            totals,
            confirmed: confirmed(KEY_BUDGET) && !items.is_empty(),
            missing_prices: items.iter().filter(|r| r.unit_price_mxn <= 0.0).count(),
        },
        schedule: ScheduleView { duration_months: schedule::duration_months(&months), confirmed: confirmed(KEY_SCHEDULE) && !activities.is_empty(), activities },
        objective: objective_of(conn, project_id)?,
        plan_ready: prepared.is_some(),
        project,
    })
}

/// Every section that has to be confirmed is, and so are the budget and the schedule.
pub fn sections_confirmed(conn: &Connection, project_id: &str) -> Result<bool, ServiceError> {
    let v = drafting_view(conn, project_id)?;
    Ok(v.sections.iter().filter(|s| s.spec.required).all(|s| match s.spec.kind {
        SectionKind::Budget => v.budget.confirmed,
        SectionKind::Schedule => v.schedule.confirmed,
        _ => s.status == SectionStatus::Confirmed,
    }))
}

// ------------------------------------------------------------------ the person's side

fn in_drafting(conn: &Connection, project_id: &str) -> Result<ProjectRow, ServiceError> {
    let project = projects::get_project(conn, project_id)?.ok_or(ServiceError::NotFound)?;
    if project.stage != Stage::Drafting {
        return Err(ServiceError::WrongStage);
    }
    Ok(project)
}

/// The person says whether the call asks for a project proposal as a document of its own.
pub fn set_asks_for_proposal(db: &SharedDb, project_id: &str, asks: bool) -> Result<DraftingView, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    store::set_asks_for_proposal(&conn, project_id, asks)?;
    drafting_view(&conn, project_id)
}

fn text_spec(conn: &Connection, project: &ProjectRow, key: &str) -> Result<SectionSpec, ServiceError> {
    let req = requirements_of(conn, project)?;
    let (_, _, plan) = plan_of(conn, project, &req)?;
    plan.into_iter().find(|s| s.key == key && s.kind == SectionKind::Text).ok_or(ServiceError::NotFound)
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum EditOutcome {
    Saved { view: DraftingView },
    Quarantine { report: QuarantineReport },
}

/// The person writes or corrects the text of a section. It is scanned like any text and goes back to a draft.
pub fn save_text(db: &SharedDb, project_id: &str, key: &str, text: &str, decision: Option<Decision>) -> Result<EditOutcome, ServiceError> {
    let conn = lock(db)?;
    let project = in_drafting(&conn, project_id)?;
    text_spec(&conn, &project, key)?;
    if text.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    if text.len() > MAX_SECTION_BYTES {
        return Err(ServiceError::TextTooLarge);
    }
    let fields = vec![("section_text".to_string(), text.to_string())];
    let clean = match guard_texts(&conn, "project_section", &fields, decision)? {
        Ok(mut t) => t.remove(0),
        Err(report) => return Ok(EditOutcome::Quarantine { report }),
    };
    store::save_section(&conn, project_id, key, &clean, "user", None)?;
    Ok(EditOutcome::Saved { view: drafting_view(&conn, project_id)? })
}

/// The person confirms the text of a section.
pub fn confirm_text(db: &SharedDb, project_id: &str, key: &str) -> Result<DraftingView, ServiceError> {
    let conn = lock(db)?;
    let project = in_drafting(&conn, project_id)?;
    text_spec(&conn, &project, key)?;
    if !store::confirm_section(&conn, project_id, key)? {
        return Err(ServiceError::Storage(StorageError::NothingToConfirm));
    }
    drafting_view(&conn, project_id)
}

// ------------------------------------------------------------------ the AI writes a section

#[derive(Debug, Serialize)]
pub struct DraftOutcome {
    pub view: DraftingView,
    pub ai: AiStatus,
}

/// The AI writes the text of a section from what is confirmed. A figure nobody gave is asked again once; if it
/// stays it is only flagged by the screen. The text is a draft (origin `ai_assumption`) until the person confirms.
pub async fn draft_section(db: &SharedDb, provider: Option<&dyn AiProvider>, project_id: &str, key: &str) -> Result<DraftOutcome, ServiceError> {
    let (spec, context, sources) = {
        let conn = lock(db)?;
        let project = in_drafting(&conn, project_id)?;
        let spec = text_spec(&conn, &project, key)?;
        let summary = projects::get_summary(&conn, project_id)?.ok_or(ServiceError::WrongStage)?;
        let root = projects::get_root(&conn, project_id)?.map(|r| r.text).unwrap_or_default();
        let objective = objective_of(&conn, project_id)?.ok_or(ServiceError::WrongStage)?;
        let budget_rows = store::budget(&conn, project_id)?;
        let budget_words = budget_text(&budget_rows, &budget::totals(&budget_items(&budget_rows)));
        let schedule_words = schedule_text(&store::schedule(&conn, project_id)?);
        let profile = profile_context(&conn)?;
        let call = call_context(&conn, &project)?;
        let turns = projects::turns(&conn, project_id)?;
        let mut sources = figure_sources(&conn, &project, &turns)?;
        let summary_json = summary.summary.to_string();
        sources.extend([summary_json.clone(), root.clone(), objective.clone(), budget_words.clone(), schedule_words.clone()]);
        let context = vec![
            format!("Sección que se redacta: {}\nQué debe decir: {}", spec.title, spec.guidance),
            format!("Perfil de la institución:\n{profile}"),
            format!("Convocatoria que la persona confirmó:\n{call}"),
            format!("Resumen del diagnóstico confirmado (JSON):\n{summary_json}"),
            format!("Causa de fondo confirmada:\n{root}"),
            format!("Objetivo del proyecto:\n{objective}"),
            format!("Presupuesto (lo calculó el programa):\n{budget_words}"),
            format!("Cronograma:\n{schedule_words}"),
        ];
        (spec, context, sources)
    };
    let user = format!("Escriba el texto de la sección «{}».", spec.title);

    let refs: Vec<&str> = sources.iter().map(String::as_str).collect();
    let unsupported = |v: &Value| figures::unsupported_figures(&[v["content"].as_str().unwrap_or("")], &refs);
    let mut result = ask_ai(db, provider, AiTask::DraftingSection, project_id, context.clone(), user.clone()).await;
    // the AI must not invent numbers: ask once more if it did
    let mut bad = result.as_ref().map(&unsupported).unwrap_or_default();
    if !bad.is_empty() {
        let again = format!("{user}\n\nEstas cifras no constan en lo que se le dio: {}. No las use; escriba [por completar: …] si hace falta.", bad.join(", "));
        if let Ok(v2) = ask_ai(db, provider, AiTask::DraftingSection, project_id, context, again).await {
            bad = unsupported(&v2);
            result = Ok(v2);
        }
    }

    let conn = lock(db)?;
    let ai = match result {
        Ok(v) => {
            let content = v["content"].as_str().unwrap_or("").trim().to_string();
            if content.is_empty() {
                AiStatus::Unavailable
            } else {
                let open: Vec<&str> = v["open_points"].as_array().into_iter().flatten().filter_map(Value::as_str).take(MAX_OPEN_POINTS).collect();
                store::save_section(&conn, project_id, &spec.key, &content, "ai_assumption", Some(&json!({ "prompt": prompts::DRAFTING_SECTION_VERSION, "open_points": open, "unsupported_figures": bad })))?;
                AiStatus::Used
            }
        }
        Err(e) => AiStatus::from(&e),
    };
    Ok(DraftOutcome { view: drafting_view(&conn, project_id)?, ai })
}

// ------------------------------------------------------------------ the assistant prepares the draft (ADR-021)

/// What every drafting call gets: the confirmed facts, as text, and where a figure may come from.
struct Gathered {
    blocks: Vec<String>,
    sources: Vec<String>,
    requirements: CallRequirements,
    specs: Vec<SectionSpec>,
}

fn gather(conn: &Connection, project_id: &str, with_budget: bool) -> Result<Gathered, ServiceError> {
    let project = in_drafting(conn, project_id)?;
    let summary = projects::get_summary(conn, project_id)?.ok_or(ServiceError::WrongStage)?;
    let root = projects::get_root(conn, project_id)?.map(|r| r.text).unwrap_or_default();
    let objective = objective_of(conn, project_id)?.ok_or(ServiceError::WrongStage)?;
    let requirements = requirements_of(conn, &project)?;
    let (_, _, specs) = plan_of(conn, &project, &requirements)?;
    let profile = profile_context(conn)?;
    let call = call_context(conn, &project)?;
    let turns = projects::turns(conn, project_id)?;
    let mut sources = figure_sources(conn, &project, &turns)?;
    let summary_json = summary.summary.to_string();
    sources.extend([summary_json.clone(), root.clone(), objective.clone()]);
    let mut blocks = vec![
        format!("Perfil de la institución:\n{profile}"),
        format!("Convocatoria que la persona confirmó:\n{call}"),
        format!("Resumen del diagnóstico confirmado (JSON):\n{summary_json}"),
        format!("Causa de fondo confirmada:\n{root}"),
        format!("Objetivo del proyecto:\n{objective}"),
    ];
    if with_budget {
        let rows = store::budget(conn, project_id)?;
        let budget_words = budget_text(&rows, &budget::totals(&budget_items(&rows)));
        let schedule_words = schedule_text(&store::schedule(conn, project_id)?);
        blocks.push(format!("Presupuesto (lo calculó el programa):\n{budget_words}"));
        blocks.push(format!("Cronograma:\n{schedule_words}"));
        sources.extend([budget_words, schedule_words]);
    }
    Ok(Gathered { blocks, sources, requirements, specs })
}

fn clip(text: &str, chars: usize) -> String {
    let t = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() <= chars { t } else { t.chars().take(chars).collect::<String>().trim_end().to_string() }
}

fn quantity_text(q: f64) -> String {
    if q.fract() == 0.0 { format!("{}", q as i64) } else { format!("{q}") }
}

/// Keeps what the assistant prepared: the clear title and the plain words of each section, the budget lines (with no
/// price) and the schedule it proposed. A line or an activity it proposed is only taken if the person has none yet,
/// and a quantity only if somebody said it: the rest of what it says is a proposal the person corrects.
fn apply_plan(conn: &Connection, project_id: &str, v: &Value, keys: &[String], sources: &[String], max_months: u32) -> Result<(), ServiceError> {
    let mut sections = serde_json::Map::new();
    for s in v["sections"].as_array().into_iter().flatten() {
        let key = s["key"].as_str().unwrap_or("");
        if !keys.iter().any(|k| k == key) {
            continue;
        }
        let (title, plain) = (clip(s["title"].as_str().unwrap_or(""), 90), clip(s["plain"].as_str().unwrap_or(""), 420));
        if !title.is_empty() || !plain.is_empty() {
            sections.insert(key.to_string(), json!({ "title": title, "plain": plain }));
        }
    }

    let refs: Vec<&str> = sources.iter().map(String::as_str).collect();
    if store::budget(conn, project_id)?.is_empty() {
        for l in v["budget_lines"].as_array().into_iter().flatten().take(MAX_PROPOSED_LINES) {
            let (description, category) = (clip(l["description"].as_str().unwrap_or(""), 160), clip(l["category"].as_str().unwrap_or(""), 60));
            if description.is_empty() || category.is_empty() {
                continue;
            }
            let quantity = l["quantity"]
                .as_f64()
                .filter(|q| *q > 0.0 && *q <= 1_000_000.0 && figures::unsupported_figures(&[quantity_text(*q).as_str()], &refs).is_empty())
                .unwrap_or(1.0);
            let funded_by = l["funded_by"].as_str().and_then(Funder::from_db).unwrap_or(Funder::Requested);
            let unit = l["unit"].as_str().map(|u| clip(u, 30)).filter(|u| !u.is_empty());
            store::insert_proposed_budget_item(
                conn,
                project_id,
                &BudgetInput { category, description, quantity, unit, unit_price_mxn: 0.0, vat_included: false, funded_by, administrative: l["administrative"].as_bool().unwrap_or(false) },
            )?;
        }
    }
    if store::schedule(conn, project_id)?.is_empty() {
        for a in v["activities"].as_array().into_iter().flatten().take(MAX_PROPOSED_ACTIVITIES) {
            let title = clip(a["title"].as_str().unwrap_or(""), 160);
            let (start, end) = (a["start_month"].as_i64().unwrap_or(0), a["end_month"].as_i64().unwrap_or(0));
            if title.is_empty() || schedule::validate(start, end).is_err() || end > i64::from(max_months) {
                continue;
            }
            store::insert_proposed_activity(conn, project_id, &title, start as u32, end as u32)?;
        }
    }
    store::save_plan(conn, project_id, prompts::DRAFTING_PLAN_VERSION, &Value::Object(sections))?;
    Ok(())
}

/// When the drafting begins the assistant prepares everything it already knows, in one call: a clear title and a
/// plain explanation for each section (so a sentence of the call is never left ambiguous), the budget lines without
/// prices and a schedule. The person then only writes the costs and corrects. It runs once; if the AI is not
/// available nothing is saved and the person can try again.
pub async fn prepare_plan(db: &SharedDb, provider: Option<&dyn AiProvider>, project_id: &str) -> Result<DraftOutcome, ServiceError> {
    let (context, keys, sources, max_months) = {
        let conn = lock(db)?;
        if store::plan(&conn, project_id)?.is_some() {
            in_drafting(&conn, project_id)?;
            return Ok(DraftOutcome { view: drafting_view(&conn, project_id)?, ai: AiStatus::Skipped });
        }
        let g = gather(&conn, project_id, false)?;
        let texts: Vec<&SectionSpec> = g.specs.iter().filter(|s| s.kind == SectionKind::Text).collect();
        let list = texts.iter().map(|s| format!("- {} | {} | lo que dice la convocatoria: {}", s.key, s.title, s.guidance)).collect::<Vec<_>>().join("\n");
        let max_months = g.requirements.max_duration_months.as_ref().map_or(DEFAULT_MAX_MONTHS, |d| d.value);
        let mut context = g.blocks;
        if let Some(d) = &g.requirements.max_duration_months {
            context.push(format!("Duración máxima que dice la convocatoria: {} meses.", d.value));
        }
        context.push(format!("Secciones que pide la propuesta (clave | título | qué dice la convocatoria):\n{list}"));
        (context, texts.iter().map(|s| s.key.clone()).collect::<Vec<_>>(), g.sources, max_months)
    };
    let user = "Prepare el borrador del proyecto: aclare cada sección, proponga las partidas del presupuesto (sin precios) y el cronograma.".to_string();
    let result = ask_ai(db, provider, AiTask::DraftingPlan, project_id, context, user).await;

    let conn = lock(db)?;
    let ai = match result {
        Ok(v) => {
            apply_plan(&conn, project_id, &v, &keys, &sources, max_months)?;
            AiStatus::Used
        }
        Err(e) => AiStatus::from(&e),
    };
    Ok(DraftOutcome { view: drafting_view(&conn, project_id)?, ai })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftMode {
    /// The text of every section.
    Full,
    /// A short guide of what to include in each, for the person to write.
    Guide,
}

/// The assistant writes every pending section in one call: it gets the whole list of sections and the person's
/// choice (a full draft or a short guide). Texts the person wrote or confirmed are left alone. A figure nobody gave
/// is asked again once; if it stays it is flagged on its section. Everything it writes is a draft until confirmed.
pub async fn draft_all(db: &SharedDb, provider: Option<&dyn AiProvider>, project_id: &str, mode: DraftMode) -> Result<DraftOutcome, ServiceError> {
    let (targets, context, sources) = {
        let conn = lock(db)?;
        let g = gather(&conn, project_id, true)?;
        let view = drafting_view(&conn, project_id)?;
        let targets: Vec<(String, String, String)> = view
            .sections
            .iter()
            .filter(|s| s.spec.kind == SectionKind::Text && matches!(s.status, SectionStatus::Empty | SectionStatus::DraftAi))
            .map(|s| (s.spec.key.clone(), s.spec.title.clone(), s.plain.clone().unwrap_or_else(|| s.spec.guidance.clone())))
            .collect();
        if targets.is_empty() {
            return Ok(DraftOutcome { view, ai: AiStatus::Skipped });
        }
        let list = targets.iter().map(|(k, t, what)| format!("- {k} | {t} | {what}")).collect::<Vec<_>>().join("\n");
        let mut context = g.blocks;
        context.push(format!("Secciones que hay que cubrir (clave | título | qué debe decir):\n{list}"));
        (targets, context, g.sources)
    };
    let user = match mode {
        DraftMode::Full => "Modo: borrador completo. Escriba el texto de todas las secciones de la lista.",
        DraftMode::Guide => "Modo: guía breve. Para cada sección de la lista escriba una guía de 3 a 5 puntos de lo que la persona debe incluir.",
    }
    .to_string();

    let refs: Vec<&str> = sources.iter().map(String::as_str).collect();
    let wanted = |key: &str| targets.iter().any(|(k, _, _)| k == key);
    let unsupported = |v: &Value| -> Vec<(String, Vec<String>)> {
        v["sections"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|s| wanted(s["key"].as_str().unwrap_or("")))
            .filter_map(|s| {
                let bad = figures::unsupported_figures(&[s["content"].as_str().unwrap_or("")], &refs);
                (!bad.is_empty()).then(|| (s["key"].as_str().unwrap_or("").to_string(), bad))
            })
            .collect()
    };
    let mut result = ask_ai(db, provider, AiTask::DraftingAll, project_id, context.clone(), user.clone()).await;
    // the AI must not invent numbers: ask once more if it did
    let mut flagged: Vec<(String, Vec<String>)> = result.as_ref().map(&unsupported).unwrap_or_default();
    if !flagged.is_empty() {
        let list = flagged.iter().map(|(k, f)| format!("{k}: {}", f.join(", "))).collect::<Vec<_>>().join("; ");
        let again = format!("{user}\n\nEstas cifras no constan en lo que se le dio ({list}). No las use; escriba [por completar: …] si hace falta.");
        if let Ok(v2) = ask_ai(db, provider, AiTask::DraftingAll, project_id, context, again).await {
            flagged = unsupported(&v2);
            result = Ok(v2);
        }
    }

    let conn = lock(db)?;
    let ai = match result {
        Ok(v) => {
            let mut saved = 0;
            for s in v["sections"].as_array().into_iter().flatten() {
                let key = s["key"].as_str().unwrap_or("");
                let content = s["content"].as_str().unwrap_or("").trim();
                if !wanted(key) || content.is_empty() {
                    continue;
                }
                let open: Vec<&str> = s["open_points"].as_array().into_iter().flatten().filter_map(Value::as_str).take(MAX_OPEN_POINTS).collect();
                let bad: Vec<&String> = flagged.iter().filter(|(k, _)| k == key).flat_map(|(_, f)| f).collect();
                let how = if mode == DraftMode::Guide { "guide" } else { "full" };
                store::save_section(&conn, project_id, key, content, "ai_assumption", Some(&json!({ "prompt": prompts::DRAFTING_ALL_VERSION, "mode": how, "open_points": open, "unsupported_figures": bad })))?;
                saved += 1;
            }
            if saved == 0 { AiStatus::Unavailable } else { AiStatus::Used }
        }
        Err(e) => AiStatus::from(&e),
    };
    Ok(DraftOutcome { view: drafting_view(&conn, project_id)?, ai })
}

/// The person confirms every text that is ready in one go. The ones that carry figures nobody gave, or that were
/// marked for review because something they depend on changed, are left for the person to look at one by one.
pub fn confirm_all_texts(db: &SharedDb, project_id: &str) -> Result<DraftingView, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    let view = drafting_view(&conn, project_id)?;
    for s in view.sections.iter().filter(|s| {
        s.spec.kind == SectionKind::Text && matches!(s.status, SectionStatus::DraftAi | SectionStatus::DraftUser) && s.unsupported_figures.is_empty()
    }) {
        store::confirm_section(&conn, project_id, &s.spec.key)?;
    }
    drafting_view(&conn, project_id)
}

// ------------------------------------------------------------------ the budget

#[derive(Debug, Clone, Deserialize)]
pub struct BudgetItemInput {
    pub id: Option<String>,
    pub category: String,
    pub description: String,
    pub quantity: f64,
    pub unit: Option<String>,
    pub unit_price_mxn: f64,
    pub vat_included: bool,
    pub funded_by: Funder,
    pub administrative: bool,
}

/// Adds or changes a budget line. The code checks the numbers; the words go through the scanner.
pub fn save_budget_item(db: &SharedDb, project_id: &str, item: BudgetItemInput, decision: Option<Decision>) -> Result<EditOutcome, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    if item.description.trim().is_empty() || item.category.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    budget::validate(item.quantity, item.unit_price_mxn).map_err(|_| ServiceError::InvalidBudgetItem)?;
    let fields = vec![
        ("budget_category".to_string(), item.category.clone()),
        ("budget_description".to_string(), item.description.clone()),
        ("budget_unit".to_string(), item.unit.clone().unwrap_or_default()),
    ];
    let texts = match guard_texts(&conn, "budget_item", &fields, decision)? {
        Ok(t) => t,
        Err(report) => return Ok(EditOutcome::Quarantine { report }),
    };
    let input = BudgetInput {
        category: texts[0].clone(),
        description: texts[1].clone(),
        quantity: item.quantity,
        unit: Some(texts[2].clone()),
        unit_price_mxn: item.unit_price_mxn,
        vat_included: item.vat_included,
        funded_by: item.funded_by,
        administrative: item.administrative,
    };
    if !store::save_budget_item(&conn, project_id, item.id.as_deref(), &input)? {
        return Err(ServiceError::NotFound);
    }
    Ok(EditOutcome::Saved { view: drafting_view(&conn, project_id)? })
}

pub fn delete_budget_item(db: &SharedDb, project_id: &str, item_id: &str) -> Result<DraftingView, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    if !store::delete_budget_item(&conn, project_id, item_id)? {
        return Err(ServiceError::NotFound);
    }
    drafting_view(&conn, project_id)
}

/// The person confirms the budget as it is: a snapshot of what the code added up is what is confirmed.
pub fn confirm_budget(db: &SharedDb, project_id: &str) -> Result<DraftingView, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    let rows = store::budget(&conn, project_id)?;
    if rows.is_empty() {
        return Err(ServiceError::Storage(StorageError::NothingToConfirm));
    }
    if rows.iter().any(|r| r.unit_price_mxn <= 0.0) {
        return Err(ServiceError::BudgetIncomplete);
    }
    let totals = budget::totals(&budget_items(&rows));
    store::save_section(&conn, project_id, KEY_BUDGET, &serde_json::to_string(&totals).unwrap_or_default(), "computed", None)?;
    store::confirm_section(&conn, project_id, KEY_BUDGET)?;
    drafting_view(&conn, project_id)
}

// ------------------------------------------------------------------ the schedule

/// Adds or changes an activity. The code checks the months; the title goes through the scanner.
pub fn save_activity(db: &SharedDb, project_id: &str, activity_id: Option<&str>, title: &str, start_month: i64, end_month: i64, decision: Option<Decision>) -> Result<EditOutcome, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    if title.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    schedule::validate(start_month, end_month).map_err(|_| ServiceError::InvalidActivity)?;
    let fields = vec![("activity_title".to_string(), title.to_string())];
    let clean = match guard_texts(&conn, "schedule_activity", &fields, decision)? {
        Ok(mut t) => t.remove(0),
        Err(report) => return Ok(EditOutcome::Quarantine { report }),
    };
    if !store::save_activity(&conn, project_id, activity_id, &clean, start_month as u32, end_month as u32)? {
        return Err(ServiceError::NotFound);
    }
    Ok(EditOutcome::Saved { view: drafting_view(&conn, project_id)? })
}

pub fn delete_activity(db: &SharedDb, project_id: &str, activity_id: &str) -> Result<DraftingView, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    if !store::delete_activity(&conn, project_id, activity_id)? {
        return Err(ServiceError::NotFound);
    }
    drafting_view(&conn, project_id)
}

pub fn confirm_schedule(db: &SharedDb, project_id: &str) -> Result<DraftingView, ServiceError> {
    let conn = lock(db)?;
    in_drafting(&conn, project_id)?;
    let rows = store::schedule(&conn, project_id)?;
    if rows.is_empty() {
        return Err(ServiceError::Storage(StorageError::NothingToConfirm));
    }
    let months: Vec<(u32, u32)> = rows.iter().map(|a| (a.start_month, a.end_month)).collect();
    let snapshot = json!({ "activities": rows.len(), "duration_months": schedule::duration_months(&months) });
    store::save_section(&conn, project_id, KEY_SCHEDULE, &snapshot.to_string(), "computed", None)?;
    store::confirm_section(&conn, project_id, KEY_SCHEDULE)?;
    drafting_view(&conn, project_id)
}

#[cfg(test)]
#[path = "drafting_service_tests.rs"]
mod tests;
