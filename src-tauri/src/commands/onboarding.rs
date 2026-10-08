//! Commands of the first start (ADR-031): thin, they call `core::onboarding`.

use super::guard;
use crate::core::access::service::Session;
use crate::error::UiError;
use crate::core::onboarding::service::{self as svc, OnboardingData, OnboardingOutcome, OnboardingStatus};
use crate::scanner::guard::Decision;
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

/// Where the institution and this person are in the first start.
#[tauri::command]
pub fn onboarding_status(session: State<'_, Session>, db: State<'_, Db>) -> Result<OnboardingStatus, UiError> {
    let gate = guard(&session, "onboarding_status")?;
    let conn = lock(&db)?;
    Ok(svc::status(&conn, gate.user()?)?)
}

#[tauri::command]
pub fn onboarding_save(session: State<'_, Session>, db: State<'_, Db>, data: OnboardingData, decision: Option<Decision>) -> Result<OnboardingOutcome, UiError> {
    let gate = guard(&session, "onboarding_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save(&mut conn, gate.user()?, data, decision)?)
}

#[tauri::command]
pub fn onboarding_finish(session: State<'_, Session>, db: State<'_, Db>) -> Result<OnboardingStatus, UiError> {
    let gate = guard(&session, "onboarding_finish")?;
    let mut conn = lock(&db)?;
    Ok(svc::finish(&mut conn, gate.user()?)?)
}

/// The person saw the welcome.
#[tauri::command]
pub fn onboarding_welcome_done(session: State<'_, Session>, db: State<'_, Db>) -> Result<(), UiError> {
    let gate = guard(&session, "onboarding_welcome_done")?;
    let conn = lock(&db)?;
    Ok(svc::welcome_done(&conn, gate.user()?)?)
}
