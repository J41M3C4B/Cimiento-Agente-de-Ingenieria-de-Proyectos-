//! The view of the guided conversation of the diagnosis (ADR-017) that the screen draws. The conversation itself
//! (the questions, the root cause) was made by the AI and no longer runs: it comes back as a capability of the
//! platform. What an earlier conversation left in the base is still read here.

use crate::modules::projects::domain::conversation::{self as conv, Kind, Phase, Role, MAX_WHYS};
use crate::modules::projects::ProjectsError;
use crate::modules::projects::storage::calls;
use crate::modules::projects::storage::projects::{self as store, ProjectRow, StoredSummary, TurnRow};
use rusqlite::Connection;
use serde::Serialize;

// ------------------------------------------------------------------ the view

#[derive(Debug, Clone, Serialize)]
pub struct TurnView {
    pub turn: i64,
    pub role: Role,
    pub kind: Kind,
    pub level: Option<u8>,
    pub text: String,
    pub options: Vec<String>,
}

/// How well the idea fits what the call funds, as the AI judged it when the person answered the opening.
#[derive(Debug, Clone, Serialize)]
pub struct FitView {
    /// `fits`, `partial` or `mismatch`.
    pub fit: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallBrief {
    pub name: String,
    pub funder: Option<String>,
    pub year: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootView {
    pub text: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationView {
    pub project: ProjectRow,
    pub call: Option<CallBrief>,
    pub turns: Vec<TurnView>,
    pub phase: Phase,
    /// The «why» being asked or answered now (0 before the first).
    pub why_level: u8,
    pub max_whys: u8,
    pub fit: Option<FitView>,
    pub root: Option<RootView>,
    pub summary: Option<StoredSummary>,
    /// Numbers in an AI summary that the person never said. Empty when all is supported.
    pub unsupported_figures: Vec<String>,
    /// The project was started with the seven fixed questions of the earlier method and has no conversation.
    pub legacy: bool,
}

pub fn conversation_view(conn: &Connection, project_id: &str) -> Result<ConversationView, ProjectsError> {
    let project = store::get_project(conn, project_id)?.ok_or(ProjectsError::NotFound)?;
    let rows = store::turns(conn, project_id)?;
    let facts: Vec<_> = rows.iter().map(TurnRow::facts).collect();
    let root = store::get_root(conn, project_id)?;
    let phase = conv::phase(&facts, root.as_ref().is_some_and(|r| r.confirmed_at.is_some()));
    let why_level = rows.iter().rev().find(|t| t.role == Role::Assistant && t.kind == Kind::Why).and_then(|t| t.level).unwrap_or(0);
    let fit = rows.iter().find(|t| t.role == Role::Assistant && t.kind == Kind::Why && t.level == Some(1)).and_then(|t| {
        t.record["fit"].as_str().map(|f| FitView { fit: f.to_string(), note: t.record["fit_note"].as_str().unwrap_or("").to_string() })
    });
    let call = match &project.call_reading_id {
        Some(id) => calls::get(conn, id)?.map(|r| CallBrief { name: r.name, funder: r.funder, year: r.year }),
        None => None,
    };
    let summary = store::get_summary(conn, project_id)?;
    // the check of the figures of a summary written by the AI belongs to the platform: it comes back with it
    let unsupported_figures = Vec::new();
    let legacy = rows.is_empty() && store::has_legacy_answers(conn, project_id)?;
    Ok(ConversationView {
        call,
        turns: rows.iter().map(|t| TurnView { turn: t.turn, role: t.role, kind: t.kind, level: t.level, text: t.text.clone(), options: t.options.clone() }).collect(),
        phase,
        why_level,
        max_whys: MAX_WHYS,
        fit,
        root: root.map(|r| RootView { text: r.text, confirmed: r.confirmed_at.is_some() }),
        summary,
        unsupported_figures,
        legacy,
        project,
    })
}
