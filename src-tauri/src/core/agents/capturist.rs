//! The Capturist (ADR-034 §1 and §4, block IA4): turns what the person tells of their institution into suggestions
//! for the fields of a window of «Mi institución». It never saves: each suggestion waits in `ai_proposal` until the
//! person puts it in its field (and saves) or discards it.
//!
//! - **The code decides what may be suggested:** only the fields of that window the AI may see (never the contact,
//!   the address, the folios, the keys nor the legal representative), each value fitting its field, and only what
//!   the person said: every suggestion carries the words it came from and the code finds them in what the person
//!   wrote, or drops it.
//! - **What the person writes goes through the scanner first,** as every text (quarantine).
//! - The person asks for it from the Assistant («Llenarlo por mí») or from the window; the code runs it, never
//!   another model (one level of delegation).

use crate::ai::agent::{self, Agent};
use crate::ai::pipeline::SqliteLedger;
use crate::ai::{AiProvider, ModelTier};
use crate::common::forms::{is_filled, AiUse, FieldKind, FieldSpec, FormSpec, Values};
use crate::core::agents::assistant::{reason, Turn, HISTORY_TURNS};
use crate::core::agents::proposals::{self, NewProposal, Origin, ProposeOutcome};
use crate::core::error::ServiceError;
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::storage::SharedDb;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Its trade, versioned like a prompt: changing it is a new file.
const TRADE: &str = include_str!("capturist.v1.md");

/// The windows it may fill: (form, the place of the window, its name).
pub const FILLABLE: &[(&str, &str, &str)] = &[
    ("institution.identity", "profile.institution", "Su institución"),
    ("institution.contact", "profile.contact", "Contacto y ubicación"),
    ("institution.legal", "profile.legal", "Datos legales y fiscales"),
    ("institution.capacity", "profile.capacity", "Capacidad y cifras rápidas"),
];

/// The longest text the person may tell at once, in characters.
pub const MAX_TOLD_CHARS: usize = 2_000;
/// The longest saved text shown to the AI as «what is saved».
const SAVED_TEXT_CHARS: usize = 200;

pub fn agent() -> Agent {
    Agent {
        task: "agent.capturist",
        name: "capturist",
        tier: ModelTier::Light,
        trade: TRADE,
        // everything it needs comes in its context: the window, its fields and what the person said
        tools: &[],
        max_tool_calls: 0,
        max_tokens: 30_000,
        max_output_tokens: 1_200,
        answer: json!({
            "type": "object",
            "properties": {
                "proposals": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "field": { "type": "string" },
                            "value": { "anyOf": [{ "type": "string" }, { "type": "integer" }, { "type": "array", "items": { "type": "string" } }] },
                            "quote": { "type": "string" }
                        },
                        "required": ["field", "value", "quote"],
                        "additionalProperties": false
                    }
                },
                "note": { "type": "string" }
            },
            "required": ["proposals", "note"],
            "additionalProperties": false
        }),
        knows_institution: true,
    }
}

/// A window it may fill, by its form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Fill {
    pub form: &'static str,
    pub place: &'static str,
}

pub fn fill_of_form(form: &str) -> Option<Fill> {
    FILLABLE.iter().find(|f| f.0 == form).map(|f| Fill { form: f.0, place: f.1 })
}

pub fn fill_of_place(place: &str) -> Option<Fill> {
    FILLABLE.iter().find(|f| f.1 == place).map(|f| Fill { form: f.0, place: f.1 })
}

/// What the screen sends: the window to fill, what the person tells now and the last turns of the conversation.
#[derive(Debug, Clone, Deserialize)]
pub struct Capture {
    pub form: String,
    pub said: String,
    #[serde(default)]
    pub history: Vec<Turn>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CaptureOutcome {
    /// The suggestions were left in the window (`fields`, by id, in order); `dropped` did not pass the code.
    Proposed { asked: String, form: &'static str, place: &'static str, fields: Vec<String>, dropped: usize, note: String },
    /// The AI could not do it (`reason`, as in the Assistant): nothing was suggested.
    Unavailable { asked: String, reason: &'static str },
    /// What the person wrote has data of a person: nothing was asked.
    Quarantine { report: QuarantineReport },
}

fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("lock".into()))
}

pub(crate) fn title(id: &str) -> String {
    crate::core::manual::entry(id).map_or_else(|| id.to_string(), |e| e.entry.title)
}

