//! The tables of the platform (ADR-003 A3): its tasks, their steps, their approvals and their trail. The platform is
//! one more owner of tables for the kernel (K4): its migrations are its own, numbered from 1, and its tables start
//! with `ai_`. The trail keeps ids and fixed codes only; the input of a task lives in the encrypted database like
//! any datum, and never in the trail.

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use super::outcome::Outcome;
use super::request::Channel;
use super::task::{Pause, Phase, Task, TaskState, Waiting};

/// The owner of the tables of the platform, for the kernel.
pub const OWNER: &str = "ai";

pub const MIGRATIONS: &[kernel::Migration] = &[kernel::Migration {
    version: 1,
    sql: "
CREATE TABLE ai_task (
    id TEXT PRIMARY KEY,
    organization TEXT NOT NULL,
    account TEXT NOT NULL,
    channel TEXT NOT NULL CHECK (channel IN ('chat', 'screen', 'event')),
    channel_detail TEXT,
    state TEXT NOT NULL CHECK (state IN ('running', 'paused', 'done', 'denied', 'limit', 'error')),
    pause TEXT CHECK (pause IN ('person', 'approval')),
    reason TEXT,
    steps INTEGER NOT NULL DEFAULT 0,
    corrections INTEGER NOT NULL DEFAULT 0,
    spent_mxn REAL NOT NULL DEFAULT 0,
    reserved_mxn REAL NOT NULL DEFAULT 0,
    opened_ms INTEGER NOT NULL,
    turn_ms INTEGER NOT NULL,
    pause_until_ms INTEGER,
    closed_ms INTEGER,
    waiting_json TEXT,
    last_json TEXT
);
CREATE TABLE ai_step (
    task_id TEXT NOT NULL REFERENCES ai_task(id),
    n INTEGER NOT NULL,
    capability TEXT NOT NULL,
    phase TEXT NOT NULL CHECK (phase IN ('read', 'prepare', 'apply')),
    input_hash TEXT NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('started', 'ok', 'invalid', 'needs_person', 'failed', 'uncertain')),
    started_ms INTEGER NOT NULL,
    ended_ms INTEGER,
    PRIMARY KEY (task_id, n)
);
CREATE TABLE ai_approval (
    task_id TEXT NOT NULL REFERENCES ai_task(id),
    fingerprint TEXT NOT NULL,
    capability TEXT NOT NULL,
    input_json TEXT NOT NULL,
    proposal_json TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('pending', 'approved', 'rejected', 'stale')),
    asked_ms INTEGER NOT NULL,
    decided_ms INTEGER,
    decided_by TEXT,
    PRIMARY KEY (task_id, fingerprint)
);
CREATE TABLE ai_event (
    task_id TEXT NOT NULL REFERENCES ai_task(id),
    at_ms INTEGER NOT NULL,
    event TEXT NOT NULL CHECK (event IN ('open', 'step', 'denied', 'approval', 'pause', 'close')),
    capability TEXT,
    code TEXT
);
CREATE INDEX ai_event_task ON ai_event (task_id, at_ms);
",
}];

/// Applies the migrations of the platform that the database does not have yet.
pub fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    kernel::migrate_owner(conn, OWNER, MIGRATIONS)
}

fn to_json<T: serde::Serialize>(v: &T) -> rusqlite::Result<String> {
    serde_json::to_string(v).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}

fn from_json<T: serde::de::DeserializeOwned>(s: &str) -> rusqlite::Result<T> {
    serde_json::from_str(s).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
}

fn pause_str(p: Pause) -> &'static str {
    match p {
        Pause::Person => "person",
        Pause::Approval => "approval",
    }
}

pub fn insert_task(conn: &Connection, t: &Task) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_task (id, organization, account, channel, channel_detail, state, opened_ms, turn_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![t.id, t.organization, t.account, t.channel.as_str(), t.channel.detail(), t.state.as_str(), t.opened_ms, t.turn_ms],
    )?;
    Ok(())
}

/// Writes the task as it is now, with the last outcome it handed over.
pub fn save_task(conn: &Connection, t: &Task, last: &Outcome) -> rusqlite::Result<()> {
    let waiting = t.waiting.as_ref().map(to_json).transpose()?;
    conn.execute(
        "UPDATE ai_task SET state = ?2, pause = ?3, reason = ?4, steps = ?5, corrections = ?6, spent_mxn = ?7,
                reserved_mxn = ?8, turn_ms = ?9, pause_until_ms = ?10, closed_ms = ?11, waiting_json = ?12, last_json = ?13
         WHERE id = ?1",
        params![
            t.id,
            t.state.as_str(),
            t.pause.map(pause_str),
            t.reason,
            t.steps,
            t.corrections,
            t.spent_mxn,
            t.reserved_mxn,
            t.turn_ms,
            t.pause_until_ms,
            t.closed_ms,
            waiting,
            to_json(last)?,
        ],
    )?;
    Ok(())
}

