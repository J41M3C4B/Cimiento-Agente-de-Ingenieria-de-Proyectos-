//! Commands of the facilities (ADR-030): thin, they call `core::bridge::facilities`.

use super::guard;
use crate::core::access::service::Session;
use crate::error::UiError;
use crate::modules::facilities::domain::group::{EquipmentData, SpaceData};
use crate::modules::facilities::domain::site::SiteData;
use crate::core::bridge::facilities::{self as svc, FacilitiesOutcome, FacilitiesOverview};
use crate::scanner::guard::Decision;
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn facilities_overview(session: State<'_, Session>, db: State<'_, Db>) -> Result<FacilitiesOverview, UiError> {
    guard(&session, "facilities_overview")?;
    let conn = lock(&db)?;
    Ok(svc::overview(&conn)?)
}

#[tauri::command]
pub fn facilities_site_save(session: State<'_, Session>, db: State<'_, Db>, data: SiteData, decision: Option<Decision>) -> Result<FacilitiesOutcome, UiError> {
    guard(&session, "facilities_site_save")?;
    let conn = lock(&db)?;
    Ok(svc::save_site(&conn, data, decision)?)
}

#[tauri::command]
pub fn facilities_space_save(session: State<'_, Session>, db: State<'_, Db>, id: Option<String>, data: SpaceData, decision: Option<Decision>) -> Result<FacilitiesOutcome, UiError> {
    guard(&session, "facilities_space_save")?;
    let conn = lock(&db)?;
    Ok(svc::save_space(&conn, id.as_deref(), data, decision)?)
}

/// Removes a group of spaces. It holds no personal datum and is written again in a minute, so it is not one of the
/// deletions that wait for the administrator (ADR-028).
#[tauri::command]
pub fn facilities_space_delete(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<FacilitiesOverview, UiError> {
    guard(&session, "facilities_space_delete")?;
    let conn = lock(&db)?;
    Ok(svc::delete_space(&conn, &id)?)
}

#[tauri::command]
pub fn facilities_equipment_save(session: State<'_, Session>, db: State<'_, Db>, id: Option<String>, data: EquipmentData, decision: Option<Decision>) -> Result<FacilitiesOutcome, UiError> {
    guard(&session, "facilities_equipment_save")?;
    let conn = lock(&db)?;
    Ok(svc::save_equipment(&conn, id.as_deref(), data, decision)?)
}

#[tauri::command]
pub fn facilities_equipment_delete(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<FacilitiesOverview, UiError> {
    guard(&session, "facilities_equipment_delete")?;
    let conn = lock(&db)?;
    Ok(svc::delete_equipment(&conn, &id)?)
}
