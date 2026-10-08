//! The guided conversation of the diagnosis (ADR-017): use cases and the view the screen draws.
//!
//! The AI words every message of the conversation; the code (`domain::conversation`) decides which step comes,
//! checks what the AI answered against what the person wrote and keeps the conversation inside its bound.
//! Only with AI: if it fails, what the person wrote stays saved and the screen offers «Reintentar»; nothing is
//! made up in its place.

use crate::ai::{AiProvider, AiTask};
use crate::diagnosis_service::{ask_ai, lock, profile_context, AiStatus, SharedDb};
use crate::core::screen::guard_texts;
use crate::documents::canonical::summary::summarize;
use crate::domain::conversation::{self as conv, Cause, Kind, Outcome, Phase, Reply, Role, Step, Tactic, MAX_WHYS};
use crate::domain::{figures, stage::Stage};
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::core::error::ServiceError;
use crate::storage::calls;
use crate::storage::projects::{self as store, ProjectRow, StoredSummary, TurnRow};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{json, Value};

/// How long what the person writes in one message may be.
const MAX_MESSAGE_BYTES: usize = 6000;
/// How much of the call goes into the context of every turn.
const CALL_CONTEXT_CHARS: usize = 3000;
/// The quick replies of a root cause proposal: the first one confirms it, the second says it is not it.
pub const CONFIRM_ROOT_OPTION: &str = "Sí, es esa";
pub const REJECT_ROOT_OPTION: &str = "No exactamente";

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

/// What the AI is told about the call: the part of the confirmed reading that says what it is for.
pub(crate) fn call_context(conn: &Connection, project: &ProjectRow) -> Result<String, ServiceError> {
    let Some(reading_id) = &project.call_reading_id else { return Ok("Convocatoria: sin lectura.".into()) };
    let ctx = calls::result(conn, reading_id)?.map(|(doc, _)| summarize(&doc).context_text(CALL_CONTEXT_CHARS)).unwrap_or_default();
    Ok(if ctx.is_empty() { "Convocatoria: sin lectura.".into() } else { ctx })
}

