//! The roster use cases (ADR-020). Records are saved on their own, apart from the profile; what the profile
//! (and so the AI, the diagnosis and the documents) gets from them is only the aggregates: how many by position,
//! how many by group, the payroll and the fees. No name, phone or mail ever leaves this module.

use crate::domain::profile::{PopulationGroupInput, ProfileInput, ProfileTotals, StaffGroupInput};
use crate::domain::roster::{self, Data, Entity, RosterEntry, RosterField};
use crate::service::{ProfileView, ServiceError};
use crate::storage::profile as profile_store;
use crate::storage::roster::{self as store, FieldInput};
use rusqlite::Connection;
use serde::Serialize;

/// Everything a tab of the roster needs to draw itself.
#[derive(Debug, Serialize)]
pub struct Overview {
    pub fields: Vec<RosterField>,
    pub entries: Vec<RosterEntry>,
    pub totals: ProfileTotals,
}

/// What changed after a record was saved or removed.
#[derive(Debug, Serialize)]
pub struct Change {
    pub entries: Vec<RosterEntry>,
    pub totals: ProfileTotals,
    /// The profile with the new aggregates, if there is one already.
    pub profile: Option<ProfileView>,
}

/// The aggregates of both lists, as the profile keeps them.
pub fn derive(conn: &Connection) -> Result<(Vec<StaffGroupInput>, Vec<PopulationGroupInput>), ServiceError> {
    let staff = roster::derive_staff(&store::list(conn, Entity::Staff)?);
    let population = roster::derive_population(&store::list(conn, Entity::Beneficiary)?);
    Ok((staff, population))
}

fn totals(conn: &Connection) -> Result<ProfileTotals, ServiceError> {
    let (staff, population) = derive(conn)?;
    Ok(ProfileInput { staff, population, ..Default::default() }.totals())
}

pub fn overview(conn: &Connection, entity: Entity) -> Result<Overview, ServiceError> {
    Ok(Overview { fields: store::fields(conn, entity)?, entries: store::list(conn, entity)?, totals: totals(conn)? })
}

pub fn save_field(conn: &Connection, entity: Entity, input: &FieldInput) -> Result<Vec<RosterField>, ServiceError> {
    if input.title.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    Ok(store::save_field(conn, entity, input)?)
}

pub fn delete_field(conn: &Connection, entity: Entity, key: &str) -> Result<Vec<RosterField>, ServiceError> {
    Ok(store::delete_field(conn, entity, key)?)
}

/// Writes the new aggregates into the current profile (a new draft if the last version was confirmed).
fn sync_profile(conn: &mut Connection) -> Result<Option<ProfileView>, ServiceError> {
    let Some(current) = profile_store::load_current(conn)? else { return Ok(None) };
    let mut input = current.input;
    let (staff, population) = derive(conn)?;
    input.staff = staff;
    input.population = population;
    Ok(Some(profile_store::save(conn, &input)?.into()))
}

fn change(conn: &mut Connection, entity: Entity) -> Result<Change, ServiceError> {
    let profile = sync_profile(conn)?;
    Ok(Change { entries: store::list(conn, entity)?, totals: totals(conn)?, profile })
}

pub fn save_entry(conn: &mut Connection, entity: Entity, id: Option<&str>, data: &Data) -> Result<Change, ServiceError> {
    let fields = store::fields(conn, entity)?;
    let clean = roster::clean_entry(&fields, data).map_err(|e| ServiceError::InvalidRoster(e.code()))?;
    if !store::upsert(conn, entity, id, &clean)? {
        return Err(ServiceError::NotFound);
    }
    change(conn, entity)
}

pub fn delete_entry(conn: &mut Connection, entity: Entity, id: &str) -> Result<Change, ServiceError> {
    if !store::delete(conn, entity, id)? {
        return Err(ServiceError::NotFound);
    }
    change(conn, entity)
}

