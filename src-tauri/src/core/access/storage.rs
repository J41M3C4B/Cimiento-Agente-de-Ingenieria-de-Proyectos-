//! Persistence of the access profiles (ADR-028): accounts, deletion requests, who is acting, and the audit log as the
//! administrator reads it.

use crate::storage::StorageError;
use crate::core::access::domain::{DeletionKind, Role};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;
use ulid::Ulid;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

/// Seconds since 1970, from the clock of the database.
pub fn now_secs(conn: &Connection) -> Result<i64, StorageError> {
    Ok(conn.query_row("SELECT CAST(strftime('%s','now') AS INTEGER)", [], |r| r.get(0))?)
}

/// An account as stored, password hash included (it never leaves the service).
#[derive(Debug, Clone)]
pub struct StoredUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub person_id: Option<String>,
    pub role: Role,
    pub active: bool,
    pub password_hash: String,
    pub must_change_password: bool,
    pub failed_attempts: i64,
    pub locked_until: Option<i64>,
    pub last_login_at: Option<String>,
}

const USER_COLUMNS: &str =
    "id, username, display_name, person_id, role, active, password_hash, must_change_password, failed_attempts, locked_until, last_login_at";

fn user_of(r: &Row) -> rusqlite::Result<StoredUser> {
    Ok(StoredUser {
        id: r.get(0)?,
        username: r.get(1)?,
        display_name: r.get(2)?,
        person_id: r.get(3)?,
        role: Role::from_db(&r.get::<_, String>(4)?).unwrap_or(Role::Manager),
        active: r.get::<_, i64>(5)? != 0,
        password_hash: r.get(6)?,
        must_change_password: r.get::<_, i64>(7)? != 0,
        failed_attempts: r.get(8)?,
        locked_until: r.get(9)?,
        last_login_at: r.get(10)?,
    })
}

pub fn count_users(conn: &Connection) -> Result<i64, StorageError> {
    Ok(conn.query_row("SELECT count(*) FROM app_user", [], |r| r.get(0))?)
}

pub fn active_admins(conn: &Connection) -> Result<i64, StorageError> {
    Ok(conn.query_row("SELECT count(*) FROM app_user WHERE role='admin' AND active=1", [], |r| r.get(0))?)
}

pub fn users(conn: &Connection) -> Result<Vec<StoredUser>, StorageError> {
    Ok(conn.prepare(&format!("SELECT {USER_COLUMNS} FROM app_user ORDER BY role, lower(display_name)"))?.query_map([], user_of)?.collect::<Result<Vec<_>, _>>()?)
}

pub fn user(conn: &Connection, id: &str) -> Result<Option<StoredUser>, StorageError> {
    Ok(conn.query_row(&format!("SELECT {USER_COLUMNS} FROM app_user WHERE id=?1"), [id], user_of).optional()?)
}

pub fn user_by_username(conn: &Connection, username: &str) -> Result<Option<StoredUser>, StorageError> {
    Ok(conn.query_row(&format!("SELECT {USER_COLUMNS} FROM app_user WHERE lower(username)=lower(?1)"), [username], user_of).optional()?)
}

pub fn user_by_person(conn: &Connection, person_id: &str) -> Result<Option<StoredUser>, StorageError> {
    Ok(conn.query_row(&format!("SELECT {USER_COLUMNS} FROM app_user WHERE person_id=?1"), [person_id], user_of).optional()?)
}

pub struct NewUser<'a> {
    pub username: &'a str,
    pub display_name: &'a str,
    pub person_id: Option<&'a str>,
    pub role: Role,
    pub password_hash: &'a str,
    pub must_change_password: bool,
    pub created_by: Option<&'a str>,
}

pub fn insert_user(conn: &Connection, u: &NewUser) -> Result<String, StorageError> {
    let id = format!("usr_{}", Ulid::generate());
    conn.execute(
        &format!(
            "INSERT INTO app_user (id,username,display_name,person_id,role,active,password_hash,must_change_password,created_by,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,1,?6,?7,?8,{NOW},{NOW})"
        ),
        params![id, u.username, u.display_name.trim(), u.person_id, u.role.as_db(), u.password_hash, u.must_change_password as i64, u.created_by],
    )?;
    Ok(id)
}

