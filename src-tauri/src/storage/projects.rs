//! Projects and the diagnosis data: answers, the open follow-up question, closed
//! dimensions, the summary and the proposed needs.

use super::StorageError;
use crate::audit::{self, AuditKind};
use crate::domain::conversation::{is_vague, Kind, Role, TurnFacts};
use crate::domain::priority::Scores;
use crate::domain::stage::{Stage, StageFacts};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};
use ulid::Ulid;

fn id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectRow {
    pub id: String,
    pub institution_id: String,
    pub profile_id: String,
    pub title: String,
    pub initial_request: Option<String>,
    pub stage: Stage,
    pub needs_review: bool,
    pub created_at: String,
    /// How it began: from a call (`call`) or from an everyday need (`internal`). See migration 0005.
    pub kind: String,
    /// The reading of the call this project was born from (`None` for an internal project, or if its files were deleted).
    pub call_reading_id: Option<String>,
    /// The color of its folder in «Mis proyectos» (one of `PROJECT_COLORS`); `None` until the person picks one.
    pub color: Option<String>,
    /// Who gives the support: `institutional`, `private` or `individual` (one of `DONOR_KINDS`); `None` if not said.
    pub donor_kind: Option<String>,
}

/// The kinds of donor a project can have.
pub const DONOR_KINDS: [&str; 3] = ["institutional", "private", "individual"];

/// The colors a folder can have. The screen draws each one its own way.
pub const PROJECT_COLORS: [&str; 8] = ["blue", "violet", "teal", "green", "amber", "orange", "pink", "red"];

fn project_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    let stage: String = r.get(5)?;
    Ok(ProjectRow {
        id: r.get(0)?,
        institution_id: r.get(1)?,
        profile_id: r.get(2)?,
        title: r.get(3)?,
        initial_request: r.get(4)?,
        stage: Stage::from_db(&stage).unwrap_or(Stage::Profile),
        needs_review: r.get::<_, i64>(6)? != 0,
        created_at: r.get(7)?,
        kind: r.get(8)?,
        call_reading_id: r.get(9)?,
        color: r.get(10)?,
        donor_kind: r.get(11)?,
    })
}

const PROJECT_COLS: &str = "id, institution_id, profile_id, title, initial_request, stage, needs_review, created_at, kind, call_reading_id, color, donor_kind";

