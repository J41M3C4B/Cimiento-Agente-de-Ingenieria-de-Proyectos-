//! The board of the people served (ADR-029): indicators of the module crossed with the rest of «Mi institución»
//! (capacity, facilities, money, staff), and findings in one sentence each, made by fixed rules. A finding carries
//! its numbers; the screen and the sheet of the AI put the words. Only findings without money and with groups of
//! `MIN_GROUP` or more may reach the AI.

use crate::care::api::MIN_GROUP;
use crate::care::domain::aggregate::Indicators;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Insight {
    pub code: &'static str,
    pub values: BTreeMap<&'static str, i64>,
    /// Names that go with it (the spaces that are not accessible).
    pub items: Vec<String>,
    pub for_ai: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Board {
    pub indicators: Indicators,
    pub waiting: i64,
    pub capacity: Option<i64>,
    pub free_seats: Option<i64>,
    pub occupancy_percent: Option<i64>,
    /// What the institution spends per person served in a month (expenses of the year / 12 / people served).
    pub cost_per_person_monthly: Option<i64>,
    pub insights: Vec<Insight>,
}

/// What the board needs from the rest of the institution.
pub struct Context<'a> {
    pub capacity: Option<i64>,
    /// Spaces that are not accessible (their names).
    pub not_accessible: Vec<&'a str>,
    pub expenses_annual: Option<i64>,
    /// People of the staff in care and health positions.
    pub carers: i64,
    pub elderly_home: bool,
    pub children_home: bool,
}

fn insight(code: &'static str, values: &[(&'static str, i64)], items: Vec<String>, for_ai: bool) -> Insight {
    Insight { code, values: values.iter().copied().collect(), items, for_ai }
}

fn count_of(list: &[crate::care::api::Count], codes: &[&str]) -> i64 {
    list.iter().filter(|c| codes.contains(&c.code.as_str())).map(|c| c.count).sum()
}

pub fn board(i: Indicators, waiting: i64, cx: &Context) -> Board {
    let served = i.served;
    let free_seats = cx.capacity.map(|c| (c - served).max(0));
    let occupancy_percent = cx.capacity.filter(|c| *c > 0).map(|c| served * 100 / c);
    let cost = cx.expenses_annual.filter(|_| served > 0).map(|e| (e + 6 * served) / (12 * served));
    let mut out = Vec::new();

    let limited = count_of(&i.mobility, &["wheelchair", "bedridden"]);
    if limited > 0 && !cx.not_accessible.is_empty() {
        out.push(insight("mobility_vs_access", &[("people", limited), ("spaces", cx.not_accessible.len() as i64)], cx.not_accessible.iter().map(|s| s.to_string()).collect(), limited >= MIN_GROUP));
    }
    if waiting > 0 {
        out.push(insight("waitlist_vs_seats", &[("waiting", waiting), ("free", free_seats.unwrap_or(-1))], vec![], true));
    }
    if let Some(p) = occupancy_percent.filter(|p| *p >= 95) {
        out.push(insight("occupancy_high", &[("percent", p)], vec![], true));
    }
    if let (Some(cost), true) = (cost, served > 0) {
        let fee = i.average_fee.unwrap_or(0);
        out.push(insight("cost_gap", &[("cost", cost), ("fee", fee), ("gap", (cost - fee).max(0))], vec![], false));
    }
    let dependent = count_of(&i.dependency, &["high", "total"]);
    if dependent >= MIN_GROUP {
        out.push(insight("dependency_vs_carers", &[("dependent", dependent), ("carers", cx.carers)], vec![], true));
    }
    if i.few_visits > 0 {
        out.push(insight("few_visits", &[("people", i.few_visits)], vec![], i.few_visits >= MIN_GROUP));
    }
    if cx.elderly_home && i.elderly_without_program > 0 {
        out.push(insight("no_program", &[("people", i.elderly_without_program)], vec![], i.elderly_without_program >= MIN_GROUP));
    }
    if cx.children_home && i.school_lag > 0 {
        out.push(insight("school_lag", &[("people", i.school_lag)], vec![], i.school_lag >= MIN_GROUP));
    }
    if i.without_consent > 0 {
        out.push(insight("without_consent", &[("people", i.without_consent)], vec![], false));
    }
    if i.incomplete > 0 {
        out.push(insight("incomplete_records", &[("people", i.incomplete)], vec![], false));
    }
    Board { indicators: i, waiting, capacity: cx.capacity, free_seats, occupancy_percent, cost_per_person_monthly: cost, insights: out }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::care::api::Count;

    fn count(code: &str, n: i64) -> Count {
        Count { code: code.into(), count: n }
    }

    #[test]
    fn findings_cross_the_people_with_the_spaces_the_money_and_the_staff() {
        let i = Indicators {
            served: 24,
            mobility: vec![count("independent", 15), count("wheelchair", 7), count("bedridden", 2)],
            dependency: vec![count("medium", 10), count("high", 9), count("total", 3)],
            few_visits: 2,
            elderly_without_program: 6,
            average_fee: Some(2_100),
            without_consent: 4,
            ..Default::default()
        };
        let cx = Context { capacity: Some(25), not_accessible: vec!["Baño", "Regadera"], expenses_annual: Some(2_419_200), carers: 4, elderly_home: true, children_home: false };
        let b = board(i, 12, &cx);
        assert_eq!((b.free_seats, b.occupancy_percent, b.cost_per_person_monthly), (Some(1), Some(96), Some(8_400)));
        let codes: Vec<_> = b.insights.iter().map(|x| (x.code, x.for_ai)).collect();
        assert_eq!(codes, vec![
            ("mobility_vs_access", true),
            ("waitlist_vs_seats", true),
            ("occupancy_high", true),
            ("cost_gap", false),
            ("dependency_vs_carers", true),
            ("few_visits", false),
            ("no_program", true),
            ("without_consent", false),
        ]);
        let gap = &b.insights[3];
        assert_eq!((gap.values["cost"], gap.values["fee"], gap.values["gap"]), (8_400, 2_100, 6_300));
        assert_eq!(b.insights[0].values["people"], 9);
        assert_eq!(b.insights[0].items, vec!["Baño", "Regadera"]);
    }

    #[test]
    fn without_people_or_data_there_are_no_made_up_findings() {
        let cx = Context { capacity: None, not_accessible: vec![], expenses_annual: Some(100_000), carers: 0, elderly_home: true, children_home: false };
        let b = board(Indicators::default(), 0, &cx);
        assert!(b.insights.is_empty());
        assert_eq!(b.cost_per_person_monthly, None, "no cost per person without people");
    }
}