/// The fields of a window the AI may suggest.
pub(crate) fn open_fields(spec: &FormSpec) -> impl Iterator<Item = &FieldSpec> {
    spec.fields().filter(|f| f.ai != AiUse::Never)
}

/// How a field is described to the AI: its id, its title, the shape of its value, when it applies and what is saved.
pub(crate) fn describe(f: &FieldSpec, saved: Option<&Value>) -> String {
    let shape = match f.kind {
        FieldKind::Text | FieldKind::Email | FieldKind::Phone => "texto corto".to_string(),
        FieldKind::LongText => "texto".to_string(),
        FieldKind::Number | FieldKind::Money => match (f.min, f.max) {
            (Some(a), Some(b)) => format!("número entero de {a} a {b}"),
            _ => "número entero".to_string(),
        },
        FieldKind::Year => "año (número entero)".to_string(),
        FieldKind::Date => "fecha AAAA-MM-DD".to_string(),
        FieldKind::YesNo => "sí o no".to_string(),
        FieldKind::Select | FieldKind::MultiSelect => {
            let codes: Vec<String> = f
                .options
                .iter()
                .map(|c| match crate::core::institution::catalog::state_name(c) {
                    Some(name) if f.id == "institution.state" => format!("{c} ({name})"),
                    _ => (*c).to_string(),
                })
                .collect();
            let one = if f.kind == FieldKind::Select { "una opción" } else { "varias opciones" };
            format!("{one}: {}", codes.join(", "))
        }
    };
    let when = match f.applies_when {
        crate::common::forms::Condition::Always => String::new(),
        crate::common::forms::Condition::Filled { field } => format!(" Solo aplica si `{field}` tiene algo."),
        crate::common::forms::Condition::AnyOf { field, values } => format!(" Solo aplica si `{field}` es {}.", values.join(" o ")),
    };
    let saved = match saved.filter(|v| is_filled(Some(v))) {
        None => "vacío".to_string(),
        Some(Value::String(s)) if s.chars().count() > SAVED_TEXT_CHARS => format!("«{}…»", s.chars().take(SAVED_TEXT_CHARS).collect::<String>()),
        Some(Value::String(s)) => format!("«{s}»"),
        Some(v) => v.to_string(),
    };
    format!("- `{}` «{}»: {shape}.{when} Guardado: {saved}.", f.id, title(f.id))
}

/// What the Capturist is told besides what the person says now: the window, the fields it may suggest with what
/// is saved, the fields only the person writes, and the last turns of the conversation.
pub fn context(spec: &FormSpec, name: &str, saved: &Values, history: &[Turn]) -> Vec<String> {
    let mut out = vec![format!("La persona quiere llenar la ventana «{name}» de «Mi institución».")];
    let fields: Vec<String> = open_fields(spec).map(|f| describe(f, saved.get(f.id))).collect();
    out.push(format!("Campos que puede sugerir:\n{}", fields.join("\n")));
    let by_hand: Vec<String> = spec.fields().filter(|f| f.ai == AiUse::Never).map(|f| format!("«{}»", title(f.id))).collect();
    if !by_hand.is_empty() {
        out.push(format!("Campos que la persona escribe a mano (no los sugiera): {}.", by_hand.join(", ")));
    }
    let turns: Vec<&Turn> = history.iter().rev().take(HISTORY_TURNS).collect::<Vec<_>>().into_iter().rev().collect();
    if !turns.is_empty() {
        let lines: String = turns
            .iter()
            .map(|t| format!("{}: {}\n", if t.from == "assistant" { "Asistente" } else { "Persona" }, t.text.trim()))
            .collect();
        out.push(format!("La conversación hasta ahora:\n{lines}"));
    }
    out
}

/// Words compared without capitals, accents nor punctuation.
pub(crate) fn plain(s: &str) -> String {
    let folded = crate::scanner::normalize::fold_str(s);
    folded.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" ")
}

/// Whether the person said these words (in what they tell now or in their earlier turns).
fn said_by_the_person(quote: &str, told: &str) -> bool {
    let q = plain(quote);
    !q.is_empty() && plain(told).contains(&q)
}

