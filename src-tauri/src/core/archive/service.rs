//! The documents of the institution (Mis documentos): a text goes in only after the scanner, and a list of people
//! never goes in.

use crate::audit::{self, AuditKind};
use crate::core::archive::storage::{self as docs, DocumentSummary, PageMarks};
use crate::documents::reading::{self, ReadError, Reading};
use crate::documents::text::clean_text;
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

/// The pages of a file after the scanner.
pub enum GuardedPages {
    /// What is kept (the name too, when one was given): covered when the person said so, and how many data were
    /// covered.
    Clean { name: Option<String>, pages: Vec<String>, redactions: i64 },
    /// Data of a person: nothing is kept until the person decides.
    Quarantine(QuarantineReport),
    /// A list of people: never kept, whatever the decision.
    Roster,
}

/// Passes the pages of a file through the scanner (each page a field, `page_1`…), for the table `entity`, with the
/// name the person gave it when there is one. `pictures`: some page was read from a picture, where an email may come
/// with its «@» misread (OCR, ADR-035).
pub fn guard_pages(conn: &Connection, entity: &str, name: Option<&str>, pages: &[String], pictures: bool, decision: Option<Decision>) -> Result<GuardedPages, ServiceError> {
    let scanner = if pictures { scanner_for(conn)?.reading_pictures() } else { scanner_for(conn)? };
    let joined = pages.join("\n\n");
    let whole = scanner.scan(&joined);
    if scanner.looks_like_roster(&joined, &whole) {
        audit::record(conn, AuditKind::ScannerQuarantine, Some(entity), None, counts_json(&whole.counts(), "rejected_roster"))?;
        return Ok(GuardedPages::Roster);
    }
    let mut fields: Vec<(String, String)> = name.map(|n| ("display_name".to_string(), n.to_string())).into_iter().collect();
    fields.extend(pages.iter().enumerate().map(|(i, p)| (format!("page_{}", i + 1), p.clone())));
    let redactions = if decision == Some(Decision::Redact) { whole.findings.len() as i64 } else { 0 };
    Ok(match crate::scanner::guard::screen_texts(conn, &scanner, entity, &fields, decision)? {
        Ok(mut texts) => {
            let name = name.map(|_| texts.remove(0));
            GuardedPages::Clean { name, pages: texts, redactions }
        }
        Err(report) => GuardedPages::Quarantine(report),
    })
}

/// The largest file Documentos takes.
pub const MAX_FILE_BYTES: usize = 40 * 1024 * 1024;
/// Fewer letters than this in every page: there is no text to keep.
const MIN_LETTERS_PER_PAGE: usize = 20;

/// Why a file was not taken in Documentos; the screen says each one in plain words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileUnreadable {
    /// Not a PDF, Word, Excel or a photo.
    UnsupportedType,
    TooLarge,
    Damaged,
    /// More scanned pages than can be read in one file.
    TooManyPages,
    /// A picture with no text that could be read, or a file with no text.
    NoText,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AddFileOutcome {
    Saved { document: DocumentSummary },
    Quarantine { report: QuarantineReport },
    /// The file looks like a list of people: not saved at all.
    RejectedRoster,
    Unreadable { reason: FileUnreadable },
}

/// Reads a file the screen sent (PDF, Word, Excel or a photo); a scanned page or a photo is read as a picture
/// (OCR, ADR-035), a few seconds a page. It does not touch the database: the caller does it without holding it.
pub fn read_upload(file_name: &str, bytes: &[u8]) -> Result<Reading, FileUnreadable> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(FileUnreadable::TooLarge);
    }
    reading::read_upload(file_name, bytes).map_err(|e| match e {
        ReadError::Unsupported => FileUnreadable::UnsupportedType,
        ReadError::Damaged => FileUnreadable::Damaged,
        ReadError::TooManyPictures => FileUnreadable::TooManyPages,
    })
}

fn mime_of(file_name: &str) -> &'static str {
    match file_name.rsplit('.').next().map(str::to_lowercase).as_deref() {
        Some("pdf") => "application/pdf",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xlsx") | Some("xlsm") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("png") => "image/png",
        _ => "image/jpeg",
    }
}

