//! «Llenar con mis documentos» (C2, ADR-036 §4): the Reader of the Expediente with the AI. Its rules already read the
//! documents of fixed format when they are uploaded; this reads what is written in any way (the mission, the services,
//! whom the institution serves, its capacity, its purpose) in every document of the Expediente:
//!
//! - **a search per field** (words, and meaning when there are vectors, IA5): the passages that speak of each empty
//!   field of the window, never whole documents;
//! - **one call to the AI per window,** with the cheapest model: a value per field and the exact words of the passage;
//! - **the code checks** that the field is one asked for, that the words are in that passage, that a figure is in them
//!   and that the value fits; what passes waits as a suggestion with its document and page, under the one policy of
//!   ADR-038 (never over what is written, never what was rejected). Nothing is saved without the person.
//!
//! The protected fields (contact, address, RFC, folios, keys, representative) are never asked: only the rules read
//! them, before the scanner covers them.

use crate::ai::agent::{self, Agent};
use crate::ai::pipeline::SqliteLedger;
use crate::ai::{AiProvider, ModelTier};
use crate::common::forms::{is_filled, FieldKind, FieldSpec, FormSpec, Values};
use crate::core::agents::assistant::reason;
use crate::core::agents::capturist::{describe, fit, open_fields, title, FILLABLE};
use crate::core::agents::proposals::{self, NewProposal, Origin, ProposeOutcome};
use crate::core::archive::storage::{self as docs, Drawer};
use crate::core::archive::{search, vectors};
use crate::core::error::ServiceError;
use crate::documents::canonical::retrieve::Embedder;
use crate::storage::SharedDb;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;

/// Its trade, versioned like a prompt: changing it is a new file.
const TRADE: &str = include_str!("../agents/reader.v2.md");

/// The passages kept for each field.
pub(crate) const PER_FIELD: usize = 3;
/// How much of a fragment a passage shows, around the words of its field.
pub(crate) const PASSAGE_CHARS: usize = 900;

/// The words each field is looked for with in the documents, besides its title in the manual: how documents write it.
/// Every field the AI may see in the windows of «Mi institución» has its line (a test says so).
pub const SEARCH: &[(&str, &str)] = &[
    ("institution.name", "nombre comercial denominación institución casa"),
    ("institution.mission", "misión objeto brindar atención finalidad giro dedica atend"),
    ("institution.populations", "personas adultas mayores niñas niños adolescentes jóvenes atendemos residentes edad"),
    ("institution.sex_served", "cualquier sexo mujeres hombres varones"),
    ("institution.modalities", "estancia permanente residentes estancia de día albergue internado domicilio"),
    ("institution.care_areas", "alimentación atención médica salud terapia cuidado discapacidad educación"),
    ("institution.services", "servicios terapia física talleres alimentación enfermería atención"),
    ("institution.age_min", "años o más edad mínima mayores de ingresar cumplidos"),
    ("institution.age_max", "edad máxima hasta años"),
    ("institution.admission_criteria", "ingreso requisitos podrán ingresar se admitirán solicitud valoración"),
    ("institution.municipality", "municipio alcaldía demarcación territorial domicilio"),
    ("institution.state", "entidad federativa estado domicilio"),
    ("institution.legal_name", "denominación razón social se denominará"),
    ("institution.purpose", "objeto social tendrá por objeto fines"),
    ("institution.legal_form", "institución de asistencia privada asociación civil régimen capital"),
    ("institution.founded_year", "constituir escritura fecha inicio de operaciones fundación"),
    ("institution.tax_regime", "régimen personas morales fines no lucrativos régimen fiscal"),
    ("institution.authorized_donee", "donataria autorizada donativos deducibles autoriza"),
    ("institution.cluni", "CLUNI clave única de inscripción registro federal organizaciones sociedad civil"),
    ("institution.donee_category", "rubro asistencial autorización donativos actividad"),
    ("institution.capacity_total", "capacidad camas lugares plazas residentes atendimos"),
    ("institution.served_estimate", "atendimos personas residentes estancia de día beneficiarias"),
    ("institution.staff_paid_estimate", "personas contratadas personal equipo nómina empleados"),
    ("institution.staff_volunteer_estimate", "personas voluntarias voluntariado"),
    ("institution.notes", "retos necesidades lista de espera"),
];

pub fn agent() -> Agent {
    Agent {
        task: "agent.reader",
        name: "reader",
        tier: ModelTier::Light,
        trade: TRADE,
        // everything it needs comes in its context: the window, its empty fields and the passages
        tools: &[],
        max_tool_calls: 0,
        max_tokens: 40_000,
        max_output_tokens: 1_500,
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
                            "passage": { "type": "string" },
                            "quote": { "type": "string" }
                        },
                        "required": ["field", "value", "passage", "quote"],
                        "additionalProperties": false
                    }
                },
                "note": { "type": "string" }
            },
            "required": ["proposals", "note"],
            "additionalProperties": false
        }),
        knows_institution: false,
    }
}

