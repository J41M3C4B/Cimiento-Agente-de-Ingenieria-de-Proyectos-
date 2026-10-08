//! Commands of the staff module (ADR-027): thin, they call `staff_service`.

use super::guard;
use crate::access_service::Session;
use crate::error::UiError;
use crate::modules::hr::domain::person::PersonData;
use crate::modules::hr::domain::position::PositionInput;
use crate::modules::hr::service::{ModalityInfo, PersonView};
use crate::modules::hr::storage::CustomField;
use crate::scanner::guard::Decision;
use crate::staff_service::{self as svc, PersonOutcome, PositionOutcome, StaffChange, StaffOverview};
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn hr_overview(session: State<'_, Session>, db: State<'_, Db>) -> Result<StaffOverview, UiError> {
    guard(&session, "hr_overview")?;
    let conn = lock(&db)?;
    Ok(svc::overview(&conn)?)
}

#[tauri::command]
pub fn hr_person_get(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<PersonView, UiError> {
    guard(&session, "hr_person_get")?;
    let conn = lock(&db)?;
    Ok(svc::person(&conn, &id)?)
}

#[tauri::command]
pub fn hr_person_save(session: State<'_, Session>, db: State<'_, Db>, id: Option<String>, data: PersonData) -> Result<PersonOutcome, UiError> {
    guard(&session, "hr_person_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save_person(&mut conn, id.as_deref(), data)?)
}

#[tauri::command]
pub fn hr_person_delete(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<StaffChange, UiError> {
    let gate = guard(&session, "hr_person_delete")?;
    let mut conn = lock(&db)?;
    if !gate.may_delete() {
        crate::access_service::request_deletion(&mut conn, gate.user()?, crate::domain::access::DeletionKind::HrPerson, &id)?;
        return Ok(svc::change(&mut conn)?);
    }
    let change = svc::delete_person(&mut conn, &id)?;
    crate::storage::access::deactivate_for_person(&conn, &id).map_err(crate::service::ServiceError::from)?;
    Ok(change)
}

/// Shows one covered identifier (`curp`, `rfc`, `nss`, `clabe`); the audit log keeps that it was looked at.
#[tauri::command]
pub fn hr_person_reveal(session: State<'_, Session>, db: State<'_, Db>, id: String, field: String) -> Result<String, UiError> {
    guard(&session, "hr_person_reveal")?;
    let conn = lock(&db)?;
    Ok(svc::reveal(&conn, &id, &field)?)
}

#[tauri::command]
pub fn hr_position_save(session: State<'_, Session>, db: State<'_, Db>, id: Option<String>, input: PositionInput, decision: Option<Decision>) -> Result<PositionOutcome, UiError> {
    guard(&session, "hr_position_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save_position(&mut conn, id.as_deref(), input, decision)?)
}

#[tauri::command]
pub fn hr_position_set_active(session: State<'_, Session>, db: State<'_, Db>, id: String, active: bool) -> Result<StaffChange, UiError> {
    guard(&session, "hr_position_set_active")?;
    let mut conn = lock(&db)?;
    Ok(svc::set_position_active(&mut conn, &id, active)?)
}

#[tauri::command]
pub fn hr_modality_create(session: State<'_, Session>, db: State<'_, Db>, title: String, behaves_as: String) -> Result<Vec<ModalityInfo>, UiError> {
    guard(&session, "hr_modality_create")?;
    let conn = lock(&db)?;
    Ok(svc::create_modality(&conn, &title, &behaves_as)?)
}

#[tauri::command]
pub fn hr_field_save(session: State<'_, Session>, db: State<'_, Db>, key: Option<String>, title: String, kind: String, options: Vec<String>) -> Result<Vec<CustomField>, UiError> {
    guard(&session, "hr_field_save")?;
    let conn = lock(&db)?;
    Ok(svc::save_field(&conn, key.as_deref(), &title, &kind, &options)?)
}

#[tauri::command]
pub fn hr_field_delete(session: State<'_, Session>, db: State<'_, Db>, key: String) -> Result<Vec<CustomField>, UiError> {
    let gate = guard(&session, "hr_field_delete")?;
    let mut conn = lock(&db)?;
    if !gate.may_delete() {
        crate::access_service::request_deletion(&mut conn, gate.user()?, crate::domain::access::DeletionKind::HrField, &key)?;
        return Ok(crate::modules::hr::storage::custom_fields(&conn).map_err(crate::service::ServiceError::from)?);
    }
    Ok(svc::delete_field(&conn, &key)?)
}
