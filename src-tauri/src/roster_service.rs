//! The roster use cases (ADR-020): today, the people served. The staff moved to its own module (ADR-027) and only
//! its anonymous lines come in here, to be written in the profile with the groups of the people served. No name,
//! phone or mail ever leaves the roster or the staff module.

use crate::domain::profile::{ContractKind, PopulationGroupInput, ProfileInput, ProfileTotals, StaffGroupInput};
use crate::hr::domain::catalog::Flavor;
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

/// The kind of institution, as the staff module understands it (to suggest positions).
pub fn flavor_of(kind: Option<&str>) -> Flavor {
    match kind {
        Some("elderly_home") => Flavor::ElderlyHome,
        Some("children_home") => Flavor::ChildrenHome,
        _ => Flavor::Other,
    }
}

/// The aggregates of the staff module and of the roster of the people served, as the profile keeps them.
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
    let population = roster::derive_population(&store::list(conn, Entity::Beneficiary)?);
    Ok((staff, population))
}

fn not_staff(entity: Entity) -> Result<(), ServiceError> {
    if entity == Entity::Staff {
        return Err(ServiceError::StaffMoved);
    }
    Ok(())
}

/// The totals with what the roster says now, and the rest of the profile (its income) when there is one.
pub(crate) fn totals(conn: &Connection) -> Result<ProfileTotals, ServiceError> {
    let (staff, population) = derive(conn)?;
    let (input, year) = match profile_store::load_current(conn)? {
        Some(p) => (p.input, p.as_of_year),
        None => (ProfileInput::default(), profile_store::current_year(conn)?),
    };
    Ok(ProfileInput { staff, population, ..input }.totals(year))
}

pub fn overview(conn: &Connection, entity: Entity) -> Result<Overview, ServiceError> {
    not_staff(entity)?;
    Ok(Overview { fields: store::fields(conn, entity)?, entries: store::list(conn, entity)?, totals: totals(conn)? })
}

pub fn save_field(conn: &Connection, entity: Entity, input: &FieldInput) -> Result<Vec<RosterField>, ServiceError> {
    not_staff(entity)?;
    if input.title.trim().is_empty() {
        return Err(ServiceError::EmptyText);
    }
    Ok(store::save_field(conn, entity, input)?)
}

pub fn delete_field(conn: &Connection, entity: Entity, key: &str) -> Result<Vec<RosterField>, ServiceError> {
    not_staff(entity)?;
    Ok(store::delete_field(conn, entity, key)?)
}

/// Writes the new aggregates into the current profile (a new draft if the last version was confirmed).
pub(crate) fn sync_profile(conn: &mut Connection) -> Result<Option<ProfileView>, ServiceError> {
    let Some(current) = profile_store::load_current(conn)? else { return Ok(None) };
    let mut input = current.input;
    let (staff, population) = derive(conn)?;
    input.staff = staff;
    input.population = population;
    Ok(Some(profile_store::save(conn, &input)?.into()))
}

pub(crate) fn change(conn: &mut Connection, entity: Entity) -> Result<Change, ServiceError> {
    let profile = sync_profile(conn)?;
    Ok(Change { entries: store::list(conn, entity)?, totals: totals(conn)?, profile })
}

pub fn save_entry(conn: &mut Connection, entity: Entity, id: Option<&str>, data: &Data) -> Result<Change, ServiceError> {
    not_staff(entity)?;
    let fields = store::fields(conn, entity)?;
    let mut clean = roster::clean_entry(&fields, data).map_err(|e| ServiceError::InvalidRoster(e.code()))?;
    // a field hidden while its deletion waits keeps its value: if the deletion is refused, nothing was lost (ADR-028)
    if let Some(old) = id.map(|i| store::stored(conn, i)).transpose()?.flatten() {
        for key in store::hidden_field_keys(conn, entity)? {
            if let Some(v) = old.get(&key) {
                clean.insert(key, v.clone());
            }
        }
    }
    if !store::upsert(conn, entity, id, &clean)? {
        return Err(ServiceError::NotFound);
    }
    change(conn, entity)
}

pub fn delete_entry(conn: &mut Connection, entity: Entity, id: &str) -> Result<Change, ServiceError> {
    not_staff(entity)?;
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
    // the staff of the example goes into the staff module, as the old roster did when it moved (ADR-027)
    let kind: Option<String> = conn.query_row("SELECT kind FROM institution LIMIT 1", [], |r| r.get(0)).ok();
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM hr_person", [])?;
    crate::hr::legacy::import(&tx, flavor_of(kind.as_deref()), &example.staff, &[])?;
    tx.commit()?;
    for (entity, rows) in [(Entity::Beneficiary, example.beneficiary)] {
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
    use crate::storage::open_encrypted;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    #[test]
    fn the_staff_is_no_longer_kept_here() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        let data: Data = [("full_name".to_string(), "Ana".to_string()), ("role".to_string(), "Cocina".to_string())].into_iter().collect();
        assert!(matches!(save_entry(&mut c, Entity::Staff, None, &data), Err(ServiceError::StaffMoved)));
        assert!(matches!(overview(&c, Entity::Staff), Err(ServiceError::StaffMoved)));
    }
}
