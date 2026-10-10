//! The institution at a glance (ADR-033, audit D4): what «Mi institución» and Inicio show of it, composed here and
//! not in the screen. The figure of each module (and whether it is the quick figure of the first start or the
//! records), how full the house is, the balance of the year, and what is still missing with where to fill it in.

use crate::core::error::ServiceError;
use crate::core::onboarding::domain::StepStatus;
use crate::core::onboarding::service::Records;
use rusqlite::Connection;
use serde::Serialize;

/// Where a missing datum is filled in: a window of «Mi institución» or the page of a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Place {
    Institution,
    Contact,
    Legal,
    Capacity,
    Finance,
    Facilities,
    Staff,
    People,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gap {
    /// What is missing (the screen says it with `es.onboarding.missing` or `es.profile.todo`).
    pub code: &'static str,
    pub place: Place,
}

/// How far the data of the institution are filled in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Completion {
    /// In the order of the first start, then what nobody registered yet.
    pub gaps: Vec<Gap>,
    /// 0 to 100; it reads 100 only with nothing missing.
    pub percent: u32,
}

/// A figure of a module: the records, or the quick figure of the first start while there are none (`approx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Figure {
    pub value: i64,
    pub approx: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstitutionOverview {
    pub people: Figure,
    pub capacity: Option<i64>,
    /// Share of the places taken (people served over capacity), up to 100; `None` without a capacity.
    pub occupied_percent: Option<u32>,
    /// Places left (never below zero); `None` without a capacity.
    pub vacant: Option<i64>,
    pub staff: Figure,
    pub spaces: i64,
    /// What is left of the year: income minus expenses (`None` while one of them is not known).
    pub balance_annual_mxn: Option<i64>,
    pub completion: Completion,
}

fn place_of(code: &str) -> Place {
    match code {
        "state" | "municipality" | "contact" => Place::Contact,
        "legal_form" | "founded_year" | "authorized_donee" | "cluni" => Place::Legal,
        "capacity_total" | "served" | "staff" => Place::Capacity,
        "expenses" | "income" => Place::Finance,
        "floors" | "tenure" => Place::Facilities,
        "staff_records" => Place::Staff,
        "served_records" => Place::People,
        "spaces" => Place::Facilities,
        _ => Place::Institution,
    }
}

/// Every check of the data, to tell how far they are: the data of the first start, the attention profile and the
/// records of the modules.
const CHECKS: usize = 21;

/// What is missing and how far the data are, from the steps of the first start, the required fields of «Su
/// institución» still empty (their ids), the records of the modules and how many spaces are registered. A record
/// that is missing is listed only when its quick figure is there: the person fills one thing at a time.
pub fn completion(steps: &[StepStatus], identity_missing: &[&str], records: &Records, spaces: i64) -> Completion {
    let mut codes: Vec<&'static str> = steps.iter().flat_map(|s| s.missing.iter().copied()).collect();
    for id in identity_missing {
        let code = match *id {
            "institution.name" => "name",
            "institution.mission" => "mission",
            "institution.populations" => "populations",
            "institution.modalities" => "modalities",
            _ => continue,
        };
        if !codes.contains(&code) {
            // the attention profile goes right after what the institution is
            let at = codes.iter().position(|c| !matches!(*c, "name" | "mission" | "populations" | "modalities")).unwrap_or(codes.len());
            codes.insert(at, code);
        }
    }
    let mut failed = codes.len();
    let has = |c: &[&str], code: &str| c.contains(&code);
    for (record_missing, code, covered_by) in [
        (records.staff == 0, "staff_records", &["staff"][..]),
        (records.served == 0, "served_records", &["served"][..]),
        (spaces == 0, "spaces", &["floors", "tenure"][..]),
    ] {
        if record_missing {
            failed += 1;
            if !covered_by.iter().any(|x| has(&codes, x)) {
                codes.push(code);
            }
        }
    }
    let percent = if codes.is_empty() { 100 } else { ((CHECKS - failed.min(CHECKS)) * 100 / CHECKS).min(99) as u32 };
    Completion { gaps: codes.into_iter().map(|code| Gap { code, place: place_of(code) }).collect(), percent }
}

/// The records win; while there are none, the quick figure of the first start stands in, and says so.
pub fn figure(records: i64, estimate: Option<i64>) -> Figure {
    match estimate {
        Some(e) if records == 0 => Figure { value: e, approx: true },
        _ => Figure { value: records, approx: false },
    }
}

pub fn occupied_percent(people: i64, capacity: Option<i64>) -> Option<u32> {
    capacity.filter(|c| *c > 0).map(|c| ((people.max(0) * 100 + c / 2) / c).min(100) as u32)
}

