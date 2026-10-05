//! Commands of the roster (ADR-020): thin, they call `roster_service`.

use crate::domain::roster::{Data, Entity, RosterField};
use crate::error::UiError;
use crate::roster_service::{self as svc, Change, Overview};
use crate::storage::roster::FieldInput;
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn roster_overview(db: State<'_, Db>, entity: Entity) -> Result<Overview, UiError> {
    let conn = lock(&db)?;
    Ok(svc::overview(&conn, entity)?)
}

#[tauri::command]
pub fn roster_field_save(db: State<'_, Db>, entity: Entity, field: FieldInput) -> Result<Vec<RosterField>, UiError> {
    let conn = lock(&db)?;
    Ok(svc::save_field(&conn, entity, &field)?)
}

#[tauri::command]
pub fn roster_field_delete(db: State<'_, Db>, entity: Entity, key: String) -> Result<Vec<RosterField>, UiError> {
    let conn = lock(&db)?;
    Ok(svc::delete_field(&conn, entity, &key)?)
}

#[tauri::command]
pub fn roster_entry_save(db: State<'_, Db>, entity: Entity, id: Option<String>, data: Data) -> Result<Change, UiError> {
    let mut conn = lock(&db)?;
    Ok(svc::save_entry(&mut conn, entity, id.as_deref(), &data)?)
}

#[tauri::command]
pub fn roster_entry_delete(db: State<'_, Db>, entity: Entity, id: String) -> Result<Change, UiError> {
    let mut conn = lock(&db)?;
    Ok(svc::delete_entry(&mut conn, entity, &id)?)
}
