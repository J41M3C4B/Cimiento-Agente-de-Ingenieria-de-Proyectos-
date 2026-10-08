//! Commands of the roster (ADR-020): thin, they call `roster_service`.

use super::guard;
use crate::access_service::Session;
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
pub fn roster_overview(session: State<'_, Session>, db: State<'_, Db>, entity: Entity) -> Result<Overview, UiError> {
    guard(&session, "roster_overview")?;
    let conn = lock(&db)?;
    Ok(svc::overview(&conn, entity)?)
}

#[tauri::command]
pub fn roster_field_save(session: State<'_, Session>, db: State<'_, Db>, entity: Entity, field: FieldInput) -> Result<Vec<RosterField>, UiError> {
    guard(&session, "roster_field_save")?;
    let conn = lock(&db)?;
    Ok(svc::save_field(&conn, entity, &field)?)
}

#[tauri::command]
pub fn roster_field_delete(session: State<'_, Session>, db: State<'_, Db>, entity: Entity, key: String) -> Result<Vec<RosterField>, UiError> {
    let gate = guard(&session, "roster_field_delete")?;
    let mut conn = lock(&db)?;
    if !gate.may_delete() && entity == Entity::Beneficiary {
        crate::access_service::request_deletion(&mut conn, gate.user()?, crate::domain::access::DeletionKind::RosterField, &key)?;
        return Ok(crate::storage::roster::fields(&conn, entity).map_err(crate::service::ServiceError::from)?);
    }
    Ok(svc::delete_field(&conn, entity, &key)?)
}

#[tauri::command]
pub fn roster_entry_save(session: State<'_, Session>, db: State<'_, Db>, entity: Entity, id: Option<String>, data: Data) -> Result<Change, UiError> {
    guard(&session, "roster_entry_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save_entry(&mut conn, entity, id.as_deref(), &data)?)
}

#[tauri::command]
pub fn roster_entry_delete(session: State<'_, Session>, db: State<'_, Db>, entity: Entity, id: String) -> Result<Change, UiError> {
    let gate = guard(&session, "roster_entry_delete")?;
    let mut conn = lock(&db)?;
    if !gate.may_delete() && entity == Entity::Beneficiary {
        crate::access_service::request_deletion(&mut conn, gate.user()?, crate::domain::access::DeletionKind::Beneficiary, &id)?;
        return Ok(svc::change(&mut conn, entity)?);
    }
    Ok(svc::delete_entry(&mut conn, entity, &id)?)
}
