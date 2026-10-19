//! Clean documents (text only), their fragments and the emergency delete.

use crate::storage::StorageError;
use crate::audit::{self, AuditKind};
#[cfg(test)]
use crate::documents::chunking::chunk_text;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize)]
pub struct DocumentSummary {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    pub mime: String,
    pub data_level: String,
    pub redactions_count: i64,
    pub chunks: i64,
    pub created_at: String,
    /// How many pages were read from a picture (OCR, ADR-035), and which of them with doubts.
    pub picture_pages: usize,
    pub doubtful_pages: Vec<usize>,
    /// The class the person chose (`legal`, `financial`, `programs`) and what the document is within it, a kind of the
    /// record (IA6), with who said it (`computed`: a rule; `user`: the person).
    pub category: Option<String>,
    pub doc_type: Option<String>,
    pub doc_type_origin: Option<String>,
    /// The drawer it belongs to (ADR-036): `record`, `projects`; none for a document of before ADR-036.
    pub drawer: Option<String>,
    /// The document of the Expediente it is the file of: its row (when written), its type and, for an `other`, its name.
    pub record_id: Option<String>,
    pub record_type: Option<String>,
    pub record_name: Option<String>,
}

/// The drawer a document belongs to (ADR-036): exactly one, with one owner. The drawers of the modules come with
/// them; meanwhile the core has the Expediente, and Proyectos keeps the files of its calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drawer {
    /// The Expediente of the institution: owned by the core, read by its Reader, fills «Mi institución».
    Record,
    /// The files of a call: owned by Proyectos, read by the canonical reading (ADR-015).
    Projects,
}

impl Drawer {
    pub fn as_str(self) -> &'static str {
        match self {
            Drawer::Record => "record",
            Drawer::Projects => "projects",
        }
    }
    fn kind(self) -> &'static str {
        match self {
            Drawer::Record => "internal",
            Drawer::Projects => "call",
        }
    }
    fn level(self) -> &'static str {
        match self {
            Drawer::Record => "yellow",
            Drawer::Projects => "green",
        }
    }
}

/// Which pages of a file were read from a picture (OCR, ADR-035) and which of those with doubts; empty for a file
/// with its own text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageMarks {
    pub from_picture: Vec<usize>,
    pub doubtful: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DeleteSummary {
    pub chunks: i64,
    pub derived_rows: i64,
}

/// Saves the CLEAN text of a document (already scanned) in its drawer, with its fragments and search index. For
/// the tests: a file of the Expediente comes in through its document (`core::records`).
#[cfg(test)]
pub fn add_text_document(
    conn: &mut Connection,
    drawer: Drawer,
    display_name: &str,
    clean_text: &str,
    redactions_count: i64,
) -> Result<DocumentSummary, StorageError> {
    let kind = drawer.kind();
    let doc_id = format!("doc_{}", Ulid::generate());
    let hash = hex::encode(Sha256::digest(clean_text.as_bytes()));
    let chunks = chunk_text(clean_text);

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO document (id,kind,display_name,mime,data_level,clean_hash,extracted_text,redactions_count,drawer,created_at)
         VALUES (?1,?2,?3,'text/plain',?4,?5,?6,?7,?8,strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![doc_id, kind, display_name.trim(), drawer.level(), hash, clean_text, redactions_count, drawer.as_str()],
    )?;
    for (i, text) in chunks.iter().enumerate() {
        tx.execute(
            "INSERT INTO document_chunk (id,document_id,ordinal,text) VALUES (?1,?2,?3,?4)",
            params![format!("chunk_{}", Ulid::generate()), doc_id, i as i64, text],
        )?;
    }
    tx.execute(
        "INSERT INTO document_chunk_fts(rowid,text) SELECT rowid,text FROM document_chunk WHERE document_id=?1",
        [&doc_id],
    )?;
    audit::record(
        &tx,
        AuditKind::DocumentUploaded,
        Some("document"),
        Some(&doc_id),
        json!({ "kind": kind, "chunks": chunks.len(), "redactions": redactions_count }),
    )?;
    tx.commit()?;
    get(conn, &doc_id)?.ok_or(StorageError::NoProfile)
}

/// Saves one file of a call: its clean text per page (one fragment per page, with the page number) and its
/// search index. The caller owns the transaction, so the files of a call go in together or not at all.
pub fn add_call_document(conn: &Connection, display_name: &str, mime: &str, pages: &[String], redactions_count: i64, marks: &PageMarks) -> Result<String, StorageError> {
    add_paged_document(conn, Drawer::Projects, display_name, mime, pages, redactions_count, marks)
}

