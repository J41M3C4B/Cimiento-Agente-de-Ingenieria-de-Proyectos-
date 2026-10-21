//! The suggestions that come when a file is uploaded (ADR-042 §5). The rules read the formats they know when a file is
//! kept; what is written in any way (the mission, the services, whom it serves) waited for «Llenar con mis
//! documentos» in each window. Now, when a file is kept:
//!
//! - **the code looks, without AI,** for the empty fields of «Mi institución» that a fragment of **that file** speaks
//!   of: a fragment with `SPEAKS_OF` different words of the field or more (the same words as `fill::SEARCH`), or that
//!   names one of its options («Ciudad de México»);
//! - **one call** with the cheapest model, with only those fields and those passages, and the same checks of
//!   `fill::keep`: the quote, the figure and the value. What passes waits as a suggestion with its document and page,
//!   under the policy of ADR-038;
//! - with no AI, no empty field or no fragment that speaks of one, nothing is called. One call per file at most.
//!
//! It looks by words only: the vectors of the new file are not made yet when it is kept (they are made after, in the
//! background). «Llenar con mis documentos» still looks in the whole Expediente, by words and by meaning.

use super::fill::{self, query_of, Passage, PASSAGE_CHARS, PER_FIELD};
use crate::ai::agent;
use crate::ai::pipeline::SqliteLedger;
use crate::ai::AiProvider;
use crate::common::forms::FieldSpec;
use crate::core::agents::assistant::reason;
use crate::core::agents::capturist::{describe, plain, FILLABLE};
use crate::core::proposals;
use crate::core::archive::search;
use crate::core::archive::storage::{self as docs, Drawer};
use crate::core::error::ServiceError;
use crate::storage::{fts, SharedDb};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;

/// The different words of a field a fragment must have to speak of it. Measured with the fictitious Expediente
/// (`upload_detection_measure`, 2026-10-17): see ADR-042, «Fase 3, al aplicarla».
pub const SPEAKS_OF: usize = 2;

/// The words a field is looked for with, folded as FTS5 compares them, each once.
fn field_words(field: &str) -> Vec<String> {
    let mut words: Vec<String> = fts::words(&query_of(field)).iter().map(|w| fts::fold(w)).collect();
    words.sort();
    words.dedup();
    words
}

/// How many different words of a field a text has, each as the start of one of its words (as FTS5 finds them).
pub fn words_of(field: &str, text: &str) -> usize {
    let said: HashSet<String> = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(fts::fold).collect();
    field_words(field).iter().filter(|w| said.iter().any(|t| t.starts_with(w.as_str()))).count()
}

/// Whether a text names one of the options of a field with words of its own («Ciudad de México», «estancia
/// permanente», «asistencial»): how a document says a state or a kind, with none of the words of the field. A short
/// name («Sí», «No», «Otro») says nothing.
pub fn names_an_option(f: &FieldSpec, text: &str) -> bool {
    let said = format!(" {} ", plain(text));
    f.options.iter().filter_map(|code| crate::core::institution::forms::option_label(f.id, code)).map(plain).any(|label| {
        (label.contains(' ') || label.chars().count() >= 8) && said.contains(&format!(" {label} "))
    })
}

/// How much a fragment speaks of a field: the different words of it it has, and `SPEAKS_OF` more if it names one of
/// its options.
pub fn score(f: &FieldSpec, text: &str) -> usize {
    words_of(f.id, text) + if names_an_option(f, text) { SPEAKS_OF } else { 0 }
}

/// The fragments that speak of a field: a `score` of `min` or more, the highest first (in their order on a tie), up
/// to `PER_FIELD`.
pub fn speaking(chunks: &[(i64, Option<i64>, String)], f: &FieldSpec, min: usize) -> Vec<i64> {
    let mut scored: Vec<(usize, usize, i64)> =
        chunks.iter().enumerate().map(|(i, (id, _, text))| (score(f, text), i, *id)).filter(|(n, _, _)| *n >= min).collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().take(PER_FIELD).map(|(_, _, id)| id).collect()
}

/// Fields a document never fills when it is uploaded: what the person wants known (`institution.notes`, «lo que
/// quiera que se sepa… y no tenga otro lugar») is the person's to say, not a file's.
const NOT_ON_UPLOAD: &[&str] = &["institution.notes"];

/// The say of a suggestion that waits: its `rank` (ADR-038); one with none (told in the chat) has the most.
fn rank_of(p: &proposals::Proposal) -> i64 {
    p.source_ref.as_deref().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()).and_then(|v| v["rank"].as_i64()).unwrap_or(i64::MAX)
}

/// The fields asked when a file of this `kind` is kept: in each window, the empty ones the AI may see
/// (`fill::asked_fields`), but those a suggestion with as much say as this file or more waits for already (the
/// rules of this same file may have just left one). A file with more say is asked, and its suggestion takes the
/// place of the other (`proposals::propose`).
pub fn asked(conn: &Connection, kind: Option<&str>) -> Result<Vec<(&'static str, &'static FieldSpec)>, ServiceError> {
    let mut out = Vec::new();
    for (form, _, _) in FILLABLE {
        let spec = crate::core::institution::forms::form(form).ok_or(ServiceError::NotFound)?;
        let saved = crate::core::profile::forms::saved_values(conn, form)?;
        let waiting = proposals::pending(conn, form, None)?;
        let held = |f: &FieldSpec| waiting.iter().any(|p| p.field_id == f.id && rank_of(p) >= super::authority(f.id, kind));
        out.extend(fill::asked_fields(spec, &saved).into_iter().filter(|f| !NOT_ON_UPLOAD.contains(&f.id) && !held(f)).map(|f| (*form, f)));
    }
    Ok(out)
}

