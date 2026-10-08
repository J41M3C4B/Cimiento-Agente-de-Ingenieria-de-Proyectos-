//! Commands of the money (ADR-026, ADR-032): thin, they call `finance_service`.

use super::guard;
use crate::access_service::Session;
use crate::error::UiError;
use crate::finance_service::{self as svc, FinanceOutcome};
use crate::modules::finance::domain::lines::FinanceInput;
use crate::modules::finance::service::FinanceView;
use crate::scanner::guard::Decision;
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn finance_get(session: State<'_, Session>, db: State<'_, Db>) -> Result<FinanceView, UiError> {
    guard(&session, "finance_get")?;
    let conn = lock(&db)?;
    Ok(svc::view(&conn)?)
}

#[tauri::command]
pub fn finance_save(session: State<'_, Session>, db: State<'_, Db>, input: FinanceInput, decision: Option<Decision>) -> Result<FinanceOutcome, UiError> {
    guard(&session, "finance_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save(&mut conn, input, decision)?)
}