/// The conversation as the AI reads it, turn by turn.
pub(crate) fn transcript(turns: &[TurnRow]) -> String {
    turns
        .iter()
        .map(|t| format!("{}: {}", if t.role == Role::Assistant { "Analista" } else { "Persona" }, t.text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// What the person said, in their own words and without the AI's questions: the facts of the conversation, for the
/// stages after it (which would otherwise know it only through a summary the AI wrote). Their plain «yes» to the
/// root cause says nothing by itself and is left out.
pub(crate) fn person_words(turns: &[TurnRow]) -> String {
    turns
        .iter()
        .filter(|t| t.role == Role::Person && t.text.trim() != CONFIRM_ROOT_OPTION)
        .map(|t| format!("- {}", t.text.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Where a figure written by the AI may come from: what the person said, the profile and the call.
pub(crate) fn figure_sources(conn: &Connection, project: &ProjectRow, turns: &[TurnRow]) -> Result<Vec<String>, ServiceError> {
    let mut v = vec![profile_context(conn)?, call_context(conn, project)?];
    v.extend(turns.iter().filter(|t| t.role == Role::Person).map(|t| t.text.clone()));
    Ok(v)
}

pub fn conversation_view(conn: &Connection, project_id: &str) -> Result<ConversationView, ServiceError> {
    let project = store::get_project(conn, project_id)?.ok_or(ServiceError::NotFound)?;
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
    let unsupported_figures = match &summary {
        Some(s) if s.origin == "ai_assumption" => crate::diagnosis_service::check_summary_figures(&s.summary, &figure_sources(conn, &project, &rows)?),
        _ => vec![],
    };
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

// ------------------------------------------------------------------ the turns

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AnswerOutcome {
    Saved { view: ConversationView, ai: AiStatus },
    Quarantine { report: QuarantineReport },
}

fn instruction(step: Step) -> String {
    let tactic = |t: Tactic| match t {
        Tactic::Open => "Táctica: pregunta abierta; `options` va vacío.",
        Tactic::Options => "Táctica: options. La persona está atorada: ofrece de 2 a 4 respuestas cerradas en `options`.",
    };
    match step {
        Step::Opening => "Paso: apertura. Escribe la pregunta de apertura con sus tres partes.".to_string(),
        Step::FirstWhy { tactic: t } => format!("Paso: primer porqué. La persona respondió la apertura: valora `fit` y pregunta el primer «¿por qué?». {}", tactic(t)),
        Step::Why { answered, tactic: t, may_propose, must_propose } => {
            let root = if must_propose {
                "Este es el último porqué: escribe en `root_hypothesis` la causa de fondo que mejor resume lo dicho."
            } else if may_propose {
                "Si ya encontraste la causa de fondo, escríbela en `root_hypothesis`; si no, déjala en null y sigue preguntando."
            } else {
                "Todavía es pronto para proponer la causa de fondo: deja `root_hypothesis` en null."
            };
            format!("Paso: porqué siguiente. La persona respondió el porqué número {answered}. {} {root}", tactic(t))
        }
        Step::Reconsider => "Paso: proponer de nuevo. La persona dijo que la causa propuesta no es esa y explicó qué es distinto: escribe en `root_hypothesis` una nueva propuesta con sus palabras.".to_string(),
        Step::TakeTheirWords => String::new(),
    }
}

fn parse_reply(v: &Value, person_text: &str) -> Reply {
    let cause = v["cause"].as_object().and_then(|c| {
        let text = c.get("text")?.as_str()?.trim().to_string();
        let quote = c.get("quote").and_then(Value::as_str).unwrap_or("").to_string();
        (!text.is_empty()).then(|| Cause { verified: conv::quote_found(person_text, &quote), quote, text })
    });
    Reply {
        message: v["message"].as_str().unwrap_or("").trim().to_string(),
        cause,
        root_hypothesis: v["root_hypothesis"].as_str().map(str::to_string),
        options: v["options"].as_array().into_iter().flatten().filter_map(|o| o.as_str().map(str::to_string)).collect(),
    }
}

fn proposal_message(hypothesis: &str) -> String {
    format!("Entonces la causa de fondo parece ser: «{hypothesis}». ¿Es así?")
}

fn saved(conn: &Connection, project_id: &str, ai: AiStatus) -> Result<AnswerOutcome, ServiceError> {
    Ok(AnswerOutcome::Saved { view: conversation_view(conn, project_id)?, ai })
}

/// The latest cause of the conversation that the person really said (its quote was found in their text).
fn last_verified_cause(turns: &[TurnRow]) -> Option<String> {
    turns
        .iter()
        .rev()
        .filter(|t| t.role == Role::Assistant)
        .find(|t| t.record["cause"]["verified"].as_bool() == Some(true))
        .and_then(|t| t.record["cause"]["text"].as_str().map(str::to_string))
}

/// Does what is owed after the turns stored so far: asks the AI for the next message, or closes the
/// conversation with the person's own words. Safe to call again (a retry): it only acts if something is owed.
async fn advance(db: &SharedDb, provider: Option<&dyn AiProvider>, project_id: &str) -> Result<AnswerOutcome, ServiceError> {
    // 1. what is owed
    let (step, project, rows, person_text) = {
        let conn = lock(db)?;
        let project = store::get_project(&conn, project_id)?.ok_or(ServiceError::NotFound)?;
        if project.stage != Stage::Diagnosis {
            return Err(ServiceError::WrongStage);
        }
        let rows = store::turns(&conn, project_id)?;
        let facts: Vec<_> = rows.iter().map(TurnRow::facts).collect();
        let Some(step) = conv::step_after(&facts) else { return saved(&conn, project_id, AiStatus::Skipped) };
        let person_text = rows.iter().rev().find(|t| t.role == Role::Person).map(|t| t.text.clone()).unwrap_or_default();
        (step, project, rows, person_text)
    };

    // the person turned the proposal down twice: their own words are the root cause, no AI needed
    if step == Step::TakeTheirWords {
        let conn = lock(db)?;
        store::set_root(&conn, project_id, &person_text, None)?;
        store::confirm_root(&conn, project_id)?;
        let message = format!("Quedó como usted la dijo: «{}».", person_text.trim());
        store::add_turn(&conn, project_id, Role::Assistant, Kind::RootProposal, None, &message, &[], &json!({ "own_words": true }))?;
        return saved(&conn, project_id, AiStatus::Skipped);
    }

    // 2. the context: the profile, the confirmed call and the conversation so far
    let (context, sources) = {
        let conn = lock(db)?;
        let sources = figure_sources(&conn, &project, &rows)?;
        let context = vec![
            format!("Perfil de la institución:\n{}", profile_context(&conn)?),
            format!("Convocatoria que la persona confirmó:\n{}", call_context(&conn, &project)?),
            format!("Conversación hasta ahora:\n{}", if rows.is_empty() { "(todavía no empieza)".to_string() } else { transcript(&rows) }),
        ];
        (context, sources)
    };

    // 3. ask; if the AI wrote a figure nobody said, ask once more
    let mut user = instruction(step);
    let mut figures_unsupported = Vec::new();
    let mut attempt = 0;
    let reply = loop {
        match ask_ai(db, provider, AiTask::ConversationTurn, project_id, context.clone(), user.clone()).await {
            Ok(v) => {
                let reply = parse_reply(&v, &person_text);
                let refs: Vec<&str> = sources.iter().map(String::as_str).collect();
                let bad = figures::unsupported_figures(&[reply.message.as_str()], &refs);
                if !bad.is_empty() && attempt == 0 {
                    attempt += 1;
                    user = format!("{}\n\nEstas cifras no las dijo la persona: {}. No las uses.", instruction(step), bad.join(", "));
                    continue;
                }
                figures_unsupported = bad;
                break Ok((reply, v));
            }
            Err(e) => break Err(e),
        }
    };
    let conn = lock(db)?;
    let (reply, raw) = match reply {
        Ok(r) => r,
        Err(e) => return saved(&conn, project_id, AiStatus::from(&e)), // the person's message is already saved
    };

    // 4. the code decides what the reply means
    let last_cause = last_verified_cause(&rows);
    let cause_json = reply.cause.as_ref().map(|c| json!({ "text": c.text, "quote": c.quote, "verified": c.verified }));
    let mut record = json!({ "cause": cause_json, "unsupported_figures": figures_unsupported, "prompt": crate::ai::prompts::CONVERSATION_TURN_VERSION });
    if matches!(step, Step::FirstWhy { .. }) {
        record["fit"] = raw["fit"].clone();
        record["fit_note"] = raw["fit_note"].clone();
    }
    match conv::decide(step, &reply, last_cause.as_deref(), &person_text) {
        Outcome::Ask { level, message, options } => {
            if message.trim().is_empty() {
                return saved(&conn, project_id, AiStatus::Unavailable);
            }
            let (kind, level) = if level == 0 { (Kind::Opening, None) } else { (Kind::Why, Some(level)) };
            store::add_turn(&conn, project_id, Role::Assistant, kind, level, &message, &options, &record)?;
        }
        Outcome::Propose { hypothesis, forced } => {
            if hypothesis.trim().is_empty() {
                return saved(&conn, project_id, AiStatus::Unavailable);
            }
            store::set_root(&conn, project_id, &hypothesis, reply.cause.as_ref().filter(|c| c.verified).map(|c| c.quote.as_str()))?;
            record["forced"] = json!(forced);
            let options = [CONFIRM_ROOT_OPTION.to_string(), REJECT_ROOT_OPTION.to_string()];
            store::add_turn(&conn, project_id, Role::Assistant, Kind::RootProposal, None, &proposal_message(&hypothesis), &options, &record)?;
        }
    }
    saved(&conn, project_id, AiStatus::Used)
}

/// Starts the conversation: asks the AI for the opening question. Idempotent: if the opening exists, nothing
/// is asked (and nothing is billed) again.
pub async fn start_conversation(db: &SharedDb, provider: Option<&dyn AiProvider>, project_id: &str) -> Result<AnswerOutcome, ServiceError> {
    {
        let conn = lock(db)?;
        let view = conversation_view(&conn, project_id)?;
        if view.project.stage != Stage::Diagnosis {
            return Err(ServiceError::WrongStage);
        }
        if view.phase != Phase::NeedsOpening {
            return Ok(AnswerOutcome::Saved { view, ai: AiStatus::Skipped });
        }
    }
    advance(db, provider, project_id).await
}

/// The AI owes a message (it failed or was interrupted): asks again without the person writing anything.
pub async fn retry(db: &SharedDb, provider: Option<&dyn AiProvider>, project_id: &str) -> Result<AnswerOutcome, ServiceError> {
    advance(db, provider, project_id).await
}

/// What the person writes (or, with `confirm_root`, the quick reply that confirms the root cause). Their message
/// is saved first; then the AI answers. Free text goes through the scanner like any other.
pub async fn send_message(
    db: &SharedDb,
    provider: Option<&dyn AiProvider>,
    project_id: &str,
    text: &str,
    confirm_root: bool,
    decision: Option<Decision>,
) -> Result<AnswerOutcome, ServiceError> {
    {
        let conn = lock(db)?;
        let view = conversation_view(&conn, project_id)?;
        if view.project.stage != Stage::Diagnosis {
            return Err(ServiceError::WrongStage);
        }
        let none: [String; 0] = [];
        match (view.phase, confirm_root) {
            // the person confirms: only an explicit «sí» closes it, and no AI is needed
            (Phase::RootProposed, true) => {
                store::add_turn(&conn, project_id, Role::Person, Kind::RootReply, None, CONFIRM_ROOT_OPTION, &none, &json!({ "confirm": true }))?;
                store::confirm_root(&conn, project_id)?;
                return saved(&conn, project_id, AiStatus::Skipped);
            }
            (Phase::AwaitingAnswer, false) | (Phase::RootProposed, false) => {}
            _ => return Err(ServiceError::WrongStage),
        }
        if text.trim().is_empty() {
            return Err(ServiceError::EmptyText);
        }
        if text.len() > MAX_MESSAGE_BYTES {
            return Err(ServiceError::TextTooLarge);
        }
        let fields = vec![("answer".to_string(), text.to_string())];
        let clean = match guard_texts(&conn, "conversation_answer", &fields, decision)? {
            Ok(mut t) => t.remove(0),
            Err(report) => return Ok(AnswerOutcome::Quarantine { report }),
        };
        // what is it answering? the opening, a «why», or the proposal of the root cause
        let last = view.turns.last().ok_or(ServiceError::WrongStage)?;
        let (kind, level) = if last.kind == Kind::RootProposal { (Kind::RootReply, None) } else { (last.kind, last.level) };
        store::add_turn(&conn, project_id, Role::Person, kind, level, &clean, &none, &json!({}))?;
    }
    advance(db, provider, project_id).await
}

#[cfg(test)]
#[path = "conversation_service_tests.rs"]
mod tests;