/// Days since the latest confirmed profile was confirmed (`None` if none is confirmed).
pub fn profile_confirmed_days_ago(conn: &Connection) -> Result<Option<i64>, StorageError> {
    Ok(conn
        .query_row(
            "SELECT CAST(julianday('now') - julianday(confirmed_at) AS INTEGER)
             FROM institution_profile WHERE confirmed_at IS NOT NULL ORDER BY version DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?)
}

/// Creates the project in `PROFILE`; the caller advances it once the profile check passes.
/// Without a call it is an `internal` project (the door is open in the model, switched off in the screen).
pub fn create_project(conn: &mut Connection, title: &str, initial_request: Option<&str>) -> Result<ProjectRow, StorageError> {
    let tx = conn.transaction()?;
    let pid = insert_project(&tx, title, initial_request, "internal", None)?;
    tx.commit()?;
    get_project(conn, &pid)?.ok_or(StorageError::NoProfile)
}

/// Inserts the project for the latest confirmed profile, inside the caller's transaction.
pub(super) fn insert_project(
    tx: &rusqlite::Transaction<'_>,
    title: &str,
    initial_request: Option<&str>,
    kind: &str,
    call_reading_id: Option<&str>,
) -> Result<String, StorageError> {
    let (institution_id, profile_id): (String, String) = tx
        .query_row(
            "SELECT institution_id, id FROM institution_profile WHERE confirmed_at IS NOT NULL ORDER BY version DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or(StorageError::NoProfile)?;
    let pid = id("proj");
    tx.execute(
        "INSERT INTO project (id,institution_id,profile_id,title,initial_request,stage,kind,call_reading_id,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,'PROFILE',?6,?7,strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![pid, institution_id, profile_id, title.trim(), initial_request.map(str::trim), kind, call_reading_id],
    )?;
    Ok(pid)
}

/// Deletes the project with everything of it: its answers, needs and sections go by cascade, and its call
/// (files, text and reading) goes through the emergency delete of each file, so nothing is left.
pub fn delete_project(conn: &mut Connection, project_id: &str) -> Result<bool, StorageError> {
    let Some(project) = get_project(conn, project_id)? else { return Ok(false) };
    conn.execute("DELETE FROM project WHERE id=?1", [project_id])?;
    if let Some(reading_id) = project.call_reading_id {
        super::calls::delete(conn, &reading_id)?;
    }
    audit::record(conn, AuditKind::EmergencyDelete, Some("project"), Some(project_id), json!({ "kind": project.kind }))?;
    Ok(true)
}

/// Sets the color of a project's folder (`None` clears it). Returns `false` if the color is not one of the list or
/// the project does not exist.
pub fn set_color(conn: &Connection, project_id: &str, color: Option<&str>) -> Result<bool, StorageError> {
    if color.is_some_and(|c| !PROJECT_COLORS.contains(&c)) {
        return Ok(false);
    }
    Ok(conn.execute("UPDATE project SET color=?2 WHERE id=?1", params![project_id, color])? > 0)
}

/// Sets who gives the support of a project (`None` clears it). Returns `false` if the kind is not of the list or the
/// project does not exist.
pub fn set_donor_kind(conn: &Connection, project_id: &str, kind: Option<&str>) -> Result<bool, StorageError> {
    if kind.is_some_and(|k| !DONOR_KINDS.contains(&k)) {
        return Ok(false);
    }
    Ok(conn.execute("UPDATE project SET donor_kind=?2 WHERE id=?1", params![project_id, kind])? > 0)
}

pub fn get_project(conn: &Connection, project_id: &str) -> Result<Option<ProjectRow>, StorageError> {
    Ok(conn
        .query_row(&format!("SELECT {PROJECT_COLS} FROM project WHERE id=?1"), [project_id], project_from_row)
        .optional()?)
}

pub fn list_projects(conn: &Connection) -> Result<Vec<ProjectRow>, StorageError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PROJECT_COLS} FROM project WHERE archived_at IS NULL AND hidden = 0 ORDER BY created_at DESC, id DESC"
    ))?;
    let rows = stmt.query_map([], project_from_row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Changes the stage and records it in the audit log. `needs_review` is set when going back.
pub fn set_stage(conn: &mut Connection, project_id: &str, to: Stage, needs_review: bool) -> Result<(), StorageError> {
    let tx = conn.transaction()?;
    let from: String = tx.query_row("SELECT stage FROM project WHERE id=?1", [project_id], |r| r.get(0))?;
    tx.execute(
        "UPDATE project SET stage=?2, needs_review=?3, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1",
        params![project_id, to.as_db(), needs_review as i64],
    )?;
    audit::record(
        &tx,
        AuditKind::StageChanged,
        Some("project"),
        Some(project_id),
        json!({ "from": from, "to": to.as_db() }),
    )?;
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------- conversation (ADR-017)

#[derive(Debug, Clone, Serialize)]
pub struct TurnRow {
    pub turn: i64,
    pub role: Role,
    pub kind: Kind,
    pub level: Option<u8>,
    pub text: String,
    /// Closed answers the assistant offered.
    pub options: Vec<String>,
    /// What the turn recorded (cause, quote, verified, fit, tactic...); counts and words already scanned.
    pub record: Value,
}

impl TurnRow {
    pub fn facts(&self) -> TurnFacts {
        TurnFacts { role: self.role, kind: self.kind, level: self.level, vague: self.role == Role::Person && is_vague(&self.text) }
    }
}

/// Appends a message to the conversation of the project and returns its number.
pub fn add_turn(
    conn: &Connection,
    project_id: &str,
    role: Role,
    kind: Kind,
    level: Option<u8>,
    text: &str,
    options: &[String],
    record: &Value,
) -> Result<i64, StorageError> {
    let turn: i64 = conn.query_row("SELECT COALESCE(MAX(turn),0)+1 FROM conversation_turn WHERE project_id=?1", [project_id], |r| r.get(0))?;
    conn.execute(
        "INSERT INTO conversation_turn (id,project_id,turn,role,kind,level,text,options_json,record_json,created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
        params![
            id("turn"),
            project_id,
            turn,
            role.as_db(),
            kind.as_db(),
            level,
            text.trim(),
            (!options.is_empty()).then(|| json!(options).to_string()),
            record.to_string()
        ],
    )?;
    Ok(turn)
}

pub fn turns(conn: &Connection, project_id: &str) -> Result<Vec<TurnRow>, StorageError> {
    let mut stmt = conn.prepare("SELECT turn, role, kind, level, text, options_json, record_json FROM conversation_turn WHERE project_id=?1 ORDER BY turn")?;
    let rows = stmt
        .query_map([project_id], |r| {
            let options: Option<String> = r.get(5)?;
            let record: Option<String> = r.get(6)?;
            Ok(TurnRow {
                turn: r.get(0)?,
                role: Role::from_db(&r.get::<_, String>(1)?).unwrap_or(Role::Person),
                kind: Kind::from_db(&r.get::<_, String>(2)?).unwrap_or(Kind::Why),
                level: r.get(3)?,
                text: r.get(4)?,
                options: options.and_then(|o| serde_json::from_str(&o).ok()).unwrap_or_default(),
                record: record.and_then(|o| serde_json::from_str(&o).ok()).unwrap_or(Value::Null),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[derive(Debug, Clone, Serialize)]
pub struct RootRow {
    pub text: String,
    pub quote: Option<String>,
    pub confirmed_at: Option<String>,
}

/// Saves the root cause that is being proposed (not confirmed); a new proposal replaces the earlier one.
pub fn set_root(conn: &Connection, project_id: &str, text: &str, quote: Option<&str>) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO conversation_root (project_id,text,quote,confirmed_at) VALUES (?1,?2,?3,NULL)
         ON CONFLICT(project_id) DO UPDATE SET text=excluded.text, quote=excluded.quote, confirmed_at=NULL",
        params![project_id, text.trim(), quote],
    )?;
    Ok(())
}

pub fn get_root(conn: &Connection, project_id: &str) -> Result<Option<RootRow>, StorageError> {
    Ok(conn
        .query_row("SELECT text, quote, confirmed_at FROM conversation_root WHERE project_id=?1", [project_id], |r| {
            Ok(RootRow { text: r.get(0)?, quote: r.get(1)?, confirmed_at: r.get(2)? })
        })
        .optional()?)
}

/// The person says the proposed root cause is the one. Returns false if none was proposed.
pub fn confirm_root(conn: &Connection, project_id: &str) -> Result<bool, StorageError> {
    let n = conn.execute("UPDATE conversation_root SET confirmed_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE project_id=?1", [project_id])?;
    Ok(n > 0)
}

/// A project that was started with the seven fixed questions of the earlier method (their answers are still there).
pub fn has_legacy_answers(conn: &Connection, project_id: &str) -> Result<bool, StorageError> {
    Ok(conn.query_row("SELECT EXISTS(SELECT 1 FROM diagnosis_answer WHERE project_id=?1)", [project_id], |r| r.get(0))?)
}

// ---------------------------------------------------------------- summary

#[derive(Debug, Clone, Serialize)]
pub struct StoredSummary {
    pub summary: Value,
    pub origin: String,
    pub confirmed_at: Option<String>,
}

/// Saves the summary as a draft (not confirmed). `origin` is `ai_assumption`, `user` or `computed`.
pub fn save_summary(conn: &Connection, project_id: &str, summary: &Value, origin: &str, source_ref: Option<&Value>) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO diagnosis_summary (project_id,summary_json,origin,source_ref,confirmed_at,confirmed_by)
         VALUES (?1,?2,?3,?4,NULL,NULL)
         ON CONFLICT(project_id) DO UPDATE SET summary_json=excluded.summary_json, origin=excluded.origin,
                                               source_ref=excluded.source_ref, confirmed_at=NULL, confirmed_by=NULL",
        params![project_id, summary.to_string(), origin, source_ref.map(|v| v.to_string())],
    )?;
    Ok(())
}

pub fn get_summary(conn: &Connection, project_id: &str) -> Result<Option<StoredSummary>, StorageError> {
    Ok(conn
        .query_row(
            "SELECT summary_json, origin, confirmed_at FROM diagnosis_summary WHERE project_id=?1",
            [project_id],
            |r| {
                let raw: String = r.get(0)?;
                Ok(StoredSummary { summary: serde_json::from_str(&raw).unwrap_or(Value::Null), origin: r.get(1)?, confirmed_at: r.get(2)? })
            },
        )
        .optional()?)
}

/// Returns false if there was no summary to confirm.
pub fn confirm_summary(conn: &Connection, project_id: &str) -> Result<bool, StorageError> {
    let n = conn.execute(
        "UPDATE diagnosis_summary SET confirmed_at=strftime('%Y-%m-%dT%H:%M:%SZ','now'), confirmed_by='manager' WHERE project_id=?1",
        [project_id],
    )?;
    Ok(n > 0)
}

// ---------------------------------------------------------------- needs

#[derive(Debug, Clone, Serialize)]
pub struct NeedRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub scores: Option<Scores>,
    pub total_score: Option<f64>,
    pub selected: bool,
    pub origin: String,
    pub confirmed: bool,
}

pub fn add_need(conn: &Connection, project_id: &str, title: &str, description: Option<&str>, origin: &str, source_ref: Option<&Value>) -> Result<String, StorageError> {
    let nid = id("need");
    let confirmed = origin == "user";
    conn.execute(
        "INSERT INTO need (id,project_id,title,description,origin,source_ref,confirmed_at,confirmed_by)
         VALUES (?1,?2,?3,?4,?5,?6,
                 CASE WHEN ?7 THEN strftime('%Y-%m-%dT%H:%M:%SZ','now') END,
                 CASE WHEN ?7 THEN 'manager' END)",
        params![nid, project_id, title.trim(), description.map(str::trim), origin, source_ref.map(|v| v.to_string()), confirmed],
    )?;
    Ok(nid)
}

pub fn list_needs(conn: &Connection, project_id: &str) -> Result<Vec<NeedRow>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id,title,description,scores_json,total_score,selected,origin,confirmed_at FROM need WHERE project_id=?1 ORDER BY rowid",
    )?;
    let rows = stmt
        .query_map([project_id], |r| {
            let scores: Option<String> = r.get(3)?;
            Ok(NeedRow {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                scores: scores.and_then(|s| serde_json::from_str(&s).ok()),
                total_score: r.get(4)?,
                selected: r.get::<_, i64>(5)? != 0,
                origin: r.get(6)?,
                confirmed: r.get::<_, Option<String>>(7)?.is_some(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Stores the ratings and the score computed by code. The person rated it, so it counts as confirmed.
pub fn set_need_scores(conn: &Connection, need_id: &str, scores: &Scores, total: f64) -> Result<bool, StorageError> {
    let n = conn.execute(
        "UPDATE need SET scores_json=?2, total_score=?3, confirmed_at=strftime('%Y-%m-%dT%H:%M:%SZ','now'), confirmed_by='manager' WHERE id=?1",
        params![need_id, serde_json::to_string(scores).unwrap(), total],
    )?;
    Ok(n > 0)
}

/// Marks exactly one need as "the project".
pub fn select_need(conn: &mut Connection, project_id: &str, need_id: &str) -> Result<bool, StorageError> {
    let tx = conn.transaction()?;
    let exists: Option<String> = tx
        .query_row("SELECT id FROM need WHERE id=?1 AND project_id=?2", params![need_id, project_id], |r| r.get(0))
        .optional()?;
    if exists.is_none() {
        return Ok(false);
    }
    tx.execute("UPDATE need SET selected=0 WHERE project_id=?1", [project_id])?;
    tx.execute(
        "UPDATE need SET selected=1, confirmed_at=COALESCE(confirmed_at,strftime('%Y-%m-%dT%H:%M:%SZ','now')),
                         confirmed_by=COALESCE(confirmed_by,'manager') WHERE id=?1",
        [need_id],
    )?;
    tx.commit()?;
    Ok(true)
}

// ---------------------------------------------------------------- stage facts

pub fn facts(conn: &Connection, project_id: &str) -> Result<StageFacts, StorageError> {
    let summary_confirmed = get_summary(conn, project_id)?.map_or(false, |s| s.confirmed_at.is_some());
    let need_selected: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM need WHERE project_id=?1 AND selected=1)",
        [project_id],
        |r| r.get(0),
    )?;
    // the call counts when it was read (fully or in part) and the person confirmed it
    let call_confirmed: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM project p JOIN call_reading r ON r.id = p.call_reading_id
                       WHERE p.id=?1 AND r.confirmed_at IS NOT NULL AND r.status IN ('ready','partial'))",
        [project_id],
        |r| r.get(0),
    )?;
    let root_cause_confirmed: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM conversation_root WHERE project_id=?1 AND confirmed_at IS NOT NULL)",
        [project_id],
        |r| r.get(0),
    )?;
    Ok(StageFacts {
        profile_confirmed_days_ago: profile_confirmed_days_ago(conn)?,
        root_cause_confirmed,
        diagnosis_summary_confirmed: summary_confirmed,
        need_selected,
        call_confirmed,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::*;
    use crate::storage::{open_encrypted, profile};

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn conn_with_confirmed_profile() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let mut c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        let input = ProfileInput {
            institution: InstitutionInput { name: "Asilo Ficticio".into(), ..Default::default() },
            population: vec![PopulationGroupInput { label: "Adultos".into(), count: 18, ..Default::default() }],
            ..Default::default()
        };
        profile::save(&mut c, &input).unwrap();
        profile::confirm(&mut c).unwrap();
        (dir, c)
    }

    #[test]
    fn project_needs_a_confirmed_profile() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        assert!(matches!(create_project(&mut c, "x", None), Err(StorageError::NoProfile)));
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", Some("Queremos remodelar un baño")).unwrap();
        assert_eq!(p.stage, Stage::Profile);
        assert_eq!(list_projects(&c).unwrap().len(), 1);
        assert_eq!(profile_confirmed_days_ago(&c).unwrap(), Some(0));
    }

    #[test]
    fn a_folder_takes_a_color_from_the_list_and_only_from_the_list() {
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", None).unwrap();
        assert_eq!(p.color, None);
        assert!(set_color(&c, &p.id, Some("violet")).unwrap());
        assert_eq!(get_project(&c, &p.id).unwrap().unwrap().color.as_deref(), Some("violet"));
        assert!(!set_color(&c, &p.id, Some("#ff0000")).unwrap(), "a color that is not in the list is refused");
        assert_eq!(list_projects(&c).unwrap()[0].color.as_deref(), Some("violet"));
        assert!(set_color(&c, &p.id, None).unwrap());
        assert_eq!(get_project(&c, &p.id).unwrap().unwrap().color, None);
        assert!(!set_color(&c, "nope", Some("teal")).unwrap());
    }

    #[test]
    fn a_project_says_who_gives_the_support_from_a_list() {
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", None).unwrap();
        assert_eq!(p.donor_kind, None);
        assert!(set_donor_kind(&c, &p.id, Some("private")).unwrap());
        assert_eq!(get_project(&c, &p.id).unwrap().unwrap().donor_kind.as_deref(), Some("private"));
        assert!(!set_donor_kind(&c, &p.id, Some("gobierno")).unwrap(), "a kind that is not in the list is refused");
        assert_eq!(list_projects(&c).unwrap()[0].donor_kind.as_deref(), Some("private"));
        assert!(set_donor_kind(&c, &p.id, None).unwrap());
        assert!(!set_donor_kind(&c, "nope", Some("individual")).unwrap());
    }

    #[test]
    fn stage_changes_are_audited_without_content() {
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", None).unwrap();
        set_stage(&mut c, &p.id, Stage::Diagnosis, false).unwrap();
        set_stage(&mut c, &p.id, Stage::Profile, true).unwrap();
        let p2 = get_project(&c, &p.id).unwrap().unwrap();
        assert_eq!(p2.stage, Stage::Profile);
        assert!(p2.needs_review);
        let d: Vec<String> = c
            .prepare("SELECT details_json FROM audit_log WHERE event='stage.changed' ORDER BY id").unwrap()
            .query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
        assert_eq!(d, vec![r#"{"from":"PROFILE","to":"DIAGNOSIS"}"#, r#"{"from":"DIAGNOSIS","to":"PROFILE"}"#]);
    }

    #[test]
    fn the_conversation_keeps_its_turns_in_order_and_the_root_cause_waits_for_the_person() {
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", None).unwrap();
        let none: [String; 0] = [];
        add_turn(&c, &p.id, Role::Assistant, Kind::Opening, None, "  ¿Qué proyecto tienen en mente?  ", &none, &json!({})).unwrap();
        add_turn(&c, &p.id, Role::Person, Kind::Opening, None, "Arreglar el baño porque no hay dinero", &none, &json!({})).unwrap();
        let options = vec!["Falta dinero".to_string(), "No lo sé todavía".to_string()];
        let n = add_turn(&c, &p.id, Role::Assistant, Kind::Why, Some(1), "¿Por qué no hay dinero?", &options, &json!({"fit": "fits"})).unwrap();
        assert_eq!(n, 3);
        let t = turns(&c, &p.id).unwrap();
        assert_eq!(t.iter().map(|x| (x.turn, x.role, x.kind, x.level)).collect::<Vec<_>>(), vec![(1, Role::Assistant, Kind::Opening, None), (2, Role::Person, Kind::Opening, None), (3, Role::Assistant, Kind::Why, Some(1))]);
        assert_eq!((t[0].text.as_str(), t[2].options.clone(), t[2].record["fit"].as_str()), ("¿Qué proyecto tienen en mente?", options, Some("fits")));
        assert!(!t[0].facts().vague && !t[1].facts().vague);
        // a turn number is never used twice
        assert!(c.execute("INSERT INTO conversation_turn (id,project_id,turn,role,kind,text,created_at) VALUES ('x',?1,3,'person','why','t','now')", [&p.id]).is_err());

        assert!(get_root(&c, &p.id).unwrap().is_none() && !confirm_root(&c, &p.id).unwrap());
        assert!(!facts(&c, &p.id).unwrap().root_cause_confirmed);
        set_root(&c, &p.id, "No hay plan de mantenimiento", Some("nadie revisa el tubo")).unwrap();
        set_root(&c, &p.id, "No hay quien dé mantenimiento", None).unwrap(); // a new proposal replaces the earlier one
        let r = get_root(&c, &p.id).unwrap().unwrap();
        assert_eq!((r.text.as_str(), r.confirmed_at.is_none()), ("No hay quien dé mantenimiento", true));
        assert!(!facts(&c, &p.id).unwrap().root_cause_confirmed, "proposed is not confirmed");
        assert!(confirm_root(&c, &p.id).unwrap());
        assert!(facts(&c, &p.id).unwrap().root_cause_confirmed);
        // a new proposal takes the confirmation away
        set_root(&c, &p.id, "Otra", None).unwrap();
        assert!(!facts(&c, &p.id).unwrap().root_cause_confirmed);
    }

    #[test]
    fn deleting_the_project_takes_its_conversation_with_it() {
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", None).unwrap();
        add_turn(&c, &p.id, Role::Assistant, Kind::Opening, None, "Hola", &[], &json!({})).unwrap();
        set_root(&c, &p.id, "Raíz", None).unwrap();
        assert!(delete_project(&mut c, &p.id).unwrap());
        let left: i64 = c.query_row("SELECT (SELECT count(*) FROM conversation_turn) + (SELECT count(*) FROM conversation_root)", [], |r| r.get(0)).unwrap();
        assert_eq!(left, 0);
    }

    #[test]
    fn summary_is_a_draft_until_confirmed() {
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", None).unwrap();
        assert!(!confirm_summary(&c, &p.id).unwrap());
        save_summary(&c, &p.id, &json!({"problem_statement": "x"}), "ai_assumption", Some(&json!({"task": "diagnosis.summary"}))).unwrap();
        let s = get_summary(&c, &p.id).unwrap().unwrap();
        assert_eq!(s.origin, "ai_assumption");
        assert!(s.confirmed_at.is_none());
        assert!(!facts(&c, &p.id).unwrap().diagnosis_summary_confirmed);
        assert!(confirm_summary(&c, &p.id).unwrap());
        assert!(facts(&c, &p.id).unwrap().diagnosis_summary_confirmed);
        // editing it again goes back to draft
        save_summary(&c, &p.id, &json!({"problem_statement": "y"}), "user", None).unwrap();
        assert!(get_summary(&c, &p.id).unwrap().unwrap().confirmed_at.is_none());
    }

    #[test]
    fn needs_scores_and_single_selection() {
        let (_d, mut c) = conn_with_confirmed_profile();
        let p = create_project(&mut c, "Baño", None).unwrap();
        let a = add_need(&c, &p.id, "Higiene segura", Some("d"), "ai_assumption", None).unwrap();
        let b = add_need(&c, &p.id, "Otra", None, "user", None).unwrap();
        let needs = list_needs(&c, &p.id).unwrap();
        assert!(!needs[0].confirmed && needs[1].confirmed);
        let sc = Scores { beneficiaries: 4, severity: 5, mission: 5, feasibility: 3, sustainability: 4 };
        assert!(set_need_scores(&c, &a, &sc, 84.0).unwrap());
        assert!(select_need(&mut c, &p.id, &a).unwrap());
        assert!(select_need(&mut c, &p.id, &b).unwrap()); // moves the selection
        let needs = list_needs(&c, &p.id).unwrap();
        assert_eq!(needs.iter().filter(|n| n.selected).count(), 1);
        assert!(needs[1].selected);
        assert_eq!(needs[0].scores, Some(sc));
        assert!(facts(&c, &p.id).unwrap().need_selected);
        assert!(!select_need(&mut c, &p.id, "need_nope").unwrap());
    }
}
