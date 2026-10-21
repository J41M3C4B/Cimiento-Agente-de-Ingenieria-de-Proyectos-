//! Commands of the projects, their diagnosis and their needs. Thin: they call `modules::projects::diagnosis`. No
//! AI: the conversation, the summary and the proposed needs came back to the platform.

use super::guard;
use crate::core::access::service::Session;
use crate::core::error::ServiceError;
use crate::error::UiError;
use crate::modules::projects::conversation::{self as convo, ConversationView};
use crate::modules::projects::diagnosis::{self as svc, AddNeedOutcome, CreateProjectOutcome, EditOutcome, NeedsView, SharedDb, SummaryEdit};
use crate::modules::projects::domain::priority::Scores;
use crate::modules::projects::domain::stage::Stage;
use crate::modules::projects::storage::projects::{self as store, ProjectRow};
use crate::scanner::guard::Decision;
use crate::Db;
use tauri::State;

pub(crate) fn shared(db: &State<'_, Db>) -> SharedDb {
    db.0.clone()
}

fn conn_err(e: impl Into<ServiceError>) -> UiError {
    UiError::from(e.into())
}

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

// ------------------------------------------------------------------ the summary of an earlier diagnosis

#[tauri::command]
pub fn conversation_get(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<ConversationView, UiError> {
    guard(&session, "conversation_get")?;
    let conn = db.0.lock().map_err(|_| UiError::internal())?;
    Ok(convo::conversation_view(&conn, &project_id)?)
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

#[tauri::command]
pub fn needs_get(session: State<'_, Session>, db: State<'_, Db>, project_id: String) -> Result<NeedsView, UiError> {
    guard(&session, "needs_get")?;
    Ok(svc::get_needs(&shared(&db), &project_id)?)
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
