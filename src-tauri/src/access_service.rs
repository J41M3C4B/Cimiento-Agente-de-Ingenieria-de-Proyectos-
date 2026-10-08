//! Access profiles (ADR-028): the accounts, the session of the person using the app, and the deletions that wait for
//! the administrator. The rules (roles, permissions, passwords) are in `domain::access`; here they meet the database.

use crate::audit::{self, AuditKind};
use crate::domain::access::{self as rules, DeletionKind, Permission, Role};
use crate::profile_sync;
use crate::security_service::{self, Attempts, Verify};
use crate::service::ServiceError;
use crate::storage::access::{self as store, NewUser, RequestRow, StoredUser};
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::json;
use std::sync::Mutex;
use std::time::Instant;

// ------------------------------------------------------------------ the session

/// The person using the app now.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub must_change_password: bool,
}

#[derive(Debug, Default)]
pub struct SessionState {
    pub user: Option<CurrentUser>,
    /// The screen is locked (after a while without use): only the same person's password opens it.
    pub locked: bool,
}

/// The session lives only in memory: closing the app closes it. It keeps the database too, to write in the audit log
/// what someone tried without permission.
#[derive(Default)]
pub struct Session {
    pub state: Mutex<SessionState>,
    pub db: Option<crate::storage::SharedDb>,
}

impl Session {
    pub fn new(db: crate::storage::SharedDb) -> Self {
        Session { state: Mutex::new(SessionState::default()), db: Some(db) }
    }
    pub fn current(&self) -> Option<CurrentUser> {
        self.state.lock().ok().and_then(|s| s.user.clone())
    }
    pub(crate) fn set(&self, user: Option<CurrentUser>) {
        if let Ok(mut s) = self.state.lock() {
            s.user = user;
            s.locked = false;
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionView {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub permissions: &'static [Permission],
    pub must_change_password: bool,
    pub locked: bool,
}

#[derive(Debug, Serialize)]
pub struct AccessStatus {
    /// There is no account yet: the first one (the administrator) has to be made.
    pub needs_setup: bool,
    /// The computer had a screen PIN: it is asked before making the first account.
    pub setup_needs_pin: bool,
    pub session: Option<SessionView>,
    /// Minutes without use before the screen locks (the app counts them; Rust keeps the lock).
    pub idle_minutes: u64,
}

fn view_of(u: &CurrentUser, locked: bool) -> SessionView {
    SessionView {
        user_id: u.id.clone(),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        role: u.role,
        permissions: u.role.permissions(),
        must_change_password: u.must_change_password,
        locked,
    }
}

fn current_of(u: &StoredUser) -> CurrentUser {
    CurrentUser { id: u.id.clone(), username: u.username.clone(), display_name: u.display_name.clone(), role: u.role, must_change_password: u.must_change_password }
}

pub fn status(conn: &Connection, session: &Session) -> Result<AccessStatus, ServiceError> {
    let needs_setup = store::count_users(conn)? == 0;
    let state = session.state.lock().map_err(|_| ServiceError::Internal("session".into()))?;
    Ok(AccessStatus {
        needs_setup,
        setup_needs_pin: needs_setup && security_service::pin_enabled(conn)?,
        session: state.user.as_ref().map(|u| view_of(u, state.locked)),
        idle_minutes: rules::IDLE_MINUTES,
    })
}

/// The rule of the check every command makes (ADR-028), apart from the app so it can be tested: who may pass, or the
/// code of why not (`not_signed_in`, `session_locked`, `must_change_password`, `access_denied`).
pub fn check(user: Option<CurrentUser>, locked: bool, need: rules::Need) -> Result<Option<CurrentUser>, &'static str> {
    use rules::Need;
    if need == Need::Open {
        return Ok(user);
    }
    let user = user.ok_or("not_signed_in")?;
    if locked {
        return Err("session_locked");
    }
    let allowed = match need {
        Need::Open | Need::Session => return Ok(Some(user)),
        _ if user.must_change_password => return Err("must_change_password"),
        Need::Permission(p) => user.role.can(p),
        Need::DeleteOrRequest => user.role.can(Permission::Use),
    };
    if allowed {
        Ok(Some(user))
    } else {
        Err("access_denied")
    }
}

// ------------------------------------------------------------------ passwords

fn hasher() -> Argon2<'static> {
    // the tests use a light cost: the default (19 MiB, 2 passes) is what protects real accounts
    #[cfg(test)]
    {
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, argon2::Params::new(64, 1, 1, None).expect("valid params"))
    }
    #[cfg(not(test))]
    {
        Argon2::default()
    }
}