pub fn institution_overview(conn: &Connection) -> Result<InstitutionOverview, ServiceError> {
    let input = crate::core::profile::storage::load_current(conn)?.map(|p| p.input).unwrap_or_default();
    let (steps, records) = crate::core::onboarding::service::steps_now(conn)?;
    let identity = crate::core::profile::forms::get(conn, "institution.identity")?;
    let spaces = crate::modules::facilities::api::indicators(conn)?.spaces;
    let staff_estimate = match (input.staff_paid_estimate, input.staff_volunteer_estimate) {
        (None, None) => None,
        (p, v) => Some(p.unwrap_or(0) + v.unwrap_or(0)),
    };
    let people = figure(records.served, input.served_estimate);
    Ok(InstitutionOverview {
        people,
        capacity: input.capacity_total,
        occupied_percent: occupied_percent(people.value, input.capacity_total),
        vacant: input.capacity_total.map(|c| (c - people.value).max(0)),
        staff: figure(records.staff, staff_estimate),
        spaces,
        balance_annual_mxn: crate::core::bridge::finance::finances(conn)?.balance_annual_mxn,
        completion: completion(&steps, &identity.missing, &records, spaces),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps(missing: &[(&'static str, &[&'static str])]) -> Vec<StepStatus> {
        ["institution", "location", "people", "team", "money", "building"]
            .iter()
            .map(|&key| {
                let m: Vec<&'static str> = missing.iter().find(|(k, _)| *k == key).map(|(_, m)| m.to_vec()).unwrap_or_default();
                StepStatus { key, complete: m.is_empty(), missing: m }
            })
            .collect()
    }

    const FULL: Records = Records { served: 3, staff: 2, fee_payers: 0 };

    #[test]
    fn complete_data_read_one_hundred() {
        let c = completion(&steps(&[]), &[], &FULL, 4);
        assert_eq!(c, Completion { gaps: vec![], percent: 100 });
    }

    #[test]
    fn what_is_missing_says_where_and_the_attention_profile_goes_after_the_institution() {
        let c = completion(&steps(&[("institution", &["mission"]), ("location", &["state", "cluni"]), ("money", &["income"])]), &["institution.mission", "institution.populations"], &FULL, 4);
        let got: Vec<_> = c.gaps.iter().map(|g| (g.code, g.place)).collect();
        assert_eq!(got, vec![("mission", Place::Institution), ("populations", Place::Institution), ("state", Place::Contact), ("cluni", Place::Legal), ("income", Place::Finance)]);
        assert!(c.percent < 100);
    }

    #[test]
    fn a_missing_record_is_listed_only_once_its_quick_figure_is_there() {
        let nobody = Records { served: 0, staff: 0, fee_payers: 0 };
        // the quick figures are missing too: only they are listed, but the records still count as not done
        let c = completion(&steps(&[("people", &["served"]), ("team", &["staff"]), ("building", &["floors"])]), &[], &nobody, 0);
        assert_eq!(c.gaps.iter().map(|g| g.code).collect::<Vec<_>>(), vec!["served", "staff", "floors"]);
        assert_eq!(c.percent, ((21 - 6) * 100 / 21) as u32);
        // with the quick figures, the records are what is left
        let c = completion(&steps(&[]), &[], &nobody, 0);
        assert_eq!(c.gaps.iter().map(|g| (g.code, g.place)).collect::<Vec<_>>(), vec![("staff_records", Place::Staff), ("served_records", Place::People), ("spaces", Place::Facilities)]);
        assert_eq!(c.percent, 85);
    }

    #[test]
    fn with_one_thing_missing_it_never_reads_one_hundred() {
        let c = completion(&steps(&[]), &["institution.modalities"], &FULL, 4);
        assert_eq!(c.percent, 95);
        assert_eq!(c.gaps, vec![Gap { code: "modalities", place: Place::Institution }]);
    }

    #[test]
    fn the_records_win_over_the_quick_figure() {
        assert_eq!(figure(0, Some(25)), Figure { value: 25, approx: true });
        assert_eq!(figure(18, Some(25)), Figure { value: 18, approx: false });
        assert_eq!(figure(0, None), Figure { value: 0, approx: false });
    }

    #[test]
    fn the_house_is_full_up_to_one_hundred() {
        assert_eq!(occupied_percent(18, Some(25)), Some(72));
        assert_eq!(occupied_percent(30, Some(25)), Some(100));
        assert_eq!(occupied_percent(5, None), None);
        assert_eq!(occupied_percent(5, Some(0)), None);
    }

    #[test]
    fn an_institution_with_its_records_and_its_quick_figures() {
        use crate::core::profile::domain::{InstitutionInput, InstitutionKind, ProfileInput};
        let dir = tempfile::tempdir().unwrap();
        let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff").unwrap();
        let input = ProfileInput {
            institution: InstitutionInput { name: "Asilo Ficticio".into(), kind: InstitutionKind::ElderlyHome, ..Default::default() },
            capacity_total: Some(20),
            served_estimate: Some(15),
            staff_paid_estimate: Some(4),
            staff_volunteer_estimate: Some(2),
            ..Default::default()
        };
        crate::core::profile::storage::save(&mut c, &input).unwrap();
        let o = institution_overview(&c).unwrap();
        assert_eq!((o.people, o.occupied_percent, o.vacant, o.staff), (Figure { value: 15, approx: true }, Some(75), Some(5), Figure { value: 6, approx: true }));
        assert_eq!(o.balance_annual_mxn, None);
        let codes: Vec<_> = o.completion.gaps.iter().map(|g| g.code).collect();
        assert!(codes.starts_with(&["mission"]) && !codes.contains(&"populations"), "the kind gave the populations: {codes:?}");
        assert!(codes.contains(&"staff_records") && codes.contains(&"served_records") && codes.contains(&"floors"), "{codes:?}");
    }
}
