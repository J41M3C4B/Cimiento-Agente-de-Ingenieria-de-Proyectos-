//! Readings of calls (ADR-015): the files of a call, what was read from them and how it went.
//! The text of every file is kept per page in `document_chunk`; the canonical document the reading
//! produced is kept whole in `call_reading`. Nothing of the original file is kept.

use crate::core::api::add_call_document;
use crate::storage::StorageError;
use crate::audit::{self, AuditKind};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ulid::Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingStatus {
    /// Saved, not read yet: no key, no connection or no allowance left.
    Waiting,
    Reading,
    Ready,
    /// Some blocks of the call could not be read.
    Partial,
    Failed,
}

impl ReadingStatus {
    pub fn as_db(self) -> &'static str {
        match self {
            ReadingStatus::Waiting => "waiting",
            ReadingStatus::Reading => "reading",
            ReadingStatus::Ready => "ready",
            ReadingStatus::Partial => "partial",
            ReadingStatus::Failed => "failed",
        }
    }

    fn from_db(s: &str) -> ReadingStatus {
        match s {
            "reading" => ReadingStatus::Reading,
            "ready" => ReadingStatus::Ready,
            "partial" => ReadingStatus::Partial,
            "failed" => ReadingStatus::Failed,
            _ => ReadingStatus::Waiting,
        }
    }
}

/// What a file is inside the package of a call, as the person marked it. It is the ground truth for the
/// kind of document: what the reading guesses (`identidad.tipo_de_documento`) only ever warns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    /// The call itself. A package has exactly one.
    Main,
    Annex,
    Guide,
    /// A form to fill in (read only: the person fills it by hand, ADR-011).
    Form,
    /// A notice, a correction or an extension of the call.
    Notice,
    Other,
}

