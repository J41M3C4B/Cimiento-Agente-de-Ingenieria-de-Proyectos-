//! Clean documents (text only), their fragments and the emergency delete.

use super::StorageError;
use crate::audit::{self, AuditKind};
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
    pub data_level: String,
    pub redactions_count: i64,
    pub chunks: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DeleteSummary {
    pub chunks: i64,
    pub derived_rows: i64,
}

/// Saves the CLEAN text of a document (already scanned) with its fragments and search index.
pub fn add_text_document(
    conn: &mut Connection,
    kind: &str,
    display_name: &str,
    clean_text: &str,
    redactions_count: i64,
) -> Result<DocumentSummary, StorageError> {
    let data_level = match kind {
        "internal" | "quote" => "yellow",
        _ => "green",
    };
    let doc_id = format!("doc_{}", Ulid::generate());
    let hash = hex::encode(Sha256::digest(clean_text.as_bytes()));
    let chunks = chunk_text(clean_text);

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO document (id,kind,display_name,mime,data_level,clean_hash,extracted_text,redactions_count,created_at)
         VALUES (?1,?2,?3,'text/plain',?4,?5,?6,?7,strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![doc_id, kind, display_name.trim(), data_level, hash, clean_text, redactions_count],
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
pub fn add_call_document(conn: &Connection, display_name: &str, mime: &str, pages: &[String], redactions_count: i64) -> Result<String, StorageError> {
    let doc_id = format!("doc_{}", Ulid::generate());
    let text = pages.join("

");
    let hash = hex::encode(Sha256::digest(text.as_bytes()));
    conn.execute(
        "INSERT INTO document (id,kind,display_name,mime,data_level,clean_hash,extracted_text,redactions_count,created_at)
         VALUES (?1,'call',?2,?3,'green',?4,?5,?6,strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![doc_id, display_name.trim(), mime, hash, text, redactions_count],
    )?;
    for (i, page) in pages.iter().enumerate() {
        conn.execute(
            "INSERT INTO document_chunk (id,document_id,ordinal,page,text) VALUES (?1,?2,?3,?4,?5)",
            params![format!("chunk_{}", Ulid::generate()), doc_id, i as i64, i as i64 + 1, page],
        )?;
    }
    conn.execute("INSERT INTO document_chunk_fts(rowid,text) SELECT rowid,text FROM document_chunk WHERE document_id=?1", [&doc_id])?;
    audit::record(conn, AuditKind::DocumentUploaded, Some("document"), Some(&doc_id), json!({ "kind": "call", "chunks": pages.len(), "redactions": redactions_count }))?;
    Ok(doc_id)
}

fn get(conn: &Connection, id: &str) -> Result<Option<DocumentSummary>, StorageError> {
    Ok(list(conn)?.into_iter().find(|d| d.id == id))
}

/// The documents of the institution: the files of a call belong to their project.
pub fn list_institution(conn: &Connection) -> Result<Vec<DocumentSummary>, StorageError> {
    Ok(list(conn)?.into_iter().filter(|d| d.kind != "call").collect())
}

pub fn list(conn: &Connection) -> Result<Vec<DocumentSummary>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.kind, d.display_name, d.data_level, d.redactions_count, d.created_at,
                (SELECT count(*) FROM document_chunk c WHERE c.document_id = d.id)
         FROM document d ORDER BY d.created_at DESC, d.id DESC",
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
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

    // Text derived from the document: every table with a `source_ref` that cites it.
    let tables: Vec<String> = {
        let mut stmt = tx.prepare(
            "SELECT m.name FROM sqlite_master m, pragma_table_info(m.name) p
             WHERE m.type='table' AND p.name='source_ref'",
        )?;
        let v = stmt.query_map([], |r| r.get(0))?.collect::<Result<Vec<String>, _>>()?;
        v
    };
    let mut derived = 0i64;
    for t in &tables {
        derived += tx.execute(
            &format!("DELETE FROM {t} WHERE json_valid(source_ref) AND json_extract(source_ref,'$.document_id') = ?1"),
            [doc_id],
        )? as i64;
    }
    // A reading of a call quotes the files it was made from: it goes with any one of them.
    derived += tx.execute("DELETE FROM call_reading WHERE id IN (SELECT reading_id FROM call_reading_file WHERE document_id=?1)", [doc_id])? as i64;
    // Direct references.
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
        let doc = add_text_document(&mut c, "call", "Convocatoria", "El monto máximo es de 150000 pesos.\n\nPlazo: 12 meses.", 0).unwrap();
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
        let keep = add_text_document(&mut c, "call", "Otra", "Texto que se conserva sobre becas.", 0).unwrap();
        let doc = add_text_document(&mut c, "internal", "Interno", "Dato secreto zanahoria morada en el documento.", 1).unwrap();
        // something derived that cites the document
        c.execute(
            "INSERT INTO institution (id,name,kind,created_at,updated_at) VALUES ('i','X','other','t','t')",
            [],
        ).unwrap();
        c.execute("INSERT INTO institution_profile (id,institution_id,version,created_at) VALUES ('p','i',1,'t')", []).unwrap();
        c.execute(
            "INSERT INTO facility (id,profile_id,kind,count,origin,source_ref) VALUES ('f1','p','Baño zanahoria',1,'document',?1)",
            [format!("{{\"document_id\":\"{}\",\"page\":1}}", doc.id)],
        ).unwrap();
        c.execute(
            "INSERT INTO facility (id,profile_id,kind,count,origin,source_ref) VALUES ('f2','p','Cocina',1,'document',?1)",
            [format!("{{\"document_id\":\"{}\"}}", keep.id)],
        ).unwrap();

        let s = emergency_delete_document(&mut c, &doc.id).unwrap();
        assert_eq!(s, DeleteSummary { chunks: 1, derived_rows: 1 });

        assert_eq!(list(&c).unwrap().len(), 1);
        let q = |sql: &str| -> i64 { c.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(q("SELECT count(*) FROM document_chunk"), 1);
        assert_eq!(q("SELECT count(*) FROM facility"), 1);
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