/// The value in the shape of its field, when the AI gave it in a near one: a number as text, one option alone…
pub(crate) fn fit(f: &FieldSpec, v: &Value) -> Value {
    match (f.kind, v) {
        (FieldKind::Number | FieldKind::Money | FieldKind::Year, Value::String(s)) => {
            let digits: String = s.chars().filter(|c| !matches!(c, ' ' | ',')).collect();
            digits.parse::<i64>().map_or_else(|_| v.clone(), Value::from)
        }
        (FieldKind::MultiSelect, Value::String(s)) => json!([s]),
        (FieldKind::Select, Value::Array(a)) if a.len() == 1 => a[0].clone(),
        (FieldKind::Text | FieldKind::LongText, Value::String(s)) => Value::String(s.trim().to_string()),
        _ => v.clone(),
    }
}

/// Keeps the suggestions the code accepts: a field of this window the AI may see, a value that fits it and is not
/// the one saved, and words the person said. Returns the fields kept and how many were dropped.
pub fn keep(conn: &rusqlite::Connection, spec: &FormSpec, saved: &Values, told: &str, answer: &Value) -> Result<(Vec<String>, usize), ServiceError> {
    let mut kept: Vec<String> = Vec::new();
    let mut dropped = 0;
    for p in answer["proposals"].as_array().into_iter().flatten() {
        let (Some(field), Some(quote)) = (p["field"].as_str(), p["quote"].as_str()) else {
            dropped += 1;
            continue;
        };
        let Some(f) = open_fields(spec).find(|f| f.id == field) else {
            dropped += 1;
            continue;
        };
        let value = fit(f, &p["value"]);
        if saved.get(field) == Some(&value) {
            continue; // already so: nothing to suggest
        }
        if !said_by_the_person(quote, told) {
            dropped += 1;
            continue;
        }
        let new = NewProposal {
            form_id: spec.id.to_string(),
            record_id: None,
            field_id: field.to_string(),
            value,
            origin: Origin::AiAssumption,
            source_ref: Some(json!({ "from": "chat", "quote": quote.trim() }).to_string()),
            agent: "capturist",
        };
        match proposals::propose(conn, new)? {
            ProposeOutcome::Kept { .. } => {
                kept.retain(|k| k != field);
                kept.push(field.to_string());
            }
            ProposeOutcome::Refused { .. } => dropped += 1,
        }
    }
    Ok((kept, dropped))
}

/// Turns what the person tells into suggestions for a window. Nothing of the institution changes.
pub async fn capture(db: &SharedDb, provider: Option<&dyn AiProvider>, c: Capture, decision: Option<Decision>) -> Result<CaptureOutcome, ServiceError> {
    let Some(&(form, place, name)) = FILLABLE.iter().find(|f| f.0 == c.form) else { return Err(ServiceError::NotFound) };
    let spec = crate::core::institution::forms::form(form).ok_or(ServiceError::NotFound)?;
    let said = c.said.trim();
    if said.is_empty() {
        return Err(ServiceError::EmptyText);
    }
    if said.chars().count() > MAX_TOLD_CHARS {
        return Err(ServiceError::TextTooLarge);
    }
    let (asked, scanner, saved) = {
        let conn = lock(db)?;
        let fields = vec![("said".to_string(), said.to_string())];
        match crate::core::screen::guard_texts(&conn, "capturist_said", &fields, decision)? {
            Ok(mut t) => (t.remove(0), crate::core::screen::scanner_for(&conn)?, crate::core::profile::forms::saved_values(&conn, form)?),
            Err(report) => return Ok(CaptureOutcome::Quarantine { report }),
        }
    };
    let Some(provider) = provider else { return Ok(CaptureOutcome::Unavailable { asked, reason: "not_configured" }) };

    let ledger = SqliteLedger(db.clone());
    let errand = agent::Errand { context: context(spec, name, &saved, &c.history), request: asked.clone() };
    let run = match agent::run(provider, &scanner, &ledger, &agent(), &[], errand).await {
        Ok(run) => run,
        Err(e) => return Ok(CaptureOutcome::Unavailable { asked, reason: reason(&e) }),
    };
    // what the person said: now and in their earlier turns
    let told = c.history.iter().filter(|t| t.from != "assistant").map(|t| t.text.as_str()).chain([asked.as_str()]).collect::<Vec<_>>().join("\n");
    let (fields, dropped) = keep(&*lock(db)?, spec, &saved, &told, &run.answer)?;
    let note = run.answer["note"].as_str().unwrap_or_default().trim().to_string();
    Ok(CaptureOutcome::Proposed { asked, form, place, fields, dropped, note })
}

#[cfg(test)]
#[path = "capturist_tests.rs"]
mod tests;
