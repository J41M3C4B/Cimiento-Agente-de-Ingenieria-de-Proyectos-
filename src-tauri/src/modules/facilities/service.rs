//! The use cases of the facilities module. The screen handles one site (the first one, made when something is first
//! saved); the tables allow several. The kind of institution comes from the app (`flavor`): the module does not read
//! the profile.

use super::domain::aggregate::{self, Indicators, SiteSummary};
use super::domain::catalog::{self, Flavor};
use super::domain::group::{EquipmentData, SpaceData};
use super::domain::site::{SiteData, DEFAULT_NAME};
use super::domain::Issue;
use super::storage::{self as store, StoredEquipment, StoredSpace};
use super::FacilitiesError;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SiteView {
    /// `None` until something is saved.
    pub id: Option<String>,
    pub data: SiteData,
    /// Heads-ups (nothing blocking is ever stored).
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpaceRow {
    #[serde(flatten)]
    pub space: StoredSpace,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquipmentRow {
    #[serde(flatten)]
    pub equipment: StoredEquipment,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub site: SiteView,
    pub spaces: Vec<SpaceRow>,
    pub equipment: Vec<EquipmentRow>,
    pub indicators: Indicators,
    /// The kinds in the order the screen offers them, by kind of institution.
    pub space_kinds: Vec<&'static str>,
    pub equipment_kinds: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SaveOutcome {
    Saved,
    Invalid { issues: Vec<Issue> },
}

fn heads_up(v: Vec<Issue>) -> Vec<Issue> {
    v.into_iter().filter(|i| !i.blocking).collect()
}

fn blocking(v: &[Issue]) -> Vec<Issue> {
    v.iter().filter(|i| i.blocking).cloned().collect()
}

/// Every site with its groups.
pub fn summaries(conn: &Connection) -> Result<Vec<SiteSummary>, FacilitiesError> {
    store::sites(conn)?
        .into_iter()
        .map(|s| {
            Ok(SiteSummary {
                spaces: store::spaces(conn, &s.id)?.into_iter().map(|x| x.data).collect(),
                equipment: store::equipment(conn, &s.id)?.into_iter().map(|x| x.data).collect(),
                site: s.data,
            })
        })
        .collect()
}

pub fn overview(conn: &Connection, flavor: Flavor) -> Result<Overview, FacilitiesError> {
    let year = store::current_year(conn)?;
    let main = store::sites(conn)?.into_iter().next();
    let (site, spaces, equipment) = match &main {
        Some(s) => {
            let floors = s.data.floors;
            let spaces = store::spaces(conn, &s.id)?.into_iter().map(|x| SpaceRow { issues: heads_up(x.data.validate(floors)), space: x }).collect();
            let equipment = store::equipment(conn, &s.id)?.into_iter().map(|x| EquipmentRow { issues: heads_up(x.data.validate()), equipment: x }).collect();
            (SiteView { id: Some(s.id.clone()), issues: heads_up(s.data.validate(year)), data: s.data.clone() }, spaces, equipment)
        }
        None => (SiteView { id: None, data: SiteData { name: DEFAULT_NAME.into(), ..Default::default() }, issues: Vec::new() }, Vec::new(), Vec::new()),
    };
    Ok(Overview {
        site,
        spaces,
        equipment,
        indicators: aggregate::indicators(&summaries(conn)?),
        space_kinds: catalog::suggested_spaces(flavor),
        equipment_kinds: catalog::suggested_equipment(flavor),
    })
}

/// Saves the building, its services and its safety. Nothing is saved while there is a blocking problem.
pub fn save_site(conn: &Connection, mut data: SiteData) -> Result<SaveOutcome, FacilitiesError> {
    let year = store::current_year(conn)?;
    data.tidy();
    let issues = blocking(&data.validate(year));
    if !issues.is_empty() {
        return Ok(SaveOutcome::Invalid { issues });
    }
    let id = store::main_site_id(conn)?;
    store::save_site(conn, &id, &data)?;
    Ok(SaveOutcome::Saved)
}

/// Saves a group of spaces (`id` = `None` adds one to the main site).
pub fn save_space(conn: &Connection, id: Option<&str>, mut data: SpaceData) -> Result<SaveOutcome, FacilitiesError> {
    let site_id = match id {
        Some(sid) => store::space(conn, sid)?.ok_or(FacilitiesError::NotFound)?.site_id,
        None => store::main_site_id(conn)?,
    };
    let floors = store::site(conn, &site_id)?.and_then(|s| s.data.floors);
    data.tidy();
    let issues = blocking(&data.validate(floors));
    if !issues.is_empty() {
        return Ok(SaveOutcome::Invalid { issues });
    }
    store::save_space(conn, id, &site_id, &data, "user", None)?.ok_or(FacilitiesError::NotFound)?;
    Ok(SaveOutcome::Saved)
}

pub fn delete_space(conn: &Connection, id: &str) -> Result<(), FacilitiesError> {
    if !store::delete_space(conn, id)? {
        return Err(FacilitiesError::NotFound);
    }
    Ok(())
}

/// Saves a group of equipment (`id` = `None` adds one to the main site).
pub fn save_equipment(conn: &Connection, id: Option<&str>, mut data: EquipmentData) -> Result<SaveOutcome, FacilitiesError> {
    let site_id = match id {
        Some(eid) => store::equipment_item(conn, eid)?.ok_or(FacilitiesError::NotFound)?.site_id,
        None => store::main_site_id(conn)?,
    };
    data.tidy();
    let issues = blocking(&data.validate());
    if !issues.is_empty() {
        return Ok(SaveOutcome::Invalid { issues });
    }
    store::save_equipment(conn, id, &site_id, &data)?.ok_or(FacilitiesError::NotFound)?;
    Ok(SaveOutcome::Saved)
}

pub fn delete_equipment(conn: &Connection, id: &str) -> Result<(), FacilitiesError> {
    if !store::delete_equipment(conn, id)? {
        return Err(FacilitiesError::NotFound);
    }
    Ok(())
}
