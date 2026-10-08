//! Numbered migrations. Once applied, a migration is never edited.

use rusqlite::{Connection, OptionalExtension, Transaction};
use std::collections::BTreeMap;

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/0001_initial.sql")),
    (2, include_str!("../../migrations/0002_diagnosis_flow.sql")),
    (3, include_str!("../../migrations/0003_ai_metrics.sql")),
    (4, include_str!("../../migrations/0004_call_reading.sql")),
    (5, include_str!("../../migrations/0005_project_from_call.sql")),
    (6, include_str!("../../migrations/0006_conversation.sql")),
    (7, include_str!("../../migrations/0007_drafting.sql")),
    (8, include_str!("../../migrations/0008_staff_pay_and_fees.sql")),
    (9, include_str!("../../migrations/0009_roster.sql")),
    (10, include_str!("../../migrations/0010_project_color.sql")),
    (11, include_str!("../../migrations/0011_project_donor_kind.sql")),
    (12, include_str!("../../migrations/0012_drafting_plan.sql")),
    (13, include_str!("../../migrations/0013_call_brief.sql")),
    (14, include_str!("../../migrations/0014_income_kinds_and_expenses.sql")),
    (15, include_str!("../../migrations/0015_hr_staff.sql")),
    (16, include_str!("../../migrations/0016_access.sql")),
    (17, include_str!("../../migrations/0017_care.sql")),
];

/// Code that runs right after the SQL of a version, inside the same transaction (moves of data that need rules).
fn after(version: i64, tx: &Transaction) -> rusqlite::Result<()> {
    if version == 15 {
        move_roster_staff(tx)?;
    }
    if version == 17 {
        move_roster_people(tx)?;
    }
    Ok(())
}

fn hr_failure(e: crate::hr::HrError) -> rusqlite::Error {
    match e {
        crate::hr::HrError::Db(e) => e,
        other => rusqlite::Error::ToSqlConversionFailure(Box::new(other)),
    }
}

fn care_failure(e: crate::care::CareError) -> rusqlite::Error {
    match e {
        crate::care::CareError::Db(e) => e,
        other => rusqlite::Error::ToSqlConversionFailure(Box::new(other)),
    }
}

