//! Commands of the drafting, the review and the guide (ADR-018). Thin: they call the services.

use super::guard;
use crate::access_service::Session;
use super::diagnosis::{as_dyn, make_provider, shared};
use crate::drafting_service::{self as svc, BudgetItemInput, DraftMode, DraftOutcome, DraftingView, EditOutcome};
use crate::error::UiError;
use crate::guide_service::{self, Exported};
use crate::review_service::{self, ReviewView};
use crate::scanner::guard::Decision;
use crate::Db;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn drafting_get(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<DraftingView, UiError> {
    guard(&session, "drafting_get")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    Ok(svc::drafting_view(&conn, &project_id)?)
}

/// The person says whether the call asks for a project proposal as a document of its own.
#[tauri::command]
pub fn drafting_set_asks(session: State<'_, Session>, db: State<'_, Db>, project_id: String, asks: bool) -> Result<DraftingView, UiError> {
    guard(&session, "drafting_set_asks")?;
    Ok(svc::set_asks_for_proposal(&shared(&db), &project_id, asks)?)
}

#[tauri::command]
pub async fn section_draft(session: State<'_, Session>, db: State<'_, Db>, project_id: String, key: String) -> Result<DraftOutcome, UiError> {
    guard(&session, "section_draft")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    Ok(svc::draft_section(&db, as_dyn(&provider), &project_id, &key).await?)
}

/// When the drafting begins: the assistant prepares the explanations, the budget lines and the schedule (once).
#[tauri::command]
pub async fn drafting_prepare(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<DraftOutcome, UiError> {
    guard(&session, "drafting_prepare")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    Ok(svc::prepare_plan(&db, as_dyn(&provider), &project_id).await?)
}

/// The assistant writes every pending section at once, as a full draft or as a short guide.
#[tauri::command]
pub async fn sections_draft_all(session: State<'_, Session>, db: State<'_, Db>, project_id: String, mode: DraftMode) -> Result<DraftOutcome, UiError> {
    guard(&session, "sections_draft_all")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    Ok(svc::draft_all(&db, as_dyn(&provider), &project_id, mode).await?)
}

/// Confirms every text that is ready.
#[tauri::command]
pub fn sections_confirm_all(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<DraftingView, UiError> {
    guard(&session, "sections_confirm_all")?;
    Ok(svc::confirm_all_texts(&shared(&db), &project_id)?)
}

#[tauri::command]
pub fn section_save(session: State<'_, Session>, db: State<'_, Db>, project_id: String, key: String, text: String, decision: Option<Decision>) -> Result<EditOutcome, UiError> {
    guard(&session, "section_save")?;
    Ok(svc::save_text(&shared(&db), &project_id, &key, &text, decision)?)
}

#[tauri::command]
pub fn section_confirm(session: State<'_, Session>, db: State<'_, Db>, project_id: String, key: String) -> Result<DraftingView, UiError> {
    guard(&session, "section_confirm")?;
    Ok(svc::confirm_text(&shared(&db), &project_id, &key)?)
}

#[tauri::command]
pub fn budget_save_item(session: State<'_, Session>, db: State<'_, Db>, project_id: String, item: BudgetItemInput, decision: Option<Decision>) -> Result<EditOutcome, UiError> {
    guard(&session, "budget_save_item")?;
    Ok(svc::save_budget_item(&shared(&db), &project_id, item, decision)?)
}

#[tauri::command]
pub fn budget_delete_item(session: State<'_, Session>, db: State<'_, Db>, project_id: String, item_id: String) -> Result<DraftingView, UiError> {
    guard(&session, "budget_delete_item")?;
    Ok(svc::delete_budget_item(&shared(&db), &project_id, &item_id)?)
}

#[tauri::command]
pub fn budget_confirm(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<DraftingView, UiError> {
    guard(&session, "budget_confirm")?;
    Ok(svc::confirm_budget(&shared(&db), &project_id)?)
}

#[tauri::command]
pub fn schedule_save_activity(session: State<'_, Session>, db: State<'_, Db>,
    project_id: String,
    activity_id: Option<String>,
    title: String,
    start_month: i64,
    end_month: i64,
    decision: Option<Decision>,
) -> Result<EditOutcome, UiError> {
    guard(&session, "schedule_save_activity")?;
    Ok(svc::save_activity(&shared(&db), &project_id, activity_id.as_deref(), &title, start_month, end_month, decision)?)
}

#[tauri::command]
pub fn schedule_delete_activity(session: State<'_, Session>, db: State<'_, Db>, project_id: String, activity_id: String) -> Result<DraftingView, UiError> {
    guard(&session, "schedule_delete_activity")?;
    Ok(svc::delete_activity(&shared(&db), &project_id, &activity_id)?)
}

#[tauri::command]
pub fn schedule_confirm(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<DraftingView, UiError> {
    guard(&session, "schedule_confirm")?;
    Ok(svc::confirm_schedule(&shared(&db), &project_id)?)
}

/// The automatic review, recomputed every time.
#[tauri::command]
pub fn review_get(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<ReviewView, UiError> {
    guard(&session, "review_get")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    Ok(review_service::review(&conn, &project_id)?)
}

/// Writes the guide in Word in the person's Downloads folder (or Documents, or the home folder).
#[tauri::command]
pub fn guide_export(session: State<'_, Session>, app: AppHandle, db: State<'_, Db>, project_id: String) -> Result<Exported, UiError> {
    guard(&session, "guide_export")?;
    let paths = app.path();
    let dir = paths.download_dir().or_else(|_| paths.document_dir()).or_else(|_| paths.home_dir()).map_err(|_| UiError::internal())?;
    Ok(guide_service::export_guide(&shared(&db), &project_id, &dir)?)
}
