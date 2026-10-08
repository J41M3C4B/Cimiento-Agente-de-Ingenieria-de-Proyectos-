//! What leaves the module of the people served (ADR-029): anonymous lines for the profile (how many per group, with
//! the stay fees), the summary the AI reads and the indicators of the board. Never a name, a CURP, a date of a
//! person, a responsible person or a legal situation of one girl. Personal attributes (health, disability,
//! language, origin, schooling, family) are told only for groups of `MIN_GROUP` or more.

use super::catalog::{self, Flavor};
use super::person::BeneficiaryData;
use serde::Serialize;

pub const MIN_GROUP: i64 = 3;

/// A person served with what the screen and the counts need about them.
pub struct Member<'a> {
    pub data: &'a BeneficiaryData,
    /// The title of their own group, if the institution gave them one.
    pub own_group: Option<&'a str>,
}

/// One anonymous line for the profile: the people of a group with the same level of support and fee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PopulationLine {
    pub label: String,
    pub dependency: Option<String>,
    pub count: i64,
    pub age_min: Option<i64>,
    pub age_max: Option<i64>,
    pub paying: i64,
    /// The fee per paying person (lines are split by fee).
    pub monthly_fee: Option<i64>,
}

fn label_of(m: &Member, flavor: Flavor, today: &str) -> String {
    m.own_group.map(str::to_string).unwrap_or_else(|| catalog::group_label(flavor, m.data.sex.as_deref(), m.data.age(today)))
}

