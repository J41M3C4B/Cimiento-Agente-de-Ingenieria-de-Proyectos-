//! Where each datum of the institution comes from and how it changed (ADR-033 §4, audit D6). A datum is a field of
//! the catalog (`core::institution::forms`), by its id.
//!
//! - `core_field` keeps the origin of what is written today: who confirmed it, when, and from where.
//! - `core_change` adds one line per change and never edits nor deletes one: with it, how a datum was on any day can
//!   be told. What is the institution's own and protected says only that it changed, never the value.
//!
//! Both are written in the same transaction as the datum, so there is never a datum without its line.

use crate::common::forms::{is_filled, Sensitivity, Values};
use crate::storage::StorageError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Where a datum comes from (principle 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Source {
    /// `user`, `document`, `ai_assumption` or `computed`.
    pub origin: &'static str,
    /// A document and its page, when it came from one.
    pub source_ref: Option<String>,
}

impl Source {
    /// A person wrote it.
    pub fn user() -> Self {
        Source { origin: "user", source_ref: None }
    }
}

/// The origin of a datum written today, as the screen and the assistant read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldOrigin {
    pub origin: String,
    pub source_ref: Option<String>,
    /// `None` for a datum written before origins were kept.
    pub confirmed_at: Option<String>,
}

/// One change of a datum.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Change {
    pub field: String,
    /// The new value; `None` when it was emptied or when the datum is protected.
    pub value: Option<Value>,
    pub protected: bool,
    pub origin: String,
    pub changed_at: String,
    pub changed_by: Option<String>,
}

/// Whether a datum is the institution's own and protected: its value never goes into the history.
fn protected(field: &str) -> bool {
    crate::core::institution::forms::FORMS
        .iter()
        .find_map(|f| f.field(field))
        .is_some_and(|f| matches!(f.sensitivity, Sensitivity::InstitutionalPrivate | Sensitivity::Personal))
}

/// Writes what changed between two sets of values of the catalog: a line in the history for each datum whose value
/// is not the same, and its origin. Returns how many changed. Who acts comes from the session of the connection, as
/// in the audit log.
pub fn record(conn: &Connection, before: &Values, after: &Values, source: &Source) -> Result<usize, StorageError> {
    conn.execute_batch("CREATE TEMP TABLE IF NOT EXISTS session_actor (user_id TEXT);")?;
    let mut fields: Vec<&String> = before.keys().chain(after.keys()).collect();
    fields.sort();
    fields.dedup();
    let mut changed = 0;
    for field in fields {
        let (old, new) = (before.get(field).filter(|v| is_filled(Some(v))), after.get(field).filter(|v| is_filled(Some(v))));
        if old == new {
            continue;
        }
        let hidden = protected(field);
        let value = new.filter(|_| !hidden).map(Value::to_string);
        conn.execute(
            "INSERT INTO core_change (field, value_json, protected, origin, changed_at, changed_by)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ','now'), (SELECT user_id FROM temp.session_actor LIMIT 1))",
            params![field, value, hidden, source.origin],
        )?;
        if new.is_some() {
            // a person wrote it or accepted it: it is confirmed now
            conn.execute(
                "INSERT INTO core_field (field, origin, source_ref, confirmed_at, confirmed_by, updated_at)
                 VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ','now'), (SELECT user_id FROM temp.session_actor LIMIT 1),
                         strftime('%Y-%m-%dT%H:%M:%SZ','now'))
                 ON CONFLICT(field) DO UPDATE SET origin=excluded.origin, source_ref=excluded.source_ref,
                        confirmed_at=excluded.confirmed_at, confirmed_by=excluded.confirmed_by, updated_at=excluded.updated_at",
                params![field, source.origin, source.source_ref],
            )?;
        } else {
            conn.execute("DELETE FROM core_field WHERE field = ?1", [field])?;
        }
        changed += 1;
    }
    Ok(changed)
}

/// The origin of each written datum of these values. One with no row was written by a person before origins were
/// kept: no document nor AI wrote the institution before (N3b).
pub fn origins(conn: &Connection, values: &Values) -> Result<BTreeMap<String, FieldOrigin>, StorageError> {
    let mut stmt = conn.prepare("SELECT origin, source_ref, confirmed_at FROM core_field WHERE field = ?1")?;
    let mut out = BTreeMap::new();
    for field in values.iter().filter(|(_, v)| is_filled(Some(v))).map(|(f, _)| f) {
        let found = stmt
            .query_row([field], |r| Ok(FieldOrigin { origin: r.get(0)?, source_ref: r.get(1)?, confirmed_at: r.get(2)? }))
            .optional()?;
        out.insert(field.clone(), found.unwrap_or(FieldOrigin { origin: "user".into(), source_ref: None, confirmed_at: None }));
    }
    Ok(out)
}