/// The people served of the old roster (ADR-020) move into their module (ADR-029), keeping their ids (a deletion
/// that was waiting still finds them) and their hiding. Then the roster, empty, goes away.
fn move_roster_people(tx: &Transaction) -> rusqlite::Result<()> {
    let rows: Vec<crate::care::legacy::LegacyRow> = tx
        .prepare("SELECT id, data, hidden FROM roster_entry WHERE entity = 'beneficiary' ORDER BY created_at, rowid")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|(id, data, hidden)| crate::care::legacy::LegacyRow { id, data: serde_json::from_str(&data).unwrap_or_default(), hidden: hidden != 0 })
        .collect();
    let own: Vec<crate::care::legacy::LegacyField> = tx
        .prepare("SELECT key, title, kind, options FROM roster_field WHERE entity = 'beneficiary' AND builtin = 0 ORDER BY position")?
        .query_map([], |r| {
            let options: Vec<serde_json::Value> = serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default();
            Ok(crate::care::legacy::LegacyField {
                key: r.get(0)?,
                title: r.get(1)?,
                kind: r.get(2)?,
                options: options.iter().filter_map(|o| o["label"].as_str().map(String::from)).collect(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let hidden_fields: Vec<String> = tx.prepare("SELECT key FROM roster_field WHERE entity = 'beneficiary' AND hidden = 1")?.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    let year: i64 = tx.query_row("SELECT CAST(strftime('%Y','now') AS INTEGER)", [], |r| r.get(0))?;
    let moved = crate::care::legacy::import(tx, year, &rows, &own).map_err(care_failure)?;
    for key in hidden_fields {
        tx.execute("UPDATE care_custom_field SET hidden = 1 WHERE key = ?1", [key])?;
    }
    if moved > 0 {
        crate::audit::record(tx, crate::audit::AuditKind::CareImported, Some("care_person"), None, serde_json::json!({ "people": moved }))
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    }
    tx.execute_batch("DROP TABLE roster_entry; DROP TABLE roster_field;")?;
    Ok(())
}

/// The staff of the old roster (ADR-020) moves into the staff module (ADR-027); the roster keeps the people served.
fn move_roster_staff(tx: &Transaction) -> rusqlite::Result<()> {
    let rows: Vec<BTreeMap<String, String>> = tx
        .prepare("SELECT data FROM roster_entry WHERE entity = 'staff' ORDER BY rowid")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|d| serde_json::from_str(d).unwrap_or_default())
        .collect();
    if rows.is_empty() {
        // nothing to move: the catalog of positions is written the first time it is used, with the institution's kind
        tx.execute("DELETE FROM roster_field WHERE entity = 'staff'", [])?;
        return Ok(());
    }
    let own: Vec<crate::hr::legacy::LegacyField> = tx
        .prepare("SELECT key, title, kind, options FROM roster_field WHERE entity = 'staff' AND builtin = 0 ORDER BY position")?
        .query_map([], |r| {
            let options: Vec<serde_json::Value> = serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default();
            Ok(crate::hr::legacy::LegacyField {
                key: r.get(0)?,
                title: r.get(1)?,
                kind: r.get(2)?,
                options: options.iter().filter_map(|o| o["label"].as_str().map(String::from)).collect(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let kind: Option<String> = tx.query_row("SELECT kind FROM institution LIMIT 1", [], |r| r.get(0)).optional()?;
    let flavor = crate::profile_sync::flavor_of(kind.as_deref());
    let moved = crate::hr::legacy::import(tx, flavor, &rows, &own).map_err(hr_failure)?;
    tx.execute("DELETE FROM roster_entry WHERE entity = 'staff'", [])?;
    tx.execute("DELETE FROM roster_field WHERE entity = 'staff'", [])?;
    // written by hand: at this version the audit log has no author yet (ADR-028 adds it in 0016)
    tx.execute(
        "INSERT INTO audit_log (at, event, entity, entity_id, details_json) VALUES (strftime('%Y-%m-%dT%H:%M:%SZ','now'), 'hr.imported', 'hr_person', NULL, ?1)",
        [serde_json::json!({ "people": moved }).to_string()],
    )?;
    Ok(())
}

pub fn run(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;
    for (version, sql) in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [version],
            |r| r.get(0),
        )?;
        if applied {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        after(*version, &tx)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
            [version],
        )?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A database left by Phase 2 (versions 1 and 2, with a usage row) must upgrade in place.
    #[test]
    fn a_phase_2_database_gains_the_metrics_columns_and_keeps_its_rows() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);").unwrap();
        for (version, sql) in &MIGRATIONS[..2] {
            conn.execute_batch(sql).unwrap();
            conn.execute("INSERT INTO schema_migrations VALUES (?1, 'then')", [version]).unwrap();
        }
        conn.execute(
            "INSERT INTO ai_usage (id,at,task,provider,model,input_tokens,output_tokens,estimated_cost_mxn,success)
             VALUES ('old','2026-09-30T10:00:00Z','diagnosis.summary','anthropic','claude-sonnet-5-5',3000,900,0.5,1)",
            [],
        )
        .unwrap();

        run(&mut conn).unwrap();

        let (input, latency, thought, kind): (i64, Option<i64>, i64, Option<String>) = conn
            .query_row("SELECT input_tokens, latency_ms, thought_tokens, error_kind FROM ai_usage WHERE id='old'", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .unwrap();
        assert_eq!((input, latency, thought, kind), (3000, None, 0, None), "old rows keep their data; new columns start empty");
        let versions: i64 = conn.query_row("SELECT count(*) FROM schema_migrations", [], |r| r.get(0)).unwrap();
        assert_eq!(versions, MIGRATIONS.len() as i64);
        run(&mut conn).unwrap(); // running again changes nothing
    }

    /// A database left by the free-text projects (versions 1 to 4, with a project) upgrades in place: the old
    /// project becomes `internal`, has no call, and the new columns of the call start empty.
    #[test]
    fn a_database_with_free_text_projects_upgrades_them_to_internal() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);").unwrap();
        for (version, sql) in &MIGRATIONS[..4] {
            conn.execute_batch(sql).unwrap();
            conn.execute("INSERT INTO schema_migrations VALUES (?1, 'then')", [version]).unwrap();
        }
        conn.execute_batch(
            "INSERT INTO institution (id,name,kind,created_at,updated_at) VALUES ('i','Asilo','other','t','t');
             INSERT INTO institution_profile (id,institution_id,version,created_at) VALUES ('p','i',1,'t');
             INSERT INTO project (id,institution_id,profile_id,title,stage,created_at,updated_at) VALUES ('old','i','p','Baño','DIAGNOSIS','t','t');
             INSERT INTO call_reading (id,name,status,created_at) VALUES ('r','Una','waiting','t');
             INSERT INTO document (id,kind,display_name,mime,data_level,clean_hash,extracted_text,redactions_count,created_at) VALUES ('d','call','a.pdf','application/pdf','green','h','x',0,'t');
             INSERT INTO call_reading_file (reading_id,document_id,position) VALUES ('r','d',0);",
        )
        .unwrap();

        run(&mut conn).unwrap();

        let (kind, reading): (String, Option<String>) = conn.query_row("SELECT kind, call_reading_id FROM project WHERE id='old'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!((kind.as_str(), reading), ("internal", None));
        let (funder, year, role): (Option<String>, Option<i64>, String) = conn
            .query_row("SELECT r.funder, r.year, f.role FROM call_reading r JOIN call_reading_file f ON f.reading_id = r.id", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap();
        assert_eq!((funder, year, role.as_str()), (None, None, "main"));
        // a project cannot be of a kind nobody knows
        assert!(conn.execute("UPDATE project SET kind='other' WHERE id='old'", []).is_err());
    }

    /// The staff of the old roster moves into the staff module (ADR-027): role -> position, contract -> modality,
    /// year -> approximate date, the institution's own fields with their values; the people served stay.
    #[test]
    fn the_staff_of_the_old_roster_moves_into_the_staff_module() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON; CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);").unwrap();
        for (version, sql) in &MIGRATIONS[..14] {
            conn.execute_batch(sql).unwrap();
            conn.execute("INSERT INTO schema_migrations VALUES (?1, 'then')", [version]).unwrap();
        }
        conn.execute_batch(
            r#"INSERT INTO institution (id,name,kind,created_at,updated_at) VALUES ('i','Asilo','elderly_home','t','t');
             INSERT INTO roster_field (entity,key,title,kind,options,builtin,locked_options,required,position)
               VALUES ('staff','full_name','Nombre completo','text','[]',1,0,1,1),
                      ('staff','own_talla','Talla','select','[{"value":"M","label":"M"}]',0,0,0,9),
                      ('beneficiary','full_name','Nombre completo','text','[]',1,0,1,1);
             INSERT INTO roster_entry (id,entity,data,created_at,updated_at) VALUES
               ('a','staff','{"full_name":"Carmen Olivia Salazar Rojas","role":"Enfermería","contract":"Por tiempo definido","shift":"Nocturno","monthly_salary_mxn":"9500","paid":"yes","start_year":"2019","phone":"55 5555 0122","own_talla":"M"}','t','t'),
               ('b','staff','{"full_name":"Lupita Voluntaria","role":"Acompañamiento","paid":"no","shift":"Sábados"}','t','t'),
               ('c','beneficiary','{"full_name":"Persona atendida"}','t','t');"#,
        )
        .unwrap();

        run(&mut conn).unwrap();

        let people: Vec<(String, Option<String>, Option<String>, String, Option<String>, i64, Option<String>, Option<i64>, String, String)> = conn
            .prepare(
                "SELECT p.first_names, p.last_name_1, p.last_name_2, j.modality, j.start_date, j.start_date_approx, j.shift, j.pay_amount_mxn, pos.title, p.extra
                 FROM hr_person p JOIN hr_job j ON j.person_id = p.id JOIN hr_position pos ON pos.id = j.position_id ORDER BY p.rowid",
            )
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(people.len(), 2);
        let carmen = &people[0];
        assert_eq!((carmen.0.as_str(), carmen.1.as_deref(), carmen.2.as_deref()), ("Carmen Olivia", Some("Salazar"), Some("Rojas")));
        assert_eq!((carmen.3.as_str(), carmen.4.as_deref(), carmen.5, carmen.6.as_deref(), carmen.7), ("fixed_term", Some("2019-01-01"), 1, Some("night"), Some(9_500)));
        assert_eq!(carmen.8, "Enfermería");
        assert!(carmen.9.contains("\"own_talla\":\"M\""), "{}", carmen.9);
        let lupita = &people[1];
        assert_eq!((lupita.3.as_str(), lupita.7, lupita.8.as_str()), ("volunteer", None, "Acompañamiento"));
        assert!(lupita.9.contains("Sábados"), "a schedule the catalog does not know is kept: {}", lupita.9);

        let served: i64 = conn.query_row("SELECT count(*) FROM care_person", [], |r| r.get(0)).unwrap();
        assert_eq!(served, 1, "the people served went to their own module (ADR-029)");
        let positions: i64 = conn.query_row("SELECT count(*) FROM hr_position WHERE title='Medicina'", [], |r| r.get(0)).unwrap();
        assert_eq!(positions, 1, "the catalog starts with the positions of an elderly home");
        let event: String = conn.query_row("SELECT details_json FROM audit_log WHERE event='hr.imported'", [], |r| r.get(0)).unwrap();
        assert_eq!(event, "{\"people\":2}");
    }

    /// The people served of the old roster move into their module (ADR-029) with their ids, so a deletion that was
    /// waiting still finds them; then the roster goes away.
    #[test]
    fn the_people_served_of_the_old_roster_move_into_their_module() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON; CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);").unwrap();
        for (version, sql) in &MIGRATIONS[..16] {
            conn.execute_batch(sql).unwrap();
            conn.execute("INSERT INTO schema_migrations VALUES (?1, 'then')", [version]).unwrap();
        }
        conn.execute_batch(
            r#"INSERT INTO institution (id,name,kind,created_at,updated_at) VALUES ('i','Asilo','elderly_home','t','t');
             INSERT INTO roster_field (entity,key,title,kind,options,builtin,locked_options,required,position,hidden)
               VALUES ('beneficiary','own_dieta','Dieta','text','[]',0,0,0,9,1);
             INSERT INTO roster_entry (id,entity,data,created_at,updated_at,hidden) VALUES
               ('ben_old_1','beneficiary','{"full_name":"María de la Luz Pérez García","category":"Mujeres adultas mayores","age":"84","dependency":"high","monthly_fee_mxn":"3500","entry_year":"2019","phone":"55 5555 0131","own_dieta":"blanda"}','t','t',0),
               ('ben_old_2','beneficiary','{"full_name":"José Ramírez","category":"Hombres adultos mayores","age":"90"}','t','t',1);
             INSERT INTO app_user (id,username,display_name,role,password_hash,created_at,updated_at) VALUES ('u','rosa','Rosa','manager','x','t','t');
             INSERT INTO access_request (id,kind,target_id,target_label,requested_by,requested_at) VALUES ('r','beneficiary','ben_old_2','José Ramírez','u','t');"#,
        )
        .unwrap();

        run(&mut conn).unwrap();

        let rows: Vec<(String, String, Option<String>, Option<String>, Option<String>, i64, Option<String>, Option<i64>, i64, String)> = conn
            .prepare("SELECT id, first_names, last_name_1, last_name_2, sex, birth_date_approx, entry_date, monthly_fee_mxn, hidden, extra FROM care_person ORDER BY id")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(rows.len(), 2);
        let luz = &rows[0];
        assert_eq!((luz.0.as_str(), luz.1.as_str(), luz.2.as_deref(), luz.3.as_deref(), luz.4.as_deref()), ("ben_old_1", "María de la Luz", Some("Pérez"), Some("García"), Some("female")));
        assert_eq!((luz.5, luz.6.as_deref(), luz.7, luz.8), (1, Some("2019-01-01"), Some(3_500), 0));
        assert!(luz.9.contains("blanda"), "{}", luz.9);
        assert_eq!((rows[1].0.as_str(), rows[1].4.as_deref(), rows[1].8), ("ben_old_2", Some("male"), 1), "the hidden one stays hidden");
        let phone: String = conn.query_row("SELECT phone FROM care_contact WHERE person_id='ben_old_1'", [], |r| r.get(0)).unwrap();
        assert_eq!(phone, "55 5555 0131");
        let hidden_field: i64 = conn.query_row("SELECT hidden FROM care_custom_field WHERE key='own_dieta'", [], |r| r.get(0)).unwrap();
        assert_eq!(hidden_field, 1);
        let roster: i64 = conn.query_row("SELECT count(*) FROM sqlite_master WHERE name IN ('roster_entry','roster_field')", [], |r| r.get(0)).unwrap();
        assert_eq!(roster, 0, "the roster is gone");
        let target: String = conn.query_row("SELECT target_id FROM access_request WHERE id='r'", [], |r| r.get(0)).unwrap();
        let found: i64 = conn.query_row("SELECT count(*) FROM care_person WHERE id=?1", [target], |r| r.get(0)).unwrap();
        assert_eq!(found, 1, "the waiting request still finds its person");
    }

    /// Income written before ADR-026 keeps its amount as a yearly one; what said «cuota» becomes a fee estimate.
    #[test]
    fn old_income_lines_become_yearly_amounts_with_a_kind() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);").unwrap();
        for (version, sql) in &MIGRATIONS[..13] {
            conn.execute_batch(sql).unwrap();
            conn.execute("INSERT INTO schema_migrations VALUES (?1, 'then')", [version]).unwrap();
        }
        conn.execute_batch(
            "INSERT INTO institution (id,name,kind,created_at,updated_at) VALUES ('i','Asilo','other','t','t');
             INSERT INTO institution_profile (id,institution_id,version,created_at) VALUES ('p','i',1,'t');
             INSERT INTO income_source (id,profile_id,label,annual_amount_mxn,origin) VALUES ('a','p','Cuotas de recuperación',720000,'user');
             INSERT INTO income_source (id,profile_id,label,annual_amount_mxn,origin) VALUES ('b','p','Donativos',680000,'user');",
        )
        .unwrap();

        run(&mut conn).unwrap();

        let rows: Vec<(String, i64, String)> = conn
            .prepare("SELECT kind, amount_mxn, period FROM income_source ORDER BY id")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(rows, vec![("fee_estimate".into(), 720_000, "annual".into()), ("other".into(), 680_000, "annual".into())]);
        assert!(conn.execute("UPDATE income_source SET kind='gift' WHERE id='a'", []).is_err());
        assert!(conn.execute("INSERT INTO expense_item (id,profile_id,label,amount_mxn,period,origin) VALUES ('e','p','Luz',-1,'annual','user')", []).is_err());
    }
}