fn hash(secret: &str) -> Result<String, ServiceError> {
    hasher().hash_password(secret.as_bytes()).map(|h| h.to_string()).map_err(|e| ServiceError::Internal(e.to_string()))
}

fn matches(stored: &str, secret: &str) -> bool {
    PasswordHash::new(stored).is_ok_and(|h| Argon2::default().verify_password(secret.as_bytes(), &h).is_ok())
}

fn check_password(password: &str, username: &str) -> Result<(), ServiceError> {
    match rules::password_problem(password, username) {
        Some(code) => Err(ServiceError::Access(code)),
        None => Ok(()),
    }
}

/// A recovery code: 16 characters without the ones that look alike, in groups of four.
fn new_recovery_code() -> Result<String, ServiceError> {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| ServiceError::Internal(e.to_string()))?;
    let chars: Vec<char> = bytes.iter().map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char).collect();
    Ok(chars.chunks(4).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join("-"))
}

fn plain_code(code: &str) -> String {
    code.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase()
}

// ------------------------------------------------------------------ entering

#[derive(Debug, Serialize)]
pub struct SetupOutcome {
    pub session: SessionView,
    /// Shown once, to write it down: it lets the administrator in again if the password is forgotten.
    pub recovery_code: String,
}

/// The first account of the app: the administrator. If the computer had a screen PIN, it is asked first (so whoever
/// happens to open the app cannot take the administration), and then it retires.
pub fn setup_admin(
    conn: &mut Connection,
    session: &Session,
    attempts: &Attempts,
    display_name: &str,
    username: &str,
    password: &str,
    pin: Option<&str>,
) -> Result<SetupOutcome, ServiceError> {
    if store::count_users(conn)? > 0 {
        return Err(ServiceError::Access("already_set_up"));
    }
    if security_service::pin_enabled(conn)? {
        match security_service::verify(conn, attempts, pin.unwrap_or_default(), Instant::now())? {
            Verify::Ok => {}
            Verify::Locked { .. } => return Err(ServiceError::Access("pin_locked")),
            _ => return Err(ServiceError::Access("wrong_pin")),
        }
    }
    let username = rules::normalize_username(username).ok_or(ServiceError::Access("username_invalid"))?;
    if display_name.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    check_password(password, &username)?;
    let code = new_recovery_code()?;
    let tx = conn.transaction()?;
    let id = store::insert_user(&tx, &NewUser { username: &username, display_name, person_id: None, role: Role::Admin, password_hash: &hash(password)?, must_change_password: false, created_by: None })?;
    store::set_recovery_hash(&tx, &hash(&plain_code(&code))?)?;
    tx.execute("DELETE FROM app_settings WHERE key = 'security.pin'", [])?;
    store::set_actor(&tx, Some(&id))?;
    audit::record(&tx, AuditKind::AccessSetup, Some("app_user"), Some(&id), json!({}))?;
    tx.commit()?;
    let user = store::user(conn, &id)?.ok_or(ServiceError::NotFound)?;
    store::record_login(conn, &id)?;
    let current = current_of(&user);
    session.set(Some(current.clone()));
    Ok(SetupOutcome { session: view_of(&current, false), recovery_code: code })
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LoginOutcome {
    Ok { session: SessionView },
    /// The user name or the password is wrong (it does not say which).
    Wrong,
    /// Too many mistakes in a row: wait before trying again.
    Waiting { wait_secs: i64 },
    Disabled,
}

/// Checks a password against an account, counting mistakes (they make the account wait) and recording the outcome.
fn try_password(conn: &Connection, user: &StoredUser, password: &str) -> Result<Option<LoginOutcome>, ServiceError> {
    let now = store::now_secs(conn)?;
    if let Some(until) = user.locked_until.filter(|u| *u > now) {
        return Ok(Some(LoginOutcome::Waiting { wait_secs: until - now }));
    }
    if matches(&user.password_hash, password) {
        return Ok(None);
    }
    let failed = user.failed_attempts + 1;
    let wait = rules::wait_after(failed);
    store::record_failure(conn, &user.id, failed, (wait > 0).then_some(now + wait))?;
    audit::record(conn, AuditKind::AuthLoginFailed, Some("app_user"), Some(&user.id), json!({ "attempt": failed }))?;
    if wait > 0 {
        audit::record(conn, AuditKind::AuthLocked, Some("app_user"), Some(&user.id), json!({ "wait_secs": wait }))?;
        return Ok(Some(LoginOutcome::Waiting { wait_secs: wait }));
    }
    Ok(Some(LoginOutcome::Wrong))
}

pub fn login(conn: &Connection, session: &Session, username: &str, password: &str) -> Result<LoginOutcome, ServiceError> {
    let Some(user) = store::user_by_username(conn, username.trim())? else {
        audit::record(conn, AuditKind::AuthLoginFailed, Some("app_user"), None, json!({}))?;
        return Ok(LoginOutcome::Wrong);
    };
    if !user.active {
        return Ok(LoginOutcome::Disabled);
    }
    if let Some(out) = try_password(conn, &user, password)? {
        return Ok(out);
    }
    store::record_login(conn, &user.id)?;
    store::set_actor(conn, Some(&user.id))?;
    audit::record(conn, AuditKind::AuthLogin, Some("app_user"), Some(&user.id), json!({}))?;
    let current = current_of(&user);
    session.set(Some(current.clone()));
    Ok(LoginOutcome::Ok { session: view_of(&current, false) })
}

/// Locks the screen: the work stays, and only the same person's password opens it.
pub fn lock(session: &Session) {
    if let Ok(mut s) = session.state.lock() {
        if s.user.is_some() {
            s.locked = true;
        }
    }
}

pub fn unlock(conn: &Connection, session: &Session, password: &str) -> Result<LoginOutcome, ServiceError> {
    let Some(current) = session.current() else { return Err(ServiceError::Access("not_signed_in")) };
    let user = store::user(conn, &current.id)?.ok_or(ServiceError::NotFound)?;
    if !user.active {
        logout(conn, session)?;
        return Ok(LoginOutcome::Disabled);
    }
    if let Some(out) = try_password(conn, &user, password)? {
        return Ok(out);
    }
    store::record_login(conn, &user.id)?;
    let current = current_of(&user);
    session.set(Some(current.clone()));
    Ok(LoginOutcome::Ok { session: view_of(&current, false) })
}

pub fn logout(conn: &Connection, session: &Session) -> Result<(), ServiceError> {
    if session.current().is_some() {
        audit::record(conn, AuditKind::AuthLogout, None, None, json!({}))?;
    }
    store::set_actor(conn, None)?;
    session.set(None);
    Ok(())
}

pub fn change_password(conn: &Connection, session: &Session, current_password: &str, new_password: &str) -> Result<SessionView, ServiceError> {
    let Some(current) = session.current() else { return Err(ServiceError::Access("not_signed_in")) };
    let user = store::user(conn, &current.id)?.ok_or(ServiceError::NotFound)?;
    if !matches(&user.password_hash, current_password) {
        return Err(ServiceError::Access("wrong_password"));
    }
    check_password(new_password, &user.username)?;
    if new_password == current_password {
        return Err(ServiceError::Access("password_same"));
    }
    store::set_password(conn, &user.id, &hash(new_password)?, false)?;
    audit::record(conn, AuditKind::PasswordChanged, Some("app_user"), Some(&user.id), json!({}))?;
    let updated = CurrentUser { must_change_password: false, ..current };
    session.set(Some(updated.clone()));
    Ok(view_of(&updated, false))
}

/// The administrator forgot the password: the recovery code lets them set a new one, and a new code is given.
pub fn recover(conn: &Connection, session: &Session, username: &str, code: &str, new_password: &str) -> Result<SetupOutcome, ServiceError> {
    let user = store::user_by_username(conn, username.trim())?.filter(|u| u.role == Role::Admin && u.active).ok_or(ServiceError::Access("recovery_wrong"))?;
    let now = store::now_secs(conn)?;
    if user.locked_until.is_some_and(|u| u > now) {
        return Err(ServiceError::Access("recovery_wait"));
    }
    let stored = store::recovery_hash(conn)?.ok_or(ServiceError::Access("recovery_wrong"))?;
    if !matches(&stored, &plain_code(code)) {
        let failed = user.failed_attempts + 1;
        let wait = rules::wait_after(failed);
        store::record_failure(conn, &user.id, failed, (wait > 0).then_some(now + wait))?;
        audit::record(conn, AuditKind::AuthLoginFailed, Some("app_user"), Some(&user.id), json!({ "attempt": failed, "recovery": true }))?;
        return Err(ServiceError::Access("recovery_wrong"));
    }
    check_password(new_password, &user.username)?;
    let next = new_recovery_code()?;
    store::set_password(conn, &user.id, &hash(new_password)?, false)?;
    store::set_recovery_hash(conn, &hash(&plain_code(&next))?)?;
    store::record_login(conn, &user.id)?;
    store::set_actor(conn, Some(&user.id))?;
    audit::record(conn, AuditKind::AuthRecovered, Some("app_user"), Some(&user.id), json!({}))?;
    let current = current_of(&StoredUser { must_change_password: false, ..user });
    session.set(Some(current.clone()));
    Ok(SetupOutcome { session: view_of(&current, false), recovery_code: next })
}

// ------------------------------------------------------------------ the administration panel

#[derive(Debug, Serialize)]
pub struct UserRow {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub person_id: Option<String>,
    pub role: Role,
    pub active: bool,
    pub must_change_password: bool,
    pub last_login_at: Option<String>,
    /// Waiting after too many mistakes.
    pub waiting: bool,
}

/// A person of the staff, with their account if they have one.
#[derive(Debug, Serialize)]
pub struct PersonAccess {
    pub person_id: String,
    pub full_name: String,
    pub position_id: Option<String>,
    pub status: String,
    pub user_id: Option<String>,
    pub suggested_username: String,
}

#[derive(Debug, Serialize)]
pub struct AdminOverview {
    pub users: Vec<UserRow>,
    pub people: Vec<PersonAccess>,
    pub pending: Vec<RequestRow>,
    /// The latest resolved requests.
    pub resolved: Vec<RequestRow>,
}

pub fn admin_overview(conn: &Connection) -> Result<AdminOverview, ServiceError> {
    let now = store::now_secs(conn)?;
    let users = store::users(conn)?;
    let staff = crate::modules::hr::service::overview(conn)?;
    let people = staff
        .people
        .into_iter()
        .map(|p| {
            let user_id = users.iter().find(|u| u.person_id.as_deref() == Some(p.id.as_str())).map(|u| u.id.clone());
            let mut words = p.full_name.split_whitespace();
            let first = words.next().unwrap_or_default().to_string();
            let last = words.next().map(String::from);
            PersonAccess { suggested_username: rules::suggest_username(&first, last.as_deref()), person_id: p.id, full_name: p.full_name, position_id: p.position_id, status: p.status, user_id }
        })
        .collect();
    let all = store::requests(conn, false)?;
    let (pending, resolved): (Vec<_>, Vec<_>) = all.into_iter().partition(|r| r.status == "pending");
    Ok(AdminOverview {
        users: users
            .into_iter()
            .map(|u| UserRow {
                waiting: u.locked_until.is_some_and(|t| t > now),
                id: u.id,
                username: u.username,
                display_name: u.display_name,
                person_id: u.person_id,
                role: u.role,
                active: u.active,
                must_change_password: u.must_change_password,
                last_login_at: u.last_login_at,
            })
            .collect(),
        people,
        pending,
        resolved: resolved.into_iter().take(50).collect(),
    })
}

/// Today the administrator gives access only as direction or accounting: there is one administrator.
fn assignable(role: Role) -> Result<(), ServiceError> {
    if role == Role::Admin {
        return Err(ServiceError::Access("role_not_allowed"));
    }
    Ok(())
}

pub struct NewAccount<'a> {
    pub person_id: Option<&'a str>,
    pub display_name: &'a str,
    pub username: &'a str,
    pub role: Role,
    pub temporary_password: &'a str,
}

