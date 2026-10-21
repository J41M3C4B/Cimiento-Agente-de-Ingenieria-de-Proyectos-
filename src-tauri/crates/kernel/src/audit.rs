//! The audit store of the kernel (K9): where the platform leaves the life of each task (ADR-003, P12), with the id of
//! the task, the capability, the result and the reason. Never data of a person, and not by goodwill: the ids must
//! have the shape of an id, and the capability, the result and the reason are codes written in the program
//! (`&'static str`), never text that came from a person or a document.

use std::fmt;

use rusqlite::{params, Connection};

/// A moment in the life of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskMoment {
    Opened,
    /// A capability or a model was invoked.
    Step,
    /// A control said no: the permission, the policy or a limit.
    Denied,
    /// A person approved a change.
    Approved,
    /// The turn ended: done, paused, denied, by a limit or with an error.
    Closed,
}

impl TaskMoment {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskMoment::Opened => "task.opened",
            TaskMoment::Step => "task.step",
            TaskMoment::Denied => "task.denied",
            TaskMoment::Approved => "task.approved",
            TaskMoment::Closed => "task.closed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskEvent<'a> {
    pub moment: TaskMoment,
    /// The id of the task, which is the id of correlation.
    pub task: &'a str,
    /// The account the task runs for.
    pub account: Option<&'a str>,
    /// The id of the capability.
    pub capability: Option<&'static str>,
    /// What came out, as a code (`done`, `paused`, `limit`…).
    pub outcome: Option<&'static str>,
    /// Why, as a code (`access_denied`, `policy`…).
    pub reason: Option<&'static str>,
}

#[derive(Debug)]
pub enum AuditError {
    Db(rusqlite::Error),
    /// The field does not have the shape of an id: a lowercase prefix and `_`, then a ULID.
    NotAnId(&'static str),
    /// The field is not a code: lowercase letters, digits, `_` and `.`, up to 64.
    NotACode(&'static str),
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditError::Db(e) => write!(f, "database error: {e}"),
            AuditError::NotAnId(field) => write!(f, "the {field} of an audit event is not an id"),
            AuditError::NotACode(field) => write!(f, "the {field} of an audit event is not a code"),
        }
    }
}

impl std::error::Error for AuditError {}

impl From<rusqlite::Error> for AuditError {
    fn from(e: rusqlite::Error) -> Self {
        AuditError::Db(e)
    }
}

/// Writes a moment of a task in the audit store.
pub fn record_task(conn: &Connection, event: &TaskEvent) -> Result<(), AuditError> {
    if !is_id(event.task) {
        return Err(AuditError::NotAnId("task"));
    }
    if event.account.is_some_and(|a| !is_id(a)) {
        return Err(AuditError::NotAnId("account"));
    }
    for (field, value) in [("capability", event.capability), ("outcome", event.outcome), ("reason", event.reason)] {
        if value.is_some_and(|v| !is_code(v)) {
            return Err(AuditError::NotACode(field));
        }
    }
    conn.execute(
        "INSERT INTO kernel_audit (at, event, correlation_id, account_id, subject, outcome, reason)
         VALUES (strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?1, ?2, ?3, ?4, ?5, ?6)",
        params![event.moment.as_str(), event.task, event.account, event.capability, event.outcome, event.reason],
    )?;
    Ok(())
}

/// An optional lowercase prefix with `_` (`usr_`, `task_`), then a ULID: 26 characters of Crockford's base 32.
fn is_id(s: &str) -> bool {
    let ulid = match s.rsplit_once('_') {
        Some((prefix, rest)) if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_lowercase()) => rest,
        Some(_) => return false,
        None => s,
    };
    ulid.len() == 26 && ulid.chars().all(|c| c.is_ascii_digit() || (c.is_ascii_uppercase() && !matches!(c, 'I' | 'L' | 'O' | 'U')))
}

/// Lowercase letters, digits, `_` and `.`, starting with a letter, up to 64.
fn is_code(s: &str) -> bool {
    s.len() <= 64
        && s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    const TASK: &str = "task_01JB3M7Q8ZK4X2N6V9R5T0W1YC";
    const ACCOUNT: &str = "usr_01JB3M7Q8ZK4X2N6V9R5T0W1YD";

    fn store() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::migrate(&mut conn, &crate::Modules::of([]).unwrap()).unwrap();
        conn
    }

    fn opened() -> TaskEvent<'static> {
        TaskEvent { moment: TaskMoment::Opened, task: TASK, account: Some(ACCOUNT), capability: None, outcome: None, reason: None }
    }

    #[test]
    fn the_life_of_a_task_is_kept_by_its_id() {
        let conn = store();
        let step = TaskEvent { moment: TaskMoment::Step, capability: Some("muestra.notas_listar"), outcome: Some("ok"), ..opened() };
        let denied = TaskEvent { moment: TaskMoment::Denied, capability: Some("muestra.notas_borrar"), reason: Some("access_denied"), ..opened() };
        let closed = TaskEvent { moment: TaskMoment::Closed, outcome: Some("denied"), reason: Some("access_denied"), ..opened() };
        for e in [opened(), step, denied, closed] {
            record_task(&conn, &e).unwrap();
        }
        let rows: Vec<(String, Option<String>, Option<String>, Option<String>)> = conn
            .prepare("SELECT event, subject, outcome, reason FROM kernel_audit WHERE correlation_id = ?1 ORDER BY id")
            .unwrap()
            .query_map([TASK], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let events: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(events, ["task.opened", "task.step", "task.denied", "task.closed"]);
        assert_eq!(rows[2].1.as_deref(), Some("muestra.notas_borrar"));
        assert_eq!((rows[3].2.as_deref(), rows[3].3.as_deref()), (Some("denied"), Some("access_denied")));
    }

    /// What could carry data of a person does not enter: a name where the id goes, or free text where a code goes.
    /// The code fields only take `&'static str`, so a text read at run time does not even compile there.
    #[test]
    fn no_data_of_a_person_enters_the_store() {
        let conn = store();
        let with = |e: TaskEvent| record_task(&conn, &e).map_err(|e| e.to_string());
        assert_eq!(with(TaskEvent { task: "María de la Luz Pérez", ..opened() }), Err("the task of an audit event is not an id".into()));
        assert_eq!(with(TaskEvent { task: "task_maria", ..opened() }), Err("the task of an audit event is not an id".into()));
        assert_eq!(with(TaskEvent { account: Some("PEGM850101MDFRRR09"), ..opened() }), Err("the account of an audit event is not an id".into()));
        assert_eq!(with(TaskEvent { reason: Some("la CURP de María es PEGM850101MDFRRR09"), ..opened() }), Err("the reason of an audit event is not a code".into()));
        assert_eq!(with(TaskEvent { outcome: Some("Rosa"), ..opened() }), Err("the outcome of an audit event is not a code".into()));
        let kept: i64 = conn.query_row("SELECT count(*) FROM kernel_audit", [], |r| r.get(0)).unwrap();
        assert_eq!(kept, 0);
        // an id without a prefix is an id too
        assert!(with(TaskEvent { task: "01JB3M7Q8ZK4X2N6V9R5T0W1YC", account: None, ..opened() }).is_ok());
    }

    #[test]
    fn the_kernel_keeps_to_its_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        assert_eq!(crate::conform("kernel", crate::migrate::KERNEL_MIGRATIONS, &mut conn), []);
    }
}
