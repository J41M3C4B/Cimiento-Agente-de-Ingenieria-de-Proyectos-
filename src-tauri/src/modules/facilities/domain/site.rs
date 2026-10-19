//! A site (ADR-030): the building, its services, and its safety and civil protection. Everything is optional: the
//! person fills in what they know, and what is missing is named, never taken as zero.

use super::catalog;
use super::{code_ok, text, Issue};
use serde::{Deserialize, Serialize};

pub const DEFAULT_NAME: &str = "Inmueble principal";
const MAX_M2: i64 = 1_000_000;
const MAX_LITERS: i64 = 10_000_000;
const OLDEST_YEAR: i64 = 1500;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SiteData {
    pub name: String,
    // the building
    pub land_m2: Option<i64>,
    pub built_m2: Option<i64>,
    pub floors: Option<i64>,
    pub floor_access: Vec<String>,
    pub built_year: Option<i64>,
    pub tenure: Option<String>,
    pub tenure_until: Option<i64>,
    pub tenure_documented: Option<bool>,
    // services
    pub water_sources: Vec<String>,
    pub water_shortage: Option<String>,
    pub water_storage_liters: Option<i64>,
    pub power_outages: Option<String>,
    pub gas: Option<String>,
    pub drainage: Option<String>,
    pub internet: Option<bool>,
    // safety and civil protection
    pub extinguishers: Option<i64>,
    pub extinguishers_current: Option<bool>,
    pub smoke_detectors: Option<i64>,
    pub marked_exits: Option<bool>,
    pub emergency_lights: Option<bool>,
    pub first_aid_kit: Option<bool>,
    pub internal_program: Option<String>,
    pub civil_protection_opinion: Option<bool>,
    pub opinion_year: Option<i64>,
    pub drills_per_year: Option<i64>,
    pub notes: Option<String>,
}

impl SiteData {
    /// Blocking problems and heads-ups. `year` is the current year.
    pub fn validate(&self, year: i64) -> Vec<Issue> {
        let mut v = Vec::new();
        let mut add = |code: &'static str, field: &str, blocking: bool| v.push(Issue { code, field: field.into(), blocking });

        if self.name.trim().is_empty() {
            add("label_missing", "name", true);
        }
        for (field, value, max) in [("land_m2", self.land_m2, MAX_M2), ("built_m2", self.built_m2, MAX_M2), ("water_storage_liters", self.water_storage_liters, MAX_LITERS)] {
            if value.is_some_and(|x| x < 0) {
                add("negative_number", field, true);
            } else if value.is_some_and(|x| x > max) {
                add("number_too_large", field, true);
            }
        }
        if self.land_m2 == Some(0) {
            add("number_invalid", "land_m2", true);
        }
        if self.built_m2 == Some(0) {
            add("number_invalid", "built_m2", true);
        }
        if self.floors.is_some_and(|f| !(1..=catalog::HIGHEST_FLOOR + 1).contains(&f)) {
            add("floors_invalid", "floors", true);
        }
        for (field, value) in [("extinguishers", self.extinguishers), ("smoke_detectors", self.smoke_detectors), ("drills_per_year", self.drills_per_year)] {
            if value.is_some_and(|x| x < 0) {
                add("negative_number", field, true);
            } else if value.is_some_and(|x| x > 1_000) {
                add("number_too_large", field, true);
            }
        }
        for (field, value) in [("built_year", self.built_year), ("opinion_year", self.opinion_year)] {
            if value.is_some_and(|y| !(OLDEST_YEAR..=year).contains(&y)) {
                add("year_invalid", field, true);
            }
        }
        if self.tenure_until.is_some_and(|y| !(OLDEST_YEAR..=year + 199).contains(&y)) {
            add("year_invalid", "tenure_until", true);
        }
        for (field, value, allowed) in [
            ("tenure", &self.tenure, catalog::TENURES),
            ("water_shortage", &self.water_shortage, catalog::FREQUENCY),
            ("power_outages", &self.power_outages, catalog::FREQUENCY),
            ("gas", &self.gas, catalog::GAS),
            ("drainage", &self.drainage, catalog::DRAINAGE),
            ("internal_program", &self.internal_program, catalog::INTERNAL_PROGRAM),
        ] {
            if !code_ok(value, allowed) {
                add("code_unknown", field, true);
            }
        }
        for (field, list, allowed) in [("floor_access", &self.floor_access, catalog::FLOOR_ACCESS), ("water_sources", &self.water_sources, catalog::WATER_SOURCES)] {
            if list.iter().any(|x| !allowed.contains(&x.as_str())) {
                add("code_unknown", field, true);
            }
        }

        // heads-ups: things that do not add up
        if let (Some(land), Some(built), Some(floors)) = (self.land_m2, self.built_m2, self.floors) {
            if land > 0 && built > land * floors {
                add("built_exceeds_land", "built_m2", false);
            }
        }
        if self.tenure_until.is_some_and(|y| y < year) && self.ends() {
            add("tenure_expired", "tenure_until", false);
        }
        if self.ends() && self.tenure_until.is_none() {
            add("tenure_until_missing", "tenure_until", false);
        }
        if self.civil_protection_opinion == Some(true) && self.opinion_year.is_none() {
            add("opinion_year_missing", "opinion_year", false);
        }
        v
    }