/// Saves the clean text of a file page by page, in its drawer: the Expediente (ADR-033 §5) or the files of a call.
/// There is no document without a drawer (ADR-036). The caller owns the transaction.
pub fn add_paged_document(conn: &Connection, drawer: Drawer, display_name: &str, mime: &str, pages: &[String], redactions_count: i64, marks: &PageMarks) -> Result<String, StorageError> {
    let kind = drawer.kind();
    let doc_id = format!("doc_{}", Ulid::generate());
    let text = pages.join("\n\n");
    let hash = hex::encode(Sha256::digest(text.as_bytes()));
    conn.execute(
        "INSERT INTO document (id,kind,display_name,mime,data_level,clean_hash,extracted_text,redactions_count,picture_pages,doubtful_pages,drawer,created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![doc_id, kind, display_name.trim(), mime, drawer.level(), hash, text, redactions_count, json!(marks.from_picture).to_string(), json!(marks.doubtful).to_string(), drawer.as_str()],
    )?;
    for (i, page) in pages.iter().enumerate() {
        conn.execute(
            "INSERT INTO document_chunk (id,document_id,ordinal,page,text) VALUES (?1,?2,?3,?4,?5)",
            params![format!("chunk_{}", Ulid::generate()), doc_id, i as i64, i as i64 + 1, page],
        )?;
    }
    conn.execute("INSERT INTO document_chunk_fts(rowid,text) SELECT rowid,text FROM document_chunk WHERE document_id=?1", [&doc_id])?;
    audit::record(conn, AuditKind::DocumentUploaded, Some("document"), Some(&doc_id), json!({ "kind": kind, "chunks": pages.len(), "redactions": redactions_count, "picture_pages": marks.from_picture.len() }))?;
    Ok(doc_id)
}

fn get(conn: &Connection, id: &str) -> Result<Option<DocumentSummary>, StorageError> {
    Ok(list(conn)?.into_iter().find(|d| d.id == id))
}

/// The documents of one drawer (ADR-036), newest first.
pub fn list_drawer(conn: &Connection, drawer: Drawer) -> Result<Vec<DocumentSummary>, StorageError> {
    Ok(list(conn)?.into_iter().filter(|d| d.drawer.as_deref() == Some(drawer.as_str())).collect())
}

pub fn list(conn: &Connection) -> Result<Vec<DocumentSummary>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.kind, d.display_name, d.data_level, d.redactions_count, d.created_at,
                (SELECT count(*) FROM document_chunk c WHERE c.document_id = d.id), d.picture_pages, d.doubtful_pages, d.mime,
                d.category, d.doc_type, d.doc_type_origin, d.drawer, r.id, r.type, r.custom_name
         FROM document d LEFT JOIN core_record r ON r.document_id = d.id
         WHERE d.hidden = 0 ORDER BY d.created_at DESC, d.id DESC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(DocumentSummary {
                id: r.get(0)?,
                kind: r.get(1)?,
                display_name: r.get(2)?,
                data_level: r.get(3)?,
                redactions_count: r.get(4)?,
                created_at: r.get(5)?,
                chunks: r.get(6)?,
                picture_pages: pages(&r.get::<_, String>(7)?).len(),
                doubtful_pages: pages(&r.get::<_, String>(8)?),
                mime: r.get(9)?,
                category: r.get(10)?,
                doc_type: r.get(11)?,
                doc_type_origin: r.get(12)?,
                drawer: r.get(13)?,
                record_id: r.get(14)?,
                record_type: r.get(15)?,
                record_name: r.get(16)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// What a document is, as a type of the record (`doc_type`), if it was said or recognized.
pub fn doc_type_of(conn: &Connection, id: &str) -> Result<Option<String>, StorageError> {
    Ok(conn.query_row("SELECT doc_type FROM document WHERE id = ?1", [id], |r| r.get::<_, Option<String>>(0)).optional()?.flatten())
}

/// Whether a drawer has any document with text to read (none waiting to be deleted).
pub fn drawer_has_text(conn: &Connection, drawer: Drawer) -> Result<bool, StorageError> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM document_chunk c JOIN document d ON d.id = c.document_id WHERE d.drawer = ?1 AND d.hidden = 0)",
        [drawer.as_str()],
        |r| r.get(0),
    )?)
}

