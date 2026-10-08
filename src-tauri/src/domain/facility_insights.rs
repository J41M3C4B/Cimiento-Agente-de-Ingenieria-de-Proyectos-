//! The board of the facilities (ADR-030): the indicators of the module crossed with the people served and the
//! capacity, and findings in one sentence each, made by fixed rules. A finding carries its numbers and the names of
//! the groups; the screen and the sheet of the AI put the words. The facilities carry no personal datum; a count of
//! people with an attribute (in a wheelchair, in bed) reaches the AI only for `MIN_GROUP` or more.
//!
//! No rule judges against a norm (NOM-031-SSA3, NOM-032-SSA3) yet: the ratios are shown, not graded, until their
//! thresholds are checked in the official text.

use crate::modules::care::api::MIN_GROUP;
use crate::domain::facility_text::group_ref;
use crate::domain::insights::{insight, Insight};
use crate::modules::facilities::domain::aggregate::Indicators;
use crate::modules::facilities::domain::site::SiteData;
use serde::Serialize;

/// Years left of a «comodato» or a lease that make it a finding.
pub const TENURE_WARNING_YEARS: i64 = 5;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FacilityBoard {
    pub indicators: Indicators,
    pub served: i64,
    pub capacity: Option<i64>,
    /// Built square meters per person served.
    pub built_m2_per_person: Option<i64>,
    /// People served per bathroom, with one decimal.
    pub people_per_bathroom: Option<f64>,
    pub insights: Vec<Insight>,
}

/// What the board needs from the rest of the institution.
pub struct Context<'a> {
    pub served: i64,
    pub capacity: Option<i64>,
    /// People in a wheelchair or in bed.
    pub limited_mobility: i64,
    pub elderly_home: bool,
    pub year: i64,
    /// The main site (its tenure, services and safety).
    pub site: Option<&'a SiteData>,
}

