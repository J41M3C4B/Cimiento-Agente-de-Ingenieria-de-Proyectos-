//! Commands for the AI settings, projects, diagnosis and needs. Thin: they build the
//! provider (the API key never leaves Rust) and call `modules::projects::diagnosis`.

use super::guard;
use crate::core::access::service::Session;
use crate::ai::metrics::{self, UsageReport};
use crate::ai::settings::{self, AiSettings, ProviderKind};
use crate::ai::{self, AiError, AiProvider, ModelCheck, ModelTier};
use crate::modules::projects::conversation::{self as convo, AnswerOutcome, ConversationView};
use crate::modules::projects::diagnosis::{
    self as svc, AddNeedOutcome, AiStatus, CreateProjectOutcome, EditOutcome, NeedsView, SharedDb, SummaryEdit, SummaryOutcome,
};
use crate::modules::projects::domain::priority::Scores;
use crate::modules::projects::domain::stage::Stage;
use crate::error::UiError;
use crate::modules::projects::jobs::{JobGuard, JobKind, JobStatus, Jobs};
use crate::scanner::guard::Decision;
use crate::core::error::ServiceError;
use crate::modules::projects::storage::projects::{self as store, ProjectRow};
use crate::storage::{self, StorageError};
use crate::Db;
use serde::Serialize;
use tauri::State;

pub(crate) fn shared(db: &State<'_, Db>) -> SharedDb {
    db.0.clone()
}

fn conn_err(e: impl Into<ServiceError>) -> UiError {
    UiError::from(e.into())
}

/// The provider chosen in the settings; it exists only if its key is saved in the keychain.
pub(crate) fn make_provider(db: &SharedDb) -> Option<Box<dyn AiProvider>> {
    let conn = db.lock().ok()?;
    let settings = settings::load(&conn).ok()?;
    drop(conn);
    let key = storage::get_api_key(settings.provider.as_str()).ok().flatten()?;
    Some(ai::build_provider(&settings, key))
}

pub(crate) fn as_dyn(p: &Option<Box<dyn AiProvider>>) -> Option<&dyn AiProvider> {
    p.as_deref()
}

// ------------------------------------------------------------------ AI settings

#[derive(Serialize)]
pub struct KeysView {
    pub gemini: bool,
    pub anthropic: bool,
}

#[derive(Serialize)]
pub struct AiStatusView {
    pub provider: ProviderKind,
    /// A key is saved for the active provider.
    pub has_key: bool,
    pub keys: KeysView,
    pub model_light: String,
    pub model_strong: String,
    pub spent_mxn: f64,
    pub cap_mxn: f64,
    pub percent: f64,
    /// 80 % or more of the monthly cap.
    pub near_cap: bool,
    /// At the cap: AI is paused, the rest of the app keeps working.
    pub paused: bool,
}

fn has_key(provider: ProviderKind) -> bool {
    storage::get_api_key(provider.as_str()).ok().flatten().is_some()
}

#[tauri::command]
pub fn ai_status(session: State<'_, Session>, db: State<'_, Db>) -> Result<AiStatusView, UiError> {
    guard(&session, "ai_status")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let s: AiSettings = settings::load(&conn).map_err(conn_err)?;
    let spent = settings::month_spend_mxn(&conn).map_err(conn_err)?;
    let percent = if s.monthly_cap_mxn > 0.0 { spent * 100.0 / s.monthly_cap_mxn } else { 100.0 };
    Ok(AiStatusView {
        provider: s.provider,
        has_key: has_key(s.provider),
        keys: KeysView { gemini: has_key(ProviderKind::Gemini), anthropic: has_key(ProviderKind::Anthropic) },
        model_light: s.model_for(ModelTier::Light).to_string(),
        model_strong: s.model_for(ModelTier::Strong).to_string(),
        spent_mxn: spent,
        cap_mxn: s.monthly_cap_mxn,
        percent,
        near_cap: percent >= 80.0,
        paused: percent >= 100.0,
    })
}

#[tauri::command]
pub fn ai_set_provider(session: State<'_, Session>, db: State<'_, Db>, provider: ProviderKind) -> Result<(), UiError> {
    guard(&session, "ai_set_provider")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let mut s = settings::load(&conn).map_err(conn_err)?;
    s.provider = provider;
    settings::save(&conn, &s).map_err(conn_err)
}