/// The fields of `asked` that the document speaks of, and the passages of it shown for them (numbered, each fragment
/// once).
pub fn detect(conn: &Connection, document_id: &str, asked: &[(&'static str, &'static FieldSpec)]) -> Result<(Vec<(&'static str, &'static FieldSpec)>, Vec<Passage>), ServiceError> {
    let chunks = search::document_chunks(conn, Drawer::Record, document_id)?;
    let (kind, name) = (docs::doc_type_of(conn, document_id)?, crate::core::records::ai_name(conn, document_id)?);
    let (mut fields, mut passages): (Vec<_>, Vec<Passage>) = (Vec::new(), Vec::new());
    let mut seen: Vec<i64> = Vec::new();
    for &(form, f) in asked {
        let ids = speaking(&chunks, f, SPEAKS_OF);
        if ids.is_empty() {
            continue;
        }
        fields.push((form, f));
        let fresh: Vec<i64> = ids.into_iter().filter(|id| !seen.contains(id)).collect();
        for p in search::passages(conn, &fresh, &query_of(f.id), PASSAGE_CHARS)? {
            seen.push(p.chunk);
            passages.push(Passage {
                id: format!("P{}", passages.len() + 1),
                document_id: p.document_id,
                document: p.document,
                name: name.clone(),
                page: p.page,
                kind: kind.clone(),
                text: p.snippet,
            });
        }
    }
    Ok((fields, passages))
}

/// What the Reader is told: the empty fields it may fill, by window, and the passages of the file, numbered, with its
/// type and page (never the name of the file).
pub fn context(fields: &[(&'static str, &'static FieldSpec)], passages: &[Passage]) -> Vec<String> {
    let mut windows: Vec<String> = Vec::new();
    for (form, _, name) in FILLABLE {
        let described: Vec<String> = fields.iter().filter(|(f, _)| f == form).map(|(_, f)| describe(f, None)).collect();
        if !described.is_empty() {
            windows.push(format!("Ventana «{name}»:\n{}", described.join("\n")));
        }
    }
    let shown: Vec<String> = passages
        .iter()
        .map(|p| format!("[{}] {}{}:\n{}", p.id, p.name, p.page.map_or(String::new(), |n| format!(", página {n}")), p.text))
        .collect();
    vec![
        format!("Los campos vacíos de «Mi institución», por ventana:\n\n{}", windows.join("\n\n")),
        format!("Pasajes del documento que se acaba de subir:\n\n{}", shown.join("\n\n")),
    ]
}

/// What came of a file kept: suggestions left (`fields`, by id, each with its window), nothing asked (`reason`:
/// `not_readable`, `all_written`, `nothing_found`), or an AI that could not do it.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UploadFill {
    Proposed { fields: Vec<(String, String)>, dropped: usize },
    Nothing { reason: &'static str },
    Unavailable { reason: &'static str },
}

fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("lock".into()))
}

/// The suggestions of a file just kept on its document (`document_id`, its text in the drawer): the empty fields of
/// «Mi institución» it speaks of, in one call. Nothing of the institution changes. The base is not held while the
/// AI answers.
pub async fn after_upload(db: &SharedDb, provider: Option<&dyn AiProvider>, document_id: &str) -> Result<UploadFill, ServiceError> {
    let (fields, passages, scanner) = {
        let conn = lock(db)?;
        if search::document_chunks(&conn, Drawer::Record, document_id)?.is_empty() {
            return Ok(UploadFill::Nothing { reason: "not_readable" });
        }
        let asked = asked(&conn, docs::doc_type_of(&conn, document_id)?.as_deref())?;
        if asked.is_empty() {
            return Ok(UploadFill::Nothing { reason: "all_written" });
        }
        let (fields, passages) = detect(&conn, document_id, &asked)?;
        (fields, passages, crate::core::screen::scanner_for(&conn)?)
    };
    if fields.is_empty() {
        return Ok(UploadFill::Nothing { reason: "nothing_found" });
    }
    let Some(provider) = provider else { return Ok(UploadFill::Unavailable { reason: "not_configured" }) };
    let errand = agent::Errand {
        context: context(&fields, &passages),
        request: "Sugiera los campos vacíos de «Mi institución» con lo que dicen los pasajes de este documento.".to_string(),
    };
    let run = match agent::run(provider, &scanner, &SqliteLedger(db.clone()), &fill::agent(), &[], errand).await {
        Ok(run) => run,
        Err(e) => return Ok(UploadFill::Unavailable { reason: reason(&e) }),
    };
    let asked: Vec<&FieldSpec> = fields.iter().map(|(_, f)| *f).collect();
    let (kept, dropped) = fill::keep(&*lock(db)?, &asked, &passages, &run.answer)?;
    let fields = kept.into_iter().filter_map(|k| fields.iter().find(|(_, f)| f.id == k).map(|(form, _)| (form.to_string(), k))).collect();
    Ok(UploadFill::Proposed { fields, dropped })
}

#[cfg(test)]
#[path = "upload_tests.rs"]
mod tests;