pub fn set_password(conn: &Connection, id: &str, hash: &str, must_change: bool) -> Result<(), StorageError> {
    conn.execute(
        &format!("UPDATE app_user SET password_hash=?2, must_change_password=?3, failed_attempts=0, locked_until=NULL, updated_at={NOW} WHERE id=?1"),
        params![id, hash, must_change as i64],
    )?;
    Ok(())
}

pub fn set_role_and_active(conn: &Connection, id: &str, role: Role, active: bool) -> Result<(), StorageError> {
    conn.execute(&format!("UPDATE app_user SET role=?2, active=?3, updated_at={NOW} WHERE id=?1"), params![id, role.as_db(), active as i64])?;
    Ok(())
}

pub fn record_failure(conn: &Connection, id: &str, failed: i64, locked_until: Option<i64>) -> Result<(), StorageError> {
    conn.execute("UPDATE app_user SET failed_attempts=?2, locked_until=?3 WHERE id=?1", params![id, failed, locked_until])?;
    Ok(())
}

pub fn record_login(conn: &Connection, id: &str) -> Result<(), StorageError> {
    conn.execute(&format!("UPDATE app_user SET failed_attempts=0, locked_until=NULL, last_login_at={NOW} WHERE id=?1"), [id])?;
    Ok(())
}

/// The accounts of these staff records stop working (the person left or was deleted).
pub fn deactivate_for_person(conn: &Connection, person_id: &str) -> Result<usize, StorageError> {
    Ok(conn.execute(&format!("UPDATE app_user SET active=0, updated_at={NOW} WHERE person_id=?1 AND role <> 'admin' AND active=1"), [person_id])?)
}

// ------------------------------------------------------------------ who is acting (for the audit log)

/// Writes who is acting on this connection: the audit log takes it from here by itself.
pub fn set_actor(conn: &Connection, user_id: Option<&str>) -> Result<(), StorageError> {
    conn.execute_batch("CREATE TEMP TABLE IF NOT EXISTS session_actor (user_id TEXT); DELETE FROM temp.session_actor;")?;
    if let Some(id) = user_id {
        conn.execute("INSERT INTO temp.session_actor (user_id) VALUES (?1)", [id])?;
    }
    Ok(())
}

// ------------------------------------------------------------------ the recovery code of the administrator

const RECOVERY_KEY: &str = "access.recovery";

