//! Thin Tauri command layer: validate input, call the service, map errors.

pub mod access;
pub mod calls;
pub mod care;
pub mod diagnosis;
pub mod drafting;
pub mod hr;
pub mod security;

use crate::access_service::{CurrentUser, Session};
use crate::domain::access::{need_of, Permission};
use crate::domain::profile::ProfileInput;
use crate::error::access_message;
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
pub fn app_info(session: State<'_, Session>) -> AppInfo {
    let _ = guard(&session, "app_info"); // open: the name and version of the app
    AppInfo { name: "Cimiento", version: env!("CARGO_PKG_VERSION") }
}

/// What a command may do once it passed its check: who is acting, and whether they may delete for good.
pub struct Gate {
    pub user: Option<CurrentUser>,
}

impl Gate {
    /// Whether a deletion runs now (the person may delete) or becomes a request for the administrator.
    pub fn may_delete(&self) -> bool {
        self.user.as_ref().is_some_and(|u| u.role.can(Permission::Delete))
    }
    pub fn user(&self) -> Result<&CurrentUser, UiError> {
        self.user.as_ref().ok_or_else(|| UiError::new("not_signed_in", access_message("not_signed_in")))
    }
}

/// The check every command makes first (ADR-028): a session that is open and not locked, and the permission the
/// command needs (`domain::access::COMMANDS`). A refusal is written in the audit log.
pub fn guard(session: &State<'_, Session>, command: &str) -> Result<Gate, UiError> {
    let need = need_of(command).ok_or_else(UiError::internal)?;
    let (user, locked) = {
        let state = session.state.lock().map_err(|_| UiError::internal())?;
        (state.user.clone(), state.locked)
    };
    match crate::access_service::check(user, locked, need) {
        Ok(user) => Ok(Gate { user }),
        Err(code) => {
            if code == "access_denied" {
                if let Some(db) = &session.db {
                    if let Ok(conn) = db.lock() {
                        let _ = crate::audit::record(&conn, crate::audit::AuditKind::AccessDenied, None, None, serde_json::json!({ "command": command }));
                    }
                }
            }
            Err(UiError::new(code, access_message(code)))
        }
    }
}

fn lock<'a>(db: &'a State<'a, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn profile_get(session: State<'_, Session>, db: State<Db>) -> Result<Option<ProfileView>, UiError> {
    guard(&session, "profile_get")?;
    let conn = lock(&db)?;
    Ok(service::get_profile(&conn)?)
}

#[tauri::command]
pub fn profile_save(session: State<'_, Session>, db: State<Db>,
    input: ProfileInput,
    decision: Option<Decision>,
) -> Result<SaveProfileOutcome, UiError> {
    guard(&session, "profile_save")?;
    let mut conn = lock(&db)?;
    Ok(service::save_profile(&mut conn, input, decision)?)
}

#[tauri::command]
pub fn profile_confirm(session: State<'_, Session>, db: State<Db>) -> Result<ProfileView, UiError> {
    guard(&session, "profile_confirm")?;
    let mut conn = lock(&db)?;
    Ok(service::confirm_profile(&mut conn)?)
}

/// Adds a document of the institution (global, never tied to a project). The files of a call come in with
/// the project born from it, so the kind is not asked here.
#[tauri::command]
pub fn document_add_text(session: State<'_, Session>, db: State<Db>,
    display_name: String,
    text: String,
    decision: Option<Decision>,
) -> Result<AddDocumentOutcome, UiError> {
    guard(&session, "document_add_text")?;
    let mut conn = lock(&db)?;
    Ok(service::add_text_document(&mut conn, "internal", &display_name, &text, decision)?)
}

/// The documents of the institution: those of a call belong to their project and are not listed here.
#[tauri::command]
pub fn documents_list(session: State<'_, Session>, db: State<Db>) -> Result<Vec<DocumentSummary>, UiError> {
    guard(&session, "documents_list")?;
    let conn = lock(&db)?;
    docs::list_institution(&conn).map_err(|e| UiError::from(service::ServiceError::from(e)))
}

#[tauri::command]
pub fn document_emergency_delete(session: State<'_, Session>, db: State<Db>, id: String) -> Result<DeleteSummary, UiError> {
    let gate = guard(&session, "document_emergency_delete")?;
    let mut conn = lock(&db)?;
    if !gate.may_delete() {
        // it disappears now; the administrator deletes it for good or brings it back (ADR-028)
        crate::access_service::request_deletion(&mut conn, gate.user()?, crate::domain::access::DeletionKind::Document, &id)?;
        return Ok(DeleteSummary { chunks: 0, derived_rows: 0 });
    }
    docs::emergency_delete_document(&mut conn, &id).map_err(|e| UiError::from(service::ServiceError::from(e)))
}

/// Development only: loads a fictitious profile from `fixtures/`.
#[tauri::command]
pub fn dev_load_fixture(session: State<'_, Session>, db: State<Db>, name: String) -> Result<ProfileView, UiError> {
    guard(&session, "dev_load_fixture")?;
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
        crate::profile_sync::seed_examples(&mut conn, padron)?;
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
