//! The institution at a glance (ADR-033, audit D4): what «Mi institución» and Inicio show of it, composed here and
//! not in the screen. The figure of each module (and whether it is the quick figure of the first start or the
//! records), how full the house is, the balance of the year, and what is still missing with where to fill it in.

use crate::core::error::ServiceError;
use rusqlite::Connection;
use serde::Serialize;

/// Where a missing datum is filled in: a window of «Mi institución». What the modules still lack is asked from
/// Inicio, in «Siguientes pasos» (ADR-042).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Place {
    Institution,
    Contact,
    Legal,
    /// The board and the bank accounts of «Mi institución» (ADR-033 §3).
    Board,
    Capacity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gap {
    /// What is missing (the screen says it with `es.onboarding.missing` or `es.profile.todo`).
    pub code: &'static str,
    pub place: Place,
}

/// How far the data of the institution are filled in: only its own data (its windows and its board), never the
/// records of the modules (ADR-042).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Completion {
    /// The indispensable data first, in the order of the first start; then the rest, in the order of the page.
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
    /// The suggestions of the AI waiting in each window (IA4), only where there are.
    pub suggestions: Vec<Suggestions>,
}

/// How many suggestions of the AI wait in a window of «Mi institución».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Suggestions {
    pub place: Place,
    pub count: usize,
}

/// The suggestions waiting in each window the Capturist fills, as the window shows them.
fn suggestions(conn: &Connection) -> Result<Vec<Suggestions>, ServiceError> {
    let mut out = Vec::new();
    for (form, _, _) in crate::core::agents::capturist::FILLABLE {
        let count = crate::core::profile::forms::get(conn, form)?.proposals.len();
        let place = match *form {
            "institution.contact" => Place::Contact,
            "institution.legal" => Place::Legal,
            "institution.capacity" => Place::Capacity,
            _ => Place::Institution,
        };
        if count > 0 {
            out.push(Suggestions { place, count });
        }
    }
    Ok(out)
}

fn place_of(code: &str) -> Place {
    match code {
        "state" | "municipality" | "contact" | "address" => Place::Contact,
        "legal_form" | "founded_year" | "authorized_donee" | "cluni" | "legal_name" | "purpose" | "legal_rfc" | "tax_regime" | "junta_folio" => {
            Place::Legal
        }
        "board" => Place::Board,
        "capacity_total" | "served" | "staff" => Place::Capacity,
        _ => Place::Institution,
    }
}

/// What a required field of a form stands for in the list of what is missing: its id without `institution.`; the
/// street and the postal code as one, the address; and the quick figures as the people served and the team.
pub fn code_of(id: &'static str) -> &'static str {
    match id {
        "institution.street" | "institution.postal_code" => "address",
        "institution.served_estimate" => "served",
        "institution.staff_paid_estimate" | "institution.staff_volunteer_estimate" => "staff",
        _ => id.strip_prefix("institution.").unwrap_or(id),
    }
}

/// Where a field stands in the list: the indispensable ones first, in the order of the first start; the rest keep
/// the order of the page.
fn order(id: &str) -> (u8, usize) {
    match crate::core::onboarding::domain::STEPS.iter().flat_map(|s| s.fields.iter()).position(|f| *f == id) {
        Some(at) => (0, at),
        None => (1, 0),
    }
}

/// What is missing and how far the data are, from the required fields still empty (their ids, in the order of the
/// page) and how many things the data must have (`checks`, each code once).
pub fn completion(missing: &[&'static str], checks: usize) -> Completion {
    let mut ids = missing.to_vec();
    // stable: what is not indispensable keeps the order of the page
    ids.sort_by_key(|id| order(id));
    let mut codes: Vec<&'static str> = Vec::new();
    for id in ids {
        let code = code_of(id);
        if !codes.contains(&code) {
            codes.push(code);
        }
    }
    let failed = codes.len();
    let checks = checks.max(failed).max(1);
    let percent = if codes.is_empty() { 100 } else { ((checks - failed) * 100 / checks).min(99) as u32 };
    Completion { gaps: codes.into_iter().map(|code| Gap { code, place: place_of(code) }).collect(), percent }
}

/// What stands for the board in the list of what is missing: someone active in it.
const BOARD: &str = "institution.board";

/// The required fields of the windows of «Mi institución» that apply and are still empty, in the order of the page,
/// and how many things the data must have (each code once, the board included). What the records answer is not
/// missing (`profile::forms::get` leaves it out).
fn forms_missing(conn: &Connection) -> Result<(Vec<&'static str>, usize), ServiceError> {
    let mut missing = Vec::new();
    let mut checks: Vec<&'static str> = Vec::new();
    for spec in crate::core::institution::forms::FORMS {
        let view = crate::core::profile::forms::get(conn, spec.id)?;
        missing.extend(view.missing);
        for f in spec.fields().filter(|f| f.required && spec.applies(f, &view.values)) {
            let code = code_of(f.id);
            if !checks.contains(&code) {
                checks.push(code);
            }
        }
    }
    // the board is asked of every institution: someone governs it (ADR-033 §3); the accounts are optional
    checks.push(code_of(BOARD));
    if !crate::core::protected::has_board(conn)? {
        missing.push(BOARD);
    }
    Ok((missing, checks.len()))
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
    let records = crate::core::onboarding::service::records(conn)?;
    let (missing, checks) = forms_missing(conn)?;
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
        completion: completion(&missing, checks),
        suggestions: suggestions(conn)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_data_read_one_hundred() {
        assert_eq!(completion(&[], 22), Completion { gaps: vec![], percent: 100 });
    }

    /// ADR-042: the indispensable data go first, in the order of the first start, whatever window they are in; the
    /// rest keep the order of the page, and each code counts once.
    #[test]
    fn the_indispensable_go_first_and_each_one_says_where() {
        let missing = [
            "institution.mission", "institution.services", "institution.street", "institution.postal_code", "institution.state",
            "institution.legal_name", "institution.cluni", "institution.staff_paid_estimate", "institution.staff_volunteer_estimate", "institution.board",
        ];
        let c = completion(&missing, 26);
        let got: Vec<_> = c.gaps.iter().map(|g| (g.code, g.place)).collect();
        assert_eq!(
            got,
            vec![
                ("mission", Place::Institution),
                ("state", Place::Contact),
                ("cluni", Place::Legal),
                ("staff", Place::Capacity),
                ("services", Place::Institution),
                ("address", Place::Contact),
                ("legal_name", Place::Legal),
                ("board", Place::Board),
            ]
        );
        assert_eq!(c.percent, ((26 - 8) * 100 / 26) as u32);
    }

    #[test]
    fn with_one_thing_missing_it_never_reads_one_hundred() {
        let c = completion(&["institution.modalities"], 200);
        assert_eq!(c.percent, 99);
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
        assert!(codes.starts_with(&["mission", "state", "municipality", "legal_form"]) && !codes.contains(&"populations"), "the kind gave the populations: {codes:?}");
        // ADR-042: the records of the modules and their data (money, building) are asked from Inicio, not here
        for module in ["staff_records", "served_records", "spaces", "floors", "expenses", "income", "served", "staff", "capacity_total"] {
            assert!(!codes.contains(&module), "{module}: {codes:?}");
        }
    }
}
