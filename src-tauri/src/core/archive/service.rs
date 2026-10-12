//! What every drawer does when a file comes in (ADR-036 §2): its pages through the scanner, a list of people refused
//! whole. The file itself is read by `documents::reading` and kept by the owner of its drawer (today the Expediente,
//! `core::records`); here only the clean text is decided.

use crate::audit::{self, AuditKind};
use crate::core::error::ServiceError;
use crate::core::screen::scanner_for;
use crate::scanner::guard::{counts_json, Decision, QuarantineReport};
use crate::scanner::SensitiveScanner;
use rusqlite::Connection;

/// The pages of a file after the scanner.
pub enum GuardedPages {
    /// What is kept: covered when the person said so, and how many data were covered.
    Clean { pages: Vec<String>, redactions: i64 },
    /// Data of a person: nothing is kept until the person decides.
    Quarantine(QuarantineReport),
    /// A list of people: never kept, whatever the decision.
    Roster,
}

/// Passes the pages of a file through the scanner (each page a field, `page_1`…), for the table `entity`.
/// `pictures`: some page was read from a picture, where an email may come with its «@» misread (OCR, ADR-035).
pub fn guard_pages(conn: &Connection, entity: &str, pages: &[String], pictures: bool, decision: Option<Decision>) -> Result<GuardedPages, ServiceError> {
    let scanner = if pictures { scanner_for(conn)?.reading_pictures() } else { scanner_for(conn)? };
    let joined = pages.join("\n\n");
    let whole = scanner.scan(&joined);
    if scanner.looks_like_roster(&joined, &whole) {
        audit::record(conn, AuditKind::ScannerQuarantine, Some(entity), None, counts_json(&whole.counts(), "rejected_roster"))?;
        return Ok(GuardedPages::Roster);
    }
    let fields: Vec<(String, String)> = pages.iter().enumerate().map(|(i, p)| (format!("page_{}", i + 1), p.clone())).collect();
    let redactions = if decision == Some(Decision::Redact) { whole.findings.len() as i64 } else { 0 };
    Ok(match crate::scanner::guard::screen_texts(conn, &scanner, entity, &fields, decision)? {
        Ok(pages) => GuardedPages::Clean { pages, redactions },
        Err(report) => GuardedPages::Quarantine(report),
    })
}
