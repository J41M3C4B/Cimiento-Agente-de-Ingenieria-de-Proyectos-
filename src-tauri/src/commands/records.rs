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

/// How a reading in the background started.
#[derive(Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AttachStart {
    /// It is being read; `record-read` says how it ended, with `key`.
    Reading { key: String },
    /// That document is already being read.
    AlreadyReading { key: String },
}

/// Puts a file on a document, read from where it is on this computer (`path`, chosen by the person). The file is
/// read in the background (a scan or a photo takes a few seconds a page, an act several minutes) without holding the
/// database: this answers at once, and the event `record-read` says how it ended, the outcome or the error, so
/// nothing ends in silence, even with the window closed (ADR-038). The progress comes with its `key`.
#[tauri::command]
pub async fn institution_record_attach(
    app: tauri::AppHandle,
    session: State<'_, Session>,
    db: State<'_, Db>,
    kind: String,
    id: Option<String>,
    path: String,
) -> Result<AttachStart, UiError> {
    guard(&session, "institution_record_attach")?;
    let key = svc::background::key(&kind, id.as_deref());
    let Some(reading) = svc::background::start(&key) else { return Ok(AttachStart::AlreadyReading { key }) };
    let shared = db.0.clone();
    let started = key.clone();
    std::thread::spawn(move || {
        let _reading = reading;
        let ended: Result<AttachOutcome, UiError> = (|| {
            let file = match crate::documents::reading::labeled(&key, || svc::read_original(Path::new(&path))) {
                Ok(file) => file,
                Err(reason) => return Ok(AttachOutcome::Unreadable { reason }),
            };
            let mut conn = shared.lock().map_err(|_| UiError::internal())?;
            Ok(svc::attach_read(&mut conn, &kind, id.as_deref(), Path::new(&path), file)?)
        })();
        let said = match &ended {
            Ok(outcome) => serde_json::json!({ "key": key, "kind": kind, "outcome": outcome }),
            Err(error) => serde_json::json!({ "key": key, "kind": kind, "error": error }),
        };
        let _ = tauri::Emitter::emit(&app, "record-read", said);
        if let Ok(outcome) = &ended {
            suggest_from(&app, &shared, &key, &kind, outcome);
        }
        super::index_documents(&shared);
    });
    Ok(AttachStart::Reading { key: started })
}

/// How an answer about a waiting file started (ADR-042 §4).
#[derive(Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PlaceStart {
    /// It is being kept on the document of `key` (where it was put, or the one its words say); `record-read` says how
    /// it ended, with that key.
    Reading { key: String, kind: String },
    /// That document is being read; the file still waits.
    AlreadyReading { key: String },
    /// Nothing was kept.
    Cancelled,
}

/// What the person answers about a file the code was not sure of (`key`: the document it was put on): keep it there,
/// on the document its words say, or not at all. The file is not read again; keeping it (scanner, rules) runs in the
/// background like a reading, and `record-read` says how it ended.
#[tauri::command]
pub async fn institution_record_place(
    app: tauri::AppHandle,
    session: State<'_, Session>,
    db: State<'_, Db>,
    key: String,
    answer: svc::placing::Answer,
) -> Result<PlaceStart, UiError> {
    guard(&session, "institution_record_place")?;
    let waiting = svc::placing::take(&key).ok_or(crate::core::error::ServiceError::NotFound)?;
    let Some((kind, target)) = waiting.target(answer) else { return Ok(PlaceStart::Cancelled) };
    let Some(reading) = svc::background::start(&target) else {
        svc::placing::put_back(&key, waiting);
        return Ok(PlaceStart::AlreadyReading { key: target });
    };
    let shared = db.0.clone();
    let started = target.clone();
    let ended_kind = kind.clone();
    std::thread::spawn(move || {
        let _reading = reading;
        let ended: Result<Option<AttachOutcome>, UiError> = (|| {
            let mut conn = shared.lock().map_err(|_| UiError::internal())?;
            Ok(svc::placing::keep_as(&mut conn, waiting, answer)?)
        })();
        let said = match &ended {
            Ok(outcome) => serde_json::json!({ "key": target, "kind": kind, "outcome": outcome }),
            Err(error) => serde_json::json!({ "key": target, "kind": kind, "error": error }),
        };
        let _ = tauri::Emitter::emit(&app, "record-read", said);
        if let Ok(Some(outcome)) = &ended {
            suggest_from(&app, &shared, &target, &kind, outcome);
        }
        super::index_documents(&shared);
    });
    Ok(PlaceStart::Reading { key: started, kind: ended_kind })
}

/// After a file is kept with its text, what it says of the empty fields of «Mi institución» (ADR-042 §5): in the
/// background, one call at most, and the event `record-suggested` says what it left, with the key of the document.
fn suggest_from(app: &tauri::AppHandle, db: &crate::storage::SharedDb, key: &str, kind: &str, outcome: &AttachOutcome) {
    let AttachOutcome::Attached { id, readable: true, .. } = outcome else { return };
    let document = match db.lock() {
        Ok(conn) => svc::document_of(&conn, id).ok().flatten(),
        Err(_) => None,
    };
    let Some(document) = document else { return };
    let (app, db, key, kind) = (app.clone(), db.clone(), key.to_string(), kind.to_string());
    tauri::async_runtime::spawn(async move {
        let provider = super::diagnosis::make_provider(&db);
        let said = match crate::core::reader::upload::after_upload(&db, provider.as_deref(), &document).await {
            Ok(outcome) => serde_json::json!({ "key": key, "kind": kind, "outcome": outcome }),
            Err(error) => serde_json::json!({ "key": key, "kind": kind, "error": UiError::from(error) }),
        };
        let _ = tauri::Emitter::emit(&app, "record-suggested", said);
    });
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
