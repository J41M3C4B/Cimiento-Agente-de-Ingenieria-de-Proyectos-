//! The profile keeps anonymous lines of the staff (ADR-027) and of the people served (ADR-029), computed by their
//! modules. This writes them into the current profile after every change, and gives the sums with them. No name,
//! phone, mail or identifier ever comes in here.

use crate::care::domain::catalog::Flavor as CareFlavor;
use crate::domain::profile::{ContractKind, DependencyLevel, PopulationGroupInput, ProfileInput, ProfileTotals, StaffGroupInput};
use crate::hr::domain::catalog::Flavor;
use crate::service::{ProfileView, ServiceError};
use crate::storage::profile as profile_store;
use rusqlite::Connection;

fn institution_kind(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT kind FROM institution LIMIT 1", [], |r| r.get(0)).ok()
}

/// The kind of institution, as the staff module understands it (to suggest positions).
pub fn flavor_of(kind: Option<&str>) -> Flavor {
    match kind {
        Some("elderly_home") => Flavor::ElderlyHome,
        Some("children_home") => Flavor::ChildrenHome,
        _ => Flavor::Other,
    }
}

/// The kind of institution, as the module of the people served understands it (extra data and age bands).
pub fn care_flavor_of(kind: Option<&str>) -> CareFlavor {
    match kind {
        Some("elderly_home") => CareFlavor::ElderlyHome,
        Some("children_home") => CareFlavor::ChildrenHome,
        _ => CareFlavor::Other,
    }
}

pub fn care_flavor(conn: &Connection) -> CareFlavor {
    care_flavor_of(institution_kind(conn).as_deref())
}

/// The anonymous lines of both modules, as the profile keeps them.
pub fn derive(conn: &Connection) -> Result<(Vec<StaffGroupInput>, Vec<PopulationGroupInput>), ServiceError> {
    let staff = crate::hr::api::staff_lines(conn)?
        .into_iter()
        .map(|l| StaffGroupInput {
            role: l.role,
            count: l.count,
            shift: l.shift,
            paid: l.paid,
            monthly_salary_mxn: l.monthly_pay,
            contract: l.contract.and_then(ContractKind::from_db),
            start_year: l.start_year,
            notes: None,
            relation: Some(l.relation.as_str().into()),
        })
        .collect();
    let population = crate::care::api::population_lines(conn, care_flavor(conn))?
        .into_iter()
        .map(|l| PopulationGroupInput {
            label: l.label,
            age_min: l.age_min,
            age_max: l.age_max,
            count: l.count,
            dependency_level: l.dependency.as_deref().and_then(DependencyLevel::from_db),
            notes: None,
            paying_count: (l.paying > 0).then_some(l.paying),
            monthly_fee_mxn: l.monthly_fee,
        })
        .collect();
    Ok((staff, population))
}

/// The totals with what the modules say now, and the rest of the profile (its income) when there is one.
pub(crate) fn totals(conn: &Connection) -> Result<ProfileTotals, ServiceError> {
    let (staff, population) = derive(conn)?;
    let (input, year) = match profile_store::load_current(conn)? {
        Some(p) => (p.input, p.as_of_year),
        None => (ProfileInput::default(), profile_store::current_year(conn)?),
    };
    Ok(ProfileInput { staff, population, ..input }.totals(year))
}

/// Writes the new lines into the current profile (a new draft if the last version was confirmed).
pub(crate) fn sync_profile(conn: &mut Connection) -> Result<Option<ProfileView>, ServiceError> {
    let Some(current) = profile_store::load_current(conn)? else { return Ok(None) };
    let mut input = current.input;
    let (staff, population) = derive(conn)?;
    input.staff = staff;
    input.population = population;
    Ok(Some(profile_store::save(conn, &input)?.into()))
}

/// Development only: replaces the staff and the people served with the fictitious ones of an example
/// (`fixtures/padron-*.json`), moved as the old roster was.
#[cfg(debug_assertions)]
pub fn seed_examples(conn: &mut Connection, raw: &str) -> Result<(), ServiceError> {
    use std::collections::BTreeMap;
    #[derive(serde::Deserialize)]
    struct Example {
        staff: Vec<BTreeMap<String, String>>,
        beneficiary: Vec<BTreeMap<String, String>>,
    }
    let example: Example = serde_json::from_str(raw).map_err(|e| ServiceError::Internal(e.to_string()))?;
    let kind = institution_kind(conn);
    let year = profile_store::current_year(conn)?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM hr_person", [])?;
    tx.execute("DELETE FROM care_person", [])?;
    crate::hr::legacy::import(&tx, flavor_of(kind.as_deref()), &example.staff, &[])?;
    let rows: Vec<crate::care::legacy::LegacyRow> = example
        .beneficiary
        .into_iter()
        .map(|data| crate::care::legacy::LegacyRow { id: crate::care::storage::new_id("ben"), data, hidden: false })
        .collect();
    crate::care::legacy::import(&tx, year, &rows, &[])?;
    tx.commit()?;
    Ok(())
}
