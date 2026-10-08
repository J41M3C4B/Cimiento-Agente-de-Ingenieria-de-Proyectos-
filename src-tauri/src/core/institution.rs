//! What the core says about the institution to the rest of the app.

use rusqlite::{Connection, OptionalExtension};

/// The kind of institution (`elderly_home`, `children_home`, …), or `None` before it is written. Nobody else reads it
/// from the table: each module turns it into its own `Flavor` (`Flavor::from_kind`).
pub fn kind(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT kind FROM institution LIMIT 1", [], |r| r.get(0)).optional().ok().flatten()
}

/// The kind of institution as the staff module understands it (the positions it suggests).
pub fn hr_flavor(conn: &Connection) -> crate::modules::hr::Flavor {
    crate::modules::hr::Flavor::from_kind(kind(conn).as_deref())
}

/// The kind of institution as the module of the people served understands it (extra data and age bands).
pub fn care_flavor(conn: &Connection) -> crate::modules::care::Flavor {
    crate::modules::care::Flavor::from_kind(kind(conn).as_deref())
}

/// The kind of institution as the facilities module understands it (the spaces and equipment it suggests first).
pub fn facilities_flavor(conn: &Connection) -> crate::modules::facilities::Flavor {
    crate::modules::facilities::Flavor::from_kind(kind(conn).as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kind_is_read_once_for_every_module() {
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::storage::open_encrypted(&dir.path().join("t.db"), "k").unwrap();
        assert_eq!(kind(&conn), None);
        assert_eq!(hr_flavor(&conn), crate::modules::hr::Flavor::Other);
        conn.execute("INSERT INTO institution (id, name, kind, created_at, updated_at) VALUES ('i','Casa','children_home','t','t')", []).unwrap();
        assert_eq!(kind(&conn).as_deref(), Some("children_home"));
        assert_eq!(care_flavor(&conn), crate::modules::care::Flavor::ChildrenHome);
        assert_eq!(facilities_flavor(&conn), crate::modules::facilities::Flavor::ChildrenHome);
    }
}
