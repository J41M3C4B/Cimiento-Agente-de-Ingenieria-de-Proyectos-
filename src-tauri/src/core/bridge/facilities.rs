//! The screens of the facilities (ADR-030): the use cases of their module, plus what the app adds around them. The
//! free texts (names and notes) go through the scanner before they are saved, because they reach the AI; the board
//! crosses the indicators of the module with the people served and the capacity.

use crate::core::insights::facilities::{self as facility_insights, Context, FacilityBoard};
use crate::core::institution::{care_flavor, facilities_flavor as flavor};
use crate::modules::facilities::Flavor;
use crate::modules::facilities::domain::group::{EquipmentData, SpaceData};
use crate::modules::facilities::domain::site::SiteData;
use crate::modules::facilities::domain::Issue;
use crate::modules::facilities::service::{self as fac, Overview, SaveOutcome};
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::core::screen::guard_texts;
use crate::core::error::ServiceError;
use crate::core::profile::storage as profile_store;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FacilitiesOverview {
    #[serde(flatten)]
    pub facilities: Overview,
    pub board: FacilityBoard,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FacilitiesOutcome {
    Saved { overview: FacilitiesOverview },
    Invalid { issues: Vec<Issue> },
    Quarantine { report: QuarantineReport },
}

/// The board: the indicators crossed with the people served (how many, how many in a wheelchair or in bed) and the
/// capacity.
pub fn board(conn: &Connection) -> Result<FacilityBoard, ServiceError> {
    let indicators = crate::modules::facilities::api::indicators(conn)?;
    let people = crate::modules::care::api::indicators(conn, care_flavor(conn))?;
    let limited = people.mobility.iter().filter(|c| matches!(c.code.as_str(), "wheelchair" | "bedridden")).map(|c| c.count).sum();
    let capacity = profile_store::load_current(conn)?.and_then(|p| p.input.capacity_total);
    let sites = crate::modules::facilities::api::summaries(conn)?;
    let cx = Context {
        served: people.served,
        capacity,
        limited_mobility: limited,
        elderly_home: flavor(conn) == Flavor::ElderlyHome,
        year: profile_store::current_year(conn)?,
        site: sites.first().map(|s| &s.site),
    };
    Ok(facility_insights::board(indicators, &cx))
}

pub fn overview(conn: &Connection) -> Result<FacilitiesOverview, ServiceError> {
    Ok(FacilitiesOverview { facilities: fac::overview(conn, flavor(conn))?, board: board(conn)? })
}

fn done(conn: &Connection, out: SaveOutcome) -> Result<FacilitiesOutcome, ServiceError> {
    Ok(match out {
        SaveOutcome::Saved => FacilitiesOutcome::Saved { overview: overview(conn)? },
        SaveOutcome::Invalid { issues } => FacilitiesOutcome::Invalid { issues },
    })
}

/// Scans the texts that may reach the AI; `Err` is the quarantine to show. Covered texts come back in place.
fn scan(conn: &Connection, entity: &str, texts: &mut [(&str, &mut Option<String>)], decision: Option<Decision>) -> Result<Result<(), QuarantineReport>, ServiceError> {
    let fields: Vec<(String, String)> = texts.iter().filter_map(|(path, t)| t.as_ref().map(|t| (path.to_string(), t.clone()))).collect();
    if fields.is_empty() {
        return Ok(Ok(()));
    }
    match guard_texts(conn, entity, &fields, decision)? {
        Err(report) => Ok(Err(report)),
        Ok(clean) => {
            let mut it = clean.into_iter();
            for (_, t) in texts.iter_mut().filter(|(_, t)| t.is_some()) {
                **t = it.next();
            }
            Ok(Ok(()))
        }
    }
}

pub fn save_site(conn: &Connection, mut data: SiteData, decision: Option<Decision>) -> Result<FacilitiesOutcome, ServiceError> {
    let mut name = Some(data.name.clone());
    if let Err(report) = scan(conn, "fac_site", &mut [("name", &mut name), ("notes", &mut data.notes)], decision)? {
        return Ok(FacilitiesOutcome::Quarantine { report });
    }
    data.name = name.unwrap_or_default();
    let out = fac::save_site(conn, data)?;
    done(conn, out)
}

pub fn save_space(conn: &Connection, id: Option<&str>, mut data: SpaceData, decision: Option<Decision>) -> Result<FacilitiesOutcome, ServiceError> {
    if let Err(report) = scan(conn, "fac_space", &mut [("label", &mut data.label), ("notes", &mut data.notes)], decision)? {
        return Ok(FacilitiesOutcome::Quarantine { report });
    }
    let out = fac::save_space(conn, id, data)?;
    done(conn, out)
}

pub fn delete_space(conn: &Connection, id: &str) -> Result<FacilitiesOverview, ServiceError> {
    fac::delete_space(conn, id)?;
    overview(conn)
}

pub fn save_equipment(conn: &Connection, id: Option<&str>, mut data: EquipmentData, decision: Option<Decision>) -> Result<FacilitiesOutcome, ServiceError> {
    if let Err(report) = scan(conn, "fac_equipment", &mut [("label", &mut data.label), ("notes", &mut data.notes)], decision)? {
        return Ok(FacilitiesOutcome::Quarantine { report });
    }
    let out = fac::save_equipment(conn, id, data)?;
    done(conn, out)
}

pub fn delete_equipment(conn: &Connection, id: &str) -> Result<FacilitiesOverview, ServiceError> {
    fac::delete_equipment(conn, id)?;
    overview(conn)
}

/// Development only: replaces the facilities with the fictitious ones of an example (`fixtures/instalaciones-*.json`).
#[cfg(debug_assertions)]
pub fn seed_example(conn: &mut Connection, raw: &str) -> Result<(), ServiceError> {
    #[derive(serde::Deserialize)]
    struct Example {
        site: SiteData,
        spaces: Vec<SpaceData>,
        equipment: Vec<EquipmentData>,
    }
    let example: Example = serde_json::from_str(raw).map_err(|e| ServiceError::Internal(e.to_string()))?;
    let tx = conn.transaction()?;
    crate::modules::facilities::api::clear_for_example(&tx)?;
    let fail = |out: SaveOutcome| match out {
        SaveOutcome::Saved => Ok(()),
        SaveOutcome::Invalid { issues } => Err(ServiceError::Internal(format!("the example is not valid: {issues:?}"))),
    };
    fail(fac::save_site(&tx, example.site)?)?;
    for s in example.spaces {
        fail(fac::save_space(&tx, None, s)?)?;
    }
    for e in example.equipment {
        fail(fac::save_equipment(&tx, None, e)?)?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
#[path = "facilities_tests.rs"]
mod tests;