impl FileRole {
    pub fn as_db(self) -> &'static str {
        match self {
            FileRole::Main => "main",
            FileRole::Annex => "annex",
            FileRole::Guide => "guide",
            FileRole::Form => "form",
            FileRole::Notice => "notice",
            FileRole::Other => "other",
        }
    }

    fn from_db(s: &str) -> FileRole {
        match s {
            "annex" => FileRole::Annex,
            "guide" => FileRole::Guide,
            "form" => FileRole::Form,
            "notice" => FileRole::Notice,
            "other" => FileRole::Other,
            _ => FileRole::Main,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadingFile {
    pub document_id: String,
    pub name: String,
    pub pages: i64,
    pub role: FileRole,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadingRow {
    pub id: String,
    pub name: String,
    /// Who gives the call and in which year, as the person wrote them.
    pub funder: Option<String>,
    pub year: Option<i64>,
    pub status: ReadingStatus,
    pub note: Option<String>,
    pub created_at: String,
    pub finished_at: Option<String>,
    /// When the person confirmed that this is the right call (cleared when it is read again).
    pub confirmed_at: Option<String>,
    pub files: Vec<ReadingFile>,
}

/// A file of the call as it goes into the database: its name, its kind and its pages, already clean.
pub struct NewFile {
    pub name: String,
    pub mime: &'static str,
    pub pages: Vec<String>,
    pub redactions: i64,
    pub role: FileRole,
}

/// What the person said about the call.
pub struct CallMeta {
    pub name: String,
    pub funder: Option<String>,
    pub year: Option<i64>,
}

/// Saves the files and a reading that has not started, inside the caller's transaction: either the whole
/// call is there or none of it is.
fn create_in(tx: &rusqlite::Transaction<'_>, meta: &CallMeta, files: &[NewFile]) -> Result<String, StorageError> {
    let id = format!("read_{}", Ulid::generate());
    tx.execute(
        "INSERT INTO call_reading (id,name,funder,year,status,created_at) VALUES (?1,?2,?3,?4,'waiting',strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![id, meta.name.trim(), meta.funder.as_deref().map(str::trim), meta.year],
    )?;
    for (position, f) in files.iter().enumerate() {
        let doc_id = add_call_document(tx, &f.name, f.mime, &f.pages, f.redactions)?;
        tx.execute(
            "INSERT INTO call_reading_file (reading_id,document_id,position,role) VALUES (?1,?2,?3,?4)",
            params![id, doc_id, position as i64, f.role.as_db()],
        )?;
    }
    Ok(id)
}

#[cfg(test)]
pub fn create(conn: &mut Connection, meta: &CallMeta, files: &[NewFile]) -> Result<String, StorageError> {
    let tx = conn.transaction()?;
    let id = create_in(&tx, meta, files)?;
    tx.commit()?;
    Ok(id)
}

/// Creates a project from its call: the project, the reading and the files go in together, and the project
/// points to the reading. Returns `(project_id, reading_id)`.
pub fn create_project_with_call(
    conn: &mut Connection,
    project_title: &str,
    initial_request: &str,
    meta: &CallMeta,
    files: &[NewFile],
) -> Result<(String, String), StorageError> {
    let tx = conn.transaction()?;
    let reading_id = create_in(&tx, meta, files)?;
    let project_id = super::projects::insert_project(&tx, project_title, Some(initial_request), "call", Some(&reading_id))?;
    tx.commit()?;
    Ok((project_id, reading_id))
}

fn files_of(conn: &Connection, reading_id: &str) -> Result<Vec<ReadingFile>, StorageError> {
    // the documents are the core's: their name and pages come through `core::api`
    let mut stmt = conn.prepare("SELECT document_id, role FROM call_reading_file WHERE reading_id = ?1 ORDER BY position")?;
    let files = stmt.query_map([reading_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?.collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::new();
    for (document_id, role) in files {
        if let Some((name, pages)) = crate::core::api::document_brief(conn, &document_id)? {
            rows.push(ReadingFile { document_id, name, pages, role: FileRole::from_db(&role) });
        }
    }
    Ok(rows)
}

fn row(conn: &Connection, r: &rusqlite::Row<'_>) -> Result<ReadingRow, StorageError> {
    let id: String = r.get(0)?;
    Ok(ReadingRow {
        files: files_of(conn, &id)?,
        name: r.get(1)?,
        status: ReadingStatus::from_db(&r.get::<_, String>(2)?),
        note: r.get(3)?,
        created_at: r.get(4)?,
        finished_at: r.get(5)?,
        funder: r.get(6)?,
        year: r.get(7)?,
        confirmed_at: r.get(8)?,
        id,
    })
}

const READING_COLS: &str = "id,name,status,note,created_at,finished_at,funder,year,confirmed_at";

#[cfg(test)]
pub fn list(conn: &Connection) -> Result<Vec<ReadingRow>, StorageError> {
    let mut stmt = conn.prepare(&format!("SELECT {READING_COLS} FROM call_reading ORDER BY created_at DESC, id DESC"))?;
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(r) = rows.next()? {
        out.push(row(conn, r)?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<ReadingRow>, StorageError> {
    let mut stmt = conn.prepare(&format!("SELECT {READING_COLS} FROM call_reading WHERE id=?1"))?;
    let mut rows = stmt.query([id])?;
    match rows.next()? {
        Some(r) => Ok(Some(row(conn, r)?)),
        None => Ok(None),
    }
}

/// The pages of every file of the reading, in order: what `Package::from_documents` takes.
pub fn pages_of(conn: &Connection, reading_id: &str) -> Result<Vec<(String, Vec<String>)>, StorageError> {
    let mut out = Vec::new();
    for f in files_of(conn, reading_id)? {
        let pages = crate::core::api::document_pages(conn, &f.document_id)?;
        out.push((f.name, pages));
    }
    Ok(out)
}

pub fn set_status(conn: &Connection, id: &str, status: ReadingStatus, note: Option<&str>) -> Result<(), StorageError> {
    // a reading that starts again is not the one the person confirmed
    conn.execute("UPDATE call_reading SET status=?2, note=?3, finished_at=NULL, confirmed_at=NULL WHERE id=?1", params![id, status.as_db(), note])?;
    Ok(())
}

/// The person confirms that the call is the right one. Only a call that was read (fully or in part) can be
/// confirmed. Returns whether it was.
pub fn confirm(conn: &Connection, id: &str) -> Result<bool, StorageError> {
    let n = conn.execute(
        "UPDATE call_reading SET confirmed_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1 AND status IN ('ready','partial')",
        [id],
    )?;
    Ok(n > 0)
}

/// The reading ended (well or not): what it produced, how it went, and a line in the log with counts only.
pub fn finish(
    conn: &Connection,
    id: &str,
    status: ReadingStatus,
    note: Option<&str>,
    canonical: Option<&Value>,
    report: &Value,
) -> Result<(), StorageError> {
    conn.execute(
        // a reading that produced nothing does not wipe what an earlier one had read; one that did makes the
        // summary written for the earlier document stale, so it goes
        "UPDATE call_reading SET status=?2, note=?3, canonical_json=COALESCE(?4, canonical_json), brief_text=CASE WHEN ?4 IS NULL THEN brief_text ELSE NULL END, report_json=?5, finished_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1",
        params![id, status.as_db(), note, canonical.map(Value::to_string), report.to_string()],
    )?;
    audit::record(conn, AuditKind::CallRead, Some("call_reading"), Some(id), json!({ "status": status.as_db(), "note": note.unwrap_or("") }))?;
    Ok(())
}

/// The canonical document and the report of a finished reading.
pub fn result(conn: &Connection, id: &str) -> Result<Option<(Value, Value)>, StorageError> {
    let r: Option<(Option<String>, Option<String>)> = conn
        .query_row("SELECT canonical_json, report_json FROM call_reading WHERE id=?1", [id], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;
    Ok(r.and_then(|(c, rep)| Some((serde_json::from_str(&c?).ok()?, rep.and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null)))))
}

/// What is kept of the «en pocas palabras» of the call (ADR-024): `None` if nobody has tried to write it, `Some("")`
/// if it was tried and the AI gave nothing usable (so the same call is not paid for again), else the text.
pub fn brief_raw(conn: &Connection, id: &str) -> Result<Option<String>, StorageError> {
    let b: Option<Option<String>> = conn.query_row("SELECT brief_text FROM call_reading WHERE id=?1", [id], |r| r.get(0)).optional()?;
    Ok(b.flatten())
}

/// Keeps the brief; an empty text records that it was tried and nothing usable came.
pub fn set_brief(conn: &Connection, id: &str, brief: &str) -> Result<(), StorageError> {
    conn.execute("UPDATE call_reading SET brief_text=?2 WHERE id=?1", params![id, brief.trim()])?;
    Ok(())
}

/// A reading left «reading» by a program that closed halfway is waiting again, so the person can resume it.
pub fn mark_interrupted(conn: &Connection) -> Result<usize, StorageError> {
    Ok(conn.execute("UPDATE call_reading SET status='waiting', note='interrupted' WHERE status='reading'", [])?)
}

/// Removes the reading and the files that came with it (the emergency delete of each one, so nothing is left).
pub fn delete(conn: &mut Connection, id: &str) -> Result<bool, StorageError> {
    let files = files_of(conn, id)?;
    if get(conn, id)?.is_none() {
        return Ok(false);
    }
    for f in &files {
        crate::core::api::emergency_delete_document(conn, &f.document_id)?;
    }
    // a reading with no files (or whose files were already gone) is removed here
    conn.execute("DELETE FROM call_reading WHERE id=?1", [id])?;
    Ok(true)
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

    fn files() -> Vec<NewFile> {
        vec![
            NewFile { name: "bases.pdf".into(), mime: "application/pdf", pages: vec!["Primera página zanahoria.".into(), "Segunda página.".into()], redactions: 0, role: FileRole::Main },
            NewFile { name: "formato.docx".into(), mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document", pages: vec!["Formato de solicitud.".into()], redactions: 1, role: FileRole::Form },
        ]
    }

    fn meta(name: &str) -> CallMeta {
        CallMeta { name: name.into(), funder: None, year: None }
    }

    #[test]
    fn a_reading_keeps_its_files_page_by_page_and_in_order() {
        let (_d, mut c) = conn();
        let id = create(&mut c, &meta(" Convocatoria de prueba "), &files()).unwrap();
        let r = get(&c, &id).unwrap().unwrap();
        assert_eq!((r.name.as_str(), r.status), ("Convocatoria de prueba", ReadingStatus::Waiting));
        assert_eq!(r.files.iter().map(|f| (f.name.as_str(), f.pages)).collect::<Vec<_>>(), vec![("bases.pdf", 2), ("formato.docx", 1)]);
        let pages = pages_of(&c, &id).unwrap();
        assert_eq!(pages[0], ("bases.pdf".to_string(), vec!["Primera página zanahoria.".to_string(), "Segunda página.".to_string()]));
        assert_eq!(pages[1].1, vec!["Formato de solicitud.".to_string()]);
        // the pages are searchable like any other document
        let n: i64 = c.query_row("SELECT count(*) FROM document_chunk_fts WHERE document_chunk_fts MATCH 'zanahoria'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
        assert_eq!(list(&c).unwrap().len(), 1);
    }

    #[test]
    fn finishing_stores_the_result_and_logs_counts_only() {
        let (_d, mut c) = conn();
        let id = create(&mut c, &meta("C"), &files()).unwrap();
        set_status(&c, &id, ReadingStatus::Reading, None).unwrap();
        finish(&c, &id, ReadingStatus::Partial, Some("quota_reached"), Some(&json!({"identidad": {}})), &json!({"calls": 8})).unwrap();
        let r = get(&c, &id).unwrap().unwrap();
        assert_eq!((r.status, r.note.as_deref()), (ReadingStatus::Partial, Some("quota_reached")));
        assert!(r.finished_at.is_some());
        let (doc, report) = result(&c, &id).unwrap().unwrap();
        assert_eq!((doc["identidad"].is_object(), report["calls"].as_i64()), (true, Some(8)));
        let d: String = c.query_row("SELECT details_json FROM audit_log WHERE event='call.read'", [], |r| r.get(0)).unwrap();
        assert!(!d.contains("identidad"));
    }

    #[test]
    fn a_second_reading_that_reads_nothing_keeps_what_the_first_had_read() {
        let (_d, mut c) = conn();
        let id = create(&mut c, &meta("C"), &files()).unwrap();
        finish(&c, &id, ReadingStatus::Partial, Some("unavailable"), Some(&json!({"identidad": {"nombre": "x"}})), &json!({"calls": 8})).unwrap();
        finish(&c, &id, ReadingStatus::Waiting, Some("quota_reached"), None, &json!({"calls": 0})).unwrap();
        let (doc, report) = result(&c, &id).unwrap().unwrap();
        assert_eq!((doc["identidad"]["nombre"].as_str(), report["calls"].as_i64()), (Some("x"), Some(0)));
        assert_eq!(get(&c, &id).unwrap().unwrap().status, ReadingStatus::Waiting);
    }

    #[test]
    fn only_a_read_call_can_be_confirmed_and_reading_it_again_takes_the_confirmation_away() {
        let (_d, mut c) = conn();
        let id = create(&mut c, &meta("C"), &files()).unwrap();
        assert!(!confirm(&c, &id).unwrap(), "a call that was not read cannot be confirmed");
        finish(&c, &id, ReadingStatus::Partial, Some("unavailable"), Some(&json!({"identidad": {}})), &json!({})).unwrap();
        assert!(confirm(&c, &id).unwrap());
        assert!(get(&c, &id).unwrap().unwrap().confirmed_at.is_some());
        set_status(&c, &id, ReadingStatus::Waiting, None).unwrap();
        assert!(get(&c, &id).unwrap().unwrap().confirmed_at.is_none());
    }

    #[test]
    fn a_reading_interrupted_by_closing_the_program_waits_again() {
        let (_d, mut c) = conn();
        let id = create(&mut c, &meta("C"), &files()).unwrap();
        set_status(&c, &id, ReadingStatus::Reading, None).unwrap();
        assert_eq!(mark_interrupted(&c).unwrap(), 1);
        let r = get(&c, &id).unwrap().unwrap();
        assert_eq!((r.status, r.note.as_deref()), (ReadingStatus::Waiting, Some("interrupted")));
    }

    #[test]
    fn deleting_a_reading_leaves_nothing_of_its_files_and_deleting_a_file_takes_the_reading() {
        let (_d, mut c) = conn();
        let id = create(&mut c, &meta("C"), &files()).unwrap();
        finish(&c, &id, ReadingStatus::Ready, None, Some(&json!({"x": "zanahoria"})), &json!({})).unwrap();
        assert!(delete(&mut c, &id).unwrap());
        fn q(c: &Connection, sql: &str) -> i64 {
            c.query_row(sql, [], |r| r.get(0)).unwrap()
        }
        assert_eq!((q(&c, "SELECT count(*) FROM call_reading"), q(&c, "SELECT count(*) FROM document"), q(&c, "SELECT count(*) FROM document_chunk")), (0, 0, 0));
        assert_eq!(q(&c, "SELECT count(*) FROM document_chunk_fts WHERE document_chunk_fts MATCH 'zanahoria'"), 0);
        assert!(!delete(&mut c, &id).unwrap());

        // the emergency delete of one file of the call removes the reading derived from it (it quotes it)
        let id = create(&mut c, &meta("D"), &files()).unwrap();
        finish(&c, &id, ReadingStatus::Ready, None, Some(&json!({"x": 1})), &json!({})).unwrap();
        let first = get(&c, &id).unwrap().unwrap().files[0].document_id.clone();
        crate::core::api::emergency_delete_document(&mut c, &first).unwrap();
        assert_eq!(q(&c, "SELECT count(*) FROM call_reading"), 0);
    }
}
