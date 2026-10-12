//! Commands of the record of the institution (ADR-033 §5, N4): thin, they call `core::records`.

use super::guard;
use crate::common::forms::Values;
use crate::core::access::service::Session;
use crate::core::records::{self as svc, AttachOutcome, Expediente, RecordForm, RecordOutcome};
use crate::error::UiError;
use crate::scanner::guard::Decision;
use crate::Db;
use std::path::Path;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

/// The documents the institution must have, with their state and validity.
#[tauri::command]
pub fn institution_records(session: State<'_, Session>, db: State<'_, Db>) -> Result<Expediente, UiError> {
    guard(&session, "institution_records")?;
    let conn = lock(&db)?;
    Ok(svc::expediente(&conn)?)
}

/// The window of a document: a type of the catalog (`kind`), or an `other` by its id (none to add one).
#[tauri::command]
pub fn institution_record_form(session: State<'_, Session>, db: State<'_, Db>, kind: String, id: Option<String>) -> Result<RecordForm, UiError> {
    guard(&session, "institution_record_form")?;
    let conn = lock(&db)?;
    Ok(svc::form(&conn, &kind, id.as_deref())?)
}

#[tauri::command]
pub fn institution_record_save(
    session: State<'_, Session>,
    db: State<'_, Db>,
    kind: String,
    id: Option<String>,
    values: Values,
    decision: Option<Decision>,
) -> Result<RecordOutcome, UiError> {
    guard(&session, "institution_record_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save(&mut conn, &kind, id.as_deref(), values, decision)?)
}

/// Puts a file on a document, read from where it is on this computer (`path`, chosen by the person). A scan or a
/// photo is read as pictures, a few seconds a page: the file is read without holding the database.
#[tauri::command]
pub async fn institution_record_attach(
    session: State<'_, Session>,
    db: State<'_, Db>,
    kind: String,
    id: Option<String>,
    path: String,
    decision: Option<Decision>,
) -> Result<AttachOutcome, UiError> {
    guard(&session, "institution_record_attach")?;
    let shared = db.0.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let file = match svc::read_original(Path::new(&path)) {
            Ok(file) => file,
            Err(reason) => return Ok(AttachOutcome::Unreadable { reason }),
        };
        let mut conn = shared.lock().map_err(|_| UiError::internal())?;
        Ok::<_, UiError>(svc::attach_read(&mut conn, &kind, id.as_deref(), Path::new(&path), file, decision)?)
    })
    .await
    .map_err(|_| UiError::internal())??;
    super::index_documents(&db.0);
    Ok(outcome)
}

#[tauri::command]
pub fn institution_record_detach(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<Expediente, UiError> {
    guard(&session, "institution_record_detach")?;
    let mut conn = lock(&db)?;
    Ok(svc::detach(&mut conn, &id)?)
}

/// Opens the original of a document with the program the computer uses for it. `false` when it is no longer where
/// it was.
#[tauri::command]
pub fn institution_record_open(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<bool, UiError> {
    guard(&session, "institution_record_open")?;
    let path = {
        let conn = lock(&db)?;
        svc::original(&conn, &id)?
    };
    let Some(path) = path else { return Ok(false) };
    open_with_system(&path).map_err(|_| UiError::internal())?;
    Ok(true)
}

/// Hands a file to the program the system opens it with.
fn open_with_system(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    let program = "explorer";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let program = "xdg-open";
    std::process::Command::new(program).arg(path).spawn().map(|_| ())
}
