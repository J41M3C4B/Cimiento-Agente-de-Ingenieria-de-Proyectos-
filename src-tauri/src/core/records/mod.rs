//! The record of the institution (ADR-033 §5, block N4): the list of what it must have, each document with its
//! state, and the file only when there is one ("we have the deed but did not upload it" counts too).
//!
//! - **Which documents apply** is decided here, from what the institution says of itself: its legal form, whether it
//!   is a donee or has its CLUNI, whom and how it serves (`catalog::RECORD_TYPES`). The institution adds the ones the
//!   catalog does not know (`other`).
//! - **Validity** is reckoned from the date written in each document: valid, expiring (30 days or less) or expired.
//!   Rules of the Junta or the SAT come later (ADR-033 §5).
//! - **A file** (PDF or Word) is read with what reads the calls; its text goes through the scanner like any text
//!   (quarantine) and only the clean text is kept, in `document`. Of the original, only where it is on this computer
//!   (principle 6). A scanned PDF or a photo has no text: the document counts as had, and the original is still
//!   opened from where it is.
//! - **Nothing of it reaches the AI yet** (N5); the folio and the notes never will.

pub mod storage;

#[cfg(test)]
mod tests;

use crate::common::dates::days_between;
use crate::common::forms::{is_filled, FormSpec, Values};
use crate::core::error::ServiceError;
use crate::core::history::{self, Source};
use crate::core::institution::catalog::{Applies, MINORS, RECORD_OTHER, RECORD_TYPES, UNDER_A_JUNTA};
use crate::core::institution::forms::{RECORD, RECORD_KNOWN};
use crate::core::profile::domain::ProfileIssue;
use crate::scanner::guard::{Decision, QuarantineReport};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;
use storage as store;

/// How many days before its date a document is said to be expiring (decision of the institution, 2026-10-10).
pub const EXPIRING_DAYS: i64 = 30;
/// The largest file taken.
pub const MAX_FILE_BYTES: u64 = 40 * 1024 * 1024;
/// Fewer letters than this in every page: there is no text to read (a scan or a photo).
const MIN_LETTERS_PER_PAGE: usize = 20;
/// What the history writes when the record changes: that it changed, never a value.
const HISTORY_FIELD: &str = "institution.record";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Validity {
    Valid,
    Expiring,
    Expired,
}

/// The file of a document, as the screen shows it: its name, and whether its text could be read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordFile {
    pub name: String,
    pub readable: bool,
}

/// A document of the record, as «Mi institución» shows it. Never its folio nor its notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    /// `None` while nothing was written of it.
    pub id: Option<String>,
    /// A type of the catalog, or `other`.
    #[serde(rename = "type")]
    pub kind: String,
    pub custom_name: Option<String>,
    /// `have`, `in_progress`, `missing` or `not_applicable`; `missing` while nothing was written.
    pub status: String,
    pub issued_on: Option<String>,
    pub valid_until: Option<String>,
    pub validity: Option<Validity>,
    /// Days until its date (negative once it passed), while the institution has it.
    pub days_left: Option<i64>,
    pub file: Option<RecordFile>,
    pub origin: Option<String>,
    pub confirmed_at: Option<String>,
}

/// «Tiene 9 de 12»: of the documents that apply (but the ones marked as not applying), how many it has, which are in
/// process or missing, and how many of the ones it has are expiring or expired.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub expected: usize,
    pub have: usize,
    pub in_progress: usize,
    pub missing: usize,
    pub expiring: usize,
    pub expired: usize,
}