/// A piece of a document shown to the AI, with what the code knows of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Passage {
    /// `P1`, `P2`…: how the AI cites it.
    pub id: String,
    pub document_id: String,
    /// The name of its file: only for the screen, never for the AI.
    pub document: String,
    /// What the AI knows it by: its type (`records::ai_name`).
    pub name: String,
    pub page: Option<i64>,
    /// What the document is (`doc_type`), for the say it has on a field.
    pub kind: Option<String>,
    pub text: String,
}

/// How a window was filled from the documents.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FillOutcome {
    /// Suggestions left in the window (`fields`, by id, in order); `dropped` did not pass the code.
    Proposed { form: &'static str, place: &'static str, fields: Vec<String>, dropped: usize, note: String },
    /// Nothing was asked: `all_written` (every field it may fill has a value), `no_documents` (the Expediente has no
    /// text), `nothing_found` (no passage speaks of the empty fields).
    Nothing { form: &'static str, place: &'static str, reason: &'static str },
    /// The AI could not do it (`reason`, as in the Assistant).
    Unavailable { form: &'static str, place: &'static str, reason: &'static str },
}

/// What a field is looked for with: its title in the manual and the words documents use for it.
pub fn query_of(field: &str) -> String {
    let words = SEARCH.iter().find(|(f, _)| *f == field).map_or("", |(_, w)| *w);
    format!("{} {words}", title(field))
}

/// The fields of a window to ask for: the ones the AI may see that are still empty (ADR-038: a written field gets
/// nothing).
pub fn asked_fields<'a>(spec: &'a FormSpec, saved: &Values) -> Vec<&'a FieldSpec> {
    open_fields(spec).filter(|f| !is_filled(saved.get(f.id))).collect()
}

/// The passages of the Expediente for these fields: up to `PER_FIELD` each, by words and, when there are vectors, by
/// meaning (`vectors_of`, one per field in the same order), each fragment once.
pub fn passages_for(conn: &rusqlite::Connection, fields: &[&FieldSpec], vectors_of: &[Option<Vec<f32>>]) -> Result<Vec<Passage>, ServiceError> {
    let mut out: Vec<Passage> = Vec::new();
    let mut seen: Vec<i64> = Vec::new();
    for (i, f) in fields.iter().enumerate() {
        let query = query_of(f.id);
        let vector = vectors_of.get(i).and_then(|v| v.as_deref());
        for p in search::search(conn, Drawer::Record, &query, vector, PER_FIELD, PASSAGE_CHARS)? {
            if seen.contains(&p.chunk) {
                continue;
            }
            seen.push(p.chunk);
            out.push(Passage {
                id: format!("P{}", out.len() + 1),
                kind: docs::doc_type_of(conn, &p.document_id)?,
                name: crate::core::records::ai_name(conn, &p.document_id)?,
                document_id: p.document_id,
                document: p.document,
                page: p.page,
                text: p.snippet,
            });
        }
    }
    Ok(out)
}

/// What the Reader is told: the window, its empty fields and the passages, numbered, each with its type and page
/// (never the name of a file).
pub fn context(name: &str, fields: &[&FieldSpec], passages: &[Passage]) -> Vec<String> {
    let described: Vec<String> = fields.iter().map(|f| describe(f, None)).collect();
    let shown: Vec<String> = passages
        .iter()
        .map(|p| format!("[{}] {}{}:\n{}", p.id, p.name, p.page.map_or(String::new(), |n| format!(", página {n}")), p.text))
        .collect();
    vec![
        format!("La ventana «{name}» de «Mi institución». Sus campos vacíos:\n{}", described.join("\n")),
        format!("Pasajes de los documentos del Expediente:\n\n{}", shown.join("\n\n")),
    ]
}

/// Whether a number is in a text, written with figures or in words («cuarenta y ocho», «dos mil quince»). «Un»,
/// «una» and «primer» alone are not a count: «voluntarios de una parroquia» does not say 1 (C4); «uno» and «treinta
/// y una» do.
pub(crate) fn says_number(text: &str, n: i64) -> bool {
    if crate::ai::figures::digit_numbers(text).contains(&n.to_string()) {
        return true;
    }
    let folded = crate::storage::fts::fold(text);
    let words: Vec<&str> = folded.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    if n == 0 {
        return words.contains(&"cero");
    }
    (0..words.len())
        .filter(|&i| !matches!(words[i], "un" | "una" | "primer" | "primero"))
        .any(|i| super::dates::leading_number_in_words(&words[i..].join(" ")) == Some(n as u32))
}

