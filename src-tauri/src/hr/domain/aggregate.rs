//! What leaves the staff module (ADR-027): anonymous lines for the profile (how many per position, with the data
//! the payroll needs) and the summary the AI reads. Never a name, an identifier, a date of a person or a pay in
//! the summary. Personal attributes (schooling, seniority) are told only for groups of `MIN_GROUP` or more.

use super::catalog::{self, Relation, Rules};
use super::person::{years_between, PersonData};
use super::position::Position;
use serde::Serialize;

/// The smallest group a personal attribute is told for: a smaller one could point at someone.
pub const MIN_GROUP: i64 = 3;

/// One person of the staff with what the rules of their modality say.
pub struct Member<'a> {
    pub data: &'a PersonData,
    /// The built-in modality whose rules apply (an institution's own modality behaves as one of these).
    pub base: &'static str,
    pub rules: Rules,
}

/// One anonymous line for the profile: the people of a position with the same pay data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaffLine {
    pub role: String,
    pub relation: Relation,
    /// Paid as payroll: a labor relation or fees.
    pub paid: bool,
    /// `permanent`, `temporary` or `fees` (the words the profile uses for the benefits of the law).
    pub contract: Option<&'static str>,
    /// Per person, as a month: the salary, the fees, the contribution or the invoice.
    pub monthly_pay: Option<i64>,
    pub start_year: Option<i64>,
    pub shift: Option<String>,
    pub count: i64,
}

fn contract_of(base: &str) -> Option<&'static str> {
    match base {
        "indefinite" | "project_based" => Some("permanent"),
        "fixed_term" | "trial" => Some("temporary"),
        "fees" | "assimilated" => Some("fees"),
        _ => None,
    }
}

fn title_of<'a>(positions: &'a [Position], id: &Option<String>) -> &'a str {
    id.as_deref().and_then(|id| positions.iter().find(|p| p.id == id)).map_or("Sin puesto", |p| p.input.title.as_str())
}

/// The lines of the people who still work here.
pub fn staff_lines(people: &[Member], positions: &[Position]) -> Vec<StaffLine> {
    let mut out: Vec<StaffLine> = Vec::new();
    for m in people.iter().filter(|m| m.data.is_current()) {
        let paid = matches!(m.rules.relation, Relation::Employee | Relation::Fees);
        let line = StaffLine {
            role: title_of(positions, &m.data.position_id).to_string(),
            relation: m.rules.relation,
            paid,
            contract: contract_of(m.base),
            monthly_pay: if m.rules.relation == Relation::Volunteer { None } else { m.data.monthly_pay() },
            start_year: m.data.start_date.as_deref().and_then(|d| d.get(..4)).and_then(|y| y.parse().ok()),
            shift: m.data.shift.clone(),
            count: 1,
        };
        match out.iter_mut().find(|l| StaffLine { count: l.count, ..line.clone() } == **l) {
            Some(l) => l.count += 1,
            None => out.push(line),
        }
    }
    out
}

/// A count by code, in the order of the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Count {
    pub code: &'static str,
    pub count: i64,
}

fn counts<'a>(codes: &[&'static str], values: impl Iterator<Item = &'a str>) -> Vec<Count> {
    let values: Vec<&str> = values.collect();
    codes
        .iter()
        .map(|c| Count { code: c, count: values.iter().filter(|v| **v == *c).count() as i64 })
        .filter(|c| c.count > 0)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PositionSummary {
    pub title: String,
    pub area: Option<String>,
    pub duties: Option<String>,
    pub people: i64,
    pub by_relation: Vec<Count>,
    pub schedules: Vec<Count>,
    pub shifts: Vec<Count>,
    pub authorized_seats: Option<i64>,
    /// Seats without a person.
    pub vacancies: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StaffSummary {
    /// The positions with people or with seats.
    pub positions: Vec<PositionSummary>,
    pub by_relation: Vec<Count>,
    pub total: i64,
    /// Ranges of schooling, only those with `MIN_GROUP` people or more.
    pub education: Vec<Count>,
    /// Ranges of years in the institution, only those with `MIN_GROUP` people or more.
    pub seniority: Vec<Count>,
}

const RELATIONS: [&str; 6] = ["employee", "fees", "religious", "volunteer", "trainee", "external"];
pub const EDUCATION_RANGES: [&str; 3] = ["basic", "high_school_or_technical", "higher"];
pub const SENIORITY_RANGES: [&str; 4] = ["under_1", "1_to_4", "5_to_9", "10_plus"];

fn education_range(code: &str) -> Option<&'static str> {
    Some(match code {
        "none" | "primary" | "secondary" => "basic",
        "high_school" | "technical" => "high_school_or_technical",
        "bachelor" | "postgraduate" => "higher",
        _ => return None,
    })
}

fn seniority_range(years: i64) -> &'static str {
    match years {
        y if y < 1 => "under_1",
        1..=4 => "1_to_4",
        5..=9 => "5_to_9",
        _ => "10_plus",
    }
}