/// Development only: replaces the roster with the fictitious people of an example (`fixtures/padron-*.json`). The
/// forms start from the defaults of the institution's kind, and the selectors gain what the examples use.
#[cfg(debug_assertions)]
pub fn seed_roster(conn: &mut Connection, raw: &str) -> Result<(), ServiceError> {
    use crate::domain::roster::FieldKind;
    #[derive(serde::Deserialize)]
    struct Example {
        staff: Vec<Data>,
        beneficiary: Vec<Data>,
    }
    let example: Example = serde_json::from_str(raw).map_err(|e| ServiceError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM roster_field", [])?;
    for (entity, rows) in [(Entity::Staff, example.staff), (Entity::Beneficiary, example.beneficiary)] {
        store::clear(conn, entity)?;
        for f in store::fields(conn, entity)?.iter().filter(|f| f.kind == FieldKind::Select && !f.locked_options) {
            let mut labels: Vec<String> = f.options.iter().map(|o| o.label.clone()).collect();
            let before = labels.len();
            for v in rows.iter().filter_map(|r| r.get(&f.key)) {
                if !labels.contains(v) {
                    labels.push(v.clone());
                }
            }
            if labels.len() > before {
                store::save_field(conn, entity, &FieldInput { key: Some(f.key.clone()), title: f.title.clone(), kind: f.kind, options: labels })?;
            }
        }
        for row in &rows {
            store::upsert(conn, entity, None, row)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::InstitutionInput;
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

    fn with_profile(c: &mut Connection) {
        let input = ProfileInput { institution: InstitutionInput { name: "Asilo Ficticio".into(), ..Default::default() }, ..Default::default() };
        profile_store::save(c, &input).unwrap();
    }

    #[test]
    fn the_profile_only_gets_aggregates_never_names_or_contact_data() {
        let (_d, mut c) = conn();
        with_profile(&mut c);
        save_entry(&mut c, Entity::Staff, None, &data(&[("full_name", "Ana Secreta Pérez"), ("role", "Cocina"), ("monthly_salary_mxn", "7000"), ("phone", "55 1234 5678"), ("email", "ana@example.org")])).unwrap();
        save_entry(&mut c, Entity::Staff, None, &data(&[("full_name", "Rosa Oculta López"), ("role", "Cocina"), ("monthly_salary_mxn", "7000")])).unwrap();
        let change = save_entry(&mut c, Entity::Beneficiary, None, &data(&[("full_name", "Luz Reservada"), ("category", "Niñas"), ("age", "8"), ("monthly_fee_mxn", "1200")])).unwrap();

        let profile = change.profile.expect("the profile exists");
        assert_eq!(profile.input.staff.len(), 1);
        assert_eq!((profile.input.staff[0].role.as_str(), profile.input.staff[0].count), ("Cocina", 2));
        assert_eq!((profile.totals.payroll_monthly_mxn, profile.totals.population, profile.totals.fees_monthly_mxn), (14_000, 1, 1_200));

        // nothing the person wrote about somebody is in the profile, however it is looked at
        let everything = serde_json::to_string(&profile.input).unwrap();
        for secret in ["Secreta", "Oculta", "Reservada", "55 1234", "ana@example.org"] {
            assert!(!everything.contains(secret), "{secret} leaked into the profile");
        }
        let summary = crate::diagnosis_service::profile_summary_for_tests(&c).unwrap();
        for secret in ["Secreta", "Oculta", "Reservada", "7000", "1200"] {
            assert!(!summary.contains(secret), "{secret} leaked into what the AI reads");
        }
    }

    #[test]
    fn totals_work_before_the_profile_exists_and_removing_updates_them() {
        let (_d, mut c) = conn();
        let first = save_entry(&mut c, Entity::Staff, None, &data(&[("full_name", "Ana"), ("role", "Cocina"), ("monthly_salary_mxn", "5000")])).unwrap();
        assert!(first.profile.is_none());
        assert_eq!(first.totals.payroll_annual_mxn, 60_000);
        let gone = delete_entry(&mut c, Entity::Staff, &first.entries[0].id).unwrap();
        assert_eq!(gone.totals.payroll_monthly_mxn, 0);
        assert!(matches!(delete_entry(&mut c, Entity::Staff, "nope"), Err(ServiceError::NotFound)));
    }

    #[test]
    fn a_record_with_a_missing_or_wrong_value_is_refused_and_nothing_is_saved() {
        let (_d, mut c) = conn();
        assert!(matches!(save_entry(&mut c, Entity::Staff, None, &data(&[("role", "Cocina")])), Err(ServiceError::InvalidRoster("required"))));
        assert!(matches!(save_entry(&mut c, Entity::Staff, None, &data(&[("full_name", "Ana"), ("role", "Cocina"), ("monthly_salary_mxn", "mucho")])), Err(ServiceError::InvalidRoster("not_a_number"))));
        assert!(overview(&c, Entity::Staff).unwrap().entries.is_empty());
    }

    #[test]
    fn saving_the_profile_form_does_not_wipe_the_aggregates() {
        let (_d, mut c) = conn();
        with_profile(&mut c);
        save_entry(&mut c, Entity::Staff, None, &data(&[("full_name", "Ana"), ("role", "Cocina")])).unwrap();
        let input = ProfileInput { institution: InstitutionInput { name: "Asilo Ficticio".into(), ..Default::default() }, ..Default::default() };
        let crate::service::SaveProfileOutcome::Saved { profile } = crate::service::save_profile(&mut c, input, None).unwrap() else { panic!("saved") };
        assert_eq!(profile.input.staff.len(), 1);
    }
}
