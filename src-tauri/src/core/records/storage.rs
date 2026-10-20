//! The rows of `core_record`: only this file writes SQL on it.

use crate::storage::StorageError;
use rusqlite::{params, Connection, OptionalExtension};

/// A row as it is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub r#type: String,
    pub custom_name: Option<String>,
    pub status: String,
    pub document_id: Option<String>,
    /// The clean text is still there (not hidden while its deletion waits, ADR-028).
    pub document_shown: bool,
    /// How many pages of its clean text were read from a picture (OCR, ADR-035), and which with doubts.
    pub picture_pages: usize,
    pub doubtful_pages: Vec<usize>,
    pub original_path: Option<String>,
    pub original_name: Option<String>,
    pub issued_on: Option<String>,
    pub valid_until: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub origin: String,
    pub confirmed_at: Option<String>,
}

const COLUMNS: &str = "r.id, r.type, r.custom_name, r.status, r.document_id, (d.id IS NOT NULL AND d.hidden = 0), r.original_path, r.original_name,
     r.issued_on, r.valid_until, r.reference, r.notes, r.origin, r.confirmed_at, coalesce(d.picture_pages, '[]'), coalesce(d.doubtful_pages, '[]')";

fn read(r: &rusqlite::Row) -> rusqlite::Result<Row> {
    Ok(Row {
        id: r.get(0)?,
        r#type: r.get(1)?,
        custom_name: r.get(2)?,
        status: r.get(3)?,
        document_id: r.get(4)?,
        document_shown: r.get(5)?,
        original_path: r.get(6)?,
        original_name: r.get(7)?,
        issued_on: r.get(8)?,
        valid_until: r.get(9)?,
        reference: r.get(10)?,
        notes: r.get(11)?,
        origin: r.get(12)?,
        confirmed_at: r.get(13)?,
        picture_pages: serde_json::from_str::<Vec<usize>>(&r.get::<_, String>(14)?).unwrap_or_default().len(),
        doubtful_pages: serde_json::from_str(&r.get::<_, String>(15)?).unwrap_or_default(),
    })
}

/// Every row, in the order they were added.
pub fn rows(conn: &Connection) -> Result<Vec<Row>, StorageError> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM core_record r LEFT JOIN document d ON d.id = r.document_id ORDER BY r.created_at, r.id"))?;
    let rows = stmt.query_map([], read)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn by_id(conn: &Connection, id: &str) -> Result<Option<Row>, StorageError> {
    Ok(conn
        .query_row(&format!("SELECT {COLUMNS} FROM core_record r LEFT JOIN document d ON d.id = r.document_id WHERE r.id = ?1"), [id], read)
        .optional()?)
}

/// The row whose file is this document of the store, if any.
pub fn by_document(conn: &Connection, document_id: &str) -> Result<Option<Row>, StorageError> {
    Ok(conn
        .query_row(&format!("SELECT {COLUMNS} FROM core_record r LEFT JOIN document d ON d.id = r.document_id WHERE r.document_id = ?1"), [document_id], read)
        .optional()?)
}

/// The row of a type of the catalog (one per type, while there is a single site).
pub fn by_type(conn: &Connection, r#type: &str) -> Result<Option<Row>, StorageError> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM core_record r LEFT JOIN document d ON d.id = r.document_id WHERE r.type = ?1 AND r.site_id IS NULL"),
            [r#type],
            read,
        )
        .optional()?)
}

/// What a person writes of a document.
pub struct Input<'a> {
    pub custom_name: Option<&'a str>,
    pub status: &'a str,
    pub issued_on: Option<&'a str>,
    pub valid_until: Option<&'a str>,
    pub reference: Option<&'a str>,
    pub notes: Option<&'a str>,
}

