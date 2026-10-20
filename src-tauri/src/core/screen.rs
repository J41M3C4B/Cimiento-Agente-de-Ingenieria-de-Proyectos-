//! The scanner with the words of the institution: its name, phone and mail are not personal data (ADR-032). The
//! rules of the scanner are in the base (`crate::scanner`); the core knows the institution.

use crate::core::error::ServiceError;
use crate::scanner::guard::{screen_texts, Decision, QuarantineReport};
use crate::scanner::RegexScanner;
use crate::storage::StorageError;
use rusqlite::Connection;

pub(crate) fn scanner_for(conn: &Connection) -> Result<RegexScanner, StorageError> {
    Ok(RegexScanner::new(crate::core::profile::storage::scanner_config(conn)?))
}

/// Scans free texts with the words of the institution (its name and contact are not personal data) and writes the
/// decision to the audit log. `Ok(Err(report))` means "quarantine: show it, save nothing".
pub(crate) fn guard_texts(
    conn: &Connection,
    entity: &str,
    fields: &[(String, String)],
    decision: Option<Decision>,
) -> Result<Result<Vec<String>, QuarantineReport>, ServiceError> {
    Ok(screen_texts(conn, &scanner_for(conn)?, entity, fields, decision)?)
}
