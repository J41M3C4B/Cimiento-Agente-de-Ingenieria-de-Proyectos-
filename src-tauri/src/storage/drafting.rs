//! Drafting data of a project (ADR-018): the text of each section, the budget lines and the schedule activities.
//! Changing the budget or the schedule takes away their confirmation and marks the texts that depend on them for
//! review; nothing is deleted behind the person's back.

use super::StorageError;
use crate::domain::budget::Funder;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use ulid::Ulid;

fn id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

// ---------------------------------------------------------------- sections

#[derive(Debug, Clone, Serialize)]
pub struct SectionRow {
    pub key: String,
    pub content: String,
    pub needs_review: bool,
    /// `ai_assumption` (written by the AI), `user` (written or edited by the person) or `computed` (a snapshot).
    pub origin: String,
    pub source_ref: Option<Value>,
    pub confirmed_at: Option<String>,
}

fn section_from(r: &rusqlite::Row<'_>) -> rusqlite::Result<SectionRow> {
    let source: Option<String> = r.get(4)?;
    Ok(SectionRow {
        key: r.get(0)?,
        content: r.get(1)?,
        needs_review: r.get::<_, i64>(2)? != 0,
        origin: r.get(3)?,
        source_ref: source.and_then(|s| serde_json::from_str(&s).ok()),
        confirmed_at: r.get(5)?,
    })
}

const SECTION_COLS: &str = "section_key, content, needs_review, origin, source_ref, confirmed_at";

