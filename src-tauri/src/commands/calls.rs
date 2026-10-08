//! Commands of the calls (convocatorias). Thin: decode what the screen sends, call `call_service`.
//! A call belongs to the project born from it. The reading runs in the background and the screen asks how it is going.

use super::guard;
use crate::access_service::Session;
use crate::call_service::{self as svc, NewProjectOutcome, PackageFile, ReadingDetail, UploadedFile};
use crate::commands::diagnosis::{as_dyn, make_provider};
use crate::error::UiError;
use crate::jobs::{JobKind, Jobs};
use crate::scanner::guard::Decision;
use crate::storage::calls::FileRole;
use crate::Db;
use base64::Engine;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct UploadFile {
    pub name: String,
    /// The bytes of the file, in base64.
    pub data: String,
    /// What the person says the file is inside the package of the call.
    pub role: FileRole,
}

/// Creates a project from its call: saves the files, creates the project and starts reading the call
/// without making the person wait.
#[tauri::command]
pub async fn project_create_from_call(session: State<'_, Session>, db: State<'_, Db>,
    files: Vec<UploadFile>,
    name: String,
    funder: Option<String>,
    year: Option<i64>,
    decision: Option<Decision>,
) -> Result<NewProjectOutcome, UiError> {
    guard(&session, "project_create_from_call")?;
    let shared = db.0.clone();
    let mut decoded = Vec::with_capacity(files.len());
    for f in files {
        let bytes = base64::engine::general_purpose::STANDARD.decode(f.data.as_bytes()).map_err(|_| UiError::internal())?;
        decoded.push(PackageFile { file: UploadedFile { name: f.name, bytes }, role: f.role });
    }
    let create_db = shared.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || svc::create_project_from_call(&create_db, decoded, &name, funder.as_deref(), year, decision))
        .await
        .map_err(|_| UiError::internal())??;
    if let NewProjectOutcome::Created { reading, .. } = &outcome {
        start(shared, reading.id.clone());
    }
    Ok(outcome)
}

/// Reads the call in the background: the screen does not wait for it.
fn start(db: crate::diagnosis_service::SharedDb, id: String) {
    tauri::async_runtime::spawn(async move {
        let plan = svc::reading_plan(&db);
        svc::read_call(&db, &plan, &id).await;
    });
}

/// The person confirms that the call is the right one. Returns whether it could be confirmed.
#[tauri::command]
pub fn call_reading_confirm(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<bool, UiError> {
    guard(&session, "call_reading_confirm")?;
    Ok(svc::confirm(&db.0, &id)?)
}

#[tauri::command]
pub fn call_reading_get(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<ReadingDetail, UiError> {
    guard(&session, "call_reading_get")?;
    Ok(svc::detail(&db.0, &id)?)
}

/// Writes the «en pocas palabras» of a read call, once. The slot is the call's (`call:<id>`), so a second request
/// while the first is running is refused instead of paying for another call.
#[tauri::command]
pub async fn call_brief_make(session: State<'_, Session>, db: State<'_, Db>, jobs: State<'_, Jobs>, id: String) -> Result<ReadingDetail, UiError> {
    guard(&session, "call_brief_make")?;
    let job = jobs.claim(&format!("call:{id}"), JobKind::Brief)?;
    let shared = db.0.clone();
    let provider = make_provider(&shared);
    let (detail, ai) = svc::make_brief(&shared, as_dyn(&provider), &id).await?;
    job.finish(ai);
    Ok(detail)
}

/// Reads again a call that is waiting, partial or failed. Returns whether a new reading started.
#[tauri::command]
pub fn call_reading_retry(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<bool, UiError> {
    guard(&session, "call_reading_retry")?;
    let shared = db.0.clone();
    let started = svc::prepare_retry(&shared, &id)?;
    if started {
        start(shared, id);
    }
    Ok(started)
}
