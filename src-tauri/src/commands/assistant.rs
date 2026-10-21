//! The commands of the Assistant (ADR-034 §4, IA3) and the Capturist (IA4): thin, they call `core::agents`.

use super::ai::make_provider;
use super::shared;
use super::guard;
use crate::core::access::service::Session;
use crate::core::agents::assistant::{self, Ask, AssistantOutcome};
use crate::core::agents::capturist::{self, Capture, CaptureOutcome};
use crate::core::reader::fill::{self, FillOutcome};
use crate::error::UiError;
use crate::Db;
use tauri::State;

/// A question from any screen. The tools give only what the person who asks may see (ADR-028).
#[tauri::command]
pub async fn assistant_ask(session: State<'_, Session>, db: State<'_, Db>, ask: Ask) -> Result<AssistantOutcome, UiError> {
    let gate = guard(&session, "assistant_ask")?;
    let role = gate.user.as_ref().map(|u| u.role);
    let db = shared(&db);
    let provider = make_provider(&db);
    let embedder = crate::core::archive::vectors::embedder(&db, true);
    Ok(assistant::ask(&db, provider.as_deref(), embedder, role, ask).await?)
}

/// «Llenarlo por mí»: what the person tells becomes suggestions in a window of «Mi institución». Nothing is saved.
#[tauri::command]
pub async fn capture_ask(session: State<'_, Session>, db: State<'_, Db>, capture: Capture) -> Result<CaptureOutcome, UiError> {
    guard(&session, "capture_ask")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    Ok(capturist::capture(&db, provider.as_deref(), capture).await?)
}

/// «Llenar con mis documentos» (C2): the empty fields of a window of «Mi institución» as suggestions from the
/// documents of the Expediente, each with its document and page. Nothing is saved.
#[tauri::command]
pub async fn institution_fill_from_documents(session: State<'_, Session>, db: State<'_, Db>, form: String) -> Result<FillOutcome, UiError> {
    guard(&session, "institution_fill_from_documents")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let embedder = crate::core::archive::vectors::embedder(&db, true);
    Ok(fill::fill_window(&db, provider.as_deref(), embedder, &form).await?)
}

/// «Llenar todo con mis documentos» (C2): the four windows, one call each.
#[tauri::command]
pub async fn institution_fill_all_from_documents(session: State<'_, Session>, db: State<'_, Db>) -> Result<Vec<FillOutcome>, UiError> {
    guard(&session, "institution_fill_all_from_documents")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    let embedder = crate::core::archive::vectors::embedder(&db, true);
    Ok(fill::fill_all(&db, provider.as_deref(), embedder).await?)
}
