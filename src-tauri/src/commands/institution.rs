//! Commands of the board and the bank accounts of the institution (ADR-033 §3): thin, they call `core::protected`.

use super::guard;
use crate::common::forms::Values;
use crate::core::access::domain::DeletionKind;
use crate::core::access::service::{request_deletion, Session};
use crate::core::protected::{self as svc, ListForm, ListKind, ListOutcome, Lists};
use crate::error::UiError;
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn institution_lists(session: State<'_, Session>, db: State<'_, Db>) -> Result<Lists, UiError> {
    guard(&session, "institution_lists")?;
    let conn = lock(&db)?;
    Ok(svc::lists(&conn)?)
}

/// The window of a row: empty to add one (`id = None`), or what a saved one holds.
#[tauri::command]
pub fn institution_list_form(session: State<'_, Session>, db: State<'_, Db>, kind: ListKind, id: Option<String>) -> Result<ListForm, UiError> {
    guard(&session, "institution_list_form")?;
    let conn = lock(&db)?;
    Ok(svc::form(&conn, kind, id.as_deref())?)
}

#[tauri::command]
pub fn institution_list_save(session: State<'_, Session>, db: State<'_, Db>, kind: ListKind, id: Option<String>, values: Values) -> Result<ListOutcome, UiError> {
    guard(&session, "institution_list_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save(&mut conn, kind, id.as_deref(), values)?)
}

/// Who left the board or an account that closed (`active = false`), or back again.
#[tauri::command]
pub fn institution_list_set_active(session: State<'_, Session>, db: State<'_, Db>, kind: ListKind, id: String, active: bool) -> Result<Lists, UiError> {
    guard(&session, "institution_list_set_active")?;
    let mut conn = lock(&db)?;
    Ok(svc::set_active(&mut conn, kind, &id, active)?)
}

/// Deletes a row written by mistake: for good when the person may delete, as a request otherwise (ADR-028).
#[tauri::command]
pub fn institution_list_delete(session: State<'_, Session>, db: State<'_, Db>, kind: ListKind, id: String) -> Result<Lists, UiError> {
    let gate = guard(&session, "institution_list_delete")?;
    let mut conn = lock(&db)?;
    if gate.may_delete() {
        svc::delete(&mut conn, kind, &id)?;
    } else {
        let deletion = match kind {
            ListKind::BoardMember => DeletionKind::BoardMember,
            ListKind::BankAccount => DeletionKind::BankAccount,
        };
        request_deletion(&mut conn, gate.user()?, deletion, &id)?;
    }
    Ok(svc::lists(&conn)?)
}

/// Shows the CLABE of an account in full; the audit log keeps that it was shown.
#[tauri::command]
pub fn institution_clabe_reveal(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<String, UiError> {
    guard(&session, "institution_clabe_reveal")?;
    let conn = lock(&db)?;
    Ok(svc::reveal_clabe(&conn, &id)?)
}