pub fn set_recovery_hash(conn: &Connection, hash: &str) -> Result<(), StorageError> {
    conn.execute("INSERT INTO app_settings (key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![RECOVERY_KEY, hash])?;
    Ok(())
}

pub fn recovery_hash(conn: &Connection) -> Result<Option<String>, StorageError> {
    Ok(conn.query_row("SELECT value FROM app_settings WHERE key=?1", [RECOVERY_KEY], |r| r.get(0)).optional()?)
}

// ------------------------------------------------------------------ deletion requests

#[derive(Debug, Clone, Serialize)]
pub struct RequestRow {
    pub id: String,
    pub kind: DeletionKind,
    pub target_id: String,
    pub target_label: String,
    pub requested_by: String,
    pub requested_by_name: String,
    pub requested_at: String,
    pub status: String,
    pub resolved_by_name: Option<String>,
    pub resolved_at: Option<String>,
}

pub fn insert_request(conn: &Connection, kind: DeletionKind, target_id: &str, label: &str, by: &str) -> Result<String, StorageError> {
    let id = format!("req_{}", Ulid::generate());
    conn.execute(
        &format!("INSERT INTO access_request (id,kind,target_id,target_label,requested_by,requested_at,status) VALUES (?1,?2,?3,?4,?5,{NOW},'pending')"),
        params![id, kind.as_db(), target_id, label, by],
    )?;
    Ok(id)
}

pub fn requests(conn: &Connection, only_pending: bool) -> Result<Vec<RequestRow>, StorageError> {
    let sql = format!(
        "SELECT r.id, r.kind, r.target_id, r.target_label, r.requested_by, coalesce(a.display_name,''), r.requested_at, r.status,
                b.display_name, r.resolved_at
         FROM access_request r LEFT JOIN app_user a ON a.id = r.requested_by LEFT JOIN app_user b ON b.id = r.resolved_by
         {} ORDER BY r.requested_at DESC, r.id DESC LIMIT 200",
        if only_pending { "WHERE r.status = 'pending'" } else { "" }
    );
    Ok(conn
        .prepare(&sql)?
        .query_map([], |r| {
            Ok(RequestRow {
                id: r.get(0)?,
                kind: DeletionKind::from_db(&r.get::<_, String>(1)?).unwrap_or(DeletionKind::Document),
                target_id: r.get(2)?,
                target_label: r.get(3)?,
                requested_by: r.get(4)?,
                requested_by_name: r.get(5)?,
                requested_at: r.get(6)?,
                status: r.get(7)?,
                resolved_by_name: r.get(8)?,
                resolved_at: r.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn request(conn: &Connection, id: &str) -> Result<Option<RequestRow>, StorageError> {
    Ok(requests(conn, false)?.into_iter().find(|r| r.id == id))
}

pub fn resolve_request(conn: &Connection, id: &str, approved: bool, by: &str) -> Result<bool, StorageError> {
    Ok(conn.execute(
        &format!("UPDATE access_request SET status=?2, resolved_by=?3, resolved_at={NOW} WHERE id=?1 AND status='pending'"),
        params![id, if approved { "approved" } else { "rejected" }, by],
    )? > 0)
}

// ------------------------------------------------------------------ the audit log, as the administrator reads it

#[derive(Debug, Clone, Serialize)]
pub struct AuditRow {
    pub at: String,
    pub event: String,
    pub entity: Option<String>,
    pub entity_id: Option<String>,
    pub actor: Option<String>,
    pub details: serde_json::Value,
}

/// The latest entries, newest first; `event` filters by the start of the event name (`auth.`, `hr.`…).
pub fn audit(conn: &Connection, event: Option<&str>, limit: i64) -> Result<Vec<AuditRow>, StorageError> {
    let like = format!("{}%", event.unwrap_or(""));
    Ok(conn
        .prepare(
            "SELECT l.at, l.event, l.entity, l.entity_id, u.display_name, l.details_json
             FROM audit_log l LEFT JOIN app_user u ON u.id = l.actor_id
             WHERE l.event LIKE ?1 ORDER BY l.at DESC, l.rowid DESC LIMIT ?2",
        )?
        .query_map(params![like, limit.clamp(1, 500)], |r| {
            Ok(AuditRow {
                at: r.get(0)?,
                event: r.get(1)?,
                entity: r.get(2)?,
                entity_id: r.get(3)?,
                actor: r.get(4)?,
                details: serde_json::from_str(&r.get::<_, String>(5)?).unwrap_or(serde_json::Value::Null),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

// ------------------------------------------------------------------ hiding a document or a project while its deletion waits

/// Hides a document or a project (or brings it back) and returns its name, if it exists.
pub fn set_hidden(conn: &Connection, table: &str, id: &str, hidden: bool) -> Result<Option<String>, StorageError> {
    let (table, label) = match table {
        "document" => ("document", "display_name"),
        "project" => ("project", "title"),
        _ => return Ok(None),
    };
    let name: Option<String> = conn.query_row(&format!("SELECT {label} FROM {table} WHERE id=?1"), [id], |r| r.get(0)).optional()?;
    if name.is_some() {
        conn.execute(&format!("UPDATE {table} SET hidden=?2 WHERE id=?1"), params![id, hidden as i64])?;
    }
    Ok(name)
}