pub fn sections(conn: &Connection, project_id: &str) -> Result<Vec<SectionRow>, StorageError> {
    let mut stmt = conn.prepare(&format!("SELECT {SECTION_COLS} FROM project_section WHERE project_id=?1 ORDER BY rowid"))?;
    let rows = stmt.query_map([project_id], section_from)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_section(conn: &Connection, project_id: &str, key: &str) -> Result<Option<SectionRow>, StorageError> {
    Ok(conn
        .query_row(&format!("SELECT {SECTION_COLS} FROM project_section WHERE project_id=?1 AND section_key=?2"), params![project_id, key], section_from)
        .optional()?)
}

/// Saves the text of a section as a draft: it is not confirmed until the person says so.
pub fn save_section(conn: &Connection, project_id: &str, key: &str, content: &str, origin: &str, source_ref: Option<&Value>) -> Result<(), StorageError> {
    conn.execute(
        &format!(
            "INSERT INTO project_section (id,project_id,section_key,content,needs_review,updated_at,origin,source_ref,confirmed_at,confirmed_by)
             VALUES (?1,?2,?3,?4,0,{NOW},?5,?6,NULL,NULL)
             ON CONFLICT(project_id,section_key) DO UPDATE SET content=excluded.content, needs_review=0, updated_at=excluded.updated_at,
                 origin=excluded.origin, source_ref=excluded.source_ref, confirmed_at=NULL, confirmed_by=NULL"
        ),
        params![id("sec"), project_id, key, content.trim(), origin, source_ref.map(Value::to_string)],
    )?;
    Ok(())
}

/// The person confirms the text. Only a section that has text can be confirmed.
pub fn confirm_section(conn: &Connection, project_id: &str, key: &str) -> Result<bool, StorageError> {
    let n = conn.execute(
        &format!("UPDATE project_section SET confirmed_at={NOW}, confirmed_by='manager', needs_review=0 WHERE project_id=?1 AND section_key=?2 AND trim(content) <> ''"),
        params![project_id, key],
    )?;
    Ok(n > 0)
}

/// Takes the confirmation away from a section (what it says depends on something that changed).
pub fn unconfirm_section(conn: &Connection, project_id: &str, key: &str) -> Result<(), StorageError> {
    conn.execute("UPDATE project_section SET confirmed_at=NULL, confirmed_by=NULL WHERE project_id=?1 AND section_key=?2", params![project_id, key])?;
    Ok(())
}

pub fn mark_needs_review(conn: &Connection, project_id: &str, keys: &[&str]) -> Result<(), StorageError> {
    for k in keys {
        conn.execute(&format!("UPDATE project_section SET needs_review=1, updated_at={NOW} WHERE project_id=?1 AND section_key=?2"), params![project_id, k])?;
    }
    Ok(())
}

// ---------------------------------------------------------------- budget

#[derive(Debug, Clone, Serialize)]
pub struct BudgetRow {
    pub id: String,
    pub category: String,
    pub description: String,
    pub quantity: f64,
    pub unit: Option<String>,
    pub unit_price_mxn: f64,
    pub vat_included: bool,
    pub funded_by: Funder,
    pub administrative: bool,
    /// `user` or `ai_assumption` (proposed by the assistant, still waiting for its cost).
    pub origin: String,
}

pub fn budget(conn: &Connection, project_id: &str) -> Result<Vec<BudgetRow>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, category, description, quantity, unit, unit_price_mxn, vat_included, funded_by, administrative, origin
         FROM budget_item WHERE project_id=?1 ORDER BY rowid",
    )?;
    let rows = stmt
        .query_map([project_id], |r| {
            Ok(BudgetRow {
                id: r.get(0)?,
                category: r.get(1)?,
                description: r.get(2)?,
                quantity: r.get(3)?,
                unit: r.get(4)?,
                unit_price_mxn: r.get(5)?,
                vat_included: r.get::<_, i64>(6)? != 0,
                funded_by: Funder::from_db(&r.get::<_, String>(7)?).unwrap_or(Funder::Requested),
                administrative: r.get::<_, i64>(8)? != 0,
                origin: r.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The fields of a budget line the person writes.
#[derive(Debug, Clone)]
pub struct BudgetInput {
    pub category: String,
    pub description: String,
    pub quantity: f64,
    pub unit: Option<String>,
    pub unit_price_mxn: f64,
    pub vat_included: bool,
    pub funded_by: Funder,
    pub administrative: bool,
}

/// Adds a line (`item_id` empty) or changes one. A change in the budget undoes its confirmation and marks the
/// texts that talk about the cost for review. Returns false if the line to change is not in this project.
pub fn save_budget_item(conn: &Connection, project_id: &str, item_id: Option<&str>, i: &BudgetInput) -> Result<bool, StorageError> {
    let unit = i.unit.as_deref().map(str::trim).filter(|u| !u.is_empty());
    match item_id {
        None => {
            conn.execute(
                "INSERT INTO budget_item (id,project_id,category,description,quantity,unit,unit_price_mxn,vat_included,funded_by,administrative,origin,confirmed_at,confirmed_by)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'user',strftime('%Y-%m-%dT%H:%M:%SZ','now'),'manager')",
                params![id("bud"), project_id, i.category.trim(), i.description.trim(), i.quantity, unit, i.unit_price_mxn, i.vat_included as i64, i.funded_by.as_db(), i.administrative as i64],
            )?;
        }
        Some(existing) => {
            let n = conn.execute(
                "UPDATE budget_item SET category=?3, description=?4, quantity=?5, unit=?6, unit_price_mxn=?7, vat_included=?8, funded_by=?9, administrative=?10, origin='user'
                 WHERE id=?1 AND project_id=?2",
                params![existing, project_id, i.category.trim(), i.description.trim(), i.quantity, unit, i.unit_price_mxn, i.vat_included as i64, i.funded_by.as_db(), i.administrative as i64],
            )?;
            if n == 0 {
                return Ok(false);
            }
        }
    }
    budget_changed(conn, project_id)?;
    Ok(true)
}

/// A line the assistant proposed: it has no price yet and nobody confirmed it. It does not undo any confirmation.
pub fn insert_proposed_budget_item(conn: &Connection, project_id: &str, i: &BudgetInput) -> Result<(), StorageError> {
    let unit = i.unit.as_deref().map(str::trim).filter(|u| !u.is_empty());
    conn.execute(
        "INSERT INTO budget_item (id,project_id,category,description,quantity,unit,unit_price_mxn,vat_included,funded_by,administrative,origin,confirmed_at,confirmed_by)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'ai_assumption',NULL,NULL)",
        params![id("bud"), project_id, i.category.trim(), i.description.trim(), i.quantity, unit, i.unit_price_mxn, i.vat_included as i64, i.funded_by.as_db(), i.administrative as i64],
    )?;
    Ok(())
}

pub fn delete_budget_item(conn: &Connection, project_id: &str, item_id: &str) -> Result<bool, StorageError> {
    let n = conn.execute("DELETE FROM budget_item WHERE id=?1 AND project_id=?2", params![item_id, project_id])?;
    if n > 0 {
        budget_changed(conn, project_id)?;
    }
    Ok(n > 0)
}

fn budget_changed(conn: &Connection, project_id: &str) -> Result<(), StorageError> {
    unconfirm_section(conn, project_id, "budget")?;
    mark_needs_review(conn, project_id, &["how_much"])
}

// ---------------------------------------------------------------- schedule

#[derive(Debug, Clone, Serialize)]
pub struct ActivityRow {
    pub id: String,
    pub title: String,
    pub start_month: u32,
    pub end_month: u32,
    /// `user` or `ai_assumption` (proposed by the assistant).
    pub origin: String,
}

pub fn schedule(conn: &Connection, project_id: &str) -> Result<Vec<ActivityRow>, StorageError> {
    let mut stmt = conn.prepare("SELECT id, title, start_month, end_month, origin FROM schedule_activity WHERE project_id=?1 ORDER BY start_month, end_month, rowid")?;
    let rows = stmt
        .query_map([project_id], |r| Ok(ActivityRow { id: r.get(0)?, title: r.get(1)?, start_month: r.get(2)?, end_month: r.get(3)?, origin: r.get(4)? }))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn save_activity(conn: &Connection, project_id: &str, activity_id: Option<&str>, title: &str, start_month: u32, end_month: u32) -> Result<bool, StorageError> {
    match activity_id {
        None => {
            conn.execute(
                "INSERT INTO schedule_activity (id,project_id,title,start_month,end_month) VALUES (?1,?2,?3,?4,?5)",
                params![id("act"), project_id, title.trim(), start_month, end_month],
            )?;
        }
        Some(existing) => {
            let n = conn.execute(
                "UPDATE schedule_activity SET title=?3, start_month=?4, end_month=?5, origin='user' WHERE id=?1 AND project_id=?2",
                params![existing, project_id, title.trim(), start_month, end_month],
            )?;
            if n == 0 {
                return Ok(false);
            }
        }
    }
    schedule_changed(conn, project_id)?;
    Ok(true)
}

/// An activity the assistant proposed.
pub fn insert_proposed_activity(conn: &Connection, project_id: &str, title: &str, start_month: u32, end_month: u32) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO schedule_activity (id,project_id,title,start_month,end_month,origin) VALUES (?1,?2,?3,?4,?5,'ai_assumption')",
        params![id("act"), project_id, title.trim(), start_month, end_month],
    )?;
    Ok(())
}

pub fn delete_activity(conn: &Connection, project_id: &str, activity_id: &str) -> Result<bool, StorageError> {
    let n = conn.execute("DELETE FROM schedule_activity WHERE id=?1 AND project_id=?2", params![activity_id, project_id])?;
    if n > 0 {
        schedule_changed(conn, project_id)?;
    }
    Ok(n > 0)
}

fn schedule_changed(conn: &Connection, project_id: &str) -> Result<(), StorageError> {
    unconfirm_section(conn, project_id, "schedule")?;
    mark_needs_review(conn, project_id, &["when"])
}

// ---------------------------------------------------------------- what the assistant prepared

/// The title and the plain explanation the assistant gave to each section, once the drafting began.
pub fn plan(conn: &Connection, project_id: &str) -> Result<Option<Value>, StorageError> {
    let raw: Option<String> = conn.query_row("SELECT sections FROM drafting_plan WHERE project_id=?1", [project_id], |r| r.get(0)).optional()?;
    Ok(raw.and_then(|s| serde_json::from_str(&s).ok()))
}

pub fn save_plan(conn: &Connection, project_id: &str, prompt_version: &str, sections: &Value) -> Result<(), StorageError> {
    conn.execute(
        &format!("INSERT OR REPLACE INTO drafting_plan (project_id,prompt_version,sections,created_at) VALUES (?1,?2,?3,{NOW})"),
        params![project_id, prompt_version, sections.to_string()],
    )?;
    Ok(())
}

// ---------------------------------------------------------------- the call asks for a proposal

pub fn asks_for_proposal(conn: &Connection, project_id: &str) -> Result<Option<bool>, StorageError> {
    let v: Option<i64> = conn.query_row("SELECT asks_for_proposal FROM project WHERE id=?1", [project_id], |r| r.get(0))?;
    Ok(v.map(|n| n != 0))
}

pub fn set_asks_for_proposal(conn: &Connection, project_id: &str, asks: bool) -> Result<(), StorageError> {
    conn.execute("UPDATE project SET asks_for_proposal=?2 WHERE id=?1", params![project_id, asks as i64])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::projects::create_project;
    use crate::storage::{open_encrypted, profile};
    use serde_json::json;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn project() -> (tempfile::TempDir, Connection, String) {
        use crate::domain::profile::*;
        let dir = tempfile::tempdir().unwrap();
        let mut c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        let input = ProfileInput { institution: InstitutionInput { name: "Asilo Ficticio".into(), ..Default::default() }, ..Default::default() };
        profile::save(&mut c, &input).unwrap();
        profile::confirm(&mut c).unwrap();
        let p = create_project(&mut c, "Proyecto", None).unwrap();
        (dir, c, p.id)
    }

    fn item(price: f64) -> BudgetInput {
        BudgetInput { category: "material".into(), description: "Tubería".into(), quantity: 2.0, unit: Some(" pieza ".into()), unit_price_mxn: price, vat_included: false, funded_by: Funder::Requested, administrative: false }
    }

    #[test]
    fn a_section_is_a_draft_until_the_person_confirms_it_and_editing_it_takes_the_confirmation_away() {
        let (_d, c, pid) = project();
        assert!(!confirm_section(&c, &pid, "what").unwrap(), "there is nothing to confirm");
        save_section(&c, &pid, "what", "  Arreglar la cocina.  ", "ai_assumption", Some(&json!({"open_points": ["monto"]}))).unwrap();
        let s = get_section(&c, &pid, "what").unwrap().unwrap();
        assert_eq!((s.content.as_str(), s.origin.as_str(), s.confirmed_at.is_none()), ("Arreglar la cocina.", "ai_assumption", true));
        assert_eq!(s.source_ref.unwrap()["open_points"][0], "monto");
        assert!(confirm_section(&c, &pid, "what").unwrap());
        assert!(get_section(&c, &pid, "what").unwrap().unwrap().confirmed_at.is_some());
        save_section(&c, &pid, "what", "Arreglar la cocina y los baños.", "user", None).unwrap();
        let s = get_section(&c, &pid, "what").unwrap().unwrap();
        assert_eq!((s.origin.as_str(), s.confirmed_at.is_none()), ("user", true));
        // an empty text cannot be confirmed
        save_section(&c, &pid, "why", "   ", "user", None).unwrap();
        assert!(!confirm_section(&c, &pid, "why").unwrap());
        assert_eq!(sections(&c, &pid).unwrap().len(), 2);
    }

    #[test]
    fn changing_the_budget_undoes_its_confirmation_and_marks_the_cost_text_for_review() {
        let (_d, c, pid) = project();
        save_section(&c, &pid, "budget", "{\"total\":1}", "computed", None).unwrap();
        confirm_section(&c, &pid, "budget").unwrap();
        save_section(&c, &pid, "how_much", "Cuesta poco.", "user", None).unwrap();
        confirm_section(&c, &pid, "how_much").unwrap();

        assert!(save_budget_item(&c, &pid, None, &item(500.0)).unwrap());
        assert!(get_section(&c, &pid, "budget").unwrap().unwrap().confirmed_at.is_none());
        let how_much = get_section(&c, &pid, "how_much").unwrap().unwrap();
        assert!(how_much.needs_review && how_much.confirmed_at.is_some(), "the text stays; it is only marked");

        let rows = budget(&c, &pid).unwrap();
        assert_eq!((rows.len(), rows[0].unit.as_deref(), rows[0].funded_by), (1, Some("pieza"), Funder::Requested));
        assert!(save_budget_item(&c, &pid, Some(&rows[0].id), &item(800.0)).unwrap());
        assert_eq!(budget(&c, &pid).unwrap()[0].unit_price_mxn, 800.0);
        assert!(!save_budget_item(&c, &pid, Some("bud_inexistente"), &item(1.0)).unwrap());
        assert!(delete_budget_item(&c, &pid, &rows[0].id).unwrap());
        assert!(!delete_budget_item(&c, &pid, &rows[0].id).unwrap());
        assert!(budget(&c, &pid).unwrap().is_empty());
    }

    #[test]
    fn the_database_refuses_a_budget_line_with_no_quantity() {
        let (_d, c, pid) = project();
        let mut bad = item(10.0);
        bad.quantity = 0.0;
        assert!(save_budget_item(&c, &pid, None, &bad).is_err());
    }

    #[test]
    fn the_schedule_is_kept_in_order_and_changing_it_undoes_its_confirmation() {
        let (_d, c, pid) = project();
        save_section(&c, &pid, "schedule", "{}", "computed", None).unwrap();
        confirm_section(&c, &pid, "schedule").unwrap();
        save_section(&c, &pid, "when", "En un año.", "user", None).unwrap();
        assert!(save_activity(&c, &pid, None, " Obra ", 4, 8).unwrap());
        assert!(save_activity(&c, &pid, None, "Compra de material", 1, 3).unwrap());
        assert!(get_section(&c, &pid, "schedule").unwrap().unwrap().confirmed_at.is_none());
        assert!(get_section(&c, &pid, "when").unwrap().unwrap().needs_review);
        let rows = schedule(&c, &pid).unwrap();
        assert_eq!(rows.iter().map(|a| (a.title.as_str(), a.start_month, a.end_month)).collect::<Vec<_>>(), vec![("Compra de material", 1, 3), ("Obra", 4, 8)]);
        assert!(save_activity(&c, &pid, Some(&rows[1].id), "Obra completa", 4, 10).unwrap());
        assert!(delete_activity(&c, &pid, &rows[0].id).unwrap());
        assert_eq!(schedule(&c, &pid).unwrap().len(), 1);
        assert!(c.execute("INSERT INTO schedule_activity (id,project_id,title,start_month,end_month) VALUES ('x',?1,'t',5,2)", [&pid]).is_err());
    }

    #[test]
    fn whether_the_call_asks_for_a_proposal_waits_for_the_person() {
        let (_d, c, pid) = project();
        assert_eq!(asks_for_proposal(&c, &pid).unwrap(), None);
        set_asks_for_proposal(&c, &pid, true).unwrap();
        assert_eq!(asks_for_proposal(&c, &pid).unwrap(), Some(true));
        set_asks_for_proposal(&c, &pid, false).unwrap();
        assert_eq!(asks_for_proposal(&c, &pid).unwrap(), Some(false));
    }

    #[test]
    fn deleting_the_project_takes_everything_of_its_drafting_with_it() {
        let (_d, mut c, pid) = project();
        save_section(&c, &pid, "what", "x", "user", None).unwrap();
        save_budget_item(&c, &pid, None, &item(1.0)).unwrap();
        save_activity(&c, &pid, None, "a", 1, 1).unwrap();
        assert!(crate::storage::projects::delete_project(&mut c, &pid).unwrap());
        let left: i64 = c.query_row("SELECT (SELECT count(*) FROM project_section)+(SELECT count(*) FROM budget_item)+(SELECT count(*) FROM schedule_activity)", [], |r| r.get(0)).unwrap();
        assert_eq!(left, 0);
    }
}
