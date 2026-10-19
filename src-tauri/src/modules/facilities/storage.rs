//! Persistence of the facilities module: its own `fac_*` tables and nothing else.

use super::domain::group::{EquipmentData, SpaceData, States};
use super::domain::site::{SiteData, DEFAULT_NAME};
use super::FacilitiesError;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;
use ulid::Ulid;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

pub fn current_year(conn: &Connection) -> Result<i64, FacilitiesError> {
    Ok(conn.query_row("SELECT CAST(strftime('%Y','now') AS INTEGER)", [], |r| r.get(0))?)
}

fn list(v: &[String]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "[]".into())
}

fn json_list(r: &Row, i: usize) -> rusqlite::Result<Vec<String>> {
    Ok(serde_json::from_str(&r.get::<_, String>(i)?).unwrap_or_default())
}

fn bool_of(v: Option<i64>) -> Option<bool> {
    v.map(|x| x != 0)
}

fn int_of(v: Option<bool>) -> Option<i64> {
    v.map(|x| x as i64)
}

// ------------------------------------------------------------------ sites

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredSite {
    pub id: String,
    pub data: SiteData,
    pub updated_at: String,
}

const SITE_COLUMNS: &str = "id, name, land_m2, built_m2, floors, floor_access, built_year, tenure, tenure_until, tenure_documented,
    water_sources, water_shortage, water_storage_liters, power_outages, gas, drainage, internet, extinguishers, extinguishers_current,
    smoke_detectors, marked_exits, emergency_lights, first_aid_kit, internal_program, civil_protection_opinion, opinion_year,
    drills_per_year, notes, updated_at";

fn site_of(r: &Row) -> rusqlite::Result<StoredSite> {
    let data = SiteData {
        name: r.get(1)?,
        land_m2: r.get(2)?,
        built_m2: r.get(3)?,
        floors: r.get(4)?,
        floor_access: json_list(r, 5)?,
        built_year: r.get(6)?,
        tenure: r.get(7)?,
        tenure_until: r.get(8)?,
        tenure_documented: bool_of(r.get(9)?),
        water_sources: json_list(r, 10)?,
        water_shortage: r.get(11)?,
        water_storage_liters: r.get(12)?,
        power_outages: r.get(13)?,
        gas: r.get(14)?,
        drainage: r.get(15)?,
        internet: bool_of(r.get(16)?),
        extinguishers: r.get(17)?,
        extinguishers_current: bool_of(r.get(18)?),
        smoke_detectors: r.get(19)?,
        marked_exits: bool_of(r.get(20)?),
        emergency_lights: bool_of(r.get(21)?),
        first_aid_kit: bool_of(r.get(22)?),
        internal_program: r.get(23)?,
        civil_protection_opinion: bool_of(r.get(24)?),
        opinion_year: r.get(25)?,
        drills_per_year: r.get(26)?,
        notes: r.get(27)?,
    };
    Ok(StoredSite { id: r.get(0)?, data, updated_at: r.get(28)? })
}

pub fn sites(conn: &Connection) -> Result<Vec<StoredSite>, FacilitiesError> {
    Ok(conn.prepare(&format!("SELECT {SITE_COLUMNS} FROM fac_site ORDER BY created_at, rowid"))?.query_map([], site_of)?.collect::<Result<Vec<_>, _>>()?)
}

pub fn site(conn: &Connection, id: &str) -> Result<Option<StoredSite>, FacilitiesError> {
    Ok(conn.query_row(&format!("SELECT {SITE_COLUMNS} FROM fac_site WHERE id = ?1"), [id], site_of).optional()?)
}

/// The first site, made (empty, with the default name) the first time it is needed.
pub fn main_site_id(conn: &Connection) -> Result<String, FacilitiesError> {
    if let Some(id) = conn.query_row("SELECT id FROM fac_site ORDER BY created_at, rowid LIMIT 1", [], |r| r.get(0)).optional()? {
        return Ok(id);
    }
    let id = new_id("site");
    conn.execute(&format!("INSERT INTO fac_site (id, name, created_at, updated_at) VALUES (?1, ?2, {NOW}, {NOW})"), params![id, DEFAULT_NAME])?;
    Ok(id)
}

