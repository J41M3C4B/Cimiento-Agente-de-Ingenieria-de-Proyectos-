//! The border of the module of the people served: the only things the rest of the app may ask it for. Everything
//! that comes out of here is anonymous (ADR-029).

use super::domain::aggregate::{self, CareSummary, Indicators, Member, PopulationLine};
use super::domain::catalog::Flavor;
use super::storage::{self as store, Group, StoredPerson};
use super::CareError;
use rusqlite::Connection;

pub use super::domain::aggregate::{Count, MIN_GROUP};

fn with_members<T>(conn: &Connection, f: impl FnOnce(&[Member], &str) -> T) -> Result<T, CareError> {
    let stored: Vec<StoredPerson> = store::people(conn)?;
    let groups: Vec<Group> = store::groups(conn)?;
    let members: Vec<Member> = stored
        .iter()
        .map(|p| Member { data: &p.data, own_group: p.data.group_id.as_deref().and_then(|g| groups.iter().find(|x| x.id == g)).map(|g| g.title.as_str()) })
        .collect();
    let today = store::today(conn)?;
    Ok(f(&members, &today))
}

/// One anonymous line per group, level of support and fee: what the profile keeps (people served and stay fees).
pub fn population_lines(conn: &Connection, flavor: Flavor) -> Result<Vec<PopulationLine>, CareError> {
    with_members(conn, |m, today| aggregate::population_lines(m, flavor, today))
}

/// The indicators of the board.
pub fn indicators(conn: &Connection, flavor: Flavor) -> Result<Indicators, CareError> {
    with_members(conn, |m, today| aggregate::indicators(m, flavor, today, |d| d.progress(flavor, d.curp.as_deref().is_some_and(|c| !c.is_empty())).percent))
}

/// Requests of the waiting list still waiting.
pub fn waiting(conn: &Connection) -> Result<i64, CareError> {
    Ok(store::waitlist(conn)?.iter().filter(|w| w.input.status == "waiting").count() as i64)
}

/// Development only: replaces the people served with the fictitious ones of an example (`fixtures/padron-*.json`),
/// moved as the old roster was. `year` is the current year (for the ages). Runs inside the caller's transaction.
#[cfg(debug_assertions)]
pub fn load_example(conn: &Connection, year: i64, rows: Vec<std::collections::BTreeMap<String, String>>) -> Result<usize, CareError> {
    conn.execute("DELETE FROM care_person", [])?;
    let rows: Vec<super::legacy::LegacyRow> =
        rows.into_iter().map(|data| super::legacy::LegacyRow { id: store::new_id("ben"), data, hidden: false }).collect();
    super::legacy::import(conn, year, &rows, &[])
}

/// What the AI may read: counts by group, and personal attributes only for groups of `MIN_GROUP` or more.
pub fn ai_summary(conn: &Connection, flavor: Flavor) -> Result<CareSummary, CareError> {
    Ok(aggregate::ai_summary(&indicators(conn, flavor)?, waiting(conn)?))
}
