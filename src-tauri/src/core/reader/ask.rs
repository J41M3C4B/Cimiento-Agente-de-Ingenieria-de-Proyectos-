//! The Reader of the Expediente answers a question (C3, ADR-036 §6): the agent of the drawer, asked by the Assistant
//! («según mi registro, ¿qué nombre legal pongo?»). The Assistant does not read documents; it asks this.
//!
//! - **It looks only in its drawer:** the passages of the Expediente that best answer the question (words, and meaning
//!   when there are vectors), or of one document when the person named its file. Never whole documents.
//! - **One call to the AI** with the cheapest model, which sees each document by its type, never by the name of its
//!   file (decision of the institution, 2026-10-13).
//! - **The code checks** that each quote is in a passage shown and that every figure of the answer is in the passages
//!   it cites (the AI does not count). An answer with no quote that passes is no answer: «no lo dice».
//! - Nothing is saved: an answer is words for the person, with its document and page from the code.

use crate::ai::agent::{self, Agent, Errand};
use crate::ai::pipeline::SqliteLedger;
use crate::ai::{AiProvider, ModelTier};
use crate::core::agents::assistant::reason;
use crate::core::archive::search::{self, Passage};
use crate::core::archive::vectors;
use crate::core::archive::storage::{self as docs, Drawer};
use crate::core::error::ServiceError;
use crate::documents::canonical::retrieve::Embedder;
use crate::storage::SharedDb;
use serde_json::{json, Value};
use std::sync::Arc;

/// Its trade, versioned like a prompt: changing it is a new file.
const TRADE: &str = include_str!("../agents/reader_answer.v1.md");

/// The passages shown for a question, and about how long each one is.
const PASSAGES: usize = 4;
const PASSAGE_CHARS: usize = 900;

