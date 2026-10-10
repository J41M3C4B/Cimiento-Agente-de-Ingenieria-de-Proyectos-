//! The command of the Assistant (ADR-034 §4, IA3): thin, it calls `core::agents::assistant`.

use super::diagnosis::{make_provider, shared};
use super::guard;
use crate::core::access::service::Session;
use crate::core::agents::assistant::{self, Ask, AssistantOutcome};
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
    Ok(assistant::ask(&db, provider.as_deref(), role, ask, decision).await?)
}