#[derive(Serialize)]
pub struct AiModelsView {
    pub provider: ProviderKind,
    pub light: String,
    pub strong: String,
    /// Thinking depth of the strong tier; empty = provider default.
    pub effort: String,
    /// What the pickers offer for the active provider.
    pub known: Vec<String>,
    /// The strong tier's chain as it runs now (main model first, backups behind it).
    pub strong_chain: Vec<String>,
}

#[tauri::command]
pub fn ai_models(session: State<'_, Session>, db: State<'_, Db>) -> Result<AiModelsView, UiError> {
    guard(&session, "ai_models")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let s = settings::load(&conn).map_err(conn_err)?;
    Ok(AiModelsView {
        provider: s.provider,
        light: s.model_of(s.provider, ModelTier::Light).to_string(),
        strong: s.model_of(s.provider, ModelTier::Strong).to_string(),
        effort: s.effort_strong.clone(),
        known: s.known_models(s.provider),
        strong_chain: s.chain_of(s.provider, ModelTier::Strong),
    })
}

/// Swaps the models of the active provider (and the strong tier's thinking depth) without rebuilding the app.
#[tauri::command]
pub fn ai_set_models(session: State<'_, Session>, db: State<'_, Db>, light: String, strong: String, effort: String) -> Result<(), UiError> {
    guard(&session, "ai_set_models")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let mut s = settings::load(&conn).map_err(conn_err)?;
    let provider = s.provider;
    s.set_models(provider, &light, &strong, &effort);
    settings::save(&conn, &s).map_err(conn_err)
}

#[tauri::command]
pub fn ai_set_key(session: State<'_, Session>, provider: ProviderKind, key: String) -> Result<(), UiError> {
    guard(&session, "ai_set_key")?;
    if key.trim().is_empty() {
        return Err(UiError::new("empty_text", "Este dato nos falta."));
    }
    storage::set_api_key(provider.as_str(), &key).map_err(|e: StorageError| conn_err(e))
}

#[tauri::command]
pub fn ai_clear_key(session: State<'_, Session>, provider: ProviderKind) -> Result<(), UiError> {
    guard(&session, "ai_clear_key")?;
    storage::delete_api_key(provider.as_str()).map_err(|e: StorageError| conn_err(e))
}

#[derive(Serialize)]
pub struct AiCheckView {
    /// The key works and every configured model exists and can answer.
    pub ok: bool,
    pub models: Vec<ModelCheck>,
    /// Why it could not be checked (same plain-language keys as `AiStatus`).
    pub problem: Option<AiStatus>,
}

/// Checks the saved key and the configured models without spending a generation call.
#[tauri::command]
pub async fn ai_check(session: State<'_, Session>, db: State<'_, Db>) -> Result<AiCheckView, UiError> {
    guard(&session, "ai_check")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let Some(provider) = as_dyn(&provider) else {
        return Ok(AiCheckView { ok: false, models: vec![], problem: Some(AiStatus::NotConfigured) });
    };
    Ok(match provider.check().await {
        Ok(models) => AiCheckView { ok: models.iter().all(|m| m.exists && m.can_generate), models, problem: None },
        Err(e @ (AiError::Auth | AiError::Offline | AiError::RateLimited)) => {
            AiCheckView { ok: false, models: vec![], problem: Some(AiStatus::from(&e)) }
        }
        Err(_) => AiCheckView { ok: false, models: vec![], problem: Some(AiStatus::Unavailable) },
    })
}

#[tauri::command]
pub fn ai_usage_report(session: State<'_, Session>, db: State<'_, Db>) -> Result<UsageReport, UiError> {
    guard(&session, "ai_usage_report")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let s = settings::load(&conn).map_err(conn_err)?;
    metrics::usage_report(&conn, &s).map_err(conn_err)
}

#[tauri::command]
pub fn ai_set_cap(session: State<'_, Session>, db: State<'_, Db>, cap_mxn: f64) -> Result<(), UiError> {
    guard(&session, "ai_set_cap")?;
    if !cap_mxn.is_finite() || cap_mxn < 0.0 {
        return Err(UiError::new("negative_number", "Este nÃºmero no puede ser menor que cero."));
    }
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    let mut s = settings::load(&conn).map_err(conn_err)?;
    s.monthly_cap_mxn = cap_mxn;
    settings::save(&conn, &s).map_err(conn_err)
}