pub fn load_task(conn: &Connection, id: &str) -> rusqlite::Result<Option<Task>> {
    conn.query_row(
        "SELECT id, organization, account, channel, channel_detail, state, pause, reason, steps, corrections, spent_mxn,
                reserved_mxn, opened_ms, turn_ms, pause_until_ms, closed_ms, waiting_json
         FROM ai_task WHERE id = ?1",
        [id],
        |r| {
            let bad = |i: usize| rusqlite::Error::InvalidColumnType(i, "value".into(), rusqlite::types::Type::Text);
            let channel: String = r.get(3)?;
            let state: String = r.get(5)?;
            let pause: Option<String> = r.get(6)?;
            let waiting: Option<String> = r.get(16)?;
            Ok(Task {
                id: r.get(0)?,
                organization: r.get(1)?,
                account: r.get(2)?,
                channel: Channel::from_parts(&channel, r.get(4)?).ok_or_else(|| bad(3))?,
                state: TaskState::parse(&state).ok_or_else(|| bad(5))?,
                pause: match pause.as_deref() {
                    None => None,
                    Some("person") => Some(Pause::Person),
                    Some("approval") => Some(Pause::Approval),
                    Some(_) => return Err(bad(6)),
                },
                reason: r.get(7)?,
                steps: r.get(8)?,
                corrections: r.get(9)?,
                spent_mxn: r.get(10)?,
                reserved_mxn: r.get(11)?,
                opened_ms: r.get(12)?,
                turn_ms: r.get(13)?,
                pause_until_ms: r.get(14)?,
                closed_ms: r.get(15)?,
                waiting: waiting.as_deref().map(from_json::<Waiting>).transpose()?,
            })
        },
    )
    .optional()
}

pub fn last_outcome(conn: &Connection, id: &str) -> rusqlite::Result<Option<Outcome>> {
    let last: Option<Option<String>> = conn.query_row("SELECT last_json FROM ai_task WHERE id = ?1", [id], |r| r.get(0)).optional()?;
    last.flatten().as_deref().map(from_json).transpose()
}

// ------------------------------------------------------------------ steps

pub fn start_step(conn: &Connection, task: &str, n: u32, capability: &str, phase: Phase, hash: &str, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_step (task_id, n, capability, phase, input_hash, result, started_ms) VALUES (?1, ?2, ?3, ?4, ?5, 'started', ?6)",
        params![task, n, capability, phase.as_str(), hash, now],
    )?;
    Ok(())
}

pub fn end_step(conn: &Connection, task: &str, n: u32, result: &'static str, now: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE ai_step SET result = ?3, ended_ms = ?4 WHERE task_id = ?1 AND n = ?2", params![task, n, result, now])?;
    Ok(())
}

/// Whether the task already ran this capability, in this phase, with this same input.
pub fn seen(conn: &Connection, task: &str, capability: &str, phase: Phase, hash: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM ai_step WHERE task_id = ?1 AND capability = ?2 AND phase = ?3 AND input_hash = ?4)",
        params![task, capability, phase.as_str(), hash],
        |r| r.get(0),
    )
}

/// An `apply` that started and never said how it ended: the app closed or failed while it ran.
pub fn unfinished_apply(conn: &Connection, task: &str) -> rusqlite::Result<Option<(u32, String)>> {
    conn.query_row(
        "SELECT n, capability FROM ai_step WHERE task_id = ?1 AND phase = 'apply' AND result = 'started' ORDER BY n DESC LIMIT 1",
        [task],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

// ------------------------------------------------------------------ approvals

/// Asks for an approval. Any other one still waiting in the task is stale now: its input is no longer the one.
pub fn ask_approval(conn: &Connection, task: &str, fingerprint: &str, capability: &str, input: &Value, proposal: &Value, now: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE ai_approval SET state = 'stale' WHERE task_id = ?1 AND state = 'pending'", [task])?;
    conn.execute(
        "INSERT OR REPLACE INTO ai_approval (task_id, fingerprint, capability, input_json, proposal_json, state, asked_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
        params![task, fingerprint, capability, to_json(input)?, to_json(proposal)?, now],
    )?;
    Ok(())
}

/// The approval the task waits for.
pub struct Pending {
    pub fingerprint: String,
    pub capability: String,
    pub input: Value,
    pub proposal: Value,
}

pub fn pending_approval(conn: &Connection, task: &str) -> rusqlite::Result<Option<Pending>> {
    let row: Option<(String, String, String, String)> = conn
        .query_row(
            "SELECT fingerprint, capability, input_json, proposal_json FROM ai_approval WHERE task_id = ?1 AND state = 'pending'",
            [task],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    row.map(|(fingerprint, capability, input, proposal)| {
        Ok(Pending { fingerprint, capability, input: from_json(&input)?, proposal: from_json(&proposal)? })
    })
    .transpose()
}

pub fn decide_approval(conn: &Connection, task: &str, fingerprint: &str, approved: bool, by: &str, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_approval SET state = ?3, decided_ms = ?4, decided_by = ?5 WHERE task_id = ?1 AND fingerprint = ?2",
        params![task, fingerprint, if approved { "approved" } else { "rejected" }, now, by],
    )?;
    Ok(())
}

// ------------------------------------------------------------------ the trail (P12)

/// One line of the trail of a task. The code is fixed text of the program, never a datum.
pub fn event(conn: &Connection, task: &str, event: &'static str, capability: Option<&str>, code: Option<&'static str>, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_event (task_id, at_ms, event, capability, code) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![task, now, event, capability, code],
    )?;
    Ok(())
}
