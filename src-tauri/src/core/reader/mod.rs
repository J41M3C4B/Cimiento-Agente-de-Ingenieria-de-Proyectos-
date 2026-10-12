//! The Reader (Lector, IA6 of ADR-034): what a document of the institution is, and the data of the institution it
//! says. Rules first, and in this part only rules: the kind by its words (`kinds`), the data by where each kind writes
//! them (`rules`), the dates as notaries write them (`dates`).
//!
//! - **It reads before the scanner covers.** The rules read the text of the file in memory, before it is saved; what
//!   is kept is still the clean text. That is how a protected datum (the RFC, a folio, the representative) can reach
//!   its field: only when a rule read it, never the AI (decision of the institution, 2026-10-12).
//! - **It proposes, the person decides.** Each datum is a suggestion of its window of «Mi institución», with the
//!   document and page it comes from (`ai_proposal`, origin `document`, agent `reader`). Nothing is saved without
//!   «Usar» and «Guardar».
//! - **Invoices are not read here:** they are uploaded only in Finanzas.

pub mod dates;
pub mod kinds;
pub mod rules;

use crate::common::forms::Sensitivity;
use crate::core::agents::proposals::{self, NewProposal, Origin, ProposeOutcome};
use crate::core::error::ServiceError;
use rules::Found;
use rusqlite::Connection;
use serde_json::json;

/// The agent that signs its proposals.
pub const AGENT: &str = "reader";

/// What the Reader understood of a document: its kind, if a rule recognized it or it was known, and the data found.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub kind: Option<&'static str>,
    pub found: Vec<Found>,
}

/// Reads the text of a file before it is saved. `known`: the kind the person already said (a document of the record).
pub fn read(pages: &[String], known: Option<&str>) -> Reading {
    let kind = known.and_then(|k| kinds::kind(k).map(|k| k.code)).or_else(|| kinds::recognize(pages));
    Reading { kind, found: kind.map(|k| rules::read(k, pages)).unwrap_or_default() }
}

fn form_of(field: &str) -> Option<&'static str> {
    use crate::core::institution::forms::FORMS;
    FORMS.iter().find(|f| f.field(field).is_some()).map(|f| f.id)
}

/// Leaves the data found in a document as suggestions of their windows, citing the document and the page; the line
/// it was read from goes too, unless the datum is protected. A datum the institution already has, with that same value,
/// is not suggested again. Says how many were kept.
pub fn propose(conn: &Connection, document_id: &str, found: &[Found]) -> Result<usize, ServiceError> {
    let mut kept = 0;
    for f in found {
        let Some(form_id) = form_of(f.field) else { continue };
        let view = crate::core::profile::forms::get(conn, form_id)?;
        if view.values.get(f.field) == Some(&f.value) {
            continue;
        }
        let protected = crate::core::institution::forms::field(f.field).is_some_and(|s| s.sensitivity == Sensitivity::InstitutionalPrivate);
        let mut source = json!({ "document_id": document_id, "page": f.page });
        if !protected {
            source["quote"] = json!(f.line);
        }
        let outcome = proposals::propose(
            conn,
            NewProposal {
                form_id: form_id.to_string(),
                record_id: None,
                field_id: f.field.to_string(),
                value: f.value.clone(),
                origin: Origin::Document,
                source_ref: Some(source.to_string()),
                agent: AGENT,
            },
        )?;
        kept += usize::from(matches!(outcome, ProposeOutcome::Kept { .. }));
    }
    Ok(kept)
}

#[cfg(test)]
mod tests;
