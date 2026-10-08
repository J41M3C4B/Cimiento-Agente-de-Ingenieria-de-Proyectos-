//! What the projects may ask the core (ADR-032): the only door from the projects into the institution. The projects
//! move onto it in block B5; until then they still reach into the core (the debt of `architecture_tests`).

use crate::core::error::ServiceError;
use crate::scanner::guard::{Decision, QuarantineReport};
use rusqlite::Connection;

#[allow(dead_code)] // the projects move onto it in block B5
/// The sheet of the institution the AI reads: everything «Mi institución» and its modules say, as aggregates.
pub fn institution_sheet(conn: &Connection) -> Result<String, ServiceError> {
    crate::core::ai_sheet::profile_context(conn)
}

#[allow(dead_code)] // the projects move onto it in block B5
/// Free texts through the scanner, with the words of the institution. `Ok(Err(report))` means quarantine.
pub fn guard_texts(conn: &Connection, entity: &str, fields: &[(String, String)], decision: Option<Decision>) -> Result<Result<Vec<String>, QuarantineReport>, ServiceError> {
    crate::core::screen::guard_texts(conn, entity, fields, decision)
}
