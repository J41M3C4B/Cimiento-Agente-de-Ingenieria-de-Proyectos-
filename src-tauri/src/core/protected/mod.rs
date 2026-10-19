//! The institution's own and protected lists (ADR-033 §3): its board of trustees or governing body and its bank
//! accounts. They live in the core, in the clear only on this computer:
//!
//! - **Never to the AI nor to a tool of an agent.** Of the board the AI learns how many people there are and which
//!   roles (`ai_board`); of the accounts, nothing. The code puts names and accounts in the documents.
//! - **The CLABE is shown covered.** Showing it in full is written in the audit log: the field, never the value.
//! - **Who leaves or an account that closes is marked**, not deleted; a deletion for good (a mistake when writing it)
//!   follows ADR-028: the administrator deletes, anyone else asks.
//! - **Every change leaves a line in the history** that says only that the list changed (`history`).
//! - **The scanner takes them as the institution's own** (`own_clabes_and_names`): a CLABE in a free text is let
//!   through only when it is exactly one of these accounts.

pub mod storage;

#[cfg(test)]
mod tests;

use crate::audit::{self, AuditKind};
use crate::common::forms::{is_filled, FormSpec, Values};
use crate::common::ids;
use crate::core::error::ServiceError;
use crate::core::history::{self, Source};
use crate::core::institution::catalog::BOARD_ROLES;
use crate::core::institution::forms::{BANK_ACCOUNT, BANK_ACCOUNT_SAVED, BOARD_MEMBER};
use crate::core::profile::domain::ProfileIssue;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use storage as store;

/// Which list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListKind {
    BoardMember,
    BankAccount,
}

impl ListKind {
    fn board(self) -> bool {
        self == ListKind::BoardMember
    }
    /// The form of a row; a saved account no longer asks its CLABE.
    fn spec(self, saved: bool) -> &'static FormSpec {
        match (self, saved) {
            (ListKind::BoardMember, _) => &BOARD_MEMBER,
            (ListKind::BankAccount, false) => &BANK_ACCOUNT,
            (ListKind::BankAccount, true) => &BANK_ACCOUNT_SAVED,
        }
    }
    /// What the history writes when the list changes.
    fn history_field(self) -> &'static str {
        self.spec(false).id
    }
}

/// A person of the board, as «Mi institución» shows it: the name in the clear, on this computer only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoardMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub role_other: Option<String>,
    pub since: Option<String>,
    pub active: bool,
    pub origin: String,
    pub confirmed_at: Option<String>,
}

/// An account, as the screen shows it: the CLABE covered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BankAccount {
    pub id: String,
    pub masked: String,
    pub bank: Option<String>,
    pub purpose: Vec<String>,
    pub in_documents: bool,
    pub active: bool,
    pub origin: String,
    pub confirmed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Lists {
    /// Who is in the board first, by role (presidencia first), then who left.
    pub board: Vec<BoardMember>,
    /// Open accounts first.
    pub accounts: Vec<BankAccount>,
}

