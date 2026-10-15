//! The scanner with the words of the institution: its name, phone and mail are not personal data (ADR-032). The
//! rules of the scanner are in the base (`crate::scanner`); the core knows the institution.

use crate::core::error::ServiceError;
use crate::scanner::guard::{screen_texts, Decision, QuarantineReport};
use crate::scanner::{RegexScanner, SensitiveScanner};
use crate::storage::StorageError;
use rusqlite::Connection;

pub(crate) fn scanner_for(conn: &Connection) -> Result<RegexScanner, StorageError> {
    Ok(RegexScanner::new(crate::core::profile::storage::scanner_config(conn)?))
}

/// The scanner of an errand in which the person said «No es dato personal» for these texts: the warnings they
/// cleared are not raised again by the last check before the AI (ADR-041). With any other decision, the usual one.
pub(crate) fn scanner_clearing(conn: &Connection, texts: &[&str], decision: Option<Decision>) -> Result<RegexScanner, StorageError> {
    let base = scanner_for(conn)?;
    if decision != Some(Decision::NotPersonal) {
        return Ok(base);
    }
    let mut cfg = crate::core::profile::storage::scanner_config(conn)?;
    for t in texts {
        let report = base.scan(t);
        cfg.cleared.extend(report.findings.iter().filter(|f| f.severity == crate::scanner::Severity::Warn).filter_map(|f| t.get(f.span.clone()).map(str::to_string)));
    }
    Ok(RegexScanner::new(cfg))
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