pub fn population_lines(people: &[Member], flavor: Flavor, today: &str) -> Vec<PopulationLine> {
    let mut out: Vec<PopulationLine> = Vec::new();
    for m in people.iter().filter(|m| m.data.is_served()) {
        let label = label_of(m, flavor, today);
        let fee = m.data.pays_fee().then_some(m.data.monthly_fee_mxn).flatten();
        let age = m.data.age(today);
        match out.iter_mut().find(|l| l.label == label && l.dependency == m.data.dependency && l.monthly_fee == fee) {
            Some(l) => {
                l.count += 1;
                l.paying += fee.is_some() as i64;
                if let Some(a) = age {
                    l.age_min = Some(l.age_min.map_or(a, |x| x.min(a)));
                    l.age_max = Some(l.age_max.map_or(a, |x| x.max(a)));
                }
            }
            None => out.push(PopulationLine { label, dependency: m.data.dependency.clone(), count: 1, age_min: age, age_max: age, paying: fee.is_some() as i64, monthly_fee: fee }),
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Count {
    pub code: String,
    pub count: i64,
}

fn counts<'a>(codes: &[&str], values: impl Iterator<Item = &'a str>) -> Vec<Count> {
    let values: Vec<&str> = values.collect();
    codes.iter().map(|c| Count { code: (*c).into(), count: values.iter().filter(|v| **v == *c).count() as i64 }).filter(|c| c.count > 0).collect()
}

fn big(v: Vec<Count>) -> Vec<Count> {
    v.into_iter().filter(|c| c.count >= MIN_GROUP).collect()
}

/// The indicators of the board: everything counted by code, for the people of the institution (ADR-029).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Indicators {
    pub served: i64,
    pub hospitalized: i64,
    /// By group (sex and age band, or the institution's own).
    pub by_group: Vec<Count>,
    pub by_sex: Vec<Count>,
    /// (band, sex, count): the pyramid.
    pub pyramid: Vec<(String, String, i64)>,
    pub without_age: i64,
    pub average_age: Option<f64>,
    pub dependency: Vec<Count>,
    pub mobility: Vec<Count>,
    pub disabilities: Vec<Count>,
    pub chronic: Vec<Count>,
    pub stay_modes: Vec<Count>,
    pub referred_by: Vec<Count>,
    pub admission_reasons: Vec<Count>,
    /// Average years in the institution of the people served.
    pub average_years: Option<f64>,
    pub admitted_this_year: i64,
    pub discharged_this_year: i64,
    pub deceased_this_year: i64,
    pub discharge_reasons_this_year: Vec<Count>,
    /// People served that are visited rarely or never.
    pub few_visits: i64,
    pub without_contact: i64,
    pub with_program: i64,
    /// People of 65 or more without a pension or social program (they may apply to the Pensión Bienestar).
    pub elderly_without_program: i64,
    pub programs: Vec<Count>,
    pub exempt: i64,
    pub paying: i64,
    /// Average monthly fee of who pays.
    pub average_fee: Option<i64>,
    pub fees_monthly: i64,
    pub without_consent: i64,
    pub indigenous_language: i64,
    /// Children's home: going to school, and behind in it.
    pub attending_school: i64,
    pub school_lag: i64,
    pub legal: Vec<Count>,
    /// Records under half filled.
    pub incomplete: i64,
}

/// The indicators on `today` (`YYYY-MM-DD`). `progress` gives each record's percent.
pub fn indicators(people: &[Member], flavor: Flavor, today: &str, progress: impl Fn(&BeneficiaryData) -> u32) -> Indicators {
    let served: Vec<&Member> = people.iter().filter(|m| m.data.is_served()).collect();
    let year = &today[..4];
    let this_year = |d: &Option<String>| d.as_deref().is_some_and(|d| d.starts_with(year));
    let mut i = Indicators { served: served.len() as i64, ..Default::default() };
    i.hospitalized = served.iter().filter(|m| m.data.status == "hospitalized").count() as i64;
    let labels: Vec<String> = served.iter().map(|m| label_of(m, flavor, today)).collect();
    let mut seen: Vec<String> = Vec::new();
    for l in &labels {
        if !seen.contains(l) {
            seen.push(l.clone());
        }
    }
    i.by_group = seen.iter().map(|l| Count { code: l.clone(), count: labels.iter().filter(|x| *x == l).count() as i64 }).collect();
    i.by_sex = counts(catalog::SEXES, served.iter().filter_map(|m| m.data.sex.as_deref()));
    let ages: Vec<i64> = served.iter().filter_map(|m| m.data.age(today)).collect();
    i.without_age = (served.len() - ages.len()) as i64;
    i.average_age = (!ages.is_empty()).then(|| ages.iter().sum::<i64>() as f64 / ages.len() as f64);
    for (band, _, _) in catalog::age_bands(flavor) {
        for sex in catalog::SEXES {
            let n = served.iter().filter(|m| m.data.sex.as_deref() == Some(*sex) && m.data.age(today).map(|a| catalog::band_of(flavor, a)) == Some(*band)).count() as i64;
            if n > 0 {
                i.pyramid.push(((*band).into(), (*sex).into(), n));
            }
        }
    }
    i.dependency = counts(catalog::DEPENDENCY, served.iter().filter_map(|m| m.data.dependency.as_deref()));
    i.mobility = counts(catalog::MOBILITY, served.iter().filter_map(|m| m.data.mobility.as_deref()));
    i.disabilities = counts(catalog::DISABILITIES, served.iter().flat_map(|m| m.data.disabilities.iter().map(String::as_str)));
    i.chronic = counts(catalog::CHRONIC, served.iter().flat_map(|m| m.data.chronic_conditions.iter().map(String::as_str)));
    i.stay_modes = counts(catalog::STAY_MODES, served.iter().filter_map(|m| m.data.stay_mode.as_deref()));
    i.referred_by = counts(catalog::REFERRED_BY, served.iter().filter_map(|m| m.data.referred_by.as_deref()));
    i.admission_reasons = counts(catalog::ADMISSION_REASONS, served.iter().flat_map(|m| m.data.admission_reasons.iter().map(String::as_str)));
    let years: Vec<i64> = served.iter().filter_map(|m| m.data.entry_date.as_deref().and_then(|d| crate::common::dates::years_between(d, today))).collect();
    i.average_years = (!years.is_empty()).then(|| years.iter().sum::<i64>() as f64 / years.len() as f64);
    i.admitted_this_year = people.iter().filter(|m| this_year(&m.data.entry_date)).count() as i64;
    i.discharged_this_year = people.iter().filter(|m| m.data.status == "discharged" && this_year(&m.data.status_date)).count() as i64;
    i.deceased_this_year = people.iter().filter(|m| m.data.status == "deceased" && this_year(&m.data.status_date)).count() as i64;
    i.discharge_reasons_this_year = counts(
        catalog::DISCHARGE_REASONS,
        people.iter().filter(|m| m.data.status == "discharged" && this_year(&m.data.status_date)).filter_map(|m| m.data.discharge_reason.as_deref()),
    );
    i.few_visits = served.iter().filter(|m| matches!(m.data.visits.as_deref(), Some("rarely" | "never"))).count() as i64;
    i.without_contact = served.iter().filter(|m| !m.data.contacts.iter().any(|c| !c.full_name.trim().is_empty())).count() as i64;
    i.with_program = served.iter().filter(|m| !m.data.programs.is_empty()).count() as i64;
    i.elderly_without_program = served.iter().filter(|m| m.data.programs.is_empty() && m.data.age(today).is_some_and(|a| a >= 65)).count() as i64;
    i.programs = counts(catalog::PROGRAMS, served.iter().flat_map(|m| m.data.programs.iter().map(String::as_str)));
    i.exempt = served.iter().filter(|m| m.data.fee_payer.as_deref() == Some("exempt")).count() as i64;
    let fees: Vec<i64> = served.iter().filter(|m| m.data.pays_fee()).filter_map(|m| m.data.monthly_fee_mxn).collect();
    i.paying = fees.len() as i64;
    i.fees_monthly = fees.iter().sum();
    i.average_fee = (!fees.is_empty()).then(|| i.fees_monthly / fees.len() as i64);
    i.without_consent = served.iter().filter(|m| m.data.consent_date.as_deref().is_none_or(str::is_empty)).count() as i64;
    i.indigenous_language = served.iter().filter(|m| m.data.indigenous_language.as_deref().is_some_and(|l| !l.trim().is_empty())).count() as i64;
    i.attending_school = served.iter().filter(|m| m.data.attends_school == Some(true)).count() as i64;
    i.school_lag = served.iter().filter(|m| m.data.school_lag == Some(true)).count() as i64;
    if flavor == Flavor::ChildrenHome {
        i.legal = counts(catalog::LEGAL_STATUSES, served.iter().filter_map(|m| m.data.legal_status.as_deref()));
    }
    i.incomplete = served.iter().filter(|m| progress(m.data) < 50).count() as i64;
    i
}

/// What the AI may read: the indicators with every personal attribute cut to groups of `MIN_GROUP` or more, and
/// without the money of the fees (the profile already says how many pay) or the internal ones (consent, records).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CareSummary {
    pub served: i64,
    pub by_group: Vec<Count>,
    pub average_age: Option<i64>,
    pub dependency: Vec<Count>,
    pub mobility: Vec<Count>,
    pub disabilities: Vec<Count>,
    pub chronic: Vec<Count>,
    pub stay_modes: Vec<Count>,
    pub admission_reasons: Vec<Count>,
    pub few_visits: Option<i64>,
    pub with_program: Option<i64>,
    pub indigenous_language: Option<i64>,
    pub attending_school: Option<i64>,
    pub school_lag: Option<i64>,
    pub legal: Vec<Count>,
    pub admitted_this_year: i64,
    pub discharged_this_year: i64,
    pub waiting: i64,
}