pub fn create_user(conn: &Connection, actor: &CurrentUser, a: &NewAccount) -> Result<AdminOverview, ServiceError> {
    assignable(a.role)?;
    let username = rules::normalize_username(a.username).ok_or(ServiceError::Access("username_invalid"))?;
    if store::user_by_username(conn, &username)?.is_some() {
        return Err(ServiceError::Access("username_taken"));
    }
    let display = match a.person_id {
        Some(pid) => {
            let person = crate::modules::hr::service::get_person(conn, pid)?.ok_or(ServiceError::NotFound)?;
            if store::user_by_person(conn, pid)?.is_some() {
                return Err(ServiceError::Access("person_has_account"));
            }
            if person.data.status == "left" {
                return Err(ServiceError::Access("person_left"));
            }
            person.data.full_name()
        }
        None => a.display_name.trim().to_string(),
    };
    if display.is_empty() {
        return Err(ServiceError::EmptyText);
    }
    check_password(a.temporary_password, &username)?;
    let id = store::insert_user(
        conn,
        &NewUser { username: &username, display_name: &display, person_id: a.person_id, role: a.role, password_hash: &hash(a.temporary_password)?, must_change_password: true, created_by: Some(&actor.id) },
    )?;
    audit::record(conn, AuditKind::UserCreated, Some("app_user"), Some(&id), json!({ "role": a.role.as_db() }))?;
    admin_overview(conn)
}