    /// Whether the institution holds the building for some years only (a «comodato» or a lease).
    pub fn ends(&self) -> bool {
        self.tenure.as_deref().is_some_and(|t| catalog::ENDING_TENURES.contains(&t))
    }

    /// A building of more than one floor that people can climb only by the stairs (as the person said it).
    pub fn only_stairs(&self) -> bool {
        self.floors.is_some_and(|f| f > 1) && self.floor_access.iter().any(|x| x == "none") && self.floor_access.len() == 1
    }

    /// Drops what does not apply (an end year of a building of its own, access between floors of a one-floor
    /// building, an opinion year without opinion) and trims the texts.
    pub fn tidy(&mut self) {
        self.name = self.name.trim().to_string();
        if self.name.is_empty() {
            self.name = DEFAULT_NAME.into();
        }
        if !self.ends() {
            self.tenure_until = None;
        }
        if self.floors == Some(1) {
            self.floor_access.clear();
        }
        // «only stairs» and a ramp cannot both be true: the ramp wins
        if self.floor_access.len() > 1 {
            self.floor_access.retain(|x| x != "none");
        }
        if self.civil_protection_opinion != Some(true) {
            self.opinion_year = None;
        }
        if self.extinguishers == Some(0) {
            self.extinguishers_current = None;
        }
        for list in [&mut self.floor_access, &mut self.water_sources] {
            let mut seen = Vec::new();
            list.retain(|x| !seen.contains(x) && {
                seen.push(x.clone());
                true
            });
        }
        for o in [&mut self.tenure, &mut self.water_shortage, &mut self.power_outages, &mut self.gas, &mut self.drainage, &mut self.internal_program, &mut self.notes] {
            *o = text(o).map(String::from);
        }
    }

    /// The key data still missing, as codes the screen names: «faltan los metros construidos».
    pub fn missing(&self) -> Vec<&'static str> {
        let mut m = Vec::new();
        if self.built_m2.is_none() {
            m.push("built_m2");
        }
        if self.floors.is_none() {
            m.push("floors");
        }
        if self.tenure.is_none() {
            m.push("tenure");
        }
        if self.water_sources.is_empty() {
            m.push("water");
        }
        if self.extinguishers.is_none() && self.internal_program.is_none() {
            m.push("safety");
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(d: &SiteData) -> Vec<(&'static str, String, bool)> {
        d.validate(2026).into_iter().map(|i| (i.code, i.field, i.blocking)).collect()
    }

    #[test]
    fn an_empty_site_with_a_name_is_valid_and_says_what_is_missing() {
        let d = SiteData { name: "Casa".into(), ..Default::default() };
        assert!(d.validate(2026).is_empty());
        assert_eq!(d.missing(), vec!["built_m2", "floors", "tenure", "water", "safety"]);
    }

    #[test]
    fn numbers_years_and_codes_are_checked() {
        let d = SiteData {
            name: " ".into(),
            land_m2: Some(-5),
            built_m2: Some(0),
            floors: Some(30),
            built_year: Some(2030),
            tenure: Some("mine".into()),
            water_sources: vec!["network".into(), "river".into()],
            extinguishers: Some(-1),
            ..Default::default()
        };
        let got = codes(&d);
        for want in [
            ("label_missing", "name"),
            ("negative_number", "land_m2"),
            ("number_invalid", "built_m2"),
            ("floors_invalid", "floors"),
            ("negative_number", "extinguishers"),
            ("year_invalid", "built_year"),
            ("code_unknown", "tenure"),
            ("code_unknown", "water_sources"),
        ] {
            assert!(got.iter().any(|(c, f, b)| *c == want.0 && f == want.1 && *b), "missing {want:?} in {got:?}");
        }
    }

    #[test]
    fn heads_ups_do_not_block() {
        let d = SiteData {
            name: "Casa".into(),
            land_m2: Some(300),
            built_m2: Some(700),
            floors: Some(2),
            tenure: Some("loan".into()),
            tenure_until: Some(2024),
            civil_protection_opinion: Some(true),
            ..Default::default()
        };
        assert_eq!(
            codes(&d),
            vec![("built_exceeds_land", "built_m2".into(), false), ("tenure_expired", "tenure_until".into(), false), ("opinion_year_missing", "opinion_year".into(), false)]
        );
        let lease = SiteData { name: "Casa".into(), tenure: Some("rent".into()), ..Default::default() };
        assert_eq!(codes(&lease), vec![("tenure_until_missing", "tenure_until".into(), false)]);
    }

    #[test]
    fn tidy_drops_what_does_not_apply() {
        let mut d = SiteData {
            name: "  ".into(),
            floors: Some(1),
            floor_access: vec!["ramp".into()],
            tenure: Some("own".into()),
            tenure_until: Some(2030),
            opinion_year: Some(2020),
            water_sources: vec!["network".into(), "network".into(), "truck".into()],
            notes: Some("   ".into()),
            ..Default::default()
        };
        d.tidy();
        assert_eq!(d.name, DEFAULT_NAME);
        assert!(d.floor_access.is_empty());
        assert_eq!((d.tenure_until, d.opinion_year, d.notes.clone()), (None, None, None));
        assert_eq!(d.water_sources, vec!["network", "truck"]);
    }
}