pub fn ai_summary(i: &Indicators, waiting: i64) -> CareSummary {
    let at_least = |n: i64| (n >= MIN_GROUP).then_some(n);
    CareSummary {
        served: i.served,
        by_group: i.by_group.clone(),
        average_age: (i.served >= MIN_GROUP).then_some(i.average_age.map(|a| a.round() as i64)).flatten(),
        dependency: big(i.dependency.clone()),
        mobility: big(i.mobility.clone()),
        disabilities: big(i.disabilities.clone()),
        chronic: big(i.chronic.clone()),
        stay_modes: big(i.stay_modes.clone()),
        admission_reasons: big(i.admission_reasons.clone()),
        few_visits: at_least(i.few_visits),
        with_program: at_least(i.with_program),
        indigenous_language: at_least(i.indigenous_language),
        attending_school: at_least(i.attending_school),
        school_lag: at_least(i.school_lag),
        legal: big(i.legal.clone()),
        admitted_this_year: i.admitted_this_year,
        discharged_this_year: i.discharged_this_year,
        waiting,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::care::domain::person::ResponsibleContact;

    fn person(sex: &str, birth: &str, f: impl FnOnce(&mut BeneficiaryData)) -> BeneficiaryData {
        let mut d = BeneficiaryData { first_names: "Persona".into(), sex: Some(sex.into()), birth_date: Some(birth.into()), status: "active".into(), ..Default::default() };
        f(&mut d);
        d
    }

    fn members(d: &[BeneficiaryData]) -> Vec<Member<'_>> {
        d.iter().map(|data| Member { data, own_group: None }).collect()
    }

    #[test]
    fn lines_group_people_by_sex_age_support_and_fee_and_leave_out_who_left() {
        let data = vec![
            person("female", "1942-01-01", |d| { d.dependency = Some("high".into()); d.monthly_fee_mxn = Some(3_000); d.fee_payer = Some("family".into()); }),
            person("female", "1945-06-01", |d| { d.dependency = Some("high".into()); d.monthly_fee_mxn = Some(3_000); d.fee_payer = Some("family".into()); }),
            person("female", "1944-01-01", |d| { d.dependency = Some("high".into()); d.fee_payer = Some("exempt".into()); }),
            person("male", "1932-01-01", |d| d.status = "deceased".into()),
        ];
        let lines = population_lines(&members(&data), Flavor::ElderlyHome, "2026-10-07");
        let got: Vec<_> = lines.iter().map(|l| (l.label.as_str(), l.count, l.paying, l.monthly_fee, l.age_min, l.age_max)).collect();
        assert_eq!(got, vec![("Mujeres de 80 a 89 años", 2, 2, Some(3_000), Some(81), Some(84)), ("Mujeres de 80 a 89 años", 1, 0, None, Some(82), Some(82))]);
    }

    #[test]
    fn indicators_count_by_code_and_the_ai_gets_only_groups_of_three() {
        let data = vec![
            person("female", "1940-01-01", |d| { d.mobility = Some("wheelchair".into()); d.chronic_conditions = vec!["diabetes".into()]; d.visits = Some("never".into()); d.entry_date = Some("2026-02-01".into()); }),
            person("female", "1941-01-01", |d| { d.mobility = Some("wheelchair".into()); d.chronic_conditions = vec!["diabetes".into(), "dementia".into()]; d.visits = Some("rarely".into()); }),
            person("male", "1938-01-01", |d| { d.mobility = Some("wheelchair".into()); d.chronic_conditions = vec!["diabetes".into()]; d.programs = vec!["bienestar_elderly".into()]; d.contacts = vec![ResponsibleContact { full_name: "Hijo".into(), ..Default::default() }]; }),
            person("male", "1950-01-01", |d| { d.status = "discharged".into(); d.status_date = Some("2026-05-01".into()); d.discharge_reason = Some("family_reintegration".into()); }),
        ];
        let i = indicators(&members(&data), Flavor::ElderlyHome, "2026-10-07", |_| 10);
        assert_eq!((i.served, i.few_visits, i.without_contact, i.with_program, i.incomplete), (3, 2, 2, 1, 3));
        assert_eq!((i.admitted_this_year, i.discharged_this_year), (1, 1));
        assert_eq!(i.mobility, vec![Count { code: "wheelchair".into(), count: 3 }]);
        assert_eq!(i.chronic, vec![Count { code: "diabetes".into(), count: 3 }, Count { code: "dementia".into(), count: 1 }]);
        assert!(i.pyramid.contains(&("80_89".into(), "female".into(), 2)));

        let ai = ai_summary(&i, 5);
        assert_eq!(ai.chronic, vec![Count { code: "diabetes".into(), count: 3 }], "one person with dementia is not told");
        assert_eq!((ai.few_visits, ai.with_program, ai.waiting), (None, None, 5), "two and one are not told");
        assert_eq!(ai.mobility.len(), 1);
    }
}
