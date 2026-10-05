//! Commands of the PIN, the encrypted backup and the scan of the database (ADR-019). Thin: they call
//! `security_service`.

use super::diagnosis::shared;
use crate::error::UiError;
use crate::security_service::{self as svc, Attempts, BackupFile, ScanSummary, Verify};
use crate::Db;
use base64::Engine;
use std::time::Instant;
use tauri::{AppHandle, Manager, State};

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

/// Whether the app asks for a PIN when it opens.
#[tauri::command]
pub fn pin_status(db: State<'_, Db>) -> Result<bool, UiError> {
    let conn = lock(&db)?;
    Ok(svc::pin_enabled(&conn)?)
}

/// Sets the PIN, or changes it (then `current` is the PIN in use).
#[tauri::command]
pub fn pin_set(db: State<'_, Db>, attempts: State<'_, Attempts>, pin: String, current: Option<String>) -> Result<(), UiError> {
    let conn = lock(&db)?;
    Ok(svc::pin_set(&conn, &attempts, &pin, current.as_deref())?)
}

#[tauri::command]
pub fn pin_clear(db: State<'_, Db>, attempts: State<'_, Attempts>, current: String) -> Result<(), UiError> {
    let conn = lock(&db)?;
    Ok(svc::pin_clear(&conn, &attempts, &current)?)
}

#[tauri::command]
pub fn pin_verify(db: State<'_, Db>, attempts: State<'_, Attempts>, pin: String) -> Result<Verify, UiError> {
    let conn = lock(&db)?;
    Ok(svc::verify(&conn, &attempts, &pin, Instant::now())?)
}

/// Looks for data that identifies a person in everything the app keeps. Only counts come back.
#[tauri::command]
pub fn security_scan(db: State<'_, Db>) -> Result<ScanSummary, UiError> {
    let conn = lock(&db)?;
    Ok(svc::scan_database(&conn)?)
}

/// Writes an encrypted copy of everything in the Downloads folder, protected by the password.
#[tauri::command]
pub fn backup_create(app: AppHandle, db: State<'_, Db>, password: String) -> Result<BackupFile, UiError> {
    let paths = app.path();
    let dir = paths.download_dir().or_else(|_| paths.document_dir()).or_else(|_| paths.home_dir()).map_err(|_| UiError::internal())?;
    Ok(svc::create_backup(&shared(&db), &dir, &password)?)
}

/// Replaces everything with what a backup holds. `data` is the file in base64. The current data is kept next to it.
#[tauri::command]
pub fn backup_restore(app: AppHandle, db: State<'_, Db>, data: String, password: String) -> Result<(), UiError> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(data.as_bytes()).map_err(|_| UiError::internal())?;
    let path = app.path().app_data_dir().map_err(|_| UiError::internal())?.join("cimiento.db");
    let key = crate::storage::get_or_create_db_key().map_err(|_| UiError::internal())?;
    svc::restore_backup(&shared(&db), &path, &key, &bytes, &password)?;
    Ok(())
}
