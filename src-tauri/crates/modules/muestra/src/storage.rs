//! The tables of the sample module and all of their SQL.

use rusqlite::{params, Connection, OptionalExtension};

use crate::contract::{NoteInput, NoteRef};

/// A note as it is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub title: String,
    pub body: Option<String>,
    pub phone: Option<String>,
    pub version: i64,
}

pub fn find(conn: &Connection, note: &NoteRef) -> rusqlite::Result<Option<Note>> {
    conn.query_row("SELECT title, body, phone, version FROM muestra_note WHERE id = ?1", [&note.0], |r| {
        Ok(Note { title: r.get(0)?, body: r.get(1)?, phone: r.get(2)?, version: r.get(3)? })
    })
    .optional()
}

/// Writes the note at its next version: a new one at version 1, or the one that is still at `base`.
pub fn save(conn: &Connection, input: &NoteInput, base: i64) -> rusqlite::Result<()> {
    if base == 0 {
        conn.execute(
            "INSERT INTO muestra_note (id, title, body, phone, version, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, strftime('%Y-%m-%dT%H:%M:%SZ','now'), strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
            params![input.id, input.title, input.body, input.phone],
        )?;
    } else {
        conn.execute(
            "UPDATE muestra_note SET title = ?2, body = ?3, phone = ?4, version = version + 1,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
             WHERE id = ?1 AND version = ?5",
            params![input.id, input.title, input.body, input.phone, base],
        )?;
    }
    Ok(())
}
