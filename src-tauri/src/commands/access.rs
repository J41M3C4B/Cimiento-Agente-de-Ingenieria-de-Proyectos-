//! Commands of the access profiles (ADR-028): entering, the session, and the administration panel. Thin: they call
//! `core::access`.

use super::guard;
use crate::core::access::service::{self as svc, AccessStatus, AdminOverview, LoginOutcome, NewAccount, Session, SessionView, SetupOutcome, TeamMember};
use crate::core::access::domain::Role;
use crate::error::UiError;
use crate::core::security::Attempts;
use crate::core::error::ServiceError;
use crate::core::access::storage::{self as store, AuditRow};
use crate::Db;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

fn role_of(s: &str) -> Result<Role, UiError> {
    Role::from_db(s).ok_or_else(|| UiError::from(ServiceError::Access("role_not_allowed")))
}

/// Whether the app needs its first account, and who is inside (if anyone).
#[tauri::command]
pub fn access_status(session: State<'_, Session>, db: State<'_, Db>) -> Result<AccessStatus, UiError> {
    guard(&session, "access_status")?;
    let conn = lock(&db)?;
    Ok(svc::status(&conn, &session)?)
}

/// The first account: the administrator. If the computer had a PIN, it is asked here.
#[tauri::command]
pub fn access_setup_admin(
    session: State<'_, Session>,
    db: State<'_, Db>,
    attempts: State<'_, Attempts>,
    display_name: String,
    username: String,
    password: String,
    pin: Option<String>,
) -> Result<SetupOutcome, UiError> {
    guard(&session, "access_setup_admin")?;
    let mut conn = lock(&db)?;
    Ok(svc::setup_admin(&mut conn, &session, &attempts, &display_name, &username, &password, pin.as_deref())?)
}

#[tauri::command]
pub fn access_login(session: State<'_, Session>, db: State<'_, Db>, username: String, password: String) -> Result<LoginOutcome, UiError> {
    guard(&session, "access_login")?;
    let conn = lock(&db)?;
    Ok(svc::login(&conn, &session, &username, &password)?)
}

/// The administrator's way back with the recovery code; a new code comes back, to write it down.
#[tauri::command]
pub fn access_recover(session: State<'_, Session>, db: State<'_, Db>, username: String, code: String, new_password: String) -> Result<SetupOutcome, UiError> {
    guard(&session, "access_recover")?;
    let conn = lock(&db)?;
    Ok(svc::recover(&conn, &session, &username, &code, &new_password)?)
}

/// Opens the locked screen with the password of the same person.
#[tauri::command]
pub fn access_unlock(session: State<'_, Session>, db: State<'_, Db>, password: String) -> Result<LoginOutcome, UiError> {
    guard(&session, "access_unlock")?;
    let conn = lock(&db)?;
    Ok(svc::unlock(&conn, &session, &password)?)
}

/// Locks the screen (the app calls it after a while without use, or with the «Bloquear» button).
#[tauri::command]
pub fn access_lock(session: State<'_, Session>) -> Result<(), UiError> {
    guard(&session, "access_lock")?;
    svc::lock(&session);
    Ok(())
}

#[tauri::command]
pub fn access_logout(session: State<'_, Session>, db: State<'_, Db>) -> Result<(), UiError> {
    guard(&session, "access_logout")?;
    let conn = lock(&db)?;
    Ok(svc::logout(&conn, &session)?)
}

#[tauri::command]
pub fn access_change_password(session: State<'_, Session>, db: State<'_, Db>, current: String, new_password: String) -> Result<SessionView, UiError> {
    guard(&session, "access_change_password")?;
    let conn = lock(&db)?;
    Ok(svc::change_password(&conn, &session, &current, &new_password)?)
}

/// Who has access to the platform: the names Inicio shows with the institution.
#[tauri::command]
pub fn access_team(session: State<'_, Session>, db: State<'_, Db>) -> Result<Vec<TeamMember>, UiError> {
    guard(&session, "access_team")?;
    let conn = lock(&db)?;
    Ok(svc::team(&conn)?)
}

#[tauri::command]
pub fn admin_overview(session: State<'_, Session>, db: State<'_, Db>) -> Result<AdminOverview, UiError> {
    guard(&session, "admin_overview")?;
    let conn = lock(&db)?;
    Ok(svc::admin_overview(&conn)?)
}

/// Gives access to a person of the staff (`person_id`) or to someone outside it (`display_name`).
#[tauri::command]
pub fn admin_user_create(
    session: State<'_, Session>,
    db: State<'_, Db>,
    person_id: Option<String>,
    display_name: Option<String>,
    username: String,
    role: String,
    temporary_password: String,
) -> Result<AdminOverview, UiError> {
    let gate = guard(&session, "admin_user_create")?;
    let conn = lock(&db)?;
    let account = NewAccount {
        person_id: person_id.as_deref(),
        display_name: display_name.as_deref().unwrap_or_default(),
        username: &username,
        role: role_of(&role)?,
        temporary_password: &temporary_password,
    };
    Ok(svc::create_user(&conn, gate.user()?, &account)?)
}

#[tauri::command]
pub fn admin_user_update(session: State<'_, Session>, db: State<'_, Db>, id: String, role: String, active: bool) -> Result<AdminOverview, UiError> {
    let gate = guard(&session, "admin_user_update")?;
    let conn = lock(&db)?;
    Ok(svc::update_user(&conn, gate.user()?, &id, role_of(&role)?, active)?)
}

#[tauri::command]
pub fn admin_user_reset_password(session: State<'_, Session>, db: State<'_, Db>, id: String, temporary_password: String) -> Result<AdminOverview, UiError> {
    guard(&session, "admin_user_reset_password")?;
    let conn = lock(&db)?;
    Ok(svc::reset_password(&conn, &id, &temporary_password)?)
}

/// Approves (deletes for good) or rejects (brings back) a deletion that waited for the administrator.
#[tauri::command]
pub fn admin_request_resolve(session: State<'_, Session>, db: State<'_, Db>, id: String, approve: bool) -> Result<AdminOverview, UiError> {
    let gate = guard(&session, "admin_request_resolve")?;
    let mut conn = lock(&db)?;
    // a project belongs to the projects module: the core is told how to delete it (ADR-032)
    let delete_project = |c: &mut rusqlite::Connection, target: &str| -> Result<(), crate::core::error::ServiceError> {
        crate::modules::projects::storage::projects::delete_project(c, target)?;
        Ok(())
    };
    Ok(svc::resolve_request(&mut conn, gate.user()?, &id, approve, &delete_project)?)
}

/// The latest entries of the audit log; `event` filters by the start of the event name (`auth.`, `hr.`…).
#[tauri::command]
pub fn admin_audit(session: State<'_, Session>, db: State<'_, Db>, event: Option<String>, limit: Option<i64>) -> Result<Vec<AuditRow>, UiError> {
    guard(&session, "admin_audit")?;
    let conn = lock(&db)?;
    Ok(store::audit(&conn, event.as_deref(), limit.unwrap_or(200)).map_err(ServiceError::from)?)
}

/// A new recovery code for the administrator (the old one stops working). Shown once.
#[tauri::command]
pub fn admin_recovery_code_new(session: State<'_, Session>, db: State<'_, Db>) -> Result<String, UiError> {
    guard(&session, "admin_recovery_code_new")?;
    let conn = lock(&db)?;
    Ok(svc::new_recovery(&conn)?)
}