pub fn save_site(conn: &Connection, id: &str, d: &SiteData) -> Result<bool, FacilitiesError> {
    let n = conn.execute(
        &format!(
            "UPDATE fac_site SET name=?2, land_m2=?3, built_m2=?4, floors=?5, floor_access=?6, built_year=?7, tenure=?8, tenure_until=?9,
                tenure_documented=?10, water_sources=?11, water_shortage=?12, water_storage_liters=?13, power_outages=?14, gas=?15,
                drainage=?16, internet=?17, extinguishers=?18, extinguishers_current=?19, smoke_detectors=?20, marked_exits=?21,
                emergency_lights=?22, first_aid_kit=?23, internal_program=?24, civil_protection_opinion=?25, opinion_year=?26,
                drills_per_year=?27, notes=?28, updated_at={NOW}
             WHERE id=?1"
        ),
        params![
            id, d.name, d.land_m2, d.built_m2, d.floors, list(&d.floor_access), d.built_year, d.tenure, d.tenure_until,
            int_of(d.tenure_documented), list(&d.water_sources), d.water_shortage, d.water_storage_liters, d.power_outages, d.gas,
            d.drainage, int_of(d.internet), d.extinguishers, int_of(d.extinguishers_current), d.smoke_detectors, int_of(d.marked_exits),
            int_of(d.emergency_lights), int_of(d.first_aid_kit), d.internal_program, int_of(d.civil_protection_opinion), d.opinion_year,
            d.drills_per_year, d.notes
        ],
    )?;
    Ok(n > 0)
}

// ------------------------------------------------------------------ spaces

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredSpace {
    pub id: String,
    pub site_id: String,
    #[serde(flatten)]
    pub data: SpaceData,
}

const SPACE_COLUMNS: &str = "id, site_id, kind, label, floor, count, good, fair, poor, unusable, problems, accessible, beds, hospital_beds,
    grab_bars, accessible_shower, notes";

fn space_of(r: &Row) -> rusqlite::Result<StoredSpace> {
    Ok(StoredSpace {
        id: r.get(0)?,
        site_id: r.get(1)?,
        data: SpaceData {
            kind: r.get(2)?,
            label: r.get(3)?,
            floor: r.get(4)?,
            count: r.get(5)?,
            states: States { good: r.get(6)?, fair: r.get(7)?, poor: r.get(8)?, unusable: r.get(9)? },
            problems: json_list(r, 10)?,
            accessible: bool_of(r.get(11)?),
            beds: r.get(12)?,
            hospital_beds: r.get(13)?,
            grab_bars: bool_of(r.get(14)?),
            accessible_shower: bool_of(r.get(15)?),
            notes: r.get(16)?,
        },
    })
}

