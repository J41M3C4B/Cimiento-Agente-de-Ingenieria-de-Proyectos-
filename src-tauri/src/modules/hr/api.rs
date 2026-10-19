//! The border of the staff module: the only things the rest of the app may ask it for. Everything that comes out
//! of here is anonymous (ADR-027).

use super::domain::aggregate::{self, Member, StaffLine, StaffSummary};
use super::domain::catalog::{self, Flavor};
use super::service::resolve;
use super::storage::{self as store, StoredPerson};
use super::HrError;
use rusqlite::Connection;

pub use super::domain::aggregate::{Count, MIN_GROUP};
pub use super::domain::payroll::annual_benefits;

/// Gets the module ready the first time: the suggested positions for the kind of institution.
pub fn prepare(conn: &Connection, flavor: Flavor) -> Result<(), HrError> {
    store::seed_positions(conn, &catalog::default_positions(flavor))
}

fn with_members<T>(conn: &Connection, f: impl FnOnce(&[Member], &[super::domain::position::Position], &str) -> T) -> Result<T, HrError> {
    let stored: Vec<StoredPerson> = store::people(conn)?;
    let mut members = Vec::new();
    for p in &stored {
        // a record whose modality is gone counts as an indefinite job: the safe side for the payroll
        let (base, rules) = resolve(conn, &p.data.modality)?.unwrap_or(("indefinite", catalog::builtin_rules("indefinite").expect("built in")));
        members.push(Member { data: &p.data, base, rules });
    }
    let positions = store::positions(conn)?;
    let today = store::today(conn)?;
    Ok(f(&members, &positions, &today))
}

/// Development only: replaces the staff with the fictitious people of an example (`fixtures/padron-*.json`), moved
/// as the old roster was. Runs inside the caller's transaction.
#[cfg(debug_assertions)]
pub fn load_example(conn: &Connection, flavor: Flavor, rows: &[std::collections::BTreeMap<String, String>]) -> Result<usize, HrError> {
    conn.execute("DELETE FROM hr_person", [])?;
    super::legacy::import(conn, flavor, rows, &[])
}

/// One anonymous line per position and pay data: what the profile keeps to add up the payroll.
pub fn staff_lines(conn: &Connection) -> Result<Vec<StaffLine>, HrError> {
    with_members(conn, |m, p, _| aggregate::staff_lines(m, p))
}

/// The staff as the AI may read it: positions, counts, vacancies; schooling and seniority only in large groups.
pub fn ai_summary(conn: &Connection) -> Result<StaffSummary, HrError> {
    with_members(conn, aggregate::summary)
}
