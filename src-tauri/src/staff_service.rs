//! The staff screens (ADR-027): the use cases of the staff module, plus what the app adds around them. After every
//! change the profile gets the new anonymous lines (so the payroll and the sheet of the AI follow), and the words
//! of a position go through the scanner before they are saved, because they may reach the AI.

use crate::audit::{self, AuditKind};
use crate::domain::profile::ProfileTotals;
use crate::hr::domain::person::{Issue, PersonData};
use crate::hr::domain::position::PositionInput;
use crate::hr::service::{self as hr, ModalityInfo, Overview, PersonView, SaveOutcome};
use crate::hr::service::CustomField;
use crate::profile_sync::{sync_profile, totals};
use crate::scanner::guard::{counts_json, guard_fields, Decision, GuardOutcome, QuarantineReport};
use crate::service::{scanner_for, ProfileView, ServiceError};
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StaffOverview {
    #[serde(flatten)]
    pub hr: Overview,
    /// The sums of the profile with the staff as it is now (payroll, benefits, contributions).
    pub totals: ProfileTotals,
}

#[derive(Debug, Serialize)]
pub struct StaffChange {
    pub overview: StaffOverview,
    /// The profile with the new lines, if there is one already.
    pub profile: Option<ProfileView>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PersonOutcome {
    Saved { person: PersonView, change: StaffChange },
    Invalid { issues: Vec<Issue> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PositionOutcome {
    Saved { change: StaffChange },
    Invalid { issues: Vec<Issue> },
    Quarantine { report: QuarantineReport },
}

/// The first time, the catalog of positions starts from the kind of institution.
fn prepare(conn: &Connection) -> Result<(), ServiceError> {
    Ok(crate::hr::api::prepare(conn, crate::core::institution::hr_flavor(conn))?)
}

pub fn overview(conn: &Connection) -> Result<StaffOverview, ServiceError> {
    prepare(conn)?;
    Ok(StaffOverview { hr: hr::overview(conn)?, totals: totals(conn)? })
}

pub(crate) fn change(conn: &mut Connection) -> Result<StaffChange, ServiceError> {
    let profile = sync_profile(conn)?;
    Ok(StaffChange { overview: overview(conn)?, profile })
}

pub fn person(conn: &Connection, id: &str) -> Result<PersonView, ServiceError> {
    hr::get_person(conn, id)?.ok_or(ServiceError::NotFound)
}

pub fn save_person(conn: &mut Connection, id: Option<&str>, data: PersonData) -> Result<PersonOutcome, ServiceError> {
    prepare(conn)?;
    Ok(match hr::save_person(conn, id, data)? {
        SaveOutcome::Invalid { issues } => PersonOutcome::Invalid { issues },
        SaveOutcome::Saved { person } => {
            // a person who left cannot use the app any more (ADR-028)
            crate::access_service::after_staff_saved(conn, &person.id, &person.data.status)?;
            PersonOutcome::Saved { person, change: change(conn)? }
        }
    })
}

pub fn delete_person(conn: &mut Connection, id: &str) -> Result<StaffChange, ServiceError> {
    hr::delete_person(conn, id)?;
    change(conn)
}

pub fn reveal(conn: &Connection, id: &str, field: &str) -> Result<String, ServiceError> {
    hr::reveal(conn, id, field)?.ok_or(ServiceError::NotFound)
}

/// Saves a position after the scanner looks at its title and duties (they may reach the AI).
pub fn save_position(conn: &mut Connection, id: Option<&str>, mut input: PositionInput, decision: Option<Decision>) -> Result<PositionOutcome, ServiceError> {
    prepare(conn)?;
    let fields = vec![("title".to_string(), input.title.clone()), ("duties".to_string(), input.duties.clone().unwrap_or_default())];
    let scanner = scanner_for(conn)?;
    let mut event = None;
    match guard_fields(&scanner, &fields, decision)? {
        GuardOutcome::Clean => {}
        GuardOutcome::Quarantine(report) => return Ok(PositionOutcome::Quarantine { report }),
        GuardOutcome::Redacted { mut texts, counts } => {
            let duties = texts.pop().unwrap_or_default();
            input.title = texts.pop().unwrap_or_default();
            input.duties = Some(duties).filter(|d| !d.trim().is_empty());
            event = Some((AuditKind::ScannerQuarantine, counts_json(&counts, "redacted")));
        }
        GuardOutcome::Overridden { counts } => event = Some((AuditKind::ScannerOverride, counts_json(&counts, "not_personal"))),
    }
    match hr::save_position(conn, id, &input)? {
        Err(issues) => Ok(PositionOutcome::Invalid { issues }),
        Ok(_) => {
            if let Some((kind, details)) = event {
                audit::record(conn, kind, Some("hr_position"), id, details)?;
            }
            Ok(PositionOutcome::Saved { change: change(conn)? })
        }
    }
}

pub fn set_position_active(conn: &mut Connection, id: &str, active: bool) -> Result<StaffChange, ServiceError> {
    hr::set_position_active(conn, id, active)?;
    change(conn)
}

pub fn create_modality(conn: &Connection, title: &str, behaves_as: &str) -> Result<Vec<ModalityInfo>, ServiceError> {
    Ok(hr::create_modality(conn, title, behaves_as)?)
}

pub fn save_field(conn: &Connection, key: Option<&str>, title: &str, kind: &str, options: &[String]) -> Result<Vec<CustomField>, ServiceError> {
    Ok(hr::save_custom_field(conn, key, title, kind, options)?)
}

pub fn delete_field(conn: &Connection, key: &str) -> Result<Vec<CustomField>, ServiceError> {
    Ok(hr::delete_custom_field(conn, key)?)
}

#[cfg(test)]
#[path = "staff_service_tests.rs"]
mod tests;
