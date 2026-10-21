//! The tables of the sample module and all of their SQL.

use rusqlite::{Connection, OptionalExtension};

use crate::contract::NoteRef;

pub fn add(conn: &Connection, id: &str, title: &str) -> rusqlite::Result<NoteRef> {
    conn.execute(
        "INSERT INTO muestra_note (id, title, created_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        [id, title],
    )?;
    Ok(NoteRef(id.to_string()))
}

pub fn title(conn: &Connection, note: &NoteRef) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT title FROM muestra_note WHERE id = ?1", [&note.0], |r| r.get(0)).optional()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_note_is_kept_and_found_by_its_reference() {
        let mut conn = Connection::open_in_memory().unwrap();
        kernel::migrate(&mut conn, &kernel::Modules::discover().unwrap()).unwrap();
        let note = add(&conn, "n1", "Primera nota").unwrap();
        assert_eq!(title(&conn, &note).unwrap().as_deref(), Some("Primera nota"));
        assert_eq!(title(&conn, &NoteRef("otra".into())).unwrap(), None);
    }
}
