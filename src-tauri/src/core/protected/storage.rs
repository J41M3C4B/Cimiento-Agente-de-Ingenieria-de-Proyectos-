//! The SQL of the board and the bank accounts of the institution (`core_board_member`, `core_bank_account`).

use crate::storage::StorageError;
use rusqlite::{params, Connection, OptionalExtension};
use ulid::Ulid;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";
/// Who acts, from the session of the connection, as in the audit log.
const ACTOR: &str = "(SELECT user_id FROM temp.session_actor LIMIT 1)";

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

fn actor_table(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch("CREATE TEMP TABLE IF NOT EXISTS session_actor (user_id TEXT);")?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberRow {
    pub id: String,
    pub name: String,
    pub role: String,
    pub role_other: Option<String>,
    pub since: Option<String>,
    pub active: bool,
    pub origin: String,
    pub confirmed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRow {
    pub id: String,
    pub clabe: String,
    pub bank: Option<String>,
    pub purpose: Vec<String>,
    pub in_documents: bool,
    pub active: bool,
    pub origin: String,
    pub confirmed_at: Option<String>,
}

const MEMBER: &str = "SELECT id, name, role, role_other, since, active, origin, confirmed_at FROM core_board_member";
const ACCOUNT: &str = "SELECT id, clabe, bank, purpose, in_documents, active, origin, confirmed_at FROM core_bank_account";

fn member(r: &rusqlite::Row) -> rusqlite::Result<MemberRow> {
    Ok(MemberRow {
        id: r.get(0)?,
        name: r.get(1)?,
        role: r.get(2)?,
        role_other: r.get(3)?,
        since: r.get(4)?,
        active: r.get::<_, i64>(5)? != 0,
        origin: r.get(6)?,
        confirmed_at: r.get(7)?,
    })
}

fn account(r: &rusqlite::Row) -> rusqlite::Result<AccountRow> {
    Ok(AccountRow {
        id: r.get(0)?,
        clabe: r.get(1)?,
        bank: r.get(2)?,
        purpose: serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default(),
        in_documents: r.get::<_, i64>(4)? != 0,
        active: r.get::<_, i64>(5)? != 0,
        origin: r.get(6)?,
        confirmed_at: r.get(7)?,
    })
}

/// The board, without those whose deletion waits for the administrator.
pub fn members(conn: &Connection) -> Result<Vec<MemberRow>, StorageError> {
    let mut stmt = conn.prepare(&format!("{MEMBER} WHERE hidden = 0 ORDER BY created_at, id"))?;
    let rows = stmt.query_map([], member)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn member_by_id(conn: &Connection, id: &str) -> Result<Option<MemberRow>, StorageError> {
    Ok(conn.query_row(&format!("{MEMBER} WHERE id = ?1 AND hidden = 0"), [id], member).optional()?)
}

pub fn accounts(conn: &Connection) -> Result<Vec<AccountRow>, StorageError> {
    let mut stmt = conn.prepare(&format!("{ACCOUNT} WHERE hidden = 0 ORDER BY created_at, id"))?;
    let rows = stmt.query_map([], account)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn account_by_id(conn: &Connection, id: &str) -> Result<Option<AccountRow>, StorageError> {
    Ok(conn.query_row(&format!("{ACCOUNT} WHERE id = ?1 AND hidden = 0"), [id], account).optional()?)
}

/// Whether a CLABE is already one of the institution's accounts (hidden or closed ones too).
pub fn clabe_taken(conn: &Connection, clabe: &str) -> Result<bool, StorageError> {
    Ok(conn.query_row("SELECT count(*) FROM core_bank_account WHERE clabe = ?1", [clabe], |r| r.get::<_, i64>(0))? > 0)
}

/// Every CLABE and every name the institution keeps, whatever their state: what the scanner takes as its own.
pub fn own_clabes_and_names(conn: &Connection) -> Result<(Vec<String>, Vec<String>), StorageError> {
    let column = |sql: &str| -> Result<Vec<String>, StorageError> {
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    };
    Ok((column("SELECT clabe FROM core_bank_account")?, column("SELECT name FROM core_board_member")?))
}

/// What a person wrote or accepted of a board member. `id = None` adds one.
pub struct MemberInput<'a> {
    pub name: &'a str,
    pub role: &'a str,
    pub role_other: Option<&'a str>,
    pub since: Option<&'a str>,
}

pub fn save_member(conn: &Connection, id: Option<&str>, m: &MemberInput, origin: &str) -> Result<String, StorageError> {
    actor_table(conn)?;
    match id {
        Some(id) => {
            conn.execute(
                &format!(
                    "UPDATE core_board_member SET name=?2, role=?3, role_other=?4, since=?5, origin=?6, source_ref=NULL,
                            confirmed_at={NOW}, confirmed_by={ACTOR}, updated_at={NOW} WHERE id=?1"
                ),
                params![id, m.name, m.role, m.role_other, m.since, origin],
            )?;
            Ok(id.to_string())
        }
        None => {
            let id = new_id("brd");
            conn.execute(
                &format!(
                    "INSERT INTO core_board_member (id, name, role, role_other, since, origin, confirmed_at, confirmed_by, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, {NOW}, {ACTOR}, {NOW}, {NOW})"
                ),
                params![id, m.name, m.role, m.role_other, m.since, origin],
            )?;
            Ok(id)
        }
    }
}

/// What a person wrote or accepted of an account. The CLABE goes only when it is added (`id = None`).
pub struct AccountInput<'a> {
    pub clabe: Option<&'a str>,
    pub bank: Option<&'a str>,
    pub purpose: &'a [String],
    pub in_documents: bool,
}

pub fn save_account(conn: &Connection, id: Option<&str>, a: &AccountInput, origin: &str) -> Result<String, StorageError> {
    actor_table(conn)?;
    let purpose = serde_json::to_string(a.purpose).unwrap_or_else(|_| "[]".into());
    match id {
        Some(id) => {
            conn.execute(
                &format!(
                    "UPDATE core_bank_account SET bank=?2, purpose=?3, in_documents=?4, origin=?5, source_ref=NULL,
                            confirmed_at={NOW}, confirmed_by={ACTOR}, updated_at={NOW} WHERE id=?1"
                ),
                params![id, a.bank, purpose, a.in_documents, origin],
            )?;
            Ok(id.to_string())
        }
        None => {
            let id = new_id("bnk");
            conn.execute(
                &format!(
                    "INSERT INTO core_bank_account (id, clabe, bank, purpose, in_documents, origin, confirmed_at, confirmed_by, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, {NOW}, {ACTOR}, {NOW}, {NOW})"
                ),
                params![id, a.clabe, a.bank, purpose, a.in_documents, origin],
            )?;
            Ok(id)
        }
    }
}

/// The two tables, by the kind of list. Only these names ever reach the SQL below.
fn table(board: bool) -> &'static str {
    if board {
        "core_board_member"
    } else {
        "core_bank_account"
    }
}

/// Marks who left or an account that closed (or brings them back). `false` when there is no such row.
pub fn set_active(conn: &Connection, board: bool, id: &str, active: bool) -> Result<bool, StorageError> {
    let t = table(board);
    Ok(conn.execute(&format!("UPDATE {t} SET active=?2, updated_at={NOW} WHERE id=?1 AND hidden = 0"), params![id, active])? > 0)
}

/// Hides a row while its deletion waits for the administrator, or brings it back. `false` when there is none.
pub fn set_hidden(conn: &Connection, board: bool, id: &str, hidden: bool) -> Result<bool, StorageError> {
    let t = table(board);
    Ok(conn.execute(&format!("UPDATE {t} SET hidden=?2, updated_at={NOW} WHERE id=?1"), params![id, hidden])? > 0)
}

/// The row whatever its state (the administrator decides on a hidden one).
pub fn label_source(conn: &Connection, board: bool, id: &str) -> Result<Option<String>, StorageError> {
    let column = if board { "name" } else { "clabe" };
    let t = table(board);
    Ok(conn.query_row(&format!("SELECT {column} FROM {t} WHERE id=?1"), [id], |r| r.get(0)).optional()?)
}

pub fn delete(conn: &Connection, board: bool, id: &str) -> Result<bool, StorageError> {
    let t = table(board);
    Ok(conn.execute(&format!("DELETE FROM {t} WHERE id=?1"), [id])? > 0)
}

/// The CLABE of an account, in full: only to show it on request (the caller writes it in the audit log).
pub fn clabe(conn: &Connection, id: &str) -> Result<Option<String>, StorageError> {
    Ok(conn.query_row("SELECT clabe FROM core_bank_account WHERE id=?1 AND hidden = 0", [id], |r| r.get(0)).optional()?)
}