/// Changes the role or turns an account off or on. There is always one active administrator.
pub fn update_user(conn: &Connection, actor: &CurrentUser, id: &str, role: Role, active: bool) -> Result<AdminOverview, ServiceError> {
    let user = store::user(conn, id)?.ok_or(ServiceError::NotFound)?;
    if role != user.role {
        assignable(role)?;
    }
    let stops_being_admin = user.role == Role::Admin && user.active && (role != Role::Admin || !active);
    if stops_being_admin && store::active_admins(conn)? <= 1 {
        return Err(ServiceError::Access("last_admin"));
    }
    if id == actor.id && !active {
        return Err(ServiceError::Access("cannot_disable_self"));
    }
    store::set_role_and_active(conn, id, role, active)?;
    audit::record(conn, AuditKind::UserUpdated, Some("app_user"), Some(id), json!({ "role": role.as_db(), "active": active }))?;
    admin_overview(conn)
}

pub fn reset_password(conn: &Connection, id: &str, temporary_password: &str) -> Result<AdminOverview, ServiceError> {
    let user = store::user(conn, id)?.ok_or(ServiceError::NotFound)?;
    check_password(temporary_password, &user.username)?;
    store::set_password(conn, id, &hash(temporary_password)?, true)?;
    audit::record(conn, AuditKind::UserPasswordReset, Some("app_user"), Some(id), json!({}))?;
    admin_overview(conn)
}