/// A row ready to draw in its window. For a saved account, `masked` is its CLABE covered: the form does not ask it.
#[derive(Debug, Serialize)]
pub struct ListForm {
    pub spec: &'static FormSpec,
    pub values: Values,
    pub missing: Vec<&'static str>,
    pub masked: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ListOutcome {
    Saved { lists: Lists },
    /// A value that cannot be kept: the field and why. Nothing was saved.
    Invalid { issues: Vec<ProfileIssue> },
}

/// How a CLABE is shown: its last four digits.
pub fn masked(clabe: &str) -> String {
    ids::mask(clabe, 0, 4)
}

fn role_rank(role: &str) -> usize {
    BOARD_ROLES.iter().position(|r| *r == role).unwrap_or(BOARD_ROLES.len())
}

pub fn lists(conn: &Connection) -> Result<Lists, ServiceError> {
    let mut board: Vec<BoardMember> = store::members(conn)?
        .into_iter()
        .map(|m| BoardMember { id: m.id, name: m.name, role: m.role, role_other: m.role_other, since: m.since, active: m.active, origin: m.origin, confirmed_at: m.confirmed_at })
        .collect();
    // stable: inside a role, in the order they were added
    board.sort_by_key(|m| (!m.active, role_rank(&m.role)));
    let mut accounts: Vec<BankAccount> = store::accounts(conn)?
        .into_iter()
        .map(|a| BankAccount {
            id: a.id,
            masked: masked(&a.clabe),
            bank: a.bank,
            purpose: a.purpose,
            in_documents: a.in_documents,
            active: a.active,
            origin: a.origin,
            confirmed_at: a.confirmed_at,
        })
        .collect();
    accounts.sort_by_key(|a| !a.active);
    Ok(Lists { board, accounts })
}

/// The window of a row: empty to add one, or what a saved one holds.
pub fn form(conn: &Connection, kind: ListKind, id: Option<&str>) -> Result<ListForm, ServiceError> {
    let spec = kind.spec(id.is_some());
    let mut values = Values::new();
    let mut covered = None;
    match (kind, id) {
        (_, None) => {}
        (ListKind::BoardMember, Some(id)) => {
            let m = store::member_by_id(conn, id)?.ok_or(ServiceError::NotFound)?;
            values.insert("institution.board.name".into(), json!(m.name));
            values.insert("institution.board.role".into(), json!(m.role));
            values.insert("institution.board.role_other".into(), json!(m.role_other));
            values.insert("institution.board.since".into(), json!(m.since));
        }
        (ListKind::BankAccount, Some(id)) => {
            let a = store::account_by_id(conn, id)?.ok_or(ServiceError::NotFound)?;
            values.insert("institution.bank.bank".into(), json!(a.bank));
            values.insert("institution.bank.purpose".into(), json!(a.purpose));
            values.insert("institution.bank.in_documents".into(), json!(a.in_documents));
            covered = Some(masked(&a.clabe));
        }
    }
    values.retain(|_, v| is_filled(Some(v)));
    Ok(ListForm { spec, missing: spec.missing(&values), values, masked: covered })
}

fn text(v: &Values, field: &str) -> Option<String> {
    v.get(field).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn invalid(code: &'static str, field: &str) -> ListOutcome {
    ListOutcome::Invalid { issues: vec![ProfileIssue { code, field: field.to_string(), blocking: true }] }
}

/// Saves a row. A value out of its list, a wrong CLABE or date, or a required field that is empty is refused with
/// its field: a person of the board needs a name and a role, an account its CLABE and what it is for. The row and
/// its line in the history go together.
pub fn save(conn: &mut Connection, kind: ListKind, id: Option<&str>, mut values: Values) -> Result<ListOutcome, ServiceError> {
    let spec = kind.spec(id.is_some());
    spec.clear_hidden(&mut values);
    let mut issues: Vec<ProfileIssue> = spec
        .validate(&values)
        .into_iter()
        // a row is not a profile filled little by little: what it needs is needed now
        .map(|i| ProfileIssue { code: if i.code == "missing" { "required" } else { i.code }, field: i.field.to_string(), blocking: true })
        .collect();
    if !issues.is_empty() {
        issues.sort_by_key(|i| spec.fields().position(|f| f.id == i.field));
        return Ok(ListOutcome::Invalid { issues });
    }
    let tx = conn.transaction()?;
    match kind {
        ListKind::BoardMember => {
            if let Some(id) = id {
                store::member_by_id(&tx, id)?.ok_or(ServiceError::NotFound)?;
            }
            let (name, role) = (text(&values, "institution.board.name").unwrap_or_default(), text(&values, "institution.board.role").unwrap_or_default());
            let (role_other, since) = (text(&values, "institution.board.role_other"), text(&values, "institution.board.since"));
            let input = store::MemberInput { name: &name, role: &role, role_other: role_other.as_deref(), since: since.as_deref() };
            store::save_member(&tx, id, &input, "user")?;
        }
        ListKind::BankAccount => {
            let clabe = match id {
                Some(id) => {
                    store::account_by_id(&tx, id)?.ok_or(ServiceError::NotFound)?;
                    None
                }
                None => {
                    let c = ids::normalize(&text(&values, "institution.bank.clabe").unwrap_or_default());
                    if store::clabe_taken(&tx, &c)? {
                        return Ok(invalid("clabe_repeated", "institution.bank.clabe"));
                    }
                    Some(c)
                }
            };
            // the bank is read from the CLABE when the person leaves it empty (a saved one, from the one it has)
            let saved = match id {
                Some(id) => store::clabe(&tx, id)?,
                None => clabe.clone(),
            };
            let bank = text(&values, "institution.bank.bank").or_else(|| saved.as_deref().and_then(ids::bank_of_clabe).map(str::to_string));
            let purpose: Vec<String> =
                values.get("institution.bank.purpose").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default();
            let in_documents = values.get("institution.bank.in_documents").and_then(Value::as_bool).unwrap_or(false);
            let input = store::AccountInput { clabe: clabe.as_deref(), bank: bank.as_deref(), purpose: &purpose, in_documents };
            store::save_account(&tx, id, &input, "user")?;
        }
    }
    history::record_list_change(&tx, kind.history_field(), &Source::user())?;
    tx.commit()?;
    Ok(ListOutcome::Saved { lists: lists(conn)? })
}

/// Marks who left the board or an account that closed, or brings them back.
pub fn set_active(conn: &mut Connection, kind: ListKind, id: &str, active: bool) -> Result<Lists, ServiceError> {
    let tx = conn.transaction()?;
    if !store::set_active(&tx, kind.board(), id, active)? {
        return Err(ServiceError::NotFound);
    }
    history::record_list_change(&tx, kind.history_field(), &Source::user())?;
    tx.commit()?;
    lists(conn)
}

/// What the administrator sees in a request to delete a row: the name, or the account covered.
fn label(kind: ListKind, source: String) -> String {
    match kind {
        ListKind::BoardMember => source,
        ListKind::BankAccount => format!("Cuenta {}", masked(&source)),
    }
}

/// Hides a row while its deletion waits for the administrator, or brings it back (ADR-028). Returns what the
/// request shows; `None` when there is no such row.
pub fn set_hidden(conn: &Connection, kind: ListKind, id: &str, hidden: bool) -> Result<Option<String>, ServiceError> {
    let Some(source) = store::label_source(conn, kind.board(), id)? else { return Ok(None) };
    store::set_hidden(conn, kind.board(), id, hidden)?;
    Ok(Some(label(kind, source)))
}

/// Deletes a row for good, with its line in the history.
pub fn delete(conn: &mut Connection, kind: ListKind, id: &str) -> Result<(), ServiceError> {
    let tx = conn.transaction()?;
    if !store::delete(&tx, kind.board(), id)? {
        return Err(ServiceError::NotFound);
    }
    history::record_list_change(&tx, kind.history_field(), &Source::user())?;
    tx.commit()?;
    Ok(())
}

/// The CLABE of an account in full, to read it or copy it. The audit log keeps that it was shown, never the value.
pub fn reveal_clabe(conn: &Connection, id: &str) -> Result<String, ServiceError> {
    let clabe = store::clabe(conn, id)?.ok_or(ServiceError::NotFound)?;
    audit::record(conn, AuditKind::InstitutionSensitiveViewed, Some("core_bank_account"), Some(id), json!({ "field": "clabe" }))?;
    Ok(clabe)
}

/// The board as the AI may know it: how many people are in it today and how many in each role. Never a name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoardSheet {
    pub total: usize,
    /// (role code, how many), in the order of `BOARD_ROLES`.
    pub roles: Vec<(&'static str, usize)>,
}

pub fn ai_board(conn: &Connection) -> Result<BoardSheet, ServiceError> {
    let active: Vec<String> = store::members(conn)?.into_iter().filter(|m| m.active).map(|m| m.role).collect();
    let roles = BOARD_ROLES.iter().map(|r| (*r, active.iter().filter(|x| x == r).count())).filter(|(_, n)| *n > 0).collect();
    Ok(BoardSheet { total: active.len(), roles })
}

/// Whether the institution registered anyone in its board today («lo que falta» asks for it).
pub fn has_board(conn: &Connection) -> Result<bool, ServiceError> {
    Ok(store::members(conn)?.iter().any(|m| m.active))
}
