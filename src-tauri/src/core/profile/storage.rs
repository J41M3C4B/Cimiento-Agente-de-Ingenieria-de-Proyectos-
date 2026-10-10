//! Institution profile persistence. Every confirmation freezes a version; editing
//! a confirmed profile starts a new draft version.

use crate::storage::StorageError;
use crate::audit::{self, AuditKind};
use crate::core::profile::domain::*;
use crate::scanner::ScannerConfig;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::json;
use ulid::Ulid;

/// The data of the institution kept as fields of the catalog (ADR-033 §1): field id → its column of `institution`.
/// The forms read and write them by id (`InstitutionInput::details`); no screen maps them one by one.
pub const DETAILS: &[(&str, &str)] = &[
    ("institution.legal_name", "legal_name"),
    ("institution.purpose", "purpose"),
    ("institution.services", "services"),
    ("institution.age_min", "age_min"),
    ("institution.age_max", "age_max"),
    ("institution.admission_criteria", "admission_criteria"),
    ("institution.street", "street"),
    ("institution.ext_number", "ext_number"),
    ("institution.int_number", "int_number"),
    ("institution.neighborhood", "neighborhood"),
    ("institution.postal_code", "postal_code"),
    ("institution.tax_regime", "tax_regime"),
    ("institution.fiscal_postal_code", "fiscal_postal_code"),
    ("institution.junta_folio", "junta_folio"),
    ("institution.donee_category", "donee_category"),
    ("institution.donee_letter_number", "donee_letter_number"),
    ("institution.donee_letter_date", "donee_letter_date"),
    ("institution.cluni_key", "cluni_key"),
    ("institution.legal_rep_valid_until", "legal_rep_valid_until"),
];

pub fn is_detail(field: &str) -> bool {
    DETAILS.iter().any(|(id, _)| *id == field)
}

/// The details as they are saved: a text or a whole number by field id; what is empty is left out.
fn load_details(conn: &Connection) -> Result<crate::common::forms::Values, StorageError> {
    use rusqlite::types::ValueRef;
    let cols: Vec<&str> = DETAILS.iter().map(|(_, c)| *c).collect();
    let found = conn
        .query_row(&format!("SELECT {} FROM institution LIMIT 1", cols.join(", ")), [], |r| {
            let mut v = crate::common::forms::Values::new();
            for (n, (id, _)) in DETAILS.iter().enumerate() {
                match r.get_ref(n)? {
                    ValueRef::Text(t) => {
                        v.insert(id.to_string(), json!(String::from_utf8_lossy(t)));
                    }
                    ValueRef::Integer(i) => {
                        v.insert(id.to_string(), json!(i));
                    }
                    _ => {}
                }
            }
            Ok(v)
        })
        .optional()?;
    Ok(found.unwrap_or_default())
}

/// Writes every detail: what the values do not carry is emptied (a form sends all of its fields).
fn save_details(tx: &rusqlite::Transaction<'_>, institution_id: &str, details: &crate::common::forms::Values) -> Result<(), StorageError> {
    use rusqlite::types::Value as Sql;
    let sets: Vec<String> = DETAILS.iter().enumerate().map(|(n, (_, c))| format!("{c}=?{}", n + 2)).collect();
    let mut values: Vec<Sql> = vec![Sql::Text(institution_id.to_string())];
    for (id, _) in DETAILS {
        values.push(match details.get(*id) {
            Some(serde_json::Value::String(s)) if !s.trim().is_empty() => Sql::Text(s.trim().to_string()),
            Some(serde_json::Value::Number(n)) => n.as_i64().map_or(Sql::Null, Sql::Integer),
            _ => Sql::Null,
        });
    }
    tx.execute(&format!("UPDATE institution SET {} WHERE id=?1", sets.join(", ")), rusqlite::params_from_iter(values))?;
    Ok(())
}

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

/// A list of codes kept as a JSON array; anything unreadable is an empty list.
fn codes(raw: String) -> Vec<String> {
    serde_json::from_str(&raw).unwrap_or_default()
}

fn codes_json(list: &[String]) -> String {
    serde_json::to_string(list).unwrap_or_else(|_| "[]".into())
}