impl Summary {
    /// What it has and is not expired: «expediente vigente» of the monthly photo.
    pub fn current(&self) -> usize {
        self.have - self.expired
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Expediente {
    pub records: Vec<Record>,
    pub summary: Summary,
}

/// A document ready to draw in its window.
#[derive(Debug, Serialize)]
pub struct RecordForm {
    pub spec: &'static FormSpec,
    pub values: Values,
    pub missing: Vec<&'static str>,
    pub file: Option<RecordFile>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RecordOutcome {
    Saved { id: String, expediente: Expediente },
    /// A value that cannot be kept: the field and why. Nothing was saved.
    Invalid { issues: Vec<ProfileIssue> },
    /// The name or the notes have data of a person: nothing was saved until the person decides.
    Quarantine { report: QuarantineReport },
}

/// Why a file was not taken; the screen says each one in plain words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unreadable {
    /// Not a PDF, a Word file (.docx) or a photo.
    UnsupportedType,
    TooLarge,
    /// It is not where the person said, or it cannot be opened.
    NotFound,
    Damaged,
    /// A list of people: it belongs to Personal or Beneficiarios, never to the record.
    Roster,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AttachOutcome {
    /// The file is on the document. `readable = false`: a scan or a photo, with no text to read.
    Attached { id: String, readable: bool, expediente: Expediente },
    Quarantine { report: QuarantineReport },
    Unreadable { reason: Unreadable },
}

fn text(v: &Values, field: &str) -> Option<String> {
    v.get(field).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn list(v: &Values, field: &str) -> Vec<String> {
    v.get(field).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default()
}

/// Whether a type applies to the institution, from what it says of itself now.
fn applies(a: Applies, identity: &Values, legal: &Values) -> bool {
    let one_of = |field: &str, codes: &[&str]| text(legal, field).is_some_and(|v| codes.contains(&v.as_str()));
    match a {
        Applies::Always => true,
        Applies::Donee => one_of("institution.authorized_donee", &["yes", "in_progress"]),
        Applies::Cluni => one_of("institution.cluni", &["yes", "in_progress"]),
        Applies::Junta => one_of("institution.legal_form", UNDER_A_JUNTA),
        Applies::Minors => list(identity, "institution.populations").iter().any(|p| MINORS.contains(&p.as_str())),
        Applies::Housing => list(identity, "institution.modalities").iter().any(|m| m == "residential" || m == "day_care"),
    }
}

fn known(kind: &str) -> bool {
    kind == RECORD_OTHER || RECORD_TYPES.iter().any(|(code, _)| *code == kind)
}

/// Today on this computer (`YYYY-MM-DD`).
fn today(conn: &Connection) -> Result<String, ServiceError> {
    Ok(conn.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))?)
}

fn file_of(row: &store::Row) -> Option<RecordFile> {
    row.original_name.clone().map(|name| RecordFile { name, readable: row.document_shown })
}

fn record(kind: &str, row: Option<&store::Row>, today: &str) -> Record {
    let status = row.map_or("missing".to_string(), |r| r.status.clone());
    let valid_until = row.and_then(|r| r.valid_until.clone());
    let days_left = valid_until.as_deref().filter(|_| status == "have").and_then(|d| days_between(today, d));
    let validity = days_left.map(|d| match d {
        d if d < 0 => Validity::Expired,
        d if d <= EXPIRING_DAYS => Validity::Expiring,
        _ => Validity::Valid,
    });
    Record {
        id: row.map(|r| r.id.clone()),
        kind: kind.to_string(),
        custom_name: row.and_then(|r| r.custom_name.clone()),
        status,
        issued_on: row.and_then(|r| r.issued_on.clone()),
        valid_until,
        validity,
        days_left,
        file: row.and_then(file_of),
        origin: row.map(|r| r.origin.clone()),
        confirmed_at: row.and_then(|r| r.confirmed_at.clone()),
    }
}

/// The record of the institution as of a day: the types that apply, in the order of the catalog, then the ones the
/// institution named. A type that no longer applies still shows while the institution has it or has it in process.
pub fn expediente_on(conn: &Connection, today: &str) -> Result<Expediente, ServiceError> {
    let rows = store::rows(conn)?;
    let identity = crate::core::profile::forms::saved_values(conn, "institution.identity")?;
    let legal = crate::core::profile::forms::saved_values(conn, "institution.legal")?;
    let mut records = Vec::new();
    for (code, rule) in RECORD_TYPES {
        let row = rows.iter().find(|r| r.r#type == *code);
        let held = row.is_some_and(|r| r.status == "have" || r.status == "in_progress");
        if applies(*rule, &identity, &legal) || held {
            records.push(record(code, row, today));
        }
    }
    records.extend(rows.iter().filter(|r| r.r#type == RECORD_OTHER).map(|r| record(RECORD_OTHER, Some(r), today)));

    let mut s = Summary::default();
    for r in records.iter().filter(|r| r.status != "not_applicable") {
        s.expected += 1;
        match r.status.as_str() {
            "have" => s.have += 1,
            "in_progress" => s.in_progress += 1,
            _ => s.missing += 1,
        }
        match r.validity {
            Some(Validity::Expiring) => s.expiring += 1,
            Some(Validity::Expired) => s.expired += 1,
            _ => {}
        }
    }
    Ok(Expediente { records, summary: s })
}

pub fn expediente(conn: &Connection) -> Result<Expediente, ServiceError> {
    expediente_on(conn, &today(conn)?)
}

/// The row a request speaks of: by its id, or the one of a type of the catalog. `Err(NotFound)` for a type the code
/// does not know or an id of another type.
fn row_of(conn: &Connection, kind: &str, id: Option<&str>) -> Result<Option<store::Row>, ServiceError> {
    if !known(kind) {
        return Err(ServiceError::NotFound);
    }
    match id {
        Some(id) => match store::by_id(conn, id)? {
            Some(r) if r.r#type == kind => Ok(Some(r)),
            _ => Err(ServiceError::NotFound),
        },
        None if kind == RECORD_OTHER => Ok(None),
        None => Ok(store::by_type(conn, kind)?),
    }
}

fn spec_of(kind: &str) -> &'static FormSpec {
    if kind == RECORD_OTHER {
        &RECORD
    } else {
        &RECORD_KNOWN
    }
}

/// The window of a document: what is written of it, or empty.
pub fn form(conn: &Connection, kind: &str, id: Option<&str>) -> Result<RecordForm, ServiceError> {
    let row = row_of(conn, kind, id)?;
    let spec = spec_of(kind);
    let mut values = Values::new();
    if let Some(r) = &row {
        values.insert("institution.record.custom_name".into(), json!(r.custom_name));
        values.insert("institution.record.status".into(), json!(r.status));
        values.insert("institution.record.issued_on".into(), json!(r.issued_on));
        values.insert("institution.record.valid_until".into(), json!(r.valid_until));
        values.insert("institution.record.reference".into(), json!(r.reference));
        values.insert("institution.record.notes".into(), json!(r.notes));
    }
    values.retain(|k, v| is_filled(Some(v)) && spec.field(k).is_some());
    Ok(RecordForm { spec, missing: spec.missing(&values), file: row.as_ref().and_then(file_of), values })
}

/// Saves what the person writes of a document. A status out of its list, a date that does not exist or a validity
/// before the issue date is refused with its field; the name of an `other` is needed. The name and the notes go
/// through the scanner. The document and its line in the history go together.
pub fn save(conn: &mut Connection, kind: &str, id: Option<&str>, mut values: Values, decision: Option<Decision>) -> Result<RecordOutcome, ServiceError> {
    let row = row_of(conn, kind, id)?;
    let spec = spec_of(kind);
    spec.clear_hidden(&mut values);
    let mut issues: Vec<ProfileIssue> = spec
        .validate(&values)
        .into_iter()
        // a document is not filled little by little: what it needs is needed now
        .map(|i| ProfileIssue { code: if i.code == "missing" { "required" } else { i.code }, field: i.field.to_string(), blocking: true })
        .collect();
    if let (Some(from), Some(to)) = (text(&values, "institution.record.issued_on"), text(&values, "institution.record.valid_until")) {
        if days_between(&from, &to).is_some_and(|d| d < 0) && issues.is_empty() {
            issues.push(ProfileIssue { code: "valid_before_issued", field: "institution.record.valid_until".into(), blocking: true });
        }
    }
    if !issues.is_empty() {
        issues.sort_by_key(|i| spec.fields().position(|f| f.id == i.field));
        return Ok(RecordOutcome::Invalid { issues });
    }
    // the free texts of the person, through the scanner
    let free: Vec<(String, String)> = ["institution.record.custom_name", "institution.record.notes"]
        .iter()
        .filter_map(|f| text(&values, f).map(|t| (f.to_string(), t)))
        .collect();
    let mut cleaned = match crate::core::screen::guard_texts(conn, "core_record", &free, decision)? {
        Ok(texts) => free.iter().map(|(f, _)| f.clone()).zip(texts).collect::<std::collections::BTreeMap<_, _>>(),
        Err(report) => return Ok(RecordOutcome::Quarantine { report }),
    };
    let (custom_name, notes) = (cleaned.remove("institution.record.custom_name"), cleaned.remove("institution.record.notes"));
    let (status, issued_on, valid_until, reference) = (
        text(&values, "institution.record.status").unwrap_or_default(),
        text(&values, "institution.record.issued_on"),
        text(&values, "institution.record.valid_until"),
        text(&values, "institution.record.reference"),
    );
    let input = store::Input {
        custom_name: custom_name.as_deref().filter(|_| kind == RECORD_OTHER),
        status: &status,
        issued_on: issued_on.as_deref(),
        valid_until: valid_until.as_deref(),
        reference: reference.as_deref(),
        notes: notes.as_deref(),
    };
    let tx = conn.transaction()?;
    let id = store::save(&tx, row.as_ref().map(|r| r.id.as_str()), kind, &input)?;
    history::record_list_change(&tx, HISTORY_FIELD, &Source::user())?;
    tx.commit()?;
    Ok(RecordOutcome::Saved { id, expediente: expediente(conn)? })
}

/// The text of a file, page by page, or `None` when it has none to read (a scan or a photo).
struct FileText {
    name: String,
    mime: &'static str,
    pages: Option<Vec<String>>,
}

fn read_file(path: &Path) -> Result<FileText, Unreadable> {
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).ok_or(Unreadable::NotFound)?;
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let mime = match ext.as_str() {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        _ => return Err(Unreadable::UnsupportedType),
    };
    let size = std::fs::metadata(path).map_err(|_| Unreadable::NotFound)?.len();
    if size > MAX_FILE_BYTES {
        return Err(Unreadable::TooLarge);
    }
    if mime.starts_with("image/") {
        return Ok(FileText { name, mime, pages: None });
    }
    let raw = crate::documents::canonical::package::read_pieces(path).map_err(|_| Unreadable::Damaged)?;
    let pages: Vec<String> = raw.iter().map(|p| crate::documents::text::clean_text(p)).collect();
    let letters = |t: &str| t.chars().filter(|c| c.is_alphabetic()).count();
    if pages.iter().all(|p| letters(p) < MIN_LETTERS_PER_PAGE) {
        return Ok(FileText { name, mime, pages: None });
    }
    Ok(FileText { name, mime, pages: Some(pages) })
}

/// Puts a file on a document: a PDF, a Word file or a photo, read from where it is on this computer. Its text goes
/// through the scanner (a deed names people: «Tapar y seguir»); only the clean text is kept, and of the original
/// only where it is. A document of the catalog with nothing written is created as had; an `other` must exist.
pub fn attach(conn: &mut Connection, kind: &str, id: Option<&str>, path: &Path, decision: Option<Decision>) -> Result<AttachOutcome, ServiceError> {
    let row = row_of(conn, kind, id)?;
    if kind == RECORD_OTHER && row.is_none() {
        return Err(ServiceError::NotFound);
    }
    let file = match read_file(path) {
        Ok(f) => f,
        Err(reason) => return Ok(AttachOutcome::Unreadable { reason }),
    };
    let mut clean: Option<(Vec<String>, i64)> = None;
    if let Some(pages) = &file.pages {
        let scanner = crate::core::screen::scanner_for(conn)?;
        let joined = pages.join("\n\n");
        let whole = crate::scanner::SensitiveScanner::scan(&scanner, &joined);
        if scanner.looks_like_roster(&joined, &whole) {
            return Ok(AttachOutcome::Unreadable { reason: Unreadable::Roster });
        }
        let fields: Vec<(String, String)> = pages.iter().enumerate().map(|(i, p)| (format!("page_{}", i + 1), p.clone())).collect();
        let redactions = if decision == Some(Decision::Redact) { whole.findings.len() as i64 } else { 0 };
        match crate::core::screen::guard_texts(conn, "core_record", &fields, decision)? {
            Ok(texts) => clean = Some((texts, redactions)),
            Err(report) => return Ok(AttachOutcome::Quarantine { report }),
        }
    }
    let tx = conn.transaction()?;
    let id = match row {
        Some(r) => r.id,
        None => store::save(&tx, None, kind, &store::Input { custom_name: None, status: "have", issued_on: None, valid_until: None, reference: None, notes: None })?,
    };
    let doc = match &clean {
        Some((pages, redactions)) => Some(crate::core::archive::storage::add_paged_document(&tx, "internal", &file.name, file.mime, pages, *redactions)?),
        None => None,
    };
    store::set_file(&tx, &id, doc.as_deref(), &path.to_string_lossy(), &file.name)?;
    history::record_list_change(&tx, HISTORY_FIELD, &Source::user())?;
    tx.commit()?;
    Ok(AttachOutcome::Attached { id, readable: doc.is_some(), expediente: expediente(conn)? })
}

/// Takes the file off a document. Its clean text stays in Documentos, where it is deleted as any document.
pub fn detach(conn: &mut Connection, id: &str) -> Result<Expediente, ServiceError> {
    let tx = conn.transaction()?;
    if !store::clear_file(&tx, id)? {
        return Err(ServiceError::NotFound);
    }
    history::record_list_change(&tx, HISTORY_FIELD, &Source::user())?;
    tx.commit()?;
    expediente(conn)
}

/// Where the original of a document is, if it is still there.
pub fn original(conn: &Connection, id: &str) -> Result<Option<std::path::PathBuf>, ServiceError> {
    let row = store::by_id(conn, id)?.ok_or(ServiceError::NotFound)?;
    Ok(row.original_path.map(std::path::PathBuf::from).filter(|p| p.is_file()))
}