/// Keeps the suggestions the code accepts, each in the window of its field: a field asked for, a passage shown, words that are in that passage, a
/// figure that is in those words, and a value that fits its field (`proposals::propose`, with the policy of ADR-038).
/// Returns the fields kept and how many were dropped.
pub fn keep(conn: &rusqlite::Connection, asked: &[&FieldSpec], passages: &[Passage], answer: &Value) -> Result<(Vec<String>, usize), ServiceError> {
    let mut kept: Vec<String> = Vec::new();
    let mut dropped = 0;
    for p in answer["proposals"].as_array().into_iter().flatten() {
        let (Some(field), Some(cited), Some(quote)) = (p["field"].as_str(), p["passage"].as_str(), p["quote"].as_str()) else {
            dropped += 1;
            continue;
        };
        let (Some(f), Some(passage)) = (asked.iter().find(|f| f.id == field), passages.iter().find(|x| x.id == cited.trim())) else {
            dropped += 1;
            continue;
        };
        // the words of the passage, never the AI's copy of them
        let Some(quote) = super::quote::find(quote, &passage.text) else {
            dropped += 1;
            continue;
        };
        let quote = quote.as_str();
        let value = fit(f, &p["value"]);
        // the AI does not count: a number is in the words it copied
        if matches!(f.kind, FieldKind::Number | FieldKind::Money | FieldKind::Year) && !value.as_i64().is_some_and(|n| says_number(quote, n)) {
            dropped += 1;
            continue;
        }
        let source = json!({
            "document_id": passage.document_id,
            "page": passage.page,
            "quote": quote.trim(),
            "rank": super::authority(field, passage.kind.as_deref()),
            "sha": proposals::fingerprint(&value),
        });
        let Some(form_id) = super::form_of(field) else {
            dropped += 1;
            continue;
        };
        let new = NewProposal {
            form_id: form_id.to_string(),
            record_id: None,
            field_id: field.to_string(),
            value,
            origin: Origin::Document,
            source_ref: Some(source.to_string()),
            agent: super::AGENT,
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

fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("lock".into()))
}

/// Fills the empty fields of a window of «Mi institución» from the documents of the Expediente, as suggestions.
/// Nothing of the institution changes. The base is not held while the services answer.
pub async fn fill_window(db: &SharedDb, provider: Option<&dyn AiProvider>, embedder: Option<Arc<dyn Embedder>>, form: &str) -> Result<FillOutcome, ServiceError> {
    let Some(&(form, place, name)) = FILLABLE.iter().find(|f| f.0 == form) else { return Err(ServiceError::NotFound) };
    let spec = crate::core::institution::forms::form(form).ok_or(ServiceError::NotFound)?;
    let nothing = |reason| Ok(FillOutcome::Nothing { form, place, reason });
    let (asked_ids, with_vectors) = {
        let conn = lock(db)?;
        if !docs::drawer_has_text(&conn, Drawer::Record)? {
            return nothing("no_documents");
        }
        let saved = crate::core::profile::forms::saved_values(&conn, form)?;
        let asked: Vec<&'static str> = asked_fields(spec, &saved).iter().map(|f| f.id).collect();
        (asked, vectors::any(&conn)?)
    };
    if asked_ids.is_empty() {
        return nothing("all_written");
    }
    let asked: Vec<&FieldSpec> = asked_ids.iter().filter_map(|id| spec.field(id)).collect();
    // the vectors of the questions, in one request: fixed words of the program, no data of anybody
    let mut vectors_of: Vec<Option<Vec<f32>>> = vec![None; asked.len()];
    if let Some(e) = embedder.filter(|_| with_vectors) {
        let queries: Vec<String> = asked.iter().map(|f| query_of(f.id)).collect();
        let started = std::time::Instant::now();
        let answer = e.embed(&queries, true).await;
        let ok = matches!(&answer, Ok(v) if v.len() == queries.len());
        vectors::record_usage(db, e.as_ref(), queries.iter().map(|q| q.chars().count()).sum(), ok, started.elapsed().as_millis() as u64);
        if let (true, Ok(v)) = (ok, answer) {
            vectors_of = v.into_iter().map(Some).collect();
        }
    }
    let (passages, scanner) = {
        let conn = lock(db)?;
        (passages_for(&conn, &asked, &vectors_of)?, crate::core::screen::scanner_for(&conn)?)
    };
    if passages.is_empty() {
        return nothing("nothing_found");
    }
    let Some(provider) = provider else { return Ok(FillOutcome::Unavailable { form, place, reason: "not_configured" }) };
    let ledger = SqliteLedger(db.clone());
    let errand = agent::Errand {
        context: context(name, &asked, &passages),
        request: format!("Sugiera los campos vacíos de la ventana «{name}» con lo que dicen los pasajes."),
    };
    let run = match agent::run(provider, &scanner, &ledger, &agent(), &[], errand).await {
        Ok(run) => run,
        Err(e) => return Ok(FillOutcome::Unavailable { form, place, reason: reason(&e) }),
    };
    let (fields, dropped) = keep(&*lock(db)?, &asked, &passages, &run.answer)?;
    let note = run.answer["note"].as_str().unwrap_or_default().trim().to_string();
    Ok(FillOutcome::Proposed { form, place, fields, dropped, note })
}

/// «Llenar todo con mis documentos»: the four windows, one after the other (one call each).
pub async fn fill_all(db: &SharedDb, provider: Option<&dyn AiProvider>, embedder: Option<Arc<dyn Embedder>>) -> Result<Vec<FillOutcome>, ServiceError> {
    let mut out = Vec::new();
    for (form, _, _) in FILLABLE {
        out.push(fill_window(db, provider, embedder.clone(), form).await?);
    }
    Ok(out)
}

#[cfg(test)]
#[path = "fill_tests.rs"]
pub(crate) mod tests;
