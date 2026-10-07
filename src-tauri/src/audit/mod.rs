//! Audit log. Records event types and counts, never content.

use rusqlite::{params, Connection};
use serde_json::Value;

/// Minimum events from `docs/03-gobernanza-datos.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AuditKind {
    DocumentUploaded,
    ScannerQuarantine,
    ScannerOverride,
    ScannerLeakPrevented,
    EmergencyDelete,
    AiCall,
    ExportCreated,
    ProfileConfirmed,
    StageChanged,
    BackupCreated,
    /// A call was read (counts only: files, pages, calls, status).
    CallRead,
    /// A covered identifier of a staff record was shown (which field; never the value).
    HrSensitiveViewed,
    /// A staff record was deleted with everything it had.
    HrPersonDeleted,
    /// The staff of the old roster moved into the staff module (counts only).
    HrImported,
}

impl AuditKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AuditKind::DocumentUploaded => "document.uploaded",
            AuditKind::ScannerQuarantine => "scanner.quarantine",
            AuditKind::ScannerOverride => "scanner.override",
            AuditKind::ScannerLeakPrevented => "scanner.leak_prevented",
            AuditKind::EmergencyDelete => "emergency.delete",
            AuditKind::AiCall => "ai.call",
            AuditKind::ExportCreated => "export.created",
            AuditKind::ProfileConfirmed => "profile.confirmed",
            AuditKind::StageChanged => "stage.changed",
            AuditKind::BackupCreated => "backup.created",
            AuditKind::CallRead => "call.read",
            AuditKind::HrSensitiveViewed => "hr.sensitive_viewed",
            AuditKind::HrPersonDeleted => "hr.person_deleted",
            AuditKind::HrImported => "hr.imported",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("audit details must not carry free text (found a string longer than {0} characters)")]
    FreeText(usize),
}

/// Longest string allowed inside `details`: enough for keys and decisions, too short for content.
const MAX_DETAIL_STRING: usize = 48;

fn check_no_free_text(v: &Value) -> Result<(), AuditError> {
    match v {
        Value::String(s) if s.chars().count() > MAX_DETAIL_STRING => {
            Err(AuditError::FreeText(MAX_DETAIL_STRING))
        }
        Value::Array(a) => a.iter().try_for_each(check_no_free_text),
        Value::Object(o) => o.values().try_for_each(check_no_free_text),
        _ => Ok(()),
    }
}

pub fn record(
    conn: &Connection,
    kind: AuditKind,
    entity: Option<&str>,
    entity_id: Option<&str>,
    details: Value,
) -> Result<(), AuditError> {
    check_no_free_text(&details)?;
    conn.execute(
        "INSERT INTO audit_log (at, event, entity, entity_id, details_json)
         VALUES (strftime('%Y-%m-%dT%H:%M:%SZ','now'), ?1, ?2, ?3, ?4)",
        params![kind.as_str(), entity, entity_id, details.to_string()],
    )?;
    Ok(())
}

/// Deletes entries older than two years (retention in `03-gobernanza-datos.md`).
#[allow(dead_code)]
pub fn purge_old(conn: &Connection) -> Result<usize, AuditError> {
    Ok(conn.execute(
        "DELETE FROM audit_log WHERE at < strftime('%Y-%m-%dT%H:%M:%SZ','now','-2 years')",
        [],
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(include_str!("../../migrations/0001_initial.sql")).unwrap();
        c
    }

    #[test]
    fn records_event_with_counts() {
        let c = db();
        record(&c, AuditKind::ScannerQuarantine, Some("document"), Some("doc_1"),
            json!({"findings": {"curp": 3, "phone": 1}, "decision": "redacted"})).unwrap();
        let (ev, d): (String, String) = c
            .query_row("SELECT event, details_json FROM audit_log", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(ev, "scanner.quarantine");
        assert!(d.contains("\"curp\":3"));
    }

    #[test]
    fn rejects_free_text_in_details() {
        let c = db();
        let long = "Rosa Hernández López vive en la calle Ejemplo 123 y tiene diabetes";
        assert!(matches!(
            record(&c, AuditKind::ProfileConfirmed, None, None, json!({"note": long})),
            Err(AuditError::FreeText(_))
        ));
        let n: i64 = c.query_row("SELECT count(*) FROM audit_log", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn purges_only_entries_older_than_two_years() {
        let c = db();
        c.execute("INSERT INTO audit_log (at,event) VALUES ('2020-01-01T00:00:00Z','old')", []).unwrap();
        record(&c, AuditKind::StageChanged, None, None, json!({})).unwrap();
        assert_eq!(purge_old(&c).unwrap(), 1);
        let n: i64 = c.query_row("SELECT count(*) FROM audit_log", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
}
