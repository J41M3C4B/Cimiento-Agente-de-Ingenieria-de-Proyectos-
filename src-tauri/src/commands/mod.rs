//! Thin Tauri command layer: validate input, call the service, map errors.

pub mod calls;
pub mod diagnosis;
pub mod drafting;
pub mod roster;
pub mod security;

use crate::domain::profile::ProfileInput;
use crate::error::UiError;
use crate::scanner::guard::Decision;
use crate::service::{self, AddDocumentOutcome, ProfileView, SaveProfileOutcome};
use crate::storage::documents::{self as docs, DeleteSummary, DocumentSummary};
use crate::Db;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo { name: "Cimiento", version: env!("CARGO_PKG_VERSION") }
}

fn lock<'a>(db: &'a State<'a, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn profile_get(db: State<Db>) -> Result<Option<ProfileView>, UiError> {
    let conn = lock(&db)?;
    Ok(service::get_profile(&conn)?)
}

#[tauri::command]
pub fn profile_save(
    db: State<Db>,
    input: ProfileInput,
    decision: Option<Decision>,
) -> Result<SaveProfileOutcome, UiError> {
    let mut conn = lock(&db)?;
    Ok(service::save_profile(&mut conn, input, decision)?)
}

#[tauri::command]
pub fn profile_confirm(db: State<Db>) -> Result<ProfileView, UiError> {
    let mut conn = lock(&db)?;
    Ok(service::confirm_profile(&mut conn)?)
}

/// Adds a document of the institution (global, never tied to a project). The files of a call come in with
/// the project born from it, so the kind is not asked here.
#[tauri::command]
pub fn document_add_text(
    db: State<Db>,
    display_name: String,
    text: String,
    decision: Option<Decision>,
) -> Result<AddDocumentOutcome, UiError> {
    let mut conn = lock(&db)?;
    Ok(service::add_text_document(&mut conn, "internal", &display_name, &text, decision)?)
}

/// The documents of the institution: those of a call belong to their project and are not listed here.
#[tauri::command]
pub fn documents_list(db: State<Db>) -> Result<Vec<DocumentSummary>, UiError> {
    let conn = lock(&db)?;
    docs::list_institution(&conn).map_err(|e| UiError::from(service::ServiceError::from(e)))
}

#[tauri::command]
pub fn document_emergency_delete(db: State<Db>, id: String) -> Result<DeleteSummary, UiError> {
    let mut conn = lock(&db)?;
    docs::emergency_delete_document(&mut conn, &id).map_err(|e| UiError::from(service::ServiceError::from(e)))
}

/// Development only: loads a fictitious profile from `fixtures/`.
#[tauri::command]
pub fn dev_load_fixture(db: State<Db>, name: String) -> Result<ProfileView, UiError> {
    #[cfg(debug_assertions)]
    {
        let (raw, padron) = match name.as_str() {
            "asilo" => (include_str!("../../../fixtures/institucion-asilo.json"), include_str!("../../../fixtures/padron-asilo.json")),
            "casa-hogar" => (include_str!("../../../fixtures/institucion-casa-hogar.json"), include_str!("../../../fixtures/padron-casa-hogar.json")),
            _ => return Err(UiError::new("unknown_fixture", "No existe ese perfil de ejemplo.")),
        };
        let input: ProfileInput = serde_json::from_str(raw).map_err(|_| UiError::internal())?;
        let mut conn = lock(&db)?;
        // the institution first, so the forms of the roster start with its kind; then the people of the example,
        // and the profile adds them up again
        service::save_profile(&mut conn, input.clone(), None)?;
        crate::roster_service::seed_roster(&mut conn, padron)?;
        return match service::save_profile(&mut conn, input, None)? {
            SaveProfileOutcome::Saved { profile } => Ok(profile),
            _ => Err(UiError::internal()),
        };
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = (db, name);
        Err(UiError::new("dev_only", "Esta opción solo existe en la versión de desarrollo."))
    }
}