/// The spaces of a site, by floor and then as they were added.
pub fn spaces(conn: &Connection, site_id: &str) -> Result<Vec<StoredSpace>, FacilitiesError> {
    Ok(conn
        .prepare(&format!("SELECT {SPACE_COLUMNS} FROM fac_space WHERE site_id = ?1 ORDER BY floor, created_at, rowid"))?
        .query_map([site_id], space_of)?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn space(conn: &Connection, id: &str) -> Result<Option<StoredSpace>, FacilitiesError> {
    Ok(conn.query_row(&format!("SELECT {SPACE_COLUMNS} FROM fac_space WHERE id = ?1"), [id], space_of).optional()?)
}

/// Inserts (`id` = `None`) or updates a group of spaces. `origin` and `source_ref` are written only on insert.
pub fn save_space(conn: &Connection, id: Option<&str>, site_id: &str, d: &SpaceData, origin: &str, source_ref: Option<&str>) -> Result<Option<String>, FacilitiesError> {
    let s = &d.states;
    match id {
        Some(sid) => {
            let n = conn.execute(
                &format!(
                    "UPDATE fac_space SET kind=?2, label=?3, floor=?4, count=?5, good=?6, fair=?7, poor=?8, unusable=?9, problems=?10,
                        accessible=?11, beds=?12, hospital_beds=?13, grab_bars=?14, accessible_shower=?15, notes=?16, updated_at={NOW}
                     WHERE id=?1"
                ),
                params![
                    sid, d.kind, d.label, d.floor, d.count, s.good, s.fair, s.poor, s.unusable, list(&d.problems), int_of(d.accessible), d.beds,
                    d.hospital_beds, int_of(d.grab_bars), int_of(d.accessible_shower), d.notes
                ],
            )?;
            Ok((n > 0).then(|| sid.to_string()))
        }
        None => {
            let sid = new_id("spc");
            conn.execute(
                &format!(
                    "INSERT INTO fac_space (id, site_id, kind, label, floor, count, good, fair, poor, unusable, problems, accessible, beds,
                        hospital_beds, grab_bars, accessible_shower, notes, origin, source_ref, created_at, updated_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,{NOW},{NOW})"
                ),
                params![
                    sid, site_id, d.kind, d.label, d.floor, d.count, s.good, s.fair, s.poor, s.unusable, list(&d.problems), int_of(d.accessible),
                    d.beds, d.hospital_beds, int_of(d.grab_bars), int_of(d.accessible_shower), d.notes, origin, source_ref
                ],
            )?;
            Ok(Some(sid))
        }
    }
}

pub fn delete_space(conn: &Connection, id: &str) -> Result<bool, FacilitiesError> {
    Ok(conn.execute("DELETE FROM fac_space WHERE id=?1", [id])? > 0)
}

// ------------------------------------------------------------------ equipment

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredEquipment {
    pub id: String,
    pub site_id: String,
    #[serde(flatten)]
    pub data: EquipmentData,
}

const EQUIPMENT_COLUMNS: &str = "id, site_id, kind, label, count, good, fair, poor, unusable, notes";

fn equipment_of(r: &Row) -> rusqlite::Result<StoredEquipment> {
    Ok(StoredEquipment {
        id: r.get(0)?,
        site_id: r.get(1)?,
        data: EquipmentData {
            kind: r.get(2)?,
            label: r.get(3)?,
            count: r.get(4)?,
            states: States { good: r.get(5)?, fair: r.get(6)?, poor: r.get(7)?, unusable: r.get(8)? },
            notes: r.get(9)?,
        },
    })
}

pub fn equipment(conn: &Connection, site_id: &str) -> Result<Vec<StoredEquipment>, FacilitiesError> {
    Ok(conn
        .prepare(&format!("SELECT {EQUIPMENT_COLUMNS} FROM fac_equipment WHERE site_id = ?1 ORDER BY created_at, rowid"))?
        .query_map([site_id], equipment_of)?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn equipment_item(conn: &Connection, id: &str) -> Result<Option<StoredEquipment>, FacilitiesError> {
    Ok(conn.query_row(&format!("SELECT {EQUIPMENT_COLUMNS} FROM fac_equipment WHERE id = ?1"), [id], equipment_of).optional()?)
}

pub fn save_equipment(conn: &Connection, id: Option<&str>, site_id: &str, d: &EquipmentData) -> Result<Option<String>, FacilitiesError> {
    let s = &d.states;
    match id {
        Some(eid) => {
            let n = conn.execute(
                &format!("UPDATE fac_equipment SET kind=?2, label=?3, count=?4, good=?5, fair=?6, poor=?7, unusable=?8, notes=?9, updated_at={NOW} WHERE id=?1"),
                params![eid, d.kind, d.label, d.count, s.good, s.fair, s.poor, s.unusable, d.notes],
            )?;
            Ok((n > 0).then(|| eid.to_string()))
        }
        None => {
            let eid = new_id("eqp");
            conn.execute(
                &format!(
                    "INSERT INTO fac_equipment (id, site_id, kind, label, count, good, fair, poor, unusable, notes, created_at, updated_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,{NOW},{NOW})"
                ),
                params![eid, site_id, d.kind, d.label, d.count, s.good, s.fair, s.poor, s.unusable, d.notes],
            )?;
            Ok(Some(eid))
        }
    }
}

pub fn delete_equipment(conn: &Connection, id: &str) -> Result<bool, FacilitiesError> {
    Ok(conn.execute("DELETE FROM fac_equipment WHERE id=?1", [id])? > 0)
}
