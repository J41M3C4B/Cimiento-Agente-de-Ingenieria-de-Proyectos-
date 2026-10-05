//! Persistence of the roster (ADR-020): the fields of each form and the records. Apart from the profile.

use super::StorageError;
use crate::domain::profile::InstitutionKind;
use crate::domain::roster::{Data, Entity, FieldKind, FieldOption, RosterEntry, RosterField};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use ulid::Ulid;

/// What the person writes when they add or edit a field. The options are plain texts, one per choice.
#[derive(Debug, Clone, Deserialize)]
pub struct FieldInput {
    /// `None` creates a field of their own.
    pub key: Option<String>,
    pub title: String,
    pub kind: FieldKind,
    #[serde(default)]
    pub options: Vec<String>,
}

fn institution_kind(conn: &Connection) -> Result<InstitutionKind, StorageError> {
    let kind: Option<String> = conn.query_row("SELECT kind FROM institution LIMIT 1", [], |r| r.get(0)).optional()?;
    Ok(kind.map_or(InstitutionKind::Other, |k| InstitutionKind::from_db(&k)))
}

fn load_fields(conn: &Connection, entity: Entity) -> Result<Vec<RosterField>, StorageError> {
    let rows = conn
        .prepare("SELECT key, title, kind, options, builtin, locked_options, required, position FROM roster_field WHERE entity=?1 ORDER BY position")?
        .query_map([entity.as_db()], |r| {
            Ok(RosterField {
                key: r.get(0)?,
                title: r.get(1)?,
                kind: FieldKind::from_db(&r.get::<_, String>(2)?).unwrap_or(FieldKind::Text),
                options: serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default(),
                builtin: r.get::<_, i64>(4)? != 0,
                locked_options: r.get::<_, i64>(5)? != 0,
                required: r.get::<_, i64>(6)? != 0,
                position: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The fields of a form. The first time, they are written from the defaults for the kind of institution.
pub fn fields(conn: &Connection, entity: Entity) -> Result<Vec<RosterField>, StorageError> {
    let existing = load_fields(conn, entity)?;
    if !existing.is_empty() {
        return Ok(existing);
    }
    for f in crate::domain::roster::default_fields(entity, institution_kind(conn)?) {
        conn.execute(
            "INSERT INTO roster_field (entity,key,title,kind,options,builtin,locked_options,required,position) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                entity.as_db(),
                f.key,
                f.title,
                f.kind.as_db(),
                serde_json::to_string(&f.options).unwrap_or_else(|_| "[]".into()),
                f.builtin as i64,
                f.locked_options as i64,
                f.required as i64,
                f.position
            ],
        )?;
    }
    load_fields(conn, entity)
}

fn option_list(labels: &[String]) -> Vec<FieldOption> {
    let mut out: Vec<FieldOption> = Vec::new();
    for l in labels.iter().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        if !out.iter().any(|o| o.label == l) {
            out.push(FieldOption { value: l.into(), label: l.into() });
        }
    }
    out
}

/// Adds a field of their own, or edits one: the title and the options of any field, and the type of their own.
pub fn save_field(conn: &Connection, entity: Entity, input: &FieldInput) -> Result<Vec<RosterField>, StorageError> {
    let current = fields(conn, entity)?;
    let title = input.title.trim();
    match &input.key {
        None => {
            // a field of their own is free text, a selector or a number
            let kind = match input.kind {
                FieldKind::Select | FieldKind::Number => input.kind,
                _ => FieldKind::Text,
            };
            let options = if kind == FieldKind::Select { option_list(&input.options) } else { vec![] };
            let position = current.iter().map(|f| f.position).max().unwrap_or(0) + 1;
            conn.execute(
                "INSERT INTO roster_field (entity,key,title,kind,options,builtin,locked_options,required,position) VALUES (?1,?2,?3,?4,?5,0,0,0,?6)",
                params![
                    entity.as_db(),
                    format!("c_{}", Ulid::generate().to_string().to_lowercase()),
                    title,
                    kind.as_db(),
                    serde_json::to_string(&options).unwrap_or_else(|_| "[]".into()),
                    position
                ],
            )?;
        }
        Some(key) => {
            let Some(field) = current.iter().find(|f| &f.key == key) else { return Ok(current) };
            let kind = if field.builtin || !matches!(input.kind, FieldKind::Select | FieldKind::Number | FieldKind::Text) {
                field.kind
            } else {
                input.kind
            };
            let options = if kind != FieldKind::Select || field.locked_options {
                field.options.clone()
            } else {
                option_list(&input.options)
            };
            conn.execute(
                "UPDATE roster_field SET title=?3, kind=?4, options=?5 WHERE entity=?1 AND key=?2",
                params![entity.as_db(), key, title, kind.as_db(), serde_json::to_string(&options).unwrap_or_else(|_| "[]".into())],
            )?;
        }
    }
    load_fields(conn, entity)
}

/// Deletes a field of their own (never a built-in one), with what was written in it.
pub fn delete_field(conn: &Connection, entity: Entity, key: &str) -> Result<Vec<RosterField>, StorageError> {
    let removed = conn.execute("DELETE FROM roster_field WHERE entity=?1 AND key=?2 AND builtin=0", params![entity.as_db(), key])?;
    if removed > 0 {
        let path = format!("$.{key}");
        conn.execute("UPDATE roster_entry SET data=json_remove(data, ?2) WHERE entity=?1", params![entity.as_db(), path])?;
    }
    load_fields(conn, entity)
}

pub fn list(conn: &Connection, entity: Entity) -> Result<Vec<RosterEntry>, StorageError> {
    let rows = conn
        .prepare("SELECT id, data FROM roster_entry WHERE entity=?1 ORDER BY created_at, rowid")?
        .query_map([entity.as_db()], |r| {
            let data: String = r.get(1)?;
            Ok(RosterEntry { id: r.get(0)?, data: serde_json::from_str::<Data>(&data).unwrap_or_default() })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Adds a record, or replaces the one with that id. Returns `false` when there is no such record to replace.
pub fn upsert(conn: &Connection, entity: Entity, id: Option<&str>, data: &Data) -> Result<bool, StorageError> {
    let json = serde_json::to_string(data).unwrap_or_else(|_| "{}".into());
    match id {
        Some(id) => {
            let n = conn.execute(
                "UPDATE roster_entry SET data=?3, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1 AND entity=?2",
                params![id, entity.as_db(), json],
            )?;
            Ok(n > 0)
        }
        None => {
            conn.execute(
                "INSERT INTO roster_entry (id,entity,data,created_at,updated_at)
                 VALUES (?1,?2,?3,strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
                params![format!("ros_{}", Ulid::generate()), entity.as_db(), json],
            )?;
            Ok(true)
        }
    }
}

pub fn delete(conn: &Connection, entity: Entity, id: &str) -> Result<bool, StorageError> {
    Ok(conn.execute("DELETE FROM roster_entry WHERE id=?1 AND entity=?2", params![id, entity.as_db()])? > 0)
}

#[cfg(debug_assertions)]
pub fn clear(conn: &Connection, entity: Entity) -> Result<(), StorageError> {
    conn.execute("DELETE FROM roster_entry WHERE entity=?1", [entity.as_db()])?;
    Ok(())
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

    fn data(pairs: &[(&str, &str)]) -> Data {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn the_form_starts_from_the_defaults_and_the_person_adds_a_field_of_their_own() {
        let (_d, c) = conn();
        let first = fields(&c, Entity::Staff).unwrap();
        assert!(first.iter().any(|f| f.key == "role" && f.builtin));
        assert_eq!(fields(&c, Entity::Staff).unwrap(), first, "written once");

        let own = FieldInput { key: None, title: " Talla de uniforme ".into(), kind: FieldKind::Select, options: vec!["CH".into(), "M".into(), "M".into(), " ".into(), "G".into()] };
        let after = save_field(&c, Entity::Staff, &own).unwrap();
        let added = after.last().unwrap();
        assert!(!added.builtin && added.key.starts_with("c_"));
        assert_eq!((added.title.as_str(), added.kind), ("Talla de uniforme", FieldKind::Select));
        assert_eq!(added.options.iter().map(|o| o.label.as_str()).collect::<Vec<_>>(), vec!["CH", "M", "G"]);
        assert_eq!(fields(&c, Entity::Beneficiary).unwrap().iter().filter(|f| !f.builtin).count(), 0, "each form has its own fields");
    }

    #[test]
    fn a_built_in_field_keeps_its_type_and_a_coded_one_keeps_its_options() {
        let (_d, c) = conn();
        let list = fields(&c, Entity::Beneficiary).unwrap();
        let dep = list.iter().find(|f| f.key == "dependency").unwrap().clone();
        let edited = save_field(&c, Entity::Beneficiary, &FieldInput { key: Some("dependency".into()), title: "Apoyo".into(), kind: FieldKind::Text, options: vec!["x".into()] }).unwrap();
        let dep2 = edited.iter().find(|f| f.key == "dependency").unwrap();
        assert_eq!((dep2.title.as_str(), dep2.kind), ("Apoyo", FieldKind::Select));
        assert_eq!(dep2.options, dep.options);
    }

    #[test]
    fn deleting_a_field_of_their_own_clears_it_from_the_records_but_a_built_in_one_stays() {
        let (_d, c) = conn();
        let after = save_field(&c, Entity::Staff, &FieldInput { key: None, title: "Alergias".into(), kind: FieldKind::Text, options: vec![] }).unwrap();
        let key = after.last().unwrap().key.clone();
        upsert(&c, Entity::Staff, None, &data(&[("full_name", "Ana"), (key.as_str(), "nueces")])).unwrap();
        delete_field(&c, Entity::Staff, "role").unwrap();
        assert!(fields(&c, Entity::Staff).unwrap().iter().any(|f| f.key == "role"));
        delete_field(&c, Entity::Staff, &key).unwrap();
        assert!(!fields(&c, Entity::Staff).unwrap().iter().any(|f| f.key == key));
        let rows = list(&c, Entity::Staff).unwrap();
        assert_eq!(rows[0].data.get("full_name").map(String::as_str), Some("Ana"));
        assert!(!rows[0].data.contains_key(&key));
    }

    #[test]
    fn records_are_added_edited_and_removed_by_kind() {
        let (_d, c) = conn();
        upsert(&c, Entity::Staff, None, &data(&[("full_name", "Ana"), ("role", "Cocina")])).unwrap();
        upsert(&c, Entity::Beneficiary, None, &data(&[("full_name", "Luz")])).unwrap();
        let staff = list(&c, Entity::Staff).unwrap();
        assert_eq!((staff.len(), list(&c, Entity::Beneficiary).unwrap().len()), (1, 1));
        assert!(upsert(&c, Entity::Staff, Some(&staff[0].id), &data(&[("full_name", "Ana María")])).unwrap());
        assert_eq!(list(&c, Entity::Staff).unwrap()[0].data["full_name"], "Ana María");
        assert!(!upsert(&c, Entity::Beneficiary, Some(&staff[0].id), &data(&[])).unwrap(), "a staff id is not a beneficiary id");
        assert!(delete(&c, Entity::Staff, &staff[0].id).unwrap());
        assert!(list(&c, Entity::Staff).unwrap().is_empty());
    }
}