/// A new recovery code for the administrator (the old one stops working). Shown once.
pub fn new_recovery(conn: &Connection) -> Result<String, ServiceError> {
    let code = new_recovery_code()?;
    store::set_recovery_hash(conn, &hash(&plain_code(&code))?)?;
    audit::record(conn, AuditKind::UserUpdated, Some("app_settings"), None, json!({ "recovery_code": "renewed" }))?;
    Ok(code)
}

// ------------------------------------------------------------------ deletions that wait for the administrator

/// Hides the thing (or brings it back) and returns its name for the request. People hidden or back change the sums
/// of the profile: the caller syncs it (`sync_after`).
fn hide(conn: &Connection, kind: DeletionKind, target: &str, hidden: bool) -> Result<String, ServiceError> {
    let label = match kind {
        DeletionKind::Document => store::set_hidden(conn, "document", target, hidden)?,
        DeletionKind::Project => store::set_hidden(conn, "project", target, hidden)?,
        DeletionKind::HrPerson => Some(crate::modules::hr::service::set_person_hidden(conn, target, hidden)?),
        DeletionKind::HrField => Some(crate::modules::hr::service::set_field_hidden(conn, target, hidden)?),
        DeletionKind::Beneficiary => Some(crate::modules::care::service::set_person_hidden(conn, target, hidden)?),
        DeletionKind::CareField => Some(crate::modules::care::service::set_field_hidden(conn, target, hidden)?),
    };
    label.ok_or(ServiceError::NotFound)
}

