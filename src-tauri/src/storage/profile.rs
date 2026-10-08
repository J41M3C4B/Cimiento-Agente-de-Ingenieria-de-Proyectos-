//! Institution profile persistence. Every confirmation freezes a version; editing
//! a confirmed profile starts a new draft version.

use super::StorageError;
use crate::audit::{self, AuditKind};
use crate::domain::profile::*;
use crate::scanner::ScannerConfig;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::json;
use ulid::Ulid;

fn id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredProfile {
    pub institution_id: String,
    pub profile_id: String,
    pub version: i64,
    /// `None` while it is a draft.
    pub confirmed_at: Option<String>,
    pub input: ProfileInput,
    /// The year the sums are made in (it sets how long each person has worked, for the benefits).
    pub as_of_year: i64,
}

/// The current year, from the clock of the database.
pub fn current_year(conn: &Connection) -> Result<i64, StorageError> {
    Ok(conn.query_row("SELECT CAST(strftime('%Y','now') AS INTEGER)", [], |r| r.get(0))?)
}

/// The lists that hang from a version of the profile. `income_source` and `expense_item` hang from the versions
/// written before the money became a module of its own (migration 0020); they are history and not written any more.
const PROFILE_LISTS: [&str; 2] = ["population_group", "staff_group"];

fn text(o: &Option<String>) -> Option<&str> {
    o.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Latest version of the profile (draft or confirmed), if any.
pub fn load_current(conn: &Connection) -> Result<Option<StoredProfile>, StorageError> {
    let head = conn
        .query_row(
            "SELECT i.id, i.name, i.kind, i.mission, i.legal_rfc, i.contact_phone, i.contact_email, i.legal_rep_name,
                    p.id, p.version, p.confirmed_at, p.capacity_total, p.notes,
                    i.state, i.municipality, i.founded_year, i.legal_form, i.authorized_donee, i.cluni,
                    p.served_estimate, p.staff_paid_estimate, p.staff_volunteer_estimate
             FROM institution i JOIN institution_profile p ON p.institution_id = i.id
             ORDER BY p.version DESC LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    InstitutionInput {
                        name: r.get(1)?,
                        kind: InstitutionKind::from_db(&r.get::<_, String>(2)?),
                        mission: r.get(3)?,
                        legal_rfc: r.get(4)?,
                        contact_phone: r.get(5)?,
                        contact_email: r.get(6)?,
                        legal_rep_name: r.get(7)?,
                        state: r.get(13)?,
                        municipality: r.get(14)?,
                        founded_year: r.get(15)?,
                        legal_form: r.get(16)?,
                        authorized_donee: r.get(17)?,
                        cluni: r.get(18)?,
                    },
                    r.get::<_, String>(8)?,
                    r.get::<_, i64>(9)?,
                    r.get::<_, Option<String>>(10)?,
                    r.get::<_, Option<i64>>(11)?,
                    r.get::<_, Option<String>>(12)?,
                    [r.get::<_, Option<i64>>(19)?, r.get::<_, Option<i64>>(20)?, r.get::<_, Option<i64>>(21)?],
                ))
            },
        )
        .optional()?;
    let Some((institution_id, institution, profile_id, version, confirmed_at, capacity_total, notes, [served_estimate, staff_paid_estimate, staff_volunteer_estimate])) =
        head
    else {
        return Ok(None);
    };

    let population = conn
        .prepare("SELECT label, age_min, age_max, count, dependency_level, notes, paying_count, monthly_fee_mxn FROM population_group WHERE profile_id = ?1 ORDER BY rowid")?
        .query_map([&profile_id], |r| {
            Ok(PopulationGroupInput {
                label: r.get(0)?,
                age_min: r.get(1)?,
                age_max: r.get(2)?,
                count: r.get(3)?,
                dependency_level: r.get::<_, Option<String>>(4)?.and_then(|s| DependencyLevel::from_db(&s)),
                notes: r.get(5)?,
                paying_count: r.get(6)?,
                monthly_fee_mxn: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let staff = conn
        .prepare("SELECT role, count, shift, paid, monthly_salary_mxn, contract, start_year, notes, relation FROM staff_group WHERE profile_id = ?1 ORDER BY rowid")?
        .query_map([&profile_id], |r| {
            Ok(StaffGroupInput {
                role: r.get(0)?,
                count: r.get(1)?,
                shift: r.get(2)?,
                paid: r.get::<_, i64>(3)? != 0,
                monthly_salary_mxn: r.get(4)?,
                contract: r.get::<_, Option<String>>(5)?.and_then(|s| ContractKind::from_db(&s)),
                start_year: r.get(6)?,
                notes: r.get(7)?,
                relation: r.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(StoredProfile {
        institution_id,
        profile_id,
        version,
        confirmed_at,
        input: ProfileInput {
            institution,
            capacity_total,
            notes,
            served_estimate,
            staff_paid_estimate,
            staff_volunteer_estimate,
            population,
            staff,
        },
        as_of_year: current_year(conn)?,
    }))
}

/// Saves the profile as the current draft. If the latest version is already
/// confirmed, a new version is started so the confirmed one stays intact.
pub fn save(conn: &mut Connection, input: &ProfileInput) -> Result<StoredProfile, StorageError> {
    let tx = conn.transaction()?;
    let inst = &input.institution;

    let existing_inst: Option<String> =
        tx.query_row("SELECT id FROM institution LIMIT 1", [], |r| r.get(0)).optional()?;
    let institution_id = match existing_inst {
        Some(iid) => {
            tx.execute(
                "UPDATE institution SET name=?2, kind=?3, mission=?4, legal_rfc=?5, contact_phone=?6, contact_email=?7,
                        legal_rep_name=?8, state=?9, municipality=?10, founded_year=?11, legal_form=?12, authorized_donee=?13,
                        cluni=?14, updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1",
                params![iid, inst.name.trim(), inst.kind.as_db(), text(&inst.mission), text(&inst.legal_rfc),
                        text(&inst.contact_phone), text(&inst.contact_email), text(&inst.legal_rep_name), text(&inst.state),
                        text(&inst.municipality), inst.founded_year, text(&inst.legal_form), text(&inst.authorized_donee),
                        text(&inst.cluni)],
            )?;
            iid
        }
        None => {
            let iid = id("inst");
            tx.execute(
                "INSERT INTO institution (id,name,kind,mission,legal_rfc,contact_phone,contact_email,legal_rep_name,state,
                        municipality,founded_year,legal_form,authorized_donee,cluni,created_at,updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
                params![iid, inst.name.trim(), inst.kind.as_db(), text(&inst.mission), text(&inst.legal_rfc),
                        text(&inst.contact_phone), text(&inst.contact_email), text(&inst.legal_rep_name), text(&inst.state),
                        text(&inst.municipality), inst.founded_year, text(&inst.legal_form), text(&inst.authorized_donee),
                        text(&inst.cluni)],
            )?;
            iid
        }
    };

    let latest: Option<(String, i64, Option<String>)> = tx
        .query_row(
            "SELECT id, version, confirmed_at FROM institution_profile WHERE institution_id=?1 ORDER BY version DESC LIMIT 1",
            [&institution_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let profile_id = match latest {
        Some((pid, _, None)) => {
            // current draft: update in place
            tx.execute(
                "UPDATE institution_profile SET capacity_total=?2, notes=?3, served_estimate=?4,
                        staff_paid_estimate=?5, staff_volunteer_estimate=?6 WHERE id=?1",
                params![pid, input.capacity_total, text(&input.notes), input.served_estimate,
                        input.staff_paid_estimate, input.staff_volunteer_estimate],
            )?;
            for t in PROFILE_LISTS {
                tx.execute(&format!("DELETE FROM {t} WHERE profile_id=?1"), [&pid])?;
            }
            pid
        }
        other => {
            let version = other.map_or(1, |(_, v, _)| v + 1);
            let pid = id("prof");
            tx.execute(
                "INSERT INTO institution_profile (id,institution_id,version,capacity_total,notes,served_estimate,
                        staff_paid_estimate,staff_volunteer_estimate,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
                params![pid, institution_id, version, input.capacity_total, text(&input.notes),
                        input.served_estimate, input.staff_paid_estimate, input.staff_volunteer_estimate],
            )?;
            pid
        }
    };

    for g in &input.population {
        tx.execute(
            "INSERT INTO population_group (id,profile_id,label,age_min,age_max,count,dependency_level,notes,paying_count,monthly_fee_mxn,origin)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'user')",
            params![id("pop"), profile_id, g.label.trim(), g.age_min, g.age_max, g.count,
                    g.dependency_level.map(|d| d.as_db()), text(&g.notes), g.paying_count, g.monthly_fee_mxn],
        )?;
    }
    for s in &input.staff {
        tx.execute(
            "INSERT INTO staff_group (id,profile_id,role,count,shift,paid,monthly_salary_mxn,contract,start_year,notes,relation,origin)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,'computed')",
            params![id("staff"), profile_id, s.role.trim(), s.count, text(&s.shift), s.paid as i64,
                    s.monthly_salary_mxn, s.contract.map(|c| c.as_db()), s.start_year, text(&s.notes), text(&s.relation)],
        )?;
    }
    tx.commit()?;
    load_current(conn)?.ok_or(StorageError::NoProfile)
}

/// Freezes the current draft as a confirmed version.
pub fn confirm(conn: &mut Connection) -> Result<StoredProfile, StorageError> {
    let tx = conn.transaction()?;
    let draft: Option<(String, i64)> = tx
        .query_row(
            "SELECT id, version FROM institution_profile WHERE confirmed_at IS NULL ORDER BY version DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (pid, version) = draft.ok_or(StorageError::NothingToConfirm)?;
    tx.execute(
        "UPDATE institution_profile SET confirmed_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1",
        [&pid],
    )?;
    for t in PROFILE_LISTS {
        tx.execute(
            &format!("UPDATE {t} SET confirmed_at=strftime('%Y-%m-%dT%H:%M:%SZ','now'), confirmed_by='manager' WHERE profile_id=?1"),
            [&pid],
        )?;
    }
    audit::record(&tx, AuditKind::ProfileConfirmed, Some("institution_profile"), Some(&pid), json!({ "version": version }))?;
    tx.commit()?;
    load_current(conn)?.ok_or(StorageError::NoProfile)
}

/// What the scanner must not flag: the institution's own contact data and names.
pub fn scanner_config(conn: &Connection) -> Result<ScannerConfig, StorageError> {
    let row = conn
        .query_row(
            "SELECT name, contact_phone, contact_email, legal_rep_name FROM institution LIMIT 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, Option<String>>(3)?)),
        )
        .optional()?;
    Ok(match row {
        None => ScannerConfig::default(),
        Some((name, phone, email, rep)) => ScannerConfig {
            institutional_phones: phone.into_iter().collect(),
            institutional_emails: email.into_iter().collect(),
            safe_phrases: std::iter::once(name).chain(rep).collect(),
        },
    })
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

    fn sample() -> ProfileInput {
        ProfileInput {
            institution: InstitutionInput {
                name: "Casa Hogar Ficticia".into(),
                kind: InstitutionKind::ElderlyHome,
                contact_phone: Some("55 1234 5678".into()),
                ..Default::default()
            },
            capacity_total: Some(25),
            population: vec![PopulationGroupInput {
                label: "Adultos mayores".into(), count: 18, dependency_level: Some(DependencyLevel::High), ..Default::default()
            }],
            staff: vec![StaffGroupInput { role: "Enfermería".into(), count: 3, paid: true, ..Default::default() }],
            ..Default::default()
        }
    }

    #[test]
    fn saves_and_loads_a_profile() {
        let (_d, mut c) = conn();
        assert!(load_current(&c).unwrap().is_none());
        let saved = save(&mut c, &sample()).unwrap();
        assert_eq!(saved.version, 1);
        assert!(saved.confirmed_at.is_none());
        assert_eq!(saved.input.population[0].count, 18);
        assert_eq!(saved.input.institution.kind, InstitutionKind::ElderlyHome);
        assert!((2026..2200).contains(&saved.as_of_year));
    }

    #[test]
    fn the_lists_are_frozen_with_the_version_and_editing_starts_a_new_one() {
        let (_d, mut c) = conn();
        save(&mut c, &sample()).unwrap();
        confirm(&mut c).unwrap();
        let open: i64 = c.query_row("SELECT count(*) FROM population_group WHERE confirmed_at IS NULL", [], |r| r.get(0)).unwrap();
        assert_eq!(open, 0);
        let mut p = sample();
        p.population.push(PopulationGroupInput { label: "Hombres".into(), count: 2, ..Default::default() });
        let v2 = save(&mut c, &p).unwrap();
        assert_eq!((v2.version, v2.input.population.len()), (2, 2));
        let all: i64 = c.query_row("SELECT count(*) FROM population_group", [], |r| r.get(0)).unwrap();
        assert_eq!(all, 3, "version 1 keeps its own list");
    }

    #[test]
    fn editing_a_draft_does_not_create_versions() {
        let (_d, mut c) = conn();
        save(&mut c, &sample()).unwrap();
        let mut p = sample();
        p.population[0].count = 19;
        p.population.push(PopulationGroupInput { label: "Otro".into(), count: 2, ..Default::default() });
        let s = save(&mut c, &p).unwrap();
        assert_eq!(s.version, 1);
        assert_eq!(s.input.population.len(), 2);
        let n: i64 = c.query_row("SELECT count(*) FROM population_group", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn confirming_freezes_a_version_and_editing_starts_the_next() {
        let (_d, mut c) = conn();
        save(&mut c, &sample()).unwrap();
        let confirmed = confirm(&mut c).unwrap();
        assert!(confirmed.confirmed_at.is_some());
        let unconfirmed: i64 = c
            .query_row("SELECT count(*) FROM population_group WHERE confirmed_at IS NULL", [], |r| r.get(0))
            .unwrap();
        assert_eq!(unconfirmed, 0);

        let mut p = sample();
        p.population[0].count = 20;
        let v2 = save(&mut c, &p).unwrap();
        assert_eq!(v2.version, 2);
        assert!(v2.confirmed_at.is_none());
        // version 1 is intact
        let old: i64 = c
            .query_row(
                "SELECT g.count FROM population_group g JOIN institution_profile p ON p.id=g.profile_id WHERE p.version=1",
                [], |r| r.get(0),
            )
            .unwrap();
        assert_eq!(old, 18);
        // audit has the event, without content
        let (ev, d): (String, String) = c
            .query_row("SELECT event, details_json FROM audit_log", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(ev, "profile.confirmed");
        assert_eq!(d, "{\"version\":1}");
    }

    #[test]
    fn nothing_to_confirm_without_a_draft() {
        let (_d, mut c) = conn();
        assert!(matches!(confirm(&mut c), Err(StorageError::NothingToConfirm)));
        save(&mut c, &sample()).unwrap();
        confirm(&mut c).unwrap();
        assert!(matches!(confirm(&mut c), Err(StorageError::NothingToConfirm)));
    }

    #[test]
    fn scanner_config_comes_from_the_institution() {
        let (_d, mut c) = conn();
        assert!(scanner_config(&c).unwrap().safe_phrases.is_empty());
        save(&mut c, &sample()).unwrap();
        let cfg = scanner_config(&c).unwrap();
        assert_eq!(cfg.institutional_phones, vec!["55 1234 5678"]);
        assert_eq!(cfg.safe_phrases, vec!["Casa Hogar Ficticia"]);
    }
}