pub fn agent() -> Agent {
    Agent {
        task: "agent.reader_answer",
        name: "reader_answer",
        tier: ModelTier::Light,
        trade: TRADE,
        // everything it needs comes in its context: the passages
        tools: &[],
        max_tool_calls: 0,
        max_tokens: 30_000,
        max_output_tokens: 600,
        answer: json!({
            "type": "object",
            "properties": {
                "found": { "type": "boolean" },
                "text": { "type": "string" },
                "citations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": { "passage": { "type": "string" }, "quote": { "type": "string" } },
                        "required": ["passage", "quote"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["found", "text", "citations"],
            "additionalProperties": false
        }),
        knows_institution: false,
    }
}

/// A passage shown to the AI: the fragment, and the name of its document for the AI (its type).
#[derive(Debug, Clone, PartialEq)]
pub struct Shown {
    /// `P1`, `P2`…: how the AI cites it.
    pub id: String,
    pub name: String,
    pub passage: Passage,
}

/// Where an answer comes from: a passage shown and the words of it the AI copied, checked by the code.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    pub name: String,
    pub passage: Passage,
    pub quote: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReaderAnswer {
    /// What the documents say, with where.
    Found { text: String, citations: Vec<Citation> },
    /// No passage says it (`looked`: how many were read), or what the AI said did not pass the code (`dropped`).
    NotFound { looked: usize, dropped: bool },
    /// The Expediente has no document with text.
    NoDocuments,
    /// The AI could not do it (`reason`, as in the Assistant).
    Unavailable { reason: &'static str },
}

fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("lock".into()))
}

/// The passages for a question, named by their type. In one document (`document`), its first pages when no word of
/// the question is in it.
pub async fn passages_for(db: &SharedDb, embedder: Option<Arc<dyn Embedder>>, question: &str, document: Option<&str>) -> Result<Vec<Shown>, ServiceError> {
    let mut found = vectors::search_documents_in(db, embedder, Drawer::Record, document, question, PASSAGES, PASSAGE_CHARS).await?.passages;
    let conn = lock(db)?;
    if found.is_empty() {
        if let Some(d) = document {
            found = search::document_start(&conn, Drawer::Record, d, PASSAGES, PASSAGE_CHARS)?;
        }
    }
    found
        .into_iter()
        .enumerate()
        .map(|(i, p)| Ok(Shown { id: format!("P{}", i + 1), name: crate::core::records::ai_name(&conn, &p.document_id)?, passage: p }))
        .collect()
}

/// What the Reader is told: the passages, numbered, each with its type and page. Never the name of a file.
pub fn context(shown: &[Shown]) -> Vec<String> {
    let lines: Vec<String> = shown
        .iter()
        .map(|s| format!("[{}] {}{}:\n{}", s.id, s.name, s.passage.page.map_or(String::new(), |n| format!(", página {n}")), s.passage.snippet))
        .collect();
    vec![format!("Pasajes de los documentos del Expediente:\n\n{}", lines.join("\n\n"))]
}

/// Keeps what the code accepts of an answer: quotes that are in a passage shown, and figures of the text that are in
/// the passages it cites or in the question.
pub fn check(answer: &Value, shown: &[Shown], question: &str) -> ReaderAnswer {
    let looked = shown.len();
    let mut citations: Vec<Citation> = Vec::new();
    for c in answer["citations"].as_array().into_iter().flatten() {
        let (Some(id), Some(quote)) = (c["passage"].as_str(), c["quote"].as_str()) else { continue };
        let Some(s) = shown.iter().find(|s| s.id == id.trim()) else { continue };
        // the words of the passage, never the AI's copy of them
        let Some(quote) = super::quote::find(quote, &s.passage.snippet) else { continue };
        if !citations.iter().any(|x| x.passage.chunk == s.passage.chunk && x.quote == quote) {
            citations.push(Citation { name: s.name.clone(), passage: s.passage.clone(), quote });
        }
    }
    let text = super::quote::unescape(answer["text"].as_str().unwrap_or_default().trim());
    if answer["found"] != json!(true) || text.is_empty() {
        return ReaderAnswer::NotFound { looked, dropped: false };
    }
    if citations.is_empty() {
        return ReaderAnswer::NotFound { looked, dropped: true };
    }
    let mut sources: Vec<&str> = citations.iter().map(|c| c.passage.snippet.as_str()).collect();
    sources.push(question);
    if !figures_are_in(&text, &sources) {
        return ReaderAnswer::NotFound { looked, dropped: true };
    }
    ReaderAnswer::Found { text, citations }
}

/// Whether every figure of a text is in one of the sources, with figures or in words («dos mil dieciséis»).
fn figures_are_in(text: &str, sources: &[&str]) -> bool {
    crate::ai::figures::unsupported_figures(&[text], sources)
        .iter()
        .all(|n| n.parse::<i64>().is_ok_and(|n| sources.iter().any(|s| super::fill::says_number(s, n))))
}

/// Answers a question with the documents of the Expediente, or of one of them (`document`, its id in the store).
/// The base is not held while the services answer.
pub async fn answer(db: &SharedDb, provider: Option<&dyn AiProvider>, embedder: Option<Arc<dyn Embedder>>, question: &str, document: Option<&str>) -> Result<ReaderAnswer, ServiceError> {
    if !docs::drawer_has_text(&*lock(db)?, Drawer::Record)? {
        return Ok(ReaderAnswer::NoDocuments);
    }
    let shown = passages_for(db, embedder, question, document).await?;
    if shown.is_empty() {
        return Ok(ReaderAnswer::NotFound { looked: 0, dropped: false });
    }
    let Some(provider) = provider else { return Ok(ReaderAnswer::Unavailable { reason: "not_configured" }) };
    let scanner = crate::core::screen::scanner_for(&*lock(db)?)?;
    let ledger = SqliteLedger(db.clone());
    let errand = Errand { context: context(&shown), request: format!("Pregunta: {}", question.trim()) };
    match agent::run(provider, &scanner, &ledger, &agent(), &[], errand).await {
        Ok(run) => Ok(check(&run.answer, &shown, question)),
        Err(e) => Ok(ReaderAnswer::Unavailable { reason: reason(&e) }),
    }
}

#[cfg(test)]
#[path = "ask_tests.rs"]
mod tests;
