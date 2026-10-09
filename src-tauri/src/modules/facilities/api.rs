//! The border of the facilities module: the only things the rest of the app may ask it for. A facility carries no
//! personal datum, so the summary may reach the AI whole (its texts went through the scanner when they were saved).

use super::domain::aggregate::{self, Indicators, SiteSummary};
use super::FacilitiesError;
use rusqlite::Connection;

pub use super::domain::aggregate::GroupRef;
pub use super::domain::group::States;

/// Every site with its spaces and equipment.
pub fn summaries(conn: &Connection) -> Result<Vec<SiteSummary>, FacilitiesError> {
    super::service::summaries(conn)
}

/// Development only: removes every site with its spaces and equipment, before an example is loaded.
#[cfg(debug_assertions)]
pub fn clear_for_example(conn: &Connection) -> Result<(), FacilitiesError> {
    conn.execute("DELETE FROM fac_site", [])?;
    Ok(())
}

/// The indicators over every site.
pub fn indicators(conn: &Connection) -> Result<Indicators, FacilitiesError> {
    Ok(aggregate::indicators(&summaries(conn)?))
}