// ------------------------------------------------------------------ projects

#[tauri::command]
pub fn project_list(session: State<'_, Session>, db: State<'_, Db>) -> Result<Vec<ProjectRow>, UiError> {
    guard(&session, "project_list")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    store::list_projects(&conn).map_err(conn_err)
}

#[tauri::command]
pub fn project_create(session: State<'_, Session>, db: State<'_, Db>, initial_request: String, decision: Option<Decision>) -> Result<CreateProjectOutcome, UiError> {
    guard(&session, "project_create")?;
    Ok(svc::create_project(&shared(&db), &initial_request, decision)?)
}

/// Deletes the project with everything of it, its call included. Cannot be undone.
#[tauri::command]
pub fn project_delete(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<bool, UiError> {
    let gate = guard(&session, "project_delete")?;
    let mut conn = db.0.lock().map_err(|_| UiError::internal())?;
    if !gate.may_delete() {
        crate::core::access::service::request_deletion(&mut conn, gate.user()?, crate::core::access::domain::DeletionKind::Project, &project_id)?;
        return Ok(true);
    }
    store::delete_project(&mut conn, &project_id).map_err(conn_err)
}

/// Paints the folder of a project (one of the colors of the list, or `None` to leave it to the app).
#[tauri::command]
pub fn project_set_color(session: State<'_, Session>, db: State<'_, Db>, project_id: String, color: Option<String>) -> Result<bool, UiError> {
    guard(&session, "project_set_color")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    if color.as_deref().is_some_and(|c| !store::PROJECT_COLORS.contains(&c)) {
        return Err(UiError::new("invalid_color", "Ese color no está en la lista."));
    }
    store::set_color(&conn, &project_id, color.as_deref()).map_err(conn_err)
}

/// Says who gives the support of a project (one of the kinds of the list, or `None` to clear it).
#[tauri::command]
pub fn project_set_donor_kind(session: State<'_, Session>, db: State<'_, Db>, project_id: String, kind: Option<String>) -> Result<bool, UiError> {
    guard(&session, "project_set_donor_kind")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    if kind.as_deref().is_some_and(|k| !store::DONOR_KINDS.contains(&k)) {
        return Err(UiError::new("invalid_donor_kind", "Ese tipo de proyecto no está en la lista."));
    }
    store::set_donor_kind(&conn, &project_id, kind.as_deref()).map_err(conn_err)
}

#[tauri::command]
pub fn project_advance(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<ProjectRow, UiError> {
    guard(&session, "project_advance")?;
    Ok(svc::advance(&shared(&db), &project_id)?)
}

#[tauri::command]
pub fn project_go_back(session: State<'_, Session>, db: State<'_, Db>, project_id: String, target: Stage) -> Result<ProjectRow, UiError> {
    guard(&session, "project_go_back")?;
    Ok(svc::go_back(&shared(&db), &project_id, target)?)
}

// ------------------------------------------------------------------ conversation (ADR-017)

#[tauri::command]
pub fn conversation_get(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<ConversationView, UiError> {
    guard(&session, "conversation_get")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    Ok(convo::conversation_view(&conn, &project_id)?)
}

/// What the AI is doing for a project right now, and how its last job ended (handed over once). The screen asks
/// this when it comes back, so work that started before the person left is still shown as work in progress.
#[tauri::command]
pub fn project_job(session: State<'_, Session>, jobs: State<'_, Jobs>, project_id: String) -> Result<JobStatus, UiError> {
    guard(&session, "project_job")?;
    Ok(jobs.status(&project_id))
}

/// Frees the project's slot, saying how the AI part went when it did run (a message held for the scanner did not).
fn finish_answer(job: JobGuard, out: &AnswerOutcome) {
    if let AnswerOutcome::Saved { ai, .. } = out {
        job.finish(*ai);
    }
}

/// Asks the AI for the opening question. Safe to call again: if the opening exists, nothing is asked.
#[tauri::command]
pub async fn conversation_start(session: State<'_, Session>, db: State<'_, Db>, jobs: State<'_, Jobs>, project_id: String) -> Result<AnswerOutcome, UiError> {
    guard(&session, "conversation_start")?;
    let job = jobs.claim(&project_id, JobKind::Turn)?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let out = convo::start_conversation(&db, as_dyn(&provider), &project_id).await?;
    finish_answer(job, &out);
    Ok(out)
}

/// What the person writes; with `confirm_root` it is the quick reply that confirms the root cause.
#[tauri::command]
pub async fn conversation_send(session: State<'_, Session>, db: State<'_, Db>,
    jobs: State<'_, Jobs>,
    project_id: String,
    text: String,
    confirm_root: bool,
    decision: Option<Decision>,
) -> Result<AnswerOutcome, UiError> {
    guard(&session, "conversation_send")?;
    let job = jobs.claim(&project_id, JobKind::Turn)?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let out = convo::send_message(&db, as_dyn(&provider), &project_id, &text, confirm_root, decision).await?;
    finish_answer(job, &out);
    Ok(out)
}

/// The AI owes a message (it failed): asks again without the person writing anything.
#[tauri::command]
pub async fn conversation_retry(session: State<'_, Session>, db: State<'_, Db>, jobs: State<'_, Jobs>, project_id: String) -> Result<AnswerOutcome, UiError> {
    guard(&session, "conversation_retry")?;
    let job = jobs.claim(&project_id, JobKind::Turn)?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let out = convo::retry(&db, as_dyn(&provider), &project_id).await?;
    finish_answer(job, &out);
    Ok(out)
}

#[tauri::command]
pub async fn diagnosis_summary_generate(session: State<'_, Session>, db: State<'_, Db>, jobs: State<'_, Jobs>, project_id: String) -> Result<SummaryOutcome, UiError> {
    guard(&session, "diagnosis_summary_generate")?;
    let job = jobs.claim(&project_id, JobKind::Summary)?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let out = svc::generate_summary(&db, as_dyn(&provider), &project_id).await?;
    job.finish(out.ai);
    Ok(out)
}

#[tauri::command]
pub fn diagnosis_summary_edit(session: State<'_, Session>, db: State<'_, Db>,
    project_id: String,
    edit: SummaryEdit,
    decision: Option<Decision>,
) -> Result<EditOutcome, UiError> {
    guard(&session, "diagnosis_summary_edit")?;
    Ok(svc::edit_summary(&shared(&db), &project_id, edit, decision)?)
}

#[tauri::command]
pub fn diagnosis_summary_confirm(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<ConversationView, UiError> {
    guard(&session, "diagnosis_summary_confirm")?;
    Ok(svc::confirm_summary(&shared(&db), &project_id)?)
}

// ------------------------------------------------------------------ needs

#[derive(Serialize)]
pub struct ProposedNeeds {
    pub view: NeedsView,
    pub ai: AiStatus,
}

#[tauri::command]
pub fn needs_get(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<NeedsView, UiError> {
    guard(&session, "needs_get")?;
    Ok(svc::get_needs(&shared(&db), &project_id)?)
}

#[tauri::command]
pub async fn needs_propose(session: State<'_, Session>, db: State<'_, Db>, jobs: State<'_, Jobs>, project_id: String) -> Result<ProposedNeeds, UiError> {
    guard(&session, "needs_propose")?;
    let job = jobs.claim(&project_id, JobKind::Needs)?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let (view, ai) = svc::propose_needs(&db, as_dyn(&provider), &project_id).await?;
    job.finish(ai);
    Ok(ProposedNeeds { view, ai })
}

#[tauri::command]
pub fn need_add(session: State<'_, Session>, db: State<'_, Db>,
    project_id: String,
    title: String,
    description: String,
    decision: Option<Decision>,
) -> Result<AddNeedOutcome, UiError> {
    guard(&session, "need_add")?;
    Ok(svc::add_need(&shared(&db), &project_id, &title, &description, decision)?)
}

#[tauri::command]
pub fn need_rate(session: State<'_, Session>, db: State<'_, Db>, project_id: String, need_id: String, scores: Scores) -> Result<NeedsView, UiError> {
    guard(&session, "need_rate")?;
    Ok(svc::rate_need(&shared(&db), &project_id, &need_id, scores)?)
}

#[tauri::command]
pub fn need_select(session: State<'_, Session>, db: State<'_, Db>, project_id: String, need_id: String) -> Result<NeedsView, UiError> {
    guard(&session, "need_select")?;
    Ok(svc::select_need(&shared(&db), &project_id, &need_id)?)
}