fn sync_after(conn: &mut Connection, kind: DeletionKind) -> Result<(), ServiceError> {
    if matches!(kind, DeletionKind::HrPerson | DeletionKind::Beneficiary) {
        profile_sync::sync_profile(conn)?;
    }
    Ok(())
}

/// Someone who may not delete asked to: the thing disappears for everyone now, and the administrator decides.
pub fn request_deletion(conn: &mut Connection, by: &CurrentUser, kind: DeletionKind, target: &str) -> Result<(), ServiceError> {
    let tx = conn.transaction()?;
    let label = hide(&tx, kind, target, true)?;
    let id = store::insert_request(&tx, kind, target, &label, &by.id).map_err(|e| match e {
        crate::storage::StorageError::Db(rusqlite::Error::SqliteFailure(f, _)) if f.code == rusqlite::ErrorCode::ConstraintViolation => {
            ServiceError::Access("already_requested")
        }
        other => other.into(),
    })?;
    audit::record(&tx, AuditKind::RequestCreated, Some("access_request"), Some(&id), json!({ "kind": kind.as_db() }))?;
    tx.commit()?;
    sync_after(conn, kind)
}

/// How a project is deleted for good: the core does not know the projects (ADR-032), so the commands say it.
pub type DeleteProject<'a> = &'a dyn Fn(&mut Connection, &str) -> Result<(), ServiceError>;

/// The deletion itself, as the administrator does it.
fn delete_for_good(conn: &mut Connection, kind: DeletionKind, target: &str, delete_project: DeleteProject) -> Result<(), ServiceError> {
    match kind {
        DeletionKind::Document => {
            crate::storage::documents::emergency_delete_document(conn, target)?;
        }
        DeletionKind::Project => {
            delete_project(conn, target)?;
        }
        DeletionKind::HrPerson => {
            crate::staff_service::delete_person(conn, target)?;
            store::deactivate_for_person(conn, target)?;
        }
        DeletionKind::HrField => {
            crate::modules::hr::service::delete_custom_field(conn, target)?;
        }
        DeletionKind::Beneficiary => {
            crate::care_service::delete_person(conn, target)?;
        }
        DeletionKind::CareField => {
            crate::modules::care::service::delete_custom_field(conn, target)?;
        }
    }
    Ok(())
}

/// The administrator approves (the thing is deleted for good) or rejects (it comes back) a request.
pub fn resolve_request(conn: &mut Connection, actor: &CurrentUser, id: &str, approve: bool, delete_project: DeleteProject) -> Result<AdminOverview, ServiceError> {
    let request = store::request(conn, id)?.filter(|r| r.status == "pending").ok_or(ServiceError::NotFound)?;
    if approve {
        delete_for_good(conn, request.kind, &request.target_id, delete_project)?;
    } else {
        hide(conn, request.kind, &request.target_id, false)?;
        sync_after(conn, request.kind)?;
    }
    store::resolve_request(conn, id, approve, &actor.id)?;
    let kind = if approve { AuditKind::RequestApproved } else { AuditKind::RequestRejected };
    audit::record(conn, kind, Some("access_request"), Some(id), json!({ "kind": request.kind.as_db() }))?;
    admin_overview(conn)
}

/// A person who left the institution cannot use the app any more.
pub fn after_staff_saved(conn: &Connection, person_id: &str, status: &str) -> Result<(), ServiceError> {
    if status == "left" && store::deactivate_for_person(conn, person_id)? > 0 {
        audit::record(conn, AuditKind::UserUpdated, Some("app_user"), None, json!({ "active": false, "reason": "left" }))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "access_service_tests.rs"]
mod tests;
