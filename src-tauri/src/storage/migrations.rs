//! Numbered migrations. Once applied, a migration is never edited.

use rusqlite::Connection;

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
];

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
