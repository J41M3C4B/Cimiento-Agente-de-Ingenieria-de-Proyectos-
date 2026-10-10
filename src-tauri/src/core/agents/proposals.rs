//! What the AI proposes to change (ADR-034 §2). The AI never writes: a proposal names a field of a form of the
//! catalog, carries a value that fits that field, says where it comes from and waits until a person accepts or
//! rejects it. Accepting runs the usual save of the form (IA4); here a proposal is only kept and resolved.

use crate::common::forms::{is_filled, problem, AiUse};
use crate::core::error::ServiceError;
use crate::scanner::guard::Decision;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// The AI worked it out from what the person told it or from another part of the ERP.
    AiAssumption,
    /// Read from a document of the institution (`source_ref` says which and where).
    Document,
}

impl Origin {
    fn as_db(self) -> &'static str {
        match self {
            Origin::AiAssumption => "ai_assumption",
            Origin::Document => "document",
        }
    }
    fn from_db(s: &str) -> Origin {
        if s == "document" { Origin::Document } else { Origin::AiAssumption }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewProposal {
    pub form_id: String,
    pub record_id: Option<String>,
    pub field_id: String,
    pub value: Value,
    pub origin: Origin,
    pub source_ref: Option<String>,
    pub agent: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Proposal {
    pub id: String,
    pub form_id: String,
    pub record_id: Option<String>,
    pub field_id: String,
    pub value: Value,
    pub origin: Origin,
    pub source_ref: Option<String>,
    pub agent: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposeOutcome {
    Kept { id: String },
    /// Why it was not kept: `form_unknown`, `field_unknown`, `not_for_the_ai` (a field the AI never sees), `empty`,
    /// or the problem of the value (`code_unknown`, `number_too_large`, …).
    Refused { code: &'static str },
}

/// Keeps a proposal if it fits its field. A text goes through the scanner first and whatever it finds is covered:
/// there is no person in front to decide, and the save that follows the acceptance scans it again. A newer proposal
/// for the same field replaces the one still waiting.
pub fn propose(conn: &Connection, p: NewProposal) -> Result<ProposeOutcome, ServiceError> {
    let refused = |code| Ok(ProposeOutcome::Refused { code });
    let Some(form) = crate::core::institution::forms::form(&p.form_id) else { return refused("form_unknown") };
    let Some(field) = form.field(&p.field_id) else { return refused("field_unknown") };
    // what the AI never sees it cannot propose; a document can (read by rules, ADR-034 §3)
    if p.origin == Origin::AiAssumption && field.ai == AiUse::Never {
        return refused("not_for_the_ai");
    }
    if !is_filled(Some(&p.value)) {
        return refused("empty");
    }
    if let Some(code) = problem(field, &p.value) {
        return refused(code);
    }
    let value = match &p.value {
        Value::String(s) => {
            let fields = [(p.field_id.clone(), s.clone())];
            match crate::core::screen::guard_texts(conn, "ai_proposal", &fields, Some(Decision::Redact))? {
                Ok(mut texts) => Value::String(texts.remove(0)),
                // with «cover it» decided there is no quarantine; should there be one, nothing is kept
                Err(_) => return refused("not_for_the_ai"),
            }
        }
        other => other.clone(),
    };
    conn.execute(
        "DELETE FROM ai_proposal WHERE form_id = ?1 AND record_id IS ?2 AND field_id = ?3 AND status = 'pending'",
        params![p.form_id, p.record_id, p.field_id],
    )?;
    let id = format!("prop_{}", ulid::Ulid::generate());
    conn.execute(
        "INSERT INTO ai_proposal (id, form_id, record_id, field_id, value_json, origin, source_ref, agent, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![id, p.form_id, p.record_id, p.field_id, value.to_string(), p.origin.as_db(), p.source_ref, p.agent],
    )?;
    Ok(ProposeOutcome::Kept { id })
}

fn read(r: &rusqlite::Row) -> rusqlite::Result<Proposal> {
    let value: String = r.get(4)?;
    let origin: String = r.get(5)?;
    Ok(Proposal {
        id: r.get(0)?,
        form_id: r.get(1)?,
        record_id: r.get(2)?,
        field_id: r.get(3)?,
        value: serde_json::from_str(&value).unwrap_or(Value::Null),
        origin: Origin::from_db(&origin),
        source_ref: r.get(6)?,
        agent: r.get(7)?,
        status: r.get(8)?,
        created_at: r.get(9)?,
    })
}

const COLUMNS: &str = "id, form_id, record_id, field_id, value_json, origin, source_ref, agent, status, created_at";

/// The proposals still waiting for a form (and its record), in the order they came.
pub fn pending(conn: &Connection, form_id: &str, record_id: Option<&str>) -> Result<Vec<Proposal>, ServiceError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM ai_proposal WHERE form_id = ?1 AND record_id IS ?2 AND status = 'pending' ORDER BY created_at, id"
    ))?;
    let rows = stmt.query_map(params![form_id, record_id], read)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Marks a waiting proposal as accepted or rejected, with who did it. Only a waiting one can be resolved, once;
/// `None` when there is none with that id waiting.
pub fn resolve(conn: &Connection, id: &str, accepted: bool, by: Option<&str>) -> Result<Option<Proposal>, ServiceError> {
    let changed = conn.execute(
        "UPDATE ai_proposal SET status = ?2, resolved_by = ?3, resolved_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?1 AND status = 'pending'",
        params![id, if accepted { "accepted" } else { "rejected" }, by],
    )?;
    if changed == 0 {
        return Ok(None);
    }
    Ok(conn.query_row(&format!("SELECT {COLUMNS} FROM ai_proposal WHERE id = ?1"), [id], read).optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, c)
    }

    fn new(field: &str, value: Value) -> NewProposal {
        NewProposal {
            form_id: "institution.identity".into(),
            record_id: None,
            field_id: field.into(),
            value,
            origin: Origin::AiAssumption,
            source_ref: Some("chat".into()),
            agent: "capturist",
        }
    }

    fn kept(o: ProposeOutcome) -> String {
        match o {
            ProposeOutcome::Kept { id } => id,
            other => panic!("not kept: {other:?}"),
        }
    }

    #[test]
    fn a_proposal_waits_until_a_person_resolves_it_once() {
        let (_d, c) = conn();
        let id = kept(propose(&c, new("institution.populations", json!(["older_adults"]))).unwrap());
        let waiting = pending(&c, "institution.identity", None).unwrap();
        assert_eq!(waiting.len(), 1);
        assert_eq!((waiting[0].value.clone(), waiting[0].origin, waiting[0].status.as_str()), (json!(["older_adults"]), Origin::AiAssumption, "pending"));
        // nothing of the profile changed: a proposal is not a save
        assert!(crate::core::profile::storage::load_current(&c).unwrap().is_none());

        let done = resolve(&c, &id, false, Some("usr_1")).unwrap().unwrap();
        assert_eq!(done.status, "rejected");
        assert!(pending(&c, "institution.identity", None).unwrap().is_empty());
        assert_eq!(resolve(&c, &id, true, Some("usr_1")).unwrap(), None, "resolved once");
        let by: String = c.query_row("SELECT resolved_by FROM ai_proposal WHERE id = ?1", [&id], |r| r.get(0)).unwrap();
        assert_eq!(by, "usr_1");
    }

    #[test]
    fn a_value_that_does_not_fit_its_field_is_refused() {
        let (_d, c) = conn();
        let refused = |p| match propose(&c, p).unwrap() {
            ProposeOutcome::Refused { code } => code,
            other => panic!("{other:?}"),
        };
        assert_eq!(refused(new("institution.populations", json!(["aliens"]))), "code_unknown");
        assert_eq!(refused(new("institution.name", json!(3))), "wrong_type");
        assert_eq!(refused(new("institution.name", json!("  "))), "empty");
        assert_eq!(refused(new("institution.nope", json!("x"))), "field_unknown");
        assert_eq!(refused(NewProposal { form_id: "nope".into(), ..new("institution.name", json!("x")) }), "form_unknown");
        assert!(pending(&c, "institution.identity", None).unwrap().is_empty());
    }

    #[test]
    fn a_text_is_scanned_before_it_is_kept_and_a_newer_proposal_replaces_the_waiting_one() {
        let (_d, c) = conn();
        kept(propose(&c, new("institution.mission", json!("Cuidar personas mayores."))).unwrap());
        kept(propose(&c, new("institution.mission", json!("Cuidar personas mayores. Llamar a maria.lopez@example.com"))).unwrap());
        let waiting = pending(&c, "institution.identity", None).unwrap();
        assert_eq!(waiting.len(), 1, "one proposal per field");
        let text = waiting[0].value.as_str().unwrap();
        assert!(!text.contains("maria.lopez") && text.starts_with("Cuidar personas mayores."), "{text}");
    }
}
