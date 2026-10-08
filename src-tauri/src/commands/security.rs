//! Commands of the encrypted backup and the scan of the database (ADR-019). Thin: they call
//! `core::security`.

use super::guard;
use crate::core::access::service::Session;
use super::diagnosis::shared;
use crate::error::UiError;
use crate::core::security::{self as svc, BackupFile, ScanSummary};
use crate::Db;
use base64::Engine;
use tauri::{AppHandle, Manager, State};

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

/// Looks for data that identifies a person in everything the app keeps. Only counts come back.
#[tauri::command]
pub fn security_scan(session: State<'_, Session>, db: State<'_, Db>) -> Result<ScanSummary, UiError> {
    guard(&session, "security_scan")?;
    let conn = lock(&db)?;
    Ok(svc::scan_database(&conn)?)
}

/// Writes an encrypted copy of everything in the Downloads folder, protected by the password.
#[tauri::command]
pub fn backup_create(session: State<'_, Session>, app: AppHandle, db: State<'_, Db>, password: String) -> Result<BackupFile, UiError> {
    guard(&session, "backup_create")?;
    let paths = app.path();
    let dir = paths.download_dir().or_else(|_| paths.document_dir()).or_else(|_| paths.home_dir()).map_err(|_| UiError::internal())?;
    Ok(svc::create_backup(&shared(&db), &dir, &password)?)
}

/// Replaces everything with what a backup holds. `data` is the file in base64. The current data is kept next to it.
#[tauri::command]
pub fn backup_restore(session: State<'_, Session>, app: AppHandle, db: State<'_, Db>, data: String, password: String) -> Result<(), UiError> {
    guard(&session, "backup_restore")?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(data.as_bytes()).map_err(|_| UiError::internal())?;
    let path = app.path().app_data_dir().map_err(|_| UiError::internal())?.join("cimiento.db");
    let key = crate::storage::get_or_create_db_key().map_err(|_| UiError::internal())?;
    svc::restore_backup(&shared(&db), &path, &key, &bytes, &password)?;
    // the restored data has its own accounts: everyone enters again
    session.set(None);
    Ok(())
}