/// Writes the class of a document and what it is (IA6). `origin`: `computed` when a rule recognized it, `user` when
/// the person said it. A kind the person said is not changed by a rule. Says if the document exists.
pub fn set_class(conn: &Connection, id: &str, category: Option<&str>, doc_type: Option<&str>, origin: &str) -> Result<bool, StorageError> {
    let n = conn.execute(
        "UPDATE document SET category = coalesce(?2, category),
                doc_type = CASE WHEN ?4 = 'computed' AND doc_type_origin = 'user' THEN doc_type ELSE ?3 END,
                doc_type_origin = CASE WHEN ?4 = 'computed' AND doc_type_origin = 'user' THEN doc_type_origin
                                       WHEN ?3 IS NULL THEN NULL ELSE ?4 END
         WHERE id = ?1 AND kind <> 'call'",
        params![id, category, doc_type, origin],
    )?;
    Ok(n > 0)
}

/// A list of page numbers kept as a JSON array; anything unreadable is none.
fn pages(json: &str) -> Vec<usize> {
    serde_json::from_str(json).unwrap_or_default()
}

/// The name of a document and how many fragments (pages, for a call) it has, if it exists.
pub fn document_brief(conn: &Connection, doc_id: &str) -> Result<Option<(String, i64)>, StorageError> {
    Ok(conn
        .query_row(
            "SELECT display_name, (SELECT count(*) FROM document_chunk c WHERE c.document_id = d.id) FROM document d WHERE d.id = ?1",
            [doc_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?)
}

/// The clean text of a document, one fragment (one page, for a call) after another.
pub fn document_pages(conn: &Connection, doc_id: &str) -> Result<Vec<String>, StorageError> {
    Ok(conn
        .prepare("SELECT text FROM document_chunk WHERE document_id=?1 ORDER BY ordinal")?
        .query_map([doc_id], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?)
}

fn table_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?1)", [name], |r| r.get(0))
}

/// "Delete this and everything that came from it": the document, its fragments,
/// search entries, vectors and any text that cites it. Then `VACUUM` so nothing
/// is left in the file. The audit log only records that a delete happened.
pub fn emergency_delete_document(conn: &mut Connection, doc_id: &str) -> Result<DeleteSummary, StorageError> {
    let exists: Option<String> = conn
        .query_row("SELECT id FROM document WHERE id=?1", [doc_id], |r| r.get(0))
        .optional()?;
    if exists.is_none() {
        return Ok(DeleteSummary { chunks: 0, derived_rows: 0 });
    }

    let tx = conn.transaction()?;

    // Search index first: the external-content FTS table needs the old text to remove entries.
    tx.execute(
        "INSERT INTO document_chunk_fts(document_chunk_fts, rowid, text)
         SELECT 'delete', rowid, text FROM document_chunk WHERE document_id=?1",
        [doc_id],
    )?;
    if table_exists(&tx, "document_chunk_vec")? {
        tx.execute(
            "DELETE FROM document_chunk_vec WHERE rowid IN (SELECT rowid FROM document_chunk WHERE document_id=?1)",
            [doc_id],
        )?;
    }
    let chunks: i64 = tx.execute("DELETE FROM document_chunk WHERE document_id=?1", [doc_id])? as i64;

    // Text derived from the document: every table with a `source_ref` that cites it. Not what a person already
    // accepted from it (IA6): the datum is the institution's now; its origin (`core_field`) and the history of its
    // changes (`core_change`, which only grows) keep saying where it came from.
    let tables: Vec<String> = {
        let mut stmt = tx.prepare(
            "SELECT m.name FROM sqlite_master m, pragma_table_info(m.name) p
             WHERE m.type='table' AND p.name='source_ref' AND m.name NOT IN ('core_field', 'core_change', 'core_record_field')",
        )?;
        let v = stmt.query_map([], |r| r.get(0))?.collect::<Result<Vec<String>, _>>()?;
        v
    };
    // what the person rejected from it is remembered only by its fingerprint (ADR-038): no value, no document
    tx.execute(
        "UPDATE ai_proposal SET value_json = 'null', source_ref = json_object('sha', json_extract(source_ref, '$.sha'))
         WHERE status = 'rejected' AND json_valid(source_ref) AND json_extract(source_ref, '$.document_id') = ?1",
        [doc_id],
    )?;
    let mut derived = 0i64;
    for t in &tables {
        derived += tx.execute(
            &format!("DELETE FROM {t} WHERE json_valid(source_ref) AND json_extract(source_ref,'$.document_id') = ?1"),
            [doc_id],
        )? as i64;
    }
    // A reading of a call quotes the files it was made from: it goes with any one of them.
    derived += tx.execute("DELETE FROM call_reading WHERE id IN (SELECT reading_id FROM call_reading_file WHERE document_id=?1)", [doc_id])? as i64;
    // Direct references. A document of the Expediente keeps its row and its state: it only loses its file.
    tx.execute("UPDATE core_record SET document_id=NULL, original_path=NULL, original_name=NULL WHERE document_id=?1", [doc_id])?;
    derived += tx.execute("DELETE FROM questionnaire WHERE source_document_id=?1", [doc_id])? as i64;
    tx.execute("UPDATE budget_item SET quote_document_id=NULL WHERE quote_document_id=?1", [doc_id])?;
    tx.execute("UPDATE call_template SET word_template_document_id=NULL WHERE word_template_document_id=?1", [doc_id])?;

    tx.execute("DELETE FROM document WHERE id=?1", [doc_id])?;
    audit::record(
        &tx,
        AuditKind::EmergencyDelete,
        Some("document"),
        None,
        json!({ "chunks": chunks, "derived": derived }),
    )?;
    tx.commit()?;
    conn.execute_batch("VACUUM;")?;
    Ok(DeleteSummary { chunks, derived_rows: derived })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::open_encrypted;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn conn() -> (tempfile::TempDir, std::path::PathBuf, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.db");
        let c = open_encrypted(&path, KEY).unwrap();
        (dir, path, c)
    }

    #[test]
    fn adds_a_document_with_chunks_and_search() {
        let (_d, _p, mut c) = conn();
        let doc = add_text_document(&mut c, Drawer::Projects, "Convocatoria", "El monto máximo es de 150000 pesos.\n\nPlazo: 12 meses.", 0).unwrap();
        assert_eq!(doc.chunks, 1);
        assert_eq!(doc.data_level, "green");
        let n: i64 = c
            .query_row("SELECT count(*) FROM document_chunk_fts WHERE document_chunk_fts MATCH 'monto'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let (ev, d): (String, String) = c
            .query_row("SELECT event, details_json FROM audit_log", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(ev, "document.uploaded");
        assert!(!d.contains("monto"));
    }

    #[test]
    fn emergency_delete_removes_everything_that_came_from_the_document() {
        let (_d, path, mut c) = conn();
        let keep = add_text_document(&mut c, Drawer::Projects, "Otra", "Texto que se conserva sobre becas.", 0).unwrap();
        let doc = add_text_document(&mut c, Drawer::Record, "Interno", "Dato secreto zanahoria morada en el documento.", 1).unwrap();
        // something derived that cites the document
        c.execute(
            "INSERT INTO institution (id,name,kind,created_at,updated_at) VALUES ('i','X','other','t','t')",
            [],
        ).unwrap();
        c.execute("INSERT INTO institution_profile (id,institution_id,version,created_at) VALUES ('p','i',1,'t')", []).unwrap();
        c.execute(
            "INSERT INTO income_source (id,profile_id,label,origin,source_ref) VALUES ('f1','p','Baño zanahoria','document',?1)",
            [format!("{{\"document_id\":\"{}\",\"page\":1}}", doc.id)],
        ).unwrap();
        c.execute(
            "INSERT INTO income_source (id,profile_id,label,origin,source_ref) VALUES ('f2','p','Cocina','document',?1)",
            [format!("{{\"document_id\":\"{}\"}}", keep.id)],
        ).unwrap();

        let s = emergency_delete_document(&mut c, &doc.id).unwrap();
        assert_eq!(s, DeleteSummary { chunks: 1, derived_rows: 1 });

        assert_eq!(list(&c).unwrap().len(), 1);
        let q = |sql: &str| -> i64 { c.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(q("SELECT count(*) FROM document_chunk"), 1);
        assert_eq!(q("SELECT count(*) FROM income_source"), 1);
        assert_eq!(q("SELECT count(*) FROM document_chunk_fts WHERE document_chunk_fts MATCH 'zanahoria'"), 0);
        assert_eq!(q("SELECT count(*) FROM document_chunk_fts WHERE document_chunk_fts MATCH 'becas'"), 1);
        // audit says it happened, without content
        let d: String = c
            .query_row("SELECT details_json FROM audit_log WHERE event='emergency.delete'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(d, "{\"chunks\":1,\"derived\":1}");
        drop(c);
        // the file itself is encrypted, but also check against a re-opened db that no leftovers exist
        let c2 = open_encrypted(&path, KEY).unwrap();
        let left: i64 = c2
            .query_row("SELECT count(*) FROM document WHERE extracted_text LIKE '%zanahoria%'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(left, 0);
    }

    #[test]
    fn deleting_an_unknown_document_does_nothing() {
        let (_d, _p, mut c) = conn();
        assert_eq!(emergency_delete_document(&mut c, "doc_nope").unwrap(), DeleteSummary { chunks: 0, derived_rows: 0 });
        let n: i64 = c.query_row("SELECT count(*) FROM audit_log", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }
}