/// The changes of one datum, or of all, newest first.
pub fn changes(conn: &Connection, field: Option<&str>, limit: usize) -> Result<Vec<Change>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT field, value_json, protected, origin, changed_at, changed_by FROM core_change
         WHERE ?1 IS NULL OR field = ?1 ORDER BY changed_at DESC, id DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![field, limit as i64], |r| {
        Ok(Change {
            field: r.get(0)?,
            value: r.get::<_, Option<String>>(1)?.and_then(|s| serde_json::from_str(&s).ok()),
            protected: r.get::<_, i64>(2)? != 0,
            origin: r.get(3)?,
            changed_at: r.get(4)?,
            changed_by: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// How a datum was at a moment (`YYYY-MM-DDTHH:MM:SSZ`): the value of its last change up to then. `None` when it was
/// empty, had not been written yet, or is protected.
pub fn value_at(conn: &Connection, field: &str, at: &str) -> Result<Option<Value>, StorageError> {
    let raw: Option<Option<String>> = conn
        .query_row(
            "SELECT value_json FROM core_change WHERE field = ?1 AND changed_at <= ?2 ORDER BY changed_at DESC, id DESC LIMIT 1",
            params![field, at],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw.flatten().and_then(|s| serde_json::from_str(&s).ok()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = crate::storage::open_encrypted(&dir.path().join("t.db"), "k").unwrap();
        (dir, c)
    }

    fn values(v: Value) -> Values {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn only_what_changed_is_written_and_its_origin_follows() {
        let (_d, c) = conn();
        let first = values(json!({ "institution.name": "Casa", "institution.mission": "Cuidar." }));
        assert_eq!(record(&c, &Values::new(), &first, &Source::user()).unwrap(), 2);
        // the same values again: nothing new
        assert_eq!(record(&c, &first, &first, &Source::user()).unwrap(), 0);
        let second = values(json!({ "institution.name": "Casa Ficticia", "institution.mission": "Cuidar." }));
        let doc = Source { origin: "document", source_ref: Some(r#"{"document_id":"doc_1","page":2}"#.into()) };
        assert_eq!(record(&c, &first, &second, &doc).unwrap(), 1);

        let all = changes(&c, None, 10).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!((all[0].field.as_str(), all[0].value.clone(), all[0].origin.as_str()), ("institution.name", Some(json!("Casa Ficticia")), "document"));
        let o = origins(&c, &second).unwrap();
        assert_eq!((o["institution.name"].origin.as_str(), o["institution.mission"].origin.as_str()), ("document", "user"));
        assert!(o["institution.name"].source_ref.as_deref().unwrap().contains("doc_1"));
        assert!(o["institution.mission"].confirmed_at.is_some());
    }

    /// A datum emptied leaves its line (with no value) and loses its origin; a blank text counts as empty.
    #[test]
    fn emptying_a_datum_is_a_change_too() {
        let (_d, c) = conn();
        let full = values(json!({ "institution.services": "Comedor." }));
        record(&c, &Values::new(), &full, &Source::user()).unwrap();
        assert_eq!(record(&c, &full, &values(json!({ "institution.services": "  " })), &Source::user()).unwrap(), 1);
        let last = &changes(&c, Some("institution.services"), 1).unwrap()[0];
        assert_eq!((last.value.clone(), last.protected), (None, false));
        assert!(origins(&c, &Values::new()).unwrap().is_empty());
        let n: i64 = c.query_row("SELECT count(*) FROM core_field", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    /// ADR-033 §4: what is protected (address, RFC, folios, keys, representative) says only that it changed.
    #[test]
    fn a_protected_datum_keeps_no_value_in_the_history() {
        let (_d, c) = conn();
        let after = values(json!({ "institution.legal_rfc": "AFI870101AB1", "institution.street": "Calle Ficticia", "institution.legal_name": "Casa, I.A.P." }));
        record(&c, &Values::new(), &after, &Source::user()).unwrap();
        let raw: Vec<(String, Option<String>, i64)> = c
            .prepare("SELECT field, value_json, protected FROM core_change ORDER BY field")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            raw,
            vec![
                ("institution.legal_name".into(), Some("\"Casa, I.A.P.\"".into()), 0),
                ("institution.legal_rfc".into(), None, 1),
                ("institution.street".into(), None, 1),
            ]
        );
        assert_eq!(value_at(&c, "institution.legal_rfc", "9999-12-31T00:00:00Z").unwrap(), None);
    }

    #[test]
    fn the_history_tells_how_a_datum_was_and_never_shrinks() {
        let (_d, c) = conn();
        record(&c, &Values::new(), &values(json!({ "institution.capacity_total": 20 })), &Source::user()).unwrap();
        // a line written earlier, as if a month ago
        c.execute("INSERT INTO core_change (field, value_json, protected, origin, changed_at) VALUES ('institution.capacity_total', '12', 0, 'user', '2000-01-01T00:00:00Z')", []).unwrap();
        assert_eq!(value_at(&c, "institution.capacity_total", "2000-06-01T00:00:00Z").unwrap(), Some(json!(12)));
        assert_eq!(value_at(&c, "institution.capacity_total", "9999-12-31T00:00:00Z").unwrap(), Some(json!(20)));
        assert_eq!(value_at(&c, "institution.capacity_total", "1999-01-01T00:00:00Z").unwrap(), None);
        assert!(c.execute("UPDATE core_change SET value_json = '99'", []).is_err(), "a line is never edited");
        assert!(c.execute("DELETE FROM core_change", []).is_err(), "nor deleted");
    }

    #[test]
    fn who_changed_it_comes_from_the_session() {
        let (_d, c) = conn();
        c.execute_batch("CREATE TEMP TABLE IF NOT EXISTS session_actor (user_id TEXT); INSERT INTO session_actor VALUES ('usr_1');").unwrap();
        record(&c, &Values::new(), &values(json!({ "institution.name": "Casa" })), &Source::user()).unwrap();
        assert_eq!(changes(&c, Some("institution.name"), 1).unwrap()[0].changed_by.as_deref(), Some("usr_1"));
    }
}