/// Latest version of the profile (draft or confirmed), if any.
pub fn load_current(conn: &Connection) -> Result<Option<StoredProfile>, StorageError> {
    let head = conn
        .query_row(
            "SELECT i.id, i.name, i.kind, i.mission, i.legal_rfc, i.contact_phone, i.contact_email, i.legal_rep_name,
                    p.id, p.version, p.confirmed_at, p.capacity_total, p.notes,
                    i.state, i.municipality, i.founded_year, i.legal_form, i.authorized_donee, i.cluni,
                    p.served_estimate, p.staff_paid_estimate, p.staff_volunteer_estimate,
                    i.populations, i.sex_served, i.modalities, i.care_areas
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
                        attention: Some(Attention {
                            populations: codes(r.get(22)?),
                            sex_served: r.get(23)?,
                            modalities: codes(r.get(24)?),
                            care_areas: codes(r.get(25)?),
                        }),
                        details: None,
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
    let Some((institution_id, mut institution, profile_id, version, confirmed_at, capacity_total, notes, [served_estimate, staff_paid_estimate, staff_volunteer_estimate])) =
        head
    else {
        return Ok(None);
    };
    institution.details = Some(load_details(conn)?);

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

    let existing_inst: Option<(String, String, String, Option<String>, String, String)> = tx
        .query_row("SELECT id, kind, populations, sex_served, modalities, care_areas FROM institution LIMIT 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
        })
        .optional()?;
    // the attention profile decides the kind; the first start may still give the kind (ADR-033 §2)
    let previous = existing_inst.as_ref().map(|(_, kind, p, sex, m, a)| {
        (InstitutionKind::from_db(kind), Attention { populations: codes(p.clone()), sex_served: sex.clone(), modalities: codes(m.clone()), care_areas: codes(a.clone()) })
    });
    let (kind, attention) = reconcile_attention(previous, inst.kind, inst.attention.as_ref());
    let attention_params = (codes_json(&attention.populations), text(&attention.sex_served), codes_json(&attention.modalities), codes_json(&attention.care_areas));
    let institution_id = match existing_inst {
        Some((iid, ..)) => {
            tx.execute(
                "UPDATE institution SET name=?2, kind=?3, mission=?4, legal_rfc=?5, contact_phone=?6, contact_email=?7,
                        legal_rep_name=?8, state=?9, municipality=?10, founded_year=?11, legal_form=?12, authorized_donee=?13,
                        cluni=?14, populations=?15, sex_served=?16, modalities=?17, care_areas=?18,
                        updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1",
                params![iid, inst.name.trim(), kind.as_db(), text(&inst.mission), text(&inst.legal_rfc),
                        text(&inst.contact_phone), text(&inst.contact_email), text(&inst.legal_rep_name), text(&inst.state),
                        text(&inst.municipality), inst.founded_year, text(&inst.legal_form), text(&inst.authorized_donee),
                        text(&inst.cluni), attention_params.0, attention_params.1, attention_params.2, attention_params.3],
            )?;
            iid
        }
        None => {
            let iid = id("inst");
            tx.execute(
                "INSERT INTO institution (id,name,kind,mission,legal_rfc,contact_phone,contact_email,legal_rep_name,state,
                        municipality,founded_year,legal_form,authorized_donee,cluni,populations,sex_served,modalities,care_areas,
                        created_at,updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,
                         strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
                params![iid, inst.name.trim(), kind.as_db(), text(&inst.mission), text(&inst.legal_rfc),
                        text(&inst.contact_phone), text(&inst.contact_email), text(&inst.legal_rep_name), text(&inst.state),
                        text(&inst.municipality), inst.founded_year, text(&inst.legal_form), text(&inst.authorized_donee),
                        text(&inst.cluni), attention_params.0, attention_params.1, attention_params.2, attention_params.3],
            )?;
            iid
        }
    };

    if let Some(details) = &inst.details {
        save_details(&tx, &institution_id, details)?;
    }

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

    insert_lines(&tx, &profile_id, &input.staff, &input.population)?;
    tx.commit()?;
    load_current(conn)?.ok_or(StorageError::NoProfile)
}

/// The anonymous lines of a version. Both lists are computed from their modules, never written by a person.
fn insert_lines(tx: &rusqlite::Transaction<'_>, profile_id: &str, staff: &[StaffGroupInput], population: &[PopulationGroupInput]) -> Result<(), StorageError> {
    for g in population {
        tx.execute(
            "INSERT INTO population_group (id,profile_id,label,age_min,age_max,count,dependency_level,notes,paying_count,monthly_fee_mxn,origin)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'computed')",
            params![id("pop"), profile_id, g.label.trim(), g.age_min, g.age_max, g.count,
                    g.dependency_level.map(|d| d.as_db()), text(&g.notes), g.paying_count, g.monthly_fee_mxn],
        )?;
    }
    for s in staff {
        tx.execute(
            "INSERT INTO staff_group (id,profile_id,role,count,shift,paid,monthly_salary_mxn,contract,start_year,notes,relation,origin)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,'computed')",
            params![id("staff"), profile_id, s.role.trim(), s.count, text(&s.shift), s.paid as i64,
                    s.monthly_salary_mxn, s.contract.map(|c| c.as_db()), s.start_year, text(&s.notes), text(&s.relation)],
        )?;
    }
    Ok(())
}

/// Replaces the anonymous lines of the latest version in place, whether it is confirmed or not (audit D1). The
/// lines only mirror what the modules hold now, so a change in a module neither opens a version nor takes the
/// confirmation of what the person wrote; the progress over time is kept by the snapshots (ADR-033 §4).
pub fn refresh_lines(conn: &mut Connection, staff: &[StaffGroupInput], population: &[PopulationGroupInput]) -> Result<Option<StoredProfile>, StorageError> {
    let tx = conn.transaction()?;
    let latest: Option<String> =
        tx.query_row("SELECT id FROM institution_profile ORDER BY version DESC LIMIT 1", [], |r| r.get(0)).optional()?;
    let Some(profile_id) = latest else { return Ok(None) };
    for t in PROFILE_LISTS {
        tx.execute(&format!("DELETE FROM {t} WHERE profile_id=?1"), [&profile_id])?;
    }
    insert_lines(&tx, &profile_id, staff, population)?;
    tx.commit()?;
    load_current(conn)
}

/// Days since the latest confirmed version was confirmed (`None` if none is). The projects ask it through
/// `core::api` (the stage «Perfil» asks for a recent confirmation).
pub fn confirmed_days_ago(conn: &Connection) -> Result<Option<i64>, StorageError> {
    Ok(conn
        .query_row(
            "SELECT CAST(julianday('now') - julianday(confirmed_at) AS INTEGER)
             FROM institution_profile WHERE confirmed_at IS NOT NULL ORDER BY version DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?)
}

/// The institution and its latest confirmed version, which a new project is tied to (`None` if none is confirmed).
pub fn latest_confirmed(conn: &Connection) -> Result<Option<(String, String)>, StorageError> {
    Ok(conn
        .query_row(
            "SELECT institution_id, id FROM institution_profile WHERE confirmed_at IS NOT NULL ORDER BY version DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?)
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
