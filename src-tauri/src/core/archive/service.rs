//! The documents of the institution (Mis documentos): a text goes in only after the scanner, and a list of people
//! never goes in.

use crate::audit::{self, AuditKind};
use crate::core::archive::storage::{self as docs, DocumentSummary};
use crate::core::error::ServiceError;
use crate::core::screen::scanner_for;
use crate::scanner::guard::{counts_json, guard_fields, Decision, GuardOutcome, QuarantineReport};
use crate::scanner::SensitiveScanner;
use rusqlite::Connection;
use serde::Serialize;

const MAX_TEXT_BYTES: usize = 2 * 1024 * 1024;
const DOCUMENT_KINDS: &[&str] = &["call", "questionnaire", "template", "internal", "quote", "other"];

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AddDocumentOutcome {
    Saved { document: DocumentSummary },
    Quarantine { report: QuarantineReport },
    /// The file looks like a list of people: not saved at all.
    RejectedRoster,
}

pub fn add_text_document(
    conn: &mut Connection,
    kind: &str,
    display_name: &str,
    text: &str,
    decision: Option<Decision>,
) -> Result<AddDocumentOutcome, ServiceError> {
    if !DOCUMENT_KINDS.contains(&kind) {
        return Err(ServiceError::UnknownKind);
    }
    if text.trim().is_empty() || display_name.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    if text.len() > MAX_TEXT_BYTES {
        return Err(ServiceError::TextTooLarge);
    }
    let scanner = scanner_for(conn)?;

    // A list of people is rejected whole, whatever the decision.
    let full = scanner.scan(text);
    if scanner.looks_like_roster(text, &full) {
        audit::record(
            conn,
            AuditKind::ScannerQuarantine,
            Some("document"),
            None,
            counts_json(&full.counts(), "rejected_roster"),
        )?;
        return Ok(AddDocumentOutcome::RejectedRoster);
    }

    let fields = vec![
        ("display_name".to_string(), display_name.to_string()),
        ("text".to_string(), text.to_string()),
    ];
    let (name, body, redactions, audit_event) = match guard_fields(&scanner, &fields, decision)? {
        GuardOutcome::Clean => (display_name.to_string(), text.to_string(), 0, None),
        GuardOutcome::Quarantine(report) => return Ok(AddDocumentOutcome::Quarantine { report }),
        GuardOutcome::Redacted { mut texts, counts } => {
            let body = texts.pop().unwrap_or_default();
            let name = texts.pop().unwrap_or_default();
            let n: usize = counts.values().sum();
            (name, body, n as i64, Some((AuditKind::ScannerQuarantine, counts_json(&counts, "redacted"))))
        }
        GuardOutcome::Overridden { counts } => (
            display_name.to_string(),
            text.to_string(),
            0,
            Some((AuditKind::ScannerOverride, counts_json(&counts, "not_personal"))),
        ),
    };

    let doc = docs::add_text_document(conn, kind, &name, &body, redactions)?;
    if let Some((k, d)) = audit_event {
        audit::record(conn, k, Some("document"), Some(&doc.id), d)?;
    }
    Ok(AddDocumentOutcome::Saved { document: doc })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::open_encrypted;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, c)
    }

    fn audit_events(c: &Connection) -> Vec<(String, String)> {
        let mut s = c.prepare("SELECT event, details_json FROM audit_log ORDER BY id").unwrap();
        let v = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(|r| r.unwrap()).collect();
        v
    }

    #[test]
    fn document_flow_quarantine_redact_and_roster() {
        let (_d, mut c) = conn();
        let text = "Convocatoria. Enviar a maria.lopez@example.com antes del viernes.";
        let AddDocumentOutcome::Quarantine { report } =
            add_text_document(&mut c, "call", "Convocatoria", text, None).unwrap() else { panic!() };
        assert!(!report.has_blocking);
        assert!(docs::list(&c).unwrap().is_empty());

        let AddDocumentOutcome::Saved { document } =
            add_text_document(&mut c, "call", "Convocatoria", text, Some(Decision::Redact)).unwrap() else { panic!() };
        assert_eq!(document.redactions_count, 1);
        let stored: String = c.query_row("SELECT extracted_text FROM document", [], |r| r.get(0)).unwrap();
        assert!(stored.contains("[CORREO OCULTO]") && !stored.contains("maria.lopez"));

        let roster = "Nombre\tEdad\tDiagnóstico\nAna\t80\tDiabetes";
        assert!(matches!(
            add_text_document(&mut c, "internal", "Lista", roster, Some(Decision::Redact)).unwrap(),
            AddDocumentOutcome::RejectedRoster
        ));
        assert_eq!(docs::list(&c).unwrap().len(), 1);
        assert!(audit_events(&c).iter().any(|(_, d)| d.contains("rejected_roster")));
    }

    #[test]
    fn document_inputs_are_checked() {
        let (_d, mut c) = conn();
        assert!(matches!(add_text_document(&mut c, "photo", "x", "y", None), Err(ServiceError::UnknownKind)));
        assert!(matches!(add_text_document(&mut c, "call", "x", "  ", None), Err(ServiceError::EmptyText)));
    }
}
