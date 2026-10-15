//! The Capturist (ADR-034 §1 and §4, block IA4): turns what the person tells of their institution into suggestions
//! for the fields of a window of «Mi institución». It never saves: each suggestion waits in `ai_proposal` until the
//! person puts it in its field (and saves) or discards it.
//!
//! - **The code decides what may be suggested:** the fields of that window the AI may see and, since ADR-041, the
//!   institution's own data the person tells (contact, address, RFC, folios, keys, legal representative), each value
//!   fitting its field, and only what the person said: every suggestion carries the words it came from and the code
//!   finds them in what the person wrote, with a figure that is in those words (it never counts), or drops it.
//! - **The institution's own data are copied, never made:** their letters and digits are the person's, one by one,
//!   and a date is one the person wrote. What is saved of them is never shown to the AI.
//! - **What the person writes goes through the scanner first,** as every text (quarantine).
//! - The person asks for it from the Assistant («Llenarlo por mí») or from the window; the code runs it, never
//!   another model (one level of delegation).

use crate::ai::agent::{self, Agent};
use crate::ai::pipeline::SqliteLedger;
use crate::ai::{AiProvider, ModelTier};
use crate::common::forms::{is_filled, AiUse, FieldKind, FieldSpec, FormSpec, Sensitivity, Values};
use crate::core::agents::assistant::{reason, Turn, HISTORY_TURNS};
use crate::core::agents::proposals::{self, NewProposal, Origin, ProposeOutcome};
use crate::core::error::ServiceError;
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::storage::SharedDb;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Its trade, versioned like a prompt: changing it is a new file.
const TRADE: &str = include_str!("capturist.v4.md");

/// The windows it may fill: (form, the place of the window, its name).
pub const FILLABLE: &[(&str, &str, &str)] = &[
    ("institution.identity", "profile.institution", "Su institución"),
    ("institution.contact", "profile.contact", "Contacto y ubicación"),
    ("institution.legal", "profile.legal", "Datos legales y fiscales"),
    ("institution.capacity", "profile.capacity", "Capacidad y cifras rápidas"),
];

/// The agent that signs its proposals.
pub const AGENT: &str = "capturist";

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
                }
            },
            "required": ["proposals"],
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
    /// The suggestions were left in the window (`fields`, by id, in order, and `left`, by their titles in the manual);
    /// `dropped` did not pass the code; `still` are the titles of the fields of the window the person may still tell.
    /// The words for the person are the program's, never the AI's: they say what the code kept.
    Proposed { asked: String, form: &'static str, place: &'static str, fields: Vec<String>, left: Vec<String>, still: Vec<String>, dropped: usize },
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

/// The fields the Capturist may suggest from what the person tells: those the AI may see and the institution's own
/// (ADR-041), copied from the person's words. The other agents keep to `open_fields`.
pub(crate) fn told_fields(spec: &FormSpec) -> impl Iterator<Item = &FieldSpec> {
    spec.fields().filter(|f| f.ai != AiUse::Never || own(f))
}

/// The institution's own, protected datum: its contact, address, RFC, folios, keys and representative.
fn own(f: &FieldSpec) -> bool {
    f.sensitivity == Sensitivity::InstitutionalPrivate
}

/// Letters and digits only, without capitals nor accents: «33 1234-5678» and «3312345678» are the same.
fn letters(s: &str) -> String {
    crate::scanner::normalize::fold_str(s).chars().filter(|c| c.is_alphanumeric()).collect()
}

/// An own datum is copied from the person's words: its letters and digits are in them, one by one, and a date is one
/// the person wrote («18 de enero de 2027»).
fn copied(f: &FieldSpec, value: &Value, quote: &str) -> bool {
    match (f.kind, value) {
        (FieldKind::Date, Value::String(d)) => crate::core::reader::dates::dates_in(quote).iter().any(|(_, said)| said == d),
        (_, Value::String(s)) => {
            let v = letters(s);
            !v.is_empty() && letters(quote).contains(&v)
        }
        _ => false,
    }
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
                // each code with its name on the screen: a document says «ASISTENCIAL», not `assistance`
                .map(|c| match crate::core::institution::catalog::state_name(c).filter(|_| f.id == "institution.state") {
                    Some(name) => format!("{c} ({name})"),
                    None => match crate::core::institution::forms::option_label(f.id, c) {
                        Some(label) => format!("{c} ({label})"),
                        None => (*c).to_string(),
                    },
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
        // what is saved of a protected datum never reaches the AI, only that there is one
        Some(_) if f.ai == AiUse::Never => "lleno".to_string(),
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
    // the «Qué poner» of the manual with each field was measured (C4): worse, 24 of 28 against 27 of 28 with the titles
    let fields: Vec<String> = told_fields(spec).map(|f| describe(f, saved.get(f.id))).collect();
    out.push(format!("Campos que puede sugerir:\n{}", fields.join("\n")));
    let by_hand: Vec<String> = spec.fields().filter(|f| !told_fields(spec).any(|t| t.id == f.id)).map(|f| format!("«{}»", title(f.id))).collect();
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

/// Words compared without capitals, accents nor punctuation (the batteries grade with it).
#[cfg(test)]
pub(crate) fn plain(s: &str) -> String {
    let folded = crate::scanner::normalize::fold_str(s);
    folded.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" ")
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
        let Some(f) = told_fields(spec).find(|f| f.id == field) else {
            dropped += 1;
            continue;
        };
        let value = fit(f, &p["value"]);
        if saved.get(field) == Some(&value) {
            continue; // already so: nothing to suggest
        }
        // the words the person said (now or in their earlier turns), never the AI's copy of them
        let Some(quote) = crate::core::reader::quote::find(quote, told) else {
            dropped += 1;
            continue;
        };
        let quote = quote.as_str();
        // the AI does not count: a number is in the words it copied (principle 2)
        if matches!(f.kind, FieldKind::Number | FieldKind::Money | FieldKind::Year) && !value.as_i64().is_some_and(|n| crate::core::reader::fill::says_number(quote, n)) {
            dropped += 1;
            continue;
        }
        // an own datum (an RFC, a phone, a folio, a date) is copied, never completed nor corrected (ADR-041)
        if (own(f) || f.kind == FieldKind::Date) && !copied(f, &value, quote) {
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
            agent: AGENT,
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
            // «No es dato personal» (the institution's own phone or RFC): what the person cleared reaches the AI
            Ok(mut t) => (t.remove(0), crate::core::screen::scanner_clearing(&conn, &[said], decision)?, crate::core::profile::forms::saved_values(&conn, form)?),
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
    let conn = lock(db)?;
    let (fields, dropped) = keep(&conn, spec, &saved, &told, &run.answer)?;
    let left = fields.iter().map(|f| title(f)).collect();
    let still = still_to_tell(&conn, spec, &saved)?;
    Ok(CaptureOutcome::Proposed { asked, form, place, fields, left, still, dropped })
}

/// The titles of the fields of a window the person may still tell: the AI may suggest them, they apply (with what is
/// saved and what waits), nothing is saved in them and no suggestion waits for them.
fn still_to_tell(conn: &rusqlite::Connection, spec: &FormSpec, saved: &Values) -> Result<Vec<String>, ServiceError> {
    let mut known = saved.clone();
    for p in proposals::pending(conn, spec.id, None)? {
        known.entry(p.field_id).or_insert(p.value);
    }
    Ok(told_fields(spec).filter(|f| spec.applies(f, &known) && !is_filled(known.get(f.id))).map(|f| title(f.id)).collect())
}

#[cfg(test)]
#[path = "capturist_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "capturist_battery.rs"]
mod battery;
