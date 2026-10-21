//! Commands of the first start (ADR-031, ADR-042): thin, they call `core::onboarding`.

use super::guard;
use crate::core::access::service::Session;
use crate::error::UiError;
use crate::common::forms::Values;
use crate::core::onboarding::service::{self as svc, OnboardingOutcome, OnboardingStatus};
use rusqlite::Connection;
use crate::scanner::guard::Decision;
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

/// Whether the AI has its key: a setting of the platform, which the core of «Mi institución» does not know. The
/// administrator's setup says it; nobody else sees it.
fn with_ai_ready(mut status: OnboardingStatus, conn: &Connection) -> Result<OnboardingStatus, UiError> {
    if let Some(setup) = status.setup.as_mut() {
        let provider = crate::ai::settings::load(conn).map_err(|_| UiError::internal())?.provider;
        setup.ai_ready = Some(crate::storage::get_api_key(provider.as_str()).ok().flatten().is_some());
    }
    Ok(status)
}

/// Where the institution and this person are in the first start.
#[tauri::command]
pub fn onboarding_status(session: State<'_, Session>, db: State<'_, Db>) -> Result<OnboardingStatus, UiError> {
    let gate = guard(&session, "onboarding_status")?;
    let conn = lock(&db)?;
    with_ai_ready(svc::status(&conn, gate.user()?)?, &conn)
}

/// Saves one step: its fields, through their window of «Mi institución».
#[tauri::command]
pub fn onboarding_save(session: State<'_, Session>, db: State<'_, Db>, step: String, values: Values, decision: Option<Decision>) -> Result<OnboardingOutcome, UiError> {
    let gate = guard(&session, "onboarding_save")?;
    let mut conn = lock(&db)?;
    Ok(match svc::save(&mut conn, gate.user()?, &step, values, decision)? {
        OnboardingOutcome::Saved { onboarding } => OnboardingOutcome::Saved { onboarding: with_ai_ready(onboarding, &conn)? },
        other => other,
    })
}

#[tauri::command]
pub fn onboarding_finish(session: State<'_, Session>, db: State<'_, Db>) -> Result<OnboardingStatus, UiError> {
    let gate = guard(&session, "onboarding_finish")?;
    let mut conn = lock(&db)?;
    let status = svc::finish(&mut conn, gate.user()?)?;
    with_ai_ready(status, &conn)
}

/// The person saw the welcome.
#[tauri::command]
pub fn onboarding_welcome_done(session: State<'_, Session>, db: State<'_, Db>) -> Result<(), UiError> {
    let gate = guard(&session, "onboarding_welcome_done")?;
    let conn = lock(&db)?;
    Ok(svc::welcome_done(&conn, gate.user()?)?)
}
