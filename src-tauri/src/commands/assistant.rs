//! The commands of the Assistant (ADR-034 §4, IA3) and the Capturist (IA4): thin, they call `core::agents`.

use super::diagnosis::{make_provider, shared};
use super::guard;
use crate::core::access::service::Session;
use crate::core::agents::assistant::{self, Ask, AssistantOutcome};
use crate::core::agents::capturist::{self, Capture, CaptureOutcome};
use crate::error::UiError;
use crate::scanner::guard::Decision;
use crate::Db;
use tauri::State;

/// A question from any screen. The tools give only what the person who asks may see (ADR-028).
#[tauri::command]
pub async fn assistant_ask(session: State<'_, Session>, db: State<'_, Db>, ask: Ask, decision: Option<Decision>) -> Result<AssistantOutcome, UiError> {
    let gate = guard(&session, "assistant_ask")?;
    let role = gate.user.as_ref().map(|u| u.role);
    let db = shared(&db);
    let provider = make_provider(&db);
    let embedder = crate::core::archive::vectors::embedder(&db, true);
    Ok(assistant::ask(&db, provider.as_deref(), embedder, role, ask, decision).await?)
}

/// «Llenarlo por mí»: what the person tells becomes suggestions in a window of «Mi institución». Nothing is saved.
#[tauri::command]
pub async fn capture_ask(session: State<'_, Session>, db: State<'_, Db>, capture: Capture, decision: Option<Decision>) -> Result<CaptureOutcome, UiError> {
    guard(&session, "capture_ask")?;
    let db = shared(&db);
    let provider = make_provider(&db);
    Ok(capturist::capture(&db, provider.as_deref(), capture, decision).await?)
}

/// The person discards suggestions of the AI: nothing of the institution changes.
#[tauri::command]
pub fn proposals_reject(session: State<'_, Session>, db: State<'_, Db>, ids: Vec<String>) -> Result<usize, UiError> {
    let gate = guard(&session, "proposals_reject")?;
    let by = gate.user.as_ref().map(|u| u.id.clone());
    let conn = super::lock(&db)?;
    Ok(crate::core::agents::proposals::reject(&conn, &ids, by.as_deref())?)
}