/// Writes a document of the record (a new one with `id = None`), confirmed now by whoever acts in this connection.
/// The file stays as it was. Returns its id.
pub fn save(conn: &Connection, id: Option<&str>, r#type: &str, i: &Input<'_>) -> Result<String, StorageError> {
    conn.execute_batch("CREATE TEMP TABLE IF NOT EXISTS session_actor (user_id TEXT);")?;
    let id = match id {
        Some(id) => {
            conn.execute(
                "UPDATE core_record SET custom_name = ?2, status = ?3, issued_on = ?4, valid_until = ?5, reference = ?6, notes = ?7,
                        origin = 'user', source_ref = NULL, confirmed_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'),
                        confirmed_by = (SELECT user_id FROM temp.session_actor LIMIT 1), updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
                 WHERE id = ?1",
                params![id, i.custom_name, i.status, i.issued_on, i.valid_until, i.reference, i.notes],
            )?;
            id.to_string()
        }
        None => {
            let id = format!("rec_{}", ulid::Ulid::generate());
            conn.execute(
                "INSERT INTO core_record (id, type, custom_name, status, issued_on, valid_until, reference, notes, origin, confirmed_at, confirmed_by, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'user', strftime('%Y-%m-%dT%H:%M:%SZ','now'), (SELECT user_id FROM temp.session_actor LIMIT 1),
                         strftime('%Y-%m-%dT%H:%M:%SZ','now'), strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
                params![id, r#type, i.custom_name, i.status, i.issued_on, i.valid_until, i.reference, i.notes],
            )?;
            id
        }
    };
    Ok(id)
}

/// Puts a file on a document: its clean text (`None` when it has no text to read) and where the original is. The
/// institution has it from now on.
pub fn set_file(conn: &Connection, id: &str, document_id: Option<&str>, path: &str, name: &str) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE core_record SET document_id = ?2, original_path = ?3, original_name = ?4, status = 'have',
                updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?1",
        params![id, document_id, path, name],
    )?;
    Ok(())
}

/// The column of `core_record` a datum of its own window is kept in.
fn column_of(field: &str) -> Option<&'static str> {
    Some(match field {
        "institution.record.issued_on" => "issued_on",
        "institution.record.valid_until" => "valid_until",
        "institution.record.reference" => "reference",
        _ => return None,
    })
}

/// Writes one datum of a document read from its file (ADR-038). `false` for a field that is not one of its own.
pub fn write_read(conn: &Connection, id: &str, field: &str, value: &str) -> Result<bool, StorageError> {
    let Some(column) = column_of(field) else { return Ok(false) };
    conn.execute(
        &format!("UPDATE core_record SET {column} = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1"),
        params![id, value],
    )?;
    Ok(true)
}

/// Where each datum of a document came from, by field: (origin, source_ref). A datum without a line is the person's.
pub fn field_origins(conn: &Connection, id: &str) -> Result<std::collections::BTreeMap<String, (String, Option<String>)>, StorageError> {
    let mut stmt = conn.prepare("SELECT field, origin, source_ref FROM core_record_field WHERE record_id = ?1")?;
    let rows = stmt.query_map([id], |r| Ok((r.get::<_, String>(0)?, (r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Says where a datum of a document came from (`user`, `document` or `computed`).
pub fn set_field_origin(conn: &Connection, id: &str, field: &str, origin: &str, source_ref: Option<&str>) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO core_record_field (record_id, field, origin, source_ref, updated_at) VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         ON CONFLICT (record_id, field) DO UPDATE SET origin = excluded.origin, source_ref = excluded.source_ref, updated_at = excluded.updated_at",
        params![id, field, origin, source_ref],
    )?;
    Ok(())
}

/// Every document of the record and where each datum came from (development only: an example replaces the record).
/// The files stay in the archive.
pub fn clear_all(conn: &Connection) -> Result<(), StorageError> {
    conn.execute("DELETE FROM core_record_field", [])?;
    conn.execute("DELETE FROM core_record", [])?;
    Ok(())
}

/// Forgets where a datum came from, once it is emptied: the next file may fill it again.
pub fn clear_field_origin(conn: &Connection, id: &str, field: &str) -> Result<(), StorageError> {
    conn.execute("DELETE FROM core_record_field WHERE record_id = ?1 AND field = ?2", params![id, field])?;
    Ok(())
}

/// Takes the file off a document (its text is deleted by the caller, ADR-036).
pub fn clear_file(conn: &Connection, id: &str) -> Result<bool, StorageError> {
    Ok(conn.execute(
        "UPDATE core_record SET document_id = NULL, original_path = NULL, original_name = NULL, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1",
        [id],
    )? > 0)
}