/// The staff as the AI may read it, on `today` (`YYYY-MM-DD`).
pub fn summary(people: &[Member], positions: &[Position], today: &str) -> StaffSummary {
    let current: Vec<&Member> = people.iter().filter(|m| m.data.is_current()).collect();
    let mut out = StaffSummary::default();
    for p in positions {
        let here: Vec<&&Member> = current.iter().filter(|m| m.data.position_id.as_deref() == Some(p.id.as_str())).collect();
        let n = here.len() as i64;
        let seats = p.input.authorized_seats.filter(|s| *s > 0);
        if n == 0 && seats.is_none() {
            continue;
        }
        out.positions.push(PositionSummary {
            title: p.input.title.clone(),
            area: p.input.area.clone().filter(|a| !a.is_empty()),
            duties: p.input.duties.clone().map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
            people: n,
            by_relation: counts(&RELATIONS, here.iter().map(|m| m.rules.relation.as_str())),
            schedules: counts(catalog::SCHEDULES, here.iter().filter_map(|m| m.data.schedule.as_deref())),
            shifts: counts(catalog::SHIFTS, here.iter().filter_map(|m| m.data.shift.as_deref())),
            authorized_seats: seats,
            vacancies: seats.map_or(0, |s| (s - n).max(0)),
        });
    }
    out.by_relation = counts(&RELATIONS, current.iter().map(|m| m.rules.relation.as_str()));
    out.total = current.len() as i64;
    let big = |v: Vec<Count>| v.into_iter().filter(|c| c.count >= MIN_GROUP).collect::<Vec<_>>();
    out.education = big(counts(&EDUCATION_RANGES, current.iter().filter_map(|m| m.data.education.as_deref().and_then(education_range))));
    out.seniority = big(counts(
        &SENIORITY_RANGES,
        current.iter().filter_map(|m| m.data.start_date.as_deref().and_then(|d| years_between(d, today)).map(seniority_range)),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hr::domain::position::PositionInput;

    fn position(id: &str, title: &str, seats: Option<i64>) -> Position {
        Position { id: id.into(), input: PositionInput { title: title.into(), area: Some("care".into()), authorized_seats: seats, ..Default::default() }, active: true }
    }

    fn person(pos: &str, modality: &str, pay: Option<i64>, start: &str, education: &str) -> PersonData {
        PersonData {
            first_names: "Persona".into(),
            position_id: Some(pos.into()),
            modality: modality.into(),
            status: "active".into(),
            pay_amount_mxn: pay,
            start_date: Some(start.into()),
            education: Some(education.into()),
            shift: Some("night".into()),
            ..Default::default()
        }
    }

    fn members(data: &[PersonData]) -> Vec<Member<'_>> {
        data.iter()
            .map(|d| {
                let base = catalog::BUILTIN_MODALITIES.iter().find(|(c, _)| *c == d.modality).map(|(c, _)| *c).unwrap();
                Member { data: d, base, rules: catalog::builtin_rules(base).unwrap() }
            })
            .collect()
    }

    #[test]
    fn lines_group_people_with_the_same_pay_data_and_leave_out_who_left() {
        let mut gone = person("p1", "indefinite", Some(9_000), "2018-01-01", "bachelor");
        gone.status = "left".into();
        let data = vec![
            person("p1", "indefinite", Some(9_000), "2018-03-01", "bachelor"),
            person("p1", "indefinite", Some(9_000), "2018-09-01", "bachelor"),
            person("p1", "fees", Some(6_000), "2020-01-01", "bachelor"),
            person("p2", "religious", Some(1_500), "2010-01-01", "higher_unknown"),
            person("p2", "volunteer", Some(100), "2025-01-01", "primary"),
            gone,
        ];
        let lines = staff_lines(&members(&data), &[position("p1", "Enfermería", None), position("p2", "Pastoral", None)]);
        let got: Vec<_> = lines.iter().map(|l| (l.role.as_str(), l.relation, l.paid, l.contract, l.monthly_pay, l.start_year, l.count)).collect();
        assert_eq!(got, vec![
            ("Enfermería", Relation::Employee, true, Some("permanent"), Some(9_000), Some(2018), 2),
            ("Enfermería", Relation::Fees, true, Some("fees"), Some(6_000), Some(2020), 1),
            ("Pastoral", Relation::Religious, false, None, Some(1_500), Some(2010), 1),
            ("Pastoral", Relation::Volunteer, false, None, None, Some(2025), 1),
        ]);
    }

    #[test]
    fn the_summary_tells_positions_and_vacancies_and_personal_attributes_only_for_groups_of_three() {
        let data = vec![
            person("p1", "indefinite", Some(9_000), "2014-01-01", "bachelor"),
            person("p1", "indefinite", Some(9_000), "2015-01-01", "bachelor"),
            person("p1", "indefinite", Some(9_000), "2016-01-01", "postgraduate"),
            person("p2", "volunteer", None, "2025-01-01", "primary"),
        ];
        let positions = [position("p1", "Enfermería", Some(5)), position("p2", "Acompañamiento", None), position("p3", "Psicología", Some(1)), position("p4", "Cocina", None)];
        let s = summary(&members(&data), &positions, "2026-10-07");
        let titles: Vec<_> = s.positions.iter().map(|p| (p.title.as_str(), p.people, p.vacancies)).collect();
        assert_eq!(titles, vec![("Enfermería", 3, 2), ("Acompañamiento", 1, 0), ("Psicología", 0, 1)], "Cocina has nobody and no seats");
        assert_eq!(s.total, 4);
        assert_eq!(s.education, vec![Count { code: "higher", count: 3 }], "one person with primary is not told");
        assert_eq!(s.seniority, vec![Count { code: "10_plus", count: 3 }], "the volunteer alone (1 to 4 years) is not told");
        assert_eq!(s.positions[0].shifts, vec![Count { code: "night", count: 3 }]);
    }
}
