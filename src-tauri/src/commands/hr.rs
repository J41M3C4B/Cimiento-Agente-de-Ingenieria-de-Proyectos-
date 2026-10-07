//! Commands of the staff module (ADR-027): thin, they call `staff_service`.

use crate::error::UiError;
use crate::hr::domain::person::PersonData;
use crate::hr::domain::position::PositionInput;
use crate::hr::service::{ModalityInfo, PersonView};
use crate::hr::storage::CustomField;
use crate::scanner::guard::Decision;
use crate::staff_service::{self as svc, PersonOutcome, PositionOutcome, StaffChange, StaffOverview};
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn hr_overview(db: State<'_, Db>) -> Result<StaffOverview, UiError> {
    let conn = lock(&db)?;
    Ok(svc::overview(&conn)?)
}

#[tauri::command]
pub fn hr_person_get(db: State<'_, Db>, id: String) -> Result<PersonView, UiError> {
    let conn = lock(&db)?;
    Ok(svc::person(&conn, &id)?)
}

#[tauri::command]
pub fn hr_person_save(db: State<'_, Db>, id: Option<String>, data: PersonData) -> Result<PersonOutcome, UiError> {
    let mut conn = lock(&db)?;
    Ok(svc::save_person(&mut conn, id.as_deref(), data)?)
}

#[tauri::command]
pub fn hr_person_delete(db: State<'_, Db>, id: String) -> Result<StaffChange, UiError> {
    let mut conn = lock(&db)?;
    Ok(svc::delete_person(&mut conn, &id)?)
}

/// Shows one covered identifier (`curp`, `rfc`, `nss`, `clabe`); the audit log keeps that it was looked at.
#[tauri::command]
pub fn hr_person_reveal(db: State<'_, Db>, id: String, field: String) -> Result<String, UiError> {
    let conn = lock(&db)?;
    Ok(svc::reveal(&conn, &id, &field)?)
}

#[tauri::command]
pub fn hr_position_save(db: State<'_, Db>, id: Option<String>, input: PositionInput, decision: Option<Decision>) -> Result<PositionOutcome, UiError> {
    let mut conn = lock(&db)?;
    Ok(svc::save_position(&mut conn, id.as_deref(), input, decision)?)
}

#[tauri::command]
pub fn hr_position_set_active(db: State<'_, Db>, id: String, active: bool) -> Result<StaffChange, UiError> {
    let mut conn = lock(&db)?;
    Ok(svc::set_position_active(&mut conn, &id, active)?)
}

#[tauri::command]
pub fn hr_modality_create(db: State<'_, Db>, title: String, behaves_as: String) -> Result<Vec<ModalityInfo>, UiError> {
    let conn = lock(&db)?;
    Ok(svc::create_modality(&conn, &title, &behaves_as)?)
}

#[tauri::command]
pub fn hr_field_save(db: State<'_, Db>, key: Option<String>, title: String, kind: String, options: Vec<String>) -> Result<Vec<CustomField>, UiError> {
    let conn = lock(&db)?;
    Ok(svc::save_field(&conn, key.as_deref(), &title, &kind, &options)?)
}

#[tauri::command]
pub fn hr_field_delete(db: State<'_, Db>, key: String) -> Result<Vec<CustomField>, UiError> {
    let conn = lock(&db)?;
    Ok(svc::delete_field(&conn, &key)?)
}
