//! What the projects may ask the core (ADR-032): the only door from the projects into the institution. Everything
//! here is what the institution is as a whole (its sheet for the AI, its name and figures, its facilities in words)
//! and the services the core lends (the scanner with the words of the institution, the archive of documents).

use crate::core::error::ServiceError as CoreError;
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::scanner::ScannerConfig;
use rusqlite::Connection;

pub use crate::core::archive::storage::{add_call_document, document_brief, document_pages, emergency_delete_document};
/// Whether, and how long ago, the institution confirmed its data, and the version a new project is tied to. The
/// projects never read the tables of the profile themselves.
pub use crate::core::profile::storage::{confirmed_days_ago as profile_confirmed_days_ago, latest_confirmed as confirmed_profile};
pub use crate::core::error::ServiceError;
pub use crate::core::profile::domain::ProfileInput;

/// The sheet of the institution the AI reads: everything «Mi institución» and its modules say, as aggregates.
pub fn institution_sheet(conn: &Connection) -> Result<String, CoreError> {
    crate::core::ai_sheet::profile_context(conn)
}

/// Free texts through the scanner, with the words of the institution. `Ok(Err(report))` means quarantine.
pub fn guard_texts(conn: &Connection, entity: &str, fields: &[(String, String)], decision: Option<Decision>) -> Result<Result<Vec<String>, QuarantineReport>, CoreError> {
    crate::core::screen::guard_texts(conn, entity, fields, decision)
}

/// What the scanner must not flag: the name and the contact of the institution.
pub fn scanner_config(conn: &Connection) -> Result<ScannerConfig, CoreError> {
    Ok(crate::core::profile::storage::scanner_config(conn)?)
}

/// The profile as it is now (draft or confirmed), for the guide in Word: who the institution is.
pub fn institution(conn: &Connection) -> Result<Option<ProfileInput>, CoreError> {
    Ok(crate::core::profile::storage::load_current(conn)?.map(|p| p.input))
}

/// How many people the institution serves now, if it is known.
pub fn people_served(conn: &Connection) -> Result<Option<u32>, CoreError> {
    Ok(crate::core::profile::storage::load_current(conn)?.map(|p| p.input.totals(p.as_of_year).population.max(0) as u32))
}

/// The facilities in plain Spanish, one line each (the building, then every group of spaces and of equipment).
pub fn facility_lines(conn: &Connection) -> Result<Vec<String>, CoreError> {
    use crate::core::insights::facility_text::{equipment_line, site_lines, space_line};
    Ok(crate::modules::facilities::api::summaries(conn)?
        .iter()
        .flat_map(|s| site_lines(&s.site).into_iter().chain(s.spaces.iter().map(space_line)).chain(s.equipment.iter().map(equipment_line)))
        .collect())
}