/// Keeps the clean text of a file already read (`read_upload`) as a document of the institution, page by page:
/// its name and its pages through the scanner, a list of people refused whole, and the pages read from a picture
/// marked. Of the file itself nothing is kept (principle 6).
pub fn add_file_document(conn: &mut Connection, display_name: &str, file_name: &str, read: Reading, decision: Option<Decision>) -> Result<AddFileOutcome, ServiceError> {
    if display_name.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    let pages: Vec<String> = read.pages.iter().map(|p| clean_text(p)).collect();
    let letters = |t: &str| t.chars().filter(|c| c.is_alphabetic()).count();
    if pages.iter().all(|p| letters(p) < MIN_LETTERS_PER_PAGE) {
        return Ok(AddFileOutcome::Unreadable { reason: FileUnreadable::NoText });
    }
    let pictures = !read.from_picture.is_empty();
    let (name, pages, redactions) = match guard_pages(conn, "document", Some(display_name.trim()), &pages, pictures, decision)? {
        GuardedPages::Clean { name, pages, redactions } => (name.unwrap_or_default(), pages, redactions),
        GuardedPages::Quarantine(report) => return Ok(AddFileOutcome::Quarantine { report }),
        GuardedPages::Roster => return Ok(AddFileOutcome::RejectedRoster),
    };
    let marks = PageMarks { from_picture: read.from_picture, doubtful: read.doubtful };
    let tx = conn.transaction()?;
    let id = docs::add_paged_document(&tx, "internal", &name, mime_of(file_name), &pages, redactions, &marks)?;
    tx.commit()?;
    let document = docs::list(conn)?.into_iter().find(|d| d.id == id).ok_or(ServiceError::NotFound)?;
    Ok(AddFileOutcome::Saved { document })
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

    /// A file is read page by page: a scan as pictures, its name and pages through the scanner, and only its clean
    /// text kept, with the pages read from a picture and the ones read with doubts.
    #[test]
    fn a_scanned_file_is_kept_as_clean_text_with_its_pages_read_from_pictures() {
        let (_d, mut c) = conn();
        let fixture = |n: &str| std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office").join(n)).unwrap();
        let read = read_upload("acta.pdf", &fixture("acta-escaneada.pdf")).unwrap();
        // the email of the deed is a datum of a person: the person decides
        let AddFileOutcome::Quarantine { report } = add_file_document(&mut c, "Acta", "acta.pdf", read.clone(), None).unwrap() else { panic!() };
        assert!(report.fields.iter().any(|f| f.path == "page_1"));
        assert!(docs::list(&c).unwrap().is_empty());
        let AddFileOutcome::Saved { document } = add_file_document(&mut c, "Acta", "acta.pdf", read, Some(Decision::Redact)).unwrap() else { panic!() };
        assert_eq!((document.kind.as_str(), document.chunks, document.picture_pages, document.doubtful_pages.clone()), ("internal", 2, 2, vec![2]));
        let text: String = c.query_row("SELECT extracted_text FROM document", [], |r| r.get(0)).unwrap();
        assert!(text.contains("ACTA CONSTITUTIVA") && !text.contains("contacto@casaficticia"), "{text}");
        let mime: String = c.query_row("SELECT mime FROM document", [], |r| r.get(0)).unwrap();
        assert_eq!(mime, "application/pdf");

        assert_eq!(read_upload("notas.txt", b"x"), Err(FileUnreadable::UnsupportedType));
        assert_eq!(read_upload("foto.png", &vec![0; MAX_FILE_BYTES + 1]), Err(FileUnreadable::TooLarge));
        let blank = read_upload("foto.png", &[1, 2, 3]).unwrap();
        assert!(matches!(add_file_document(&mut c, "Foto", "foto.png", blank, None).unwrap(), AddFileOutcome::Unreadable { reason: FileUnreadable::NoText }));
    }

    #[test]
    fn document_inputs_are_checked() {
        let (_d, mut c) = conn();
        assert!(matches!(add_text_document(&mut c, "photo", "x", "y", None), Err(ServiceError::UnknownKind)));
        assert!(matches!(add_text_document(&mut c, "call", "x", "  ", None), Err(ServiceError::EmptyText)));
    }
}
