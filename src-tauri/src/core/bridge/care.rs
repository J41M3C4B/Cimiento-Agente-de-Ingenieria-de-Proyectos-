//! The screens of the people served (ADR-029): the use cases of their module, plus what the app adds around them.
//! After every change the profile gets the new anonymous lines (people served and stay fees), and the board crosses
//! the indicators of the module with the rest of «Mi institución».

use crate::modules::care::domain::person::{BeneficiaryData, Issue};
use crate::modules::care::domain::waitlist::WaitlistInput;
use crate::modules::care::service::{self as care, Overview, PersonView, SaveOutcome};
use crate::modules::care::service::{CustomField, Group, WaitlistRow};
use crate::core::insights::people::{self as insights, Board, Context};
use crate::core::profile::domain::ProfileTotals;
use crate::core::institution::care_flavor;
use crate::core::profile::sync::{sync_profile, totals};
use crate::core::profile::service::ProfileView;
use crate::core::error::ServiceError;
use crate::core::profile::storage as profile_store;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CareOverview {
    #[serde(flatten)]
    pub care: Overview,
    pub totals: ProfileTotals,
    pub board: Board,
    /// `elderly_home`, `children_home` or `other`: the screen shows the data of that kind.
    pub flavor: &'static str,
}

#[derive(Debug, Serialize)]
pub struct CareChange {
    pub overview: CareOverview,
    pub profile: Option<ProfileView>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PersonOutcome {
    Saved { person: PersonView, change: CareChange },
    Invalid { issues: Vec<Issue> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WaitlistOutcome {
    Saved { waitlist: Vec<WaitlistRow>, change: CareChange },
    Invalid { issues: Vec<Issue> },
}

/// The board: the indicators crossed with capacity, spaces, money and staff.
pub fn board(conn: &Connection) -> Result<Board, ServiceError> {
    use crate::modules::care::domain::catalog::Flavor;
    let flavor = care_flavor(conn);
    let indicators = crate::modules::care::api::indicators(conn, flavor)?;
    let waiting = crate::modules::care::api::waiting(conn)?;
    let profile = profile_store::load_current(conn)?;
    let capacity = profile.as_ref().and_then(|p| p.input.capacity_total);
    let expenses = crate::core::bridge::finance::finances(conn)?.expenses_annual_mxn;
    // the spaces live in their module (ADR-030)
    let blocked = crate::modules::facilities::api::indicators(conn)?.not_accessible;
    let staff = crate::modules::hr::api::ai_summary(conn)?;
    let carers = staff.positions.iter().filter(|p| matches!(p.area.as_deref(), Some("care" | "health"))).map(|p| p.people).sum();
    let cx = Context {
        capacity,
        not_accessible: blocked.iter().map(|g| format!("{} {} ({})", g.count, crate::core::insights::facility_text::group_name(&g.kind, g.label.as_deref(), g.count, true), crate::core::insights::facility_text::floor(g.floor))).collect(),
        not_accessible_spaces: blocked.iter().map(|g| g.count).sum(),

        expenses_annual: expenses,
        carers,
        elderly_home: flavor == Flavor::ElderlyHome,
        children_home: flavor == Flavor::ChildrenHome,
    };
    Ok(insights::board(indicators, waiting, &cx))
}

pub fn overview(conn: &Connection) -> Result<CareOverview, ServiceError> {
    use crate::modules::care::domain::catalog::Flavor;
    let flavor = care_flavor(conn);
    Ok(CareOverview {
        care: care::overview(conn, flavor)?,
        totals: totals(conn)?,
        board: board(conn)?,
        flavor: match flavor {
            Flavor::ElderlyHome => "elderly_home",
            Flavor::ChildrenHome => "children_home",
            Flavor::Other => "other",
        },
    })
}

pub(crate) fn change(conn: &mut Connection) -> Result<CareChange, ServiceError> {
    let profile = sync_profile(conn)?;
    Ok(CareChange { overview: overview(conn)?, profile })
}

pub fn person(conn: &Connection, id: &str) -> Result<PersonView, ServiceError> {
    care::get_person(conn, care_flavor(conn), id)?.ok_or(ServiceError::NotFound)
}

pub fn save_person(conn: &mut Connection, id: Option<&str>, data: BeneficiaryData) -> Result<PersonOutcome, ServiceError> {
    let flavor = care_flavor(conn);
    Ok(match care::save_person(conn, flavor, id, data)? {
        SaveOutcome::Invalid { issues } => PersonOutcome::Invalid { issues },
        SaveOutcome::Saved { person } => PersonOutcome::Saved { person, change: change(conn)? },
    })
}

pub fn delete_person(conn: &mut Connection, id: &str) -> Result<CareChange, ServiceError> {
    care::delete_person(conn, id)?;
    change(conn)
}

pub fn reveal_curp(conn: &Connection, id: &str) -> Result<String, ServiceError> {
    care::reveal_curp(conn, id)?.ok_or(ServiceError::NotFound)
}

pub fn save_group(conn: &mut Connection, id: Option<&str>, title: &str, active: bool, decision: Option<crate::scanner::guard::Decision>) -> Result<Result<Vec<Group>, crate::scanner::guard::QuarantineReport>, ServiceError> {
    use crate::scanner::guard::{guard_fields, GuardOutcome};
    // a group's title may reach the AI (it names the people of the group): it goes through the scanner
    let scanner = crate::core::screen::scanner_for(conn)?;
    let title = match guard_fields(&scanner, &[("title".to_string(), title.to_string())], decision)? {
        GuardOutcome::Quarantine(report) => return Ok(Err(report)),
        GuardOutcome::Redacted { mut texts, .. } => texts.pop().unwrap_or_default(),
        GuardOutcome::Clean | GuardOutcome::Overridden { .. } => title.to_string(),
    };
    let groups = care::save_group(conn, id, &title, active)?;
    sync_profile(conn)?;
    Ok(Ok(groups))
}

pub fn save_field(conn: &Connection, key: Option<&str>, title: &str, kind: &str, options: &[String]) -> Result<Vec<CustomField>, ServiceError> {
    Ok(care::save_custom_field(conn, key, title, kind, options)?)
}

pub fn delete_field(conn: &Connection, key: &str) -> Result<Vec<CustomField>, ServiceError> {
    Ok(care::delete_custom_field(conn, key)?)
}

pub fn save_waitlist(conn: &mut Connection, id: Option<&str>, input: &WaitlistInput) -> Result<WaitlistOutcome, ServiceError> {
    Ok(match care::save_waitlist(conn, id, input)? {
        Err(issues) => WaitlistOutcome::Invalid { issues },
        Ok(waitlist) => WaitlistOutcome::Saved { waitlist, change: change(conn)? },
    })
}

/// A request of the waiting list comes in: a record is made from it, to complete.
pub fn admit_waitlist(conn: &mut Connection, id: &str) -> Result<PersonOutcome, ServiceError> {
    let flavor = care_flavor(conn);
    let person = care::admit_waitlist(conn, flavor, id)?;
    Ok(PersonOutcome::Saved { person, change: change(conn)? })
}

pub fn delete_waitlist(conn: &mut Connection, id: &str) -> Result<CareChange, ServiceError> {
    care::delete_waitlist(conn, id)?;
    change(conn)
}

#[cfg(test)]
#[path = "care_tests.rs"]
mod tests;