pub fn board(i: Indicators, cx: &Context) -> FacilityBoard {
    let served = cx.served;
    let built_m2_per_person = i.built_m2.filter(|_| served > 0).map(|m| (m + served / 2) / served);
    let people_per_bathroom = (served > 0 && i.bathrooms > 0).then(|| (served as f64 * 10.0 / i.bathrooms as f64).round() / 10.0);
    let mut out = Vec::new();

    if !i.broken.is_empty() {
        let bad: i64 = i.broken.iter().map(|g| g.bad).sum();
        out.push(insight("broken_spaces", &[("spaces", bad), ("groups", i.broken.len() as i64)], i.broken.iter().map(|g| group_ref(g, true)).collect(), true));
    }
    if !i.broken_equipment.is_empty() {
        let bad: i64 = i.broken_equipment.iter().map(|g| g.bad).sum();
        out.push(insight("broken_equipment", &[("items", bad)], i.broken_equipment.iter().map(|g| group_ref(g, false)).collect(), true));
    }
    if i.structural > 0 {
        out.push(insight("structural", &[("groups", i.structural)], vec![], true));
    }
    if i.only_stairs && i.spaces_upstairs > 0 && (cx.limited_mobility > 0 || cx.elderly_home) {
        let people = cx.limited_mobility;
        out.push(insight("only_stairs", &[("people", people), ("spaces", i.spaces_upstairs)], vec![], people == 0 || people >= MIN_GROUP));
    }
    if i.bathrooms_without_bars > 0 && (cx.elderly_home || cx.limited_mobility > 0) {
        out.push(insight("bathrooms_without_bars", &[("without", i.bathrooms_without_bars), ("bathrooms", i.bathrooms)], vec![], true));
    }
    if let Some(beds) = i.beds {
        let short_now = served > beds;
        let short_capacity = cx.capacity.is_some_and(|c| c > beds);
        if short_now || short_capacity {
            out.push(insight("beds_short", &[("beds", beds), ("served", served), ("capacity", cx.capacity.unwrap_or(-1))], vec![], true));
        }
    }
    if let Some(site) = cx.site {
        match site.tenure.as_deref() {
            Some("rent" | "borrowed" | "other") => out.push(insight("tenure_weak", &[], vec![site.tenure.clone().unwrap_or_default()], true)),
            Some("loan") => {
                if let Some(until) = site.tenure_until.filter(|u| u - cx.year < TENURE_WARNING_YEARS) {
                    out.push(insight("tenure_ending", &[("until", until), ("years", (until - cx.year).max(0))], vec![], true));
                }
            }
            _ => {}
        }
        if site.tenure_documented == Some(false) {
            out.push(insight("tenure_undocumented", &[], vec![], true));
        }
        let mut civil = Vec::new();
        if matches!(site.internal_program.as_deref(), Some("no" | "in_progress")) {
            civil.push(format!("internal_program_{}", site.internal_program.as_deref().unwrap_or_default()));
        }
        if site.civil_protection_opinion == Some(false) {
            civil.push("opinion".to_string());
        }
        if !civil.is_empty() {
            out.push(insight("civil_protection_gap", &[], civil, true));
        }
        let mut fire = Vec::new();
        if site.extinguishers == Some(0) {
            fire.push("no_extinguishers".to_string());
        } else if site.extinguishers_current == Some(false) {
            fire.push("extinguishers_expired".to_string());
        }
        if site.smoke_detectors == Some(0) {
            fire.push("no_smoke_detectors".to_string());
        }
        if !fire.is_empty() {
            out.push(insight("fire_safety_gap", &[], fire, true));
        }
        if matches!(site.water_shortage.as_deref(), Some("sometimes" | "often")) {
            out.push(insight("water_shortage", &[("often", (site.water_shortage.as_deref() == Some("often")) as i64), ("liters", site.water_storage_liters.unwrap_or(-1))], vec![], true));
        }
        if matches!(site.power_outages.as_deref(), Some("sometimes" | "often")) && i.generator_works != Some(true) {
            out.push(insight("power_without_backup", &[("often", (site.power_outages.as_deref() == Some("often")) as i64), ("hospital_beds", i.hospital_beds.unwrap_or(0))], vec![], true));
        }
    }
    if i.spaces_unchecked > 0 {
        out.push(insight("spaces_unchecked", &[("spaces", i.spaces_unchecked)], vec![], false));
    }
    FacilityBoard { indicators: i, served, capacity: cx.capacity, built_m2_per_person, people_per_bathroom, insights: out }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::facilities::domain::aggregate::{indicators, SiteSummary};
    use crate::modules::facilities::domain::group::{EquipmentData, SpaceData, States};

    fn house() -> SiteSummary {
        let st = |good, poor, unusable| States { good, poor, unusable, ..Default::default() };
        SiteSummary {
            site: SiteData {
                name: "Casa".into(),
                built_m2: Some(650),
                floors: Some(2),
                floor_access: vec!["none".into()],
                tenure: Some("loan".into()),
                tenure_until: Some(2029),
                tenure_documented: Some(true),
                extinguishers: Some(4),
                extinguishers_current: Some(false),
                smoke_detectors: Some(0),
                internal_program: Some("no".into()),
                civil_protection_opinion: Some(false),
                water_shortage: Some("often".into()),
                water_storage_liters: Some(5_000),
                power_outages: Some("sometimes".into()),
                ..Default::default()
            },
            spaces: vec![
                SpaceData { kind: "bathroom".into(), floor: 1, count: 4, states: st(3, 1, 0), grab_bars: Some(false), problems: vec!["cracks".into()], ..Default::default() },
                SpaceData { kind: "bathroom".into(), floor: 0, count: 1, states: st(1, 0, 0), grab_bars: Some(true), ..Default::default() },
                SpaceData { kind: "bedroom".into(), floor: 1, count: 6, states: st(4, 0, 0), beds: Some(18), hospital_beds: Some(2), ..Default::default() },
            ],
            equipment: vec![EquipmentData { kind: "generator".into(), count: 1, states: st(0, 0, 1), ..Default::default() }],
        }
    }

    #[test]
    fn findings_cross_the_building_with_the_people_and_the_capacity() {
        let h = house();
        let cx = Context { served: 22, capacity: Some(25), limited_mobility: 5, elderly_home: true, year: 2026, site: Some(&h.site) };
        let b = board(indicators(std::slice::from_ref(&h)), &cx);
        assert_eq!((b.built_m2_per_person, b.people_per_bathroom), (Some(30), Some(4.4)));
        let codes: Vec<_> = b.insights.iter().map(|x| (x.code, x.for_ai)).collect();
        assert_eq!(codes, vec![
            ("broken_spaces", true),
            ("broken_equipment", true),
            ("structural", true),
            ("only_stairs", true),
            ("bathrooms_without_bars", true),
            ("beds_short", true),
            ("tenure_ending", true),
            ("civil_protection_gap", true),
            ("fire_safety_gap", true),
            ("water_shortage", true),
            ("power_without_backup", true),
            ("spaces_unchecked", false),
        ]);
        assert_eq!(b.insights[0].items, vec!["1 de 4 baños (primer piso)"]);
        assert_eq!(b.insights[1].items, vec!["1 de 1 planta de luz de emergencia"]);
        assert_eq!((b.insights[3].values["people"], b.insights[3].values["spaces"]), (5, 10));
        assert_eq!((b.insights[5].values["beds"], b.insights[5].values["served"]), (18, 22));
        assert_eq!((b.insights[6].values["until"], b.insights[6].values["years"]), (2029, 3));
        assert_eq!(b.insights[7].items, vec!["internal_program_no", "opinion"]);
        assert_eq!(b.insights[8].items, vec!["extinguishers_expired", "no_smoke_detectors"]);
    }

    #[test]
    fn few_people_with_limited_mobility_stay_out_of_the_ai() {
        let h = house();
        let cx = Context { served: 22, capacity: None, limited_mobility: 2, elderly_home: false, year: 2026, site: None };
        let b = board(indicators(std::slice::from_ref(&h)), &cx);
        let stairs = b.insights.iter().find(|x| x.code == "only_stairs").unwrap();
        assert!(!stairs.for_ai, "2 people in a wheelchair are too few to tell");
        assert!(b.insights.iter().all(|x| !x.code.starts_with("tenure")), "without the site there is nothing to say about it");
    }

    #[test]
    fn without_data_there_are_no_made_up_findings() {
        let cx = Context { served: 0, capacity: Some(20), limited_mobility: 0, elderly_home: true, year: 2026, site: None };
        let b = board(Indicators::default(), &cx);
        assert!(b.insights.is_empty());
        assert_eq!((b.built_m2_per_person, b.people_per_bathroom), (None, None));
    }
}
