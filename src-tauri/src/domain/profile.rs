//! Institution profile: only aggregated data (how many, never who). Staff are one anonymous line per position
//! (role, pay, contract; never a name), and people served are grouped, never one by one.

use super::finances;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InstitutionKind {
    ElderlyHome,
    ChildrenHome,
    #[default]
    Other,
}

impl InstitutionKind {
    pub fn as_db(self) -> &'static str {
        match self {
            InstitutionKind::ElderlyHome => "elderly_home",
            InstitutionKind::ChildrenHome => "children_home",
            InstitutionKind::Other => "other",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "elderly_home" => InstitutionKind::ElderlyHome,
            "children_home" => InstitutionKind::ChildrenHome,
            _ => InstitutionKind::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyLevel {
    Low,
    Medium,
    High,
    Total,
}

impl DependencyLevel {
    pub fn as_db(self) -> &'static str {
        match self {
            DependencyLevel::Low => "low",
            DependencyLevel::Medium => "medium",
            DependencyLevel::High => "high",
            DependencyLevel::Total => "total",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        Some(match s {
            "low" => DependencyLevel::Low,
            "medium" => DependencyLevel::Medium,
            "high" => DependencyLevel::High,
            "total" => DependencyLevel::Total,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractKind {
    /// Fixed position with benefits.
    Permanent,
    /// For a set time or project.
    Temporary,
    /// Paid against an invoice (honorarios).
    Fees,
}

impl ContractKind {
    pub fn as_db(self) -> &'static str {
        match self {
            ContractKind::Permanent => "permanent",
            ContractKind::Temporary => "temporary",
            ContractKind::Fees => "fees",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        Some(match s {
            "permanent" => ContractKind::Permanent,
            "temporary" => ContractKind::Temporary,
            "fees" => ContractKind::Fees,
            _ => return None,
        })
    }
}

/// Whether an amount is written per month or per year. The code turns it into a year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    Monthly,
    #[default]
    Annual,
}

impl Period {
    pub fn as_db(self) -> &'static str {
        match self {
            Period::Monthly => "monthly",
            Period::Annual => "annual",
        }
    }
    pub fn from_db(s: &str) -> Self {
        if s == "monthly" { Period::Monthly } else { Period::Annual }
    }
}

/// Where an income written by the person comes from. The stay fees of the roster are not one of these: the app
/// computes them (`finances`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IncomeKind {
    /// What the people served pay, written by hand while the roster has no fees.
    FeeEstimate,
    /// People or companies that give the same amount every month or year.
    RecurringDonor,
    /// People or companies that give when they want.
    OccasionalDonation,
    /// Money won with a project (a call, a foundation).
    ProjectGrant,
    #[default]
    Other,
}

impl IncomeKind {
    pub const ALL: [IncomeKind; 5] =
        [IncomeKind::FeeEstimate, IncomeKind::RecurringDonor, IncomeKind::OccasionalDonation, IncomeKind::ProjectGrant, IncomeKind::Other];
    pub fn as_db(self) -> &'static str {
        match self {
            IncomeKind::FeeEstimate => "fee_estimate",
            IncomeKind::RecurringDonor => "recurring_donor",
            IncomeKind::OccasionalDonation => "occasional_donation",
            IncomeKind::ProjectGrant => "project_grant",
            IncomeKind::Other => "other",
        }
    }
    pub fn from_db(s: &str) -> Self {
        IncomeKind::ALL.into_iter().find(|k| k.as_db() == s).unwrap_or_default()
    }
}

/// Institutional data. Contact fields are institutional, never sent to the AI,
/// and the scanner does not look at them (see `03-gobernanza-datos.md`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstitutionInput {
    pub name: String,
    pub kind: InstitutionKind,
    pub mission: Option<String>,
    pub legal_rfc: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub legal_rep_name: Option<String>,
    /// Where it is (ADR-031): a code of `onboarding::STATES` and the municipality. Not personal: it reaches the AI.
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub municipality: Option<String>,
    #[serde(default)]
    pub founded_year: Option<i64>,
    /// A code of `onboarding::LEGAL_FORMS`.
    #[serde(default)]
    pub legal_form: Option<String>,
    /// Donataria autorizada (SAT) and CLUNI: `yes`, `in_progress` or `no`.
    #[serde(default)]
    pub authorized_donee: Option<String>,
    #[serde(default)]
    pub cluni: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationGroupInput {
    pub label: String,
    pub age_min: Option<i64>,
    pub age_max: Option<i64>,
    pub count: i64,
    pub dependency_level: Option<DependencyLevel>,
    pub notes: Option<String>,
    /// How many of the group pay a stay fee (the rest do not).
    #[serde(default)]
    pub paying_count: Option<i64>,
    /// Monthly stay fee per paying person, in pesos.
    #[serde(default)]
    pub monthly_fee_mxn: Option<i64>,
}

/// A position: one person when `count` is 1 (what the app writes); older data may group several.
/// The pay is per person. There is never a name here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StaffGroupInput {
    pub role: String,
    pub count: i64,
    pub shift: Option<String>,
    pub paid: bool,
    #[serde(default)]
    pub monthly_salary_mxn: Option<i64>,
    #[serde(default)]
    pub contract: Option<ContractKind>,
    #[serde(default)]
    pub start_year: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
    /// The kind of relation of the staff module (`employee`, `fees`, `religious`, `volunteer`, `trainee`,
    /// `external`; ADR-027): it says where the money of the line counts. `None` in data older than the module.
    #[serde(default)]
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncomeSourceInput {
    pub label: String,
    #[serde(default)]
    pub kind: IncomeKind,
    /// As the person wrote it, per `period`.
    pub amount_mxn: Option<i64>,
    #[serde(default)]
    pub period: Period,
}

/// One concept of what the institution spends (food, utilities…). The payroll is not written here: the app
/// computes it from the roster.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpenseItemInput {
    pub label: String,
    pub amount_mxn: Option<i64>,
    #[serde(default)]
    pub period: Period,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileInput {
    pub institution: InstitutionInput,
    pub capacity_total: Option<i64>,
    /// «Gasto anual aproximado»: what the institution spends in a year, all included, as one approximate figure.
    /// It is the quick way to start; once the list of expenses has a line, the list is the total (ADR-026).
    pub annual_budget_mxn: Option<i64>,
    pub notes: Option<String>,
    /// Quick figures said by the person (ADR-031): how many people are served and work there while their records
    /// are not in the modules yet. The records win when they exist.
    #[serde(default)]
    pub served_estimate: Option<i64>,
    #[serde(default)]
    pub staff_paid_estimate: Option<i64>,
    #[serde(default)]
    pub staff_volunteer_estimate: Option<i64>,
    #[serde(default)]
    pub population: Vec<PopulationGroupInput>,
    #[serde(default)]
    pub staff: Vec<StaffGroupInput>,
    #[serde(default)]
    pub income: Vec<IncomeSourceInput>,
    #[serde(default)]
    pub expenses: Vec<ExpenseItemInput>,
}

/// Something the person should look at. The UI turns `code` into friendly text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileIssue {
    pub code: &'static str,
    pub field: String,
    /// `true` blocks saving; `false` is only a heads-up ("algo no cuadra").
    pub blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileTotals {
    pub population: i64,
    pub staff_paid: i64,
    pub staff_volunteer: i64,
    pub income_annual_mxn: i64,
    /// Sum of the monthly pay of the people who receive a salary.
    pub payroll_monthly_mxn: i64,
    /// The pay of twelve months, without benefits.
    pub payroll_annual_mxn: i64,
    /// Aguinaldo and vacation premium of the jobs that carry them (not fees).
    pub payroll_benefits_annual_mxn: i64,
    /// What the payroll costs in a year: pay plus benefits.
    pub payroll_cost_annual_mxn: i64,
    /// Paid people whose benefits rest on an assumption (no contract or no start year in the roster).
    pub benefits_assumed: i64,
    /// Contributions to the congregation and grants of social service, in a year (not payroll).
    pub staff_support_annual_mxn: i64,
    /// What the outside companies bill for their staff, in a year (not payroll).
    pub external_staff_annual_mxn: i64,
    /// People who pay a stay fee, and what they bring in.
    pub fee_payers: i64,
    pub fees_monthly_mxn: i64,
    pub fees_annual_mxn: i64,
}

fn normalized(s: &str) -> String {
    s.to_lowercase().replace(['á', 'à'], "a").replace('é', "e").replace('í', "i").replace('ó', "o").replace(['ú', 'ü'], "u")
}

/// A Mexican RFC: three letters (moral person) or four (physical), six digits of the date and a three-character
/// key. Spaces and dashes are ignored.
fn rfc_looks_right(s: &str) -> bool {
    let c: Vec<char> = s.to_uppercase().chars().filter(|c| !c.is_whitespace() && *c != '-').collect();
    if !(12..=13).contains(&c.len()) {
        return false;
    }
    let letters = c.len() - 9;
    c[..letters].iter().all(|c| c.is_ascii_uppercase() || *c == 'Ñ' || *c == '&')
        && c[letters..letters + 6].iter().all(|c| c.is_ascii_digit())
        && c[letters + 6..].iter().all(|c| c.is_ascii_alphanumeric())
}

/// Ten digits (a Mexican number), or up to thirteen with the country code or an extension.
fn phone_looks_right(s: &str) -> bool {
    (10..=13).contains(&s.chars().filter(char::is_ascii_digit).count())
}

fn email_looks_right(s: &str) -> bool {
    let s = s.trim();
    match s.split_once('@') {
        Some((user, domain)) => {
            !user.is_empty()
                && !domain.contains('@')
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !s.contains(char::is_whitespace)
        }
        None => false,
    }
}

impl ProfileInput {
    /// The sums of the profile in `year` (the year sets how long each person has worked, for the vacation premium).
    pub fn totals(&self, year: i64) -> ProfileTotals {
        let payroll_monthly_mxn: i64 =
            self.staff.iter().filter(|s| s.paid).map(|s| s.count * s.monthly_salary_mxn.unwrap_or(0)).sum();
        let mut payroll_benefits_annual_mxn = 0;
        let mut benefits_assumed = 0;
        for s in self.staff.iter().filter(|s| s.paid && s.monthly_salary_mxn.unwrap_or(0) > 0) {
            if !finances::has_benefits(s.contract) {
                continue;
            }
            let years = s.start_year.map_or(1, |y| year - y);
            payroll_benefits_annual_mxn += s.count * finances::annual_benefits(s.monthly_salary_mxn.unwrap_or(0), years);
            if s.contract.is_none() || s.start_year.is_none() {
                benefits_assumed += s.count;
            }
        }
        let fees_monthly_mxn: i64 =
            self.population.iter().map(|g| g.paying_count.unwrap_or(0) * g.monthly_fee_mxn.unwrap_or(0)).sum();
        ProfileTotals {
            population: self.population.iter().map(|g| g.count).sum(),
            staff_paid: self.staff.iter().filter(|s| s.paid).map(|s| s.count).sum(),
            staff_volunteer: self.staff.iter().filter(|s| !s.paid).map(|s| s.count).sum(),
            income_annual_mxn: self.income_annual(fees_monthly_mxn * 12),
            payroll_monthly_mxn,
            payroll_annual_mxn: payroll_monthly_mxn * 12,
            payroll_benefits_annual_mxn,
            payroll_cost_annual_mxn: payroll_monthly_mxn * 12 + payroll_benefits_annual_mxn,
            benefits_assumed,
            staff_support_annual_mxn: self.unpaid_staff_annual(&["religious", "trainee"]),
            external_staff_annual_mxn: self.unpaid_staff_annual(&["external"]),
            fee_payers: self.population.iter().map(|g| g.paying_count.unwrap_or(0)).sum(),
            fees_monthly_mxn,
            fees_annual_mxn: fees_monthly_mxn * 12,
        }
    }

    /// What the lines of staff outside the payroll with one of these relations cost in a year.
    fn unpaid_staff_annual(&self, relations: &[&str]) -> i64 {
        self.staff
            .iter()
            .filter(|s| !s.paid && s.relation.as_deref().is_some_and(|r| relations.contains(&r)))
            .map(|s| s.count * s.monthly_salary_mxn.unwrap_or(0) * 12)
            .sum()
    }

    /// Counted income in a year: the roster fees, and the written lines (a fee estimate only while the roster has no
    /// fees). The same rule as `finances`, which also gives the detail.
    fn income_annual(&self, roster_fees_annual: i64) -> i64 {
        let written: i64 = self
            .income
            .iter()
            .filter(|i| !(roster_fees_annual > 0 && i.kind == IncomeKind::FeeEstimate))
            .filter_map(|i| i.amount_mxn.map(|a| finances::annual(a, i.period)))
            .sum();
        roster_fees_annual + written
    }

    /// People served counted by group (the same group may come in several lines when its fees differ).
    pub fn population_by_label(&self) -> Vec<(String, i64)> {
        let mut out: Vec<(String, i64)> = Vec::new();
        for g in &self.population {
            match out.iter_mut().find(|(l, _)| *l == g.label) {
                Some((_, n)) => *n += g.count,
                None => out.push((g.label.clone(), g.count)),
            }
        }
        out
    }

    /// Problems that stop the save (`blocking`) and heads-ups, in `year` (see `totals`).
    pub fn validate(&self, year: i64) -> Vec<ProfileIssue> {
        let mut v = Vec::new();
        let mut add = |code, field: String, blocking| v.push(ProfileIssue { code, field, blocking });
        let too_large = |n: Option<i64>| n.map_or(false, |x| x > finances::MAX_MXN);

        if self.institution.name.trim().is_empty() {
            add("name_missing", "institution.name".into(), true);
        }
        let neg = |n: Option<i64>| n.map_or(false, |x| x < 0);
        if neg(self.capacity_total) {
            add("negative_number", "capacity_total".into(), true);
        }
        if neg(self.annual_budget_mxn) {
            add("negative_number", "annual_budget_mxn".into(), true);
        }
        if too_large(self.annual_budget_mxn) {
            add("amount_too_large", "annual_budget_mxn".into(), true);
        }
        for (field, n) in [("served_estimate", self.served_estimate), ("staff_paid_estimate", self.staff_paid_estimate), ("staff_volunteer_estimate", self.staff_volunteer_estimate)] {
            if neg(n) {
                add("negative_number", field.into(), true);
            } else if n.is_some_and(|x| x > super::onboarding::MAX_ESTIMATE) {
                add("number_too_large", field.into(), true);
            }
        }
        let inst = &self.institution;
        if inst.founded_year.is_some_and(|y| !(super::onboarding::OLDEST_YEAR..=year).contains(&y)) {
            add("year_invalid", "institution.founded_year".into(), true);
        }
        let known = |v: &Option<String>, list: &[&str]| v.as_deref().is_none_or(|x| x.trim().is_empty() || list.contains(&x));
        if !known(&inst.state, &super::onboarding::STATES.iter().map(|(c, _)| *c).collect::<Vec<_>>()) {
            add("code_unknown", "institution.state".into(), true);
        }
        for (field, value, list) in [
            ("institution.legal_form", &inst.legal_form, super::onboarding::LEGAL_FORMS),
            ("institution.authorized_donee", &inst.authorized_donee, super::onboarding::REGISTRY),
            ("institution.cluni", &inst.cluni, super::onboarding::REGISTRY),
        ] {
            if !known(value, list) {
                add("code_unknown", field.into(), true);
            }
        }
        if let (Some(served), Some(capacity)) = (self.served_estimate, self.capacity_total) {
            if served > capacity {
                add("served_over_capacity", "served_estimate".into(), false);
            }
        }
        for (i, g) in self.population.iter().enumerate() {
            let p = format!("population[{i}]");
            if g.label.trim().is_empty() {
                add("label_missing", format!("{p}.label"), true);
            }
            if g.count < 0 {
                add("negative_number", format!("{p}.count"), true);
            }
            if let (Some(a), Some(b)) = (g.age_min, g.age_max) {
                if a > b {
                    add("age_range", format!("{p}.age_min"), true);
                }
            }
            if neg(g.age_min) || neg(g.age_max) {
                add("negative_number", format!("{p}.age_min"), true);
            }
            if neg(g.paying_count) {
                add("negative_number", format!("{p}.paying_count"), true);
            }
            if neg(g.monthly_fee_mxn) {
                add("negative_number", format!("{p}.monthly_fee_mxn"), true);
            }
            if g.paying_count.map_or(false, |n| n > g.count) {
                add("paying_over_count", format!("{p}.paying_count"), true);
            }
        }
        for (i, s) in self.staff.iter().enumerate() {
            if s.role.trim().is_empty() {
                add("label_missing", format!("staff[{i}].role"), true);
            }
            if s.count < 0 {
                add("negative_number", format!("staff[{i}].count"), true);
            }
            if neg(s.monthly_salary_mxn) {
                add("negative_number", format!("staff[{i}].monthly_salary_mxn"), true);
            }
            if s.start_year.map_or(false, |y| !(1900..=2100).contains(&y)) {
                add("year_invalid", format!("staff[{i}].start_year"), true);
            }
        }
        for (i, inc) in self.income.iter().enumerate() {
            if inc.label.trim().is_empty() {
                add("label_missing", format!("income[{i}].label"), true);
            }
            if neg(inc.amount_mxn) {
                add("negative_number", format!("income[{i}].amount_mxn"), true);
            }
            if too_large(inc.amount_mxn) {
                add("amount_too_large", format!("income[{i}].amount_mxn"), true);
            }
        }
        for (i, e) in self.expenses.iter().enumerate() {
            if e.label.trim().is_empty() {
                add("label_missing", format!("expenses[{i}].label"), true);
            }
            if neg(e.amount_mxn) {
                add("negative_number", format!("expenses[{i}].amount_mxn"), true);
            }
            if too_large(e.amount_mxn) {
                add("amount_too_large", format!("expenses[{i}].amount_mxn"), true);
            }
        }
        for (i, s) in self.staff.iter().enumerate() {
            if too_large(s.monthly_salary_mxn) {
                add("amount_too_large", format!("staff[{i}].monthly_salary_mxn"), true);
            }
        }
        for (i, g) in self.population.iter().enumerate() {
            if too_large(g.monthly_fee_mxn) {
                add("amount_too_large", format!("population[{i}].monthly_fee_mxn"), true);
            }
        }
        if v.iter().any(|i| i.blocking) {
            // the sums below would rest on numbers that are about to be corrected
            return v;
        }
        let mut add = |code, field: String, blocking| v.push(ProfileIssue { code, field, blocking });
        // Heads-ups
        let t = self.totals(year);
        if let Some(cap) = self.capacity_total {
            if t.population > cap {
                add("population_over_capacity", "population".into(), false);
            }
        }
        if t.fees_annual_mxn > 0 {
            if let Some(i) = self.income.iter().position(|i| i.kind == IncomeKind::FeeEstimate) {
                add("fee_estimate_ignored", format!("income[{i}]"), false);
            }
        }
        if self.expenses.is_empty() {
            if let Some(estimate) = self.annual_budget_mxn {
                if t.payroll_cost_annual_mxn > estimate {
                    add("payroll_over_estimate", "annual_budget_mxn".into(), false);
                }
            }
        }
        if t.payroll_cost_annual_mxn > 0 {
            let payroll_words = ["nomina", "sueldo", "salario", "aguinaldo"];
            if let Some(i) = self.expenses.iter().position(|e| payroll_words.iter().any(|w| normalized(&e.label).contains(w))) {
                add("expense_looks_like_payroll", format!("expenses[{i}].label"), false);
            }
        }
        let filled = |o: &Option<String>| o.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        let inst = &self.institution;
        if filled(&inst.legal_rfc).is_some_and(|r| !rfc_looks_right(&r)) {
            add("rfc_format", "institution.legal_rfc".into(), false);
        }
        if filled(&inst.contact_phone).is_some_and(|p| !phone_looks_right(&p)) {
            add("phone_format", "institution.contact_phone".into(), false);
        }
        if filled(&inst.contact_email).is_some_and(|e| !email_looks_right(&e)) {
            add("email_format", "institution.contact_email".into(), false);
        }
        v
    }

    /// Visits every free-text field that must be scanned (institutional contact data excluded).
    pub fn for_each_text_mut(&mut self, f: &mut dyn FnMut(&str, &mut String)) {
        fn opt(path: &str, o: &mut Option<String>, f: &mut dyn FnMut(&str, &mut String)) {
            if let Some(s) = o {
                f(path, s);
            }
        }
        opt("institution.mission", &mut self.institution.mission, f);
        opt("institution.municipality", &mut self.institution.municipality, f);
        opt("notes", &mut self.notes, f);
        for (i, g) in self.population.iter_mut().enumerate() {
            f(&format!("population[{i}].label"), &mut g.label);
            opt(&format!("population[{i}].notes"), &mut g.notes, f);
        }
        for (i, s) in self.staff.iter_mut().enumerate() {
            f(&format!("staff[{i}].role"), &mut s.role);
            opt(&format!("staff[{i}].shift"), &mut s.shift, f);
            opt(&format!("staff[{i}].notes"), &mut s.notes, f);
        }
        for (i, inc) in self.income.iter_mut().enumerate() {
            f(&format!("income[{i}].label"), &mut inc.label);
        }
        for (i, e) in self.expenses.iter_mut().enumerate() {
            f(&format!("expenses[{i}].label"), &mut e.label);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> ProfileInput {
        ProfileInput {
            institution: InstitutionInput { name: "Casa Hogar Ficticia".into(), ..Default::default() },
            capacity_total: Some(25),
            annual_budget_mxn: Some(1_000_000),
            population: vec![
                PopulationGroupInput { label: "Adultos mayores".into(), count: 18, ..Default::default() },
                PopulationGroupInput { label: "Con dependencia alta".into(), count: 4, ..Default::default() },
            ],
            staff: vec![
                StaffGroupInput { role: "Enfermería".into(), count: 3, paid: true, ..Default::default() },
                StaffGroupInput { role: "Voluntariado".into(), count: 5, paid: false, ..Default::default() },
            ],
            income: vec![
                IncomeSourceInput { label: "Cuotas".into(), kind: IncomeKind::FeeEstimate, amount_mxn: Some(600_000), period: Period::Annual },
                IncomeSourceInput { label: "Donativos".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(400_000), period: Period::Annual },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn totals_are_computed_in_code() {
        let t = base().totals(2026);
        assert_eq!(
            t,
            ProfileTotals {
                population: 22,
                staff_paid: 3,
                staff_volunteer: 5,
                income_annual_mxn: 1_000_000,
                payroll_monthly_mxn: 0,
                payroll_annual_mxn: 0,
                payroll_benefits_annual_mxn: 0,
                payroll_cost_annual_mxn: 0,
                benefits_assumed: 0,
                staff_support_annual_mxn: 0,
                external_staff_annual_mxn: 0,
                fee_payers: 0,
                fees_monthly_mxn: 0,
                fees_annual_mxn: 0,
            }
        );
    }

    #[test]
    fn payroll_counts_only_paid_people_and_fees_only_those_who_pay() {
        let mut p = base();
        p.staff[0].monthly_salary_mxn = Some(9_000); // 3 nurses with pay
        p.staff[1].monthly_salary_mxn = Some(5_000); // volunteers: never part of the payroll
        p.population[0].paying_count = Some(10);
        p.population[0].monthly_fee_mxn = Some(2_500);
        let t = p.totals(2026);
        assert_eq!(t.payroll_monthly_mxn, 27_000);
        assert_eq!(t.payroll_annual_mxn, 324_000);
        assert_eq!((t.fee_payers, t.fees_monthly_mxn, t.fees_annual_mxn), (10, 25_000, 300_000));
    }

    #[test]
    fn more_payers_than_people_and_odd_numbers_are_blocked() {
        let mut p = base();
        p.population[0].paying_count = Some(19); // the group has 18
        p.staff[0].monthly_salary_mxn = Some(-1);
        p.staff[0].start_year = Some(20);
        let codes: Vec<_> = p.validate(2026).into_iter().filter(|i| i.blocking).map(|i| (i.code, i.field)).collect();
        assert!(codes.contains(&("paying_over_count", "population[0].paying_count".into())));
        assert!(codes.contains(&("negative_number", "staff[0].monthly_salary_mxn".into())));
        assert!(codes.contains(&("year_invalid", "staff[0].start_year".into())));
    }

    #[test]
    fn valid_profile_has_no_issues() {
        assert!(base().validate(2026).is_empty());
    }

    #[test]
    fn blocking_problems() {
        let mut p = base();
        p.institution.name = "  ".into();
        p.population[0].count = -1;
        p.population[1].age_min = Some(80);
        p.population[1].age_max = Some(60);
        p.staff[0].role.clear();
        let codes: Vec<_> = p.validate(2026).into_iter().filter(|i| i.blocking).map(|i| (i.code, i.field)).collect();
        assert!(codes.contains(&("name_missing", "institution.name".into())));
        assert!(codes.contains(&("negative_number", "population[0].count".into())));
        assert!(codes.contains(&("age_range", "population[1].age_min".into())));
        assert!(codes.contains(&("label_missing", "staff[0].role".into())));
    }

    #[test]
    fn heads_up_when_numbers_do_not_add_up() {
        let mut p = base();
        p.capacity_total = Some(20); // 22 people, 20 beds
        p.staff[0].monthly_salary_mxn = Some(30_000); // 3 x 30,000 x 12 = 1,080,000 + benefits: over the estimate
        let issues = p.validate(2026);
        assert!(issues.iter().all(|i| !i.blocking));
        let codes: Vec<_> = issues.iter().map(|i| i.code).collect();
        assert_eq!(codes, vec!["population_over_capacity", "payroll_over_estimate"]);
        // income that differs from what is spent is not «something wrong»: it is the balance
        p.income[1].amount_mxn = Some(1);
        p.staff[0].monthly_salary_mxn = None;
        p.capacity_total = None;
        assert!(p.validate(2026).is_empty());
    }

    #[test]
    fn heads_up_for_fees_counted_twice_and_payroll_written_as_an_expense() {
        let mut p = base();
        p.population[0].paying_count = Some(10);
        p.population[0].monthly_fee_mxn = Some(2_000);
        p.staff[0].monthly_salary_mxn = Some(9_000);
        p.annual_budget_mxn = None;
        p.expenses = vec![
            ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(10_000), period: Period::Monthly },
            ExpenseItemInput { label: "Nómina y aguinaldos".into(), amount_mxn: Some(300_000), period: Period::Annual },
        ];
        let issues = p.validate(2026);
        let found: Vec<_> = issues.iter().map(|i| (i.code, i.field.as_str(), i.blocking)).collect();
        assert_eq!(found, vec![("fee_estimate_ignored", "income[0]", false), ("expense_looks_like_payroll", "expenses[1].label", false)]);
    }

    #[test]
    fn amounts_out_of_reason_and_expenses_without_name_are_blocked() {
        let mut p = base();
        p.income[0].amount_mxn = Some(finances::MAX_MXN + 1);
        p.expenses = vec![ExpenseItemInput { label: " ".into(), amount_mxn: Some(-5), period: Period::Annual }];
        let codes: Vec<_> = p.validate(2026).into_iter().filter(|i| i.blocking).map(|i| (i.code, i.field)).collect();
        assert!(codes.contains(&("amount_too_large", "income[0].amount_mxn".into())));
        assert!(codes.contains(&("label_missing", "expenses[0].label".into())));
        assert!(codes.contains(&("negative_number", "expenses[0].amount_mxn".into())));
    }

    #[test]
    fn contact_and_rfc_that_do_not_look_right_are_a_heads_up_never_a_block() {
        let mut p = base();
        for (rfc, phone, email) in [("AHE200101AB1", "55 5555 0101", "contacto@asilo.org"), ("ROHL800101AB1", "+52 1 55 5555 0101", "a@b.mx"), ("ahe-200101-ab1", "(55) 5555-0101 ext 12", " x@y.com ")] {
            p.institution.legal_rfc = Some(rfc.into());
            p.institution.contact_phone = Some(phone.into());
            p.institution.contact_email = Some(email.into());
            assert!(p.validate(2026).is_empty(), "{rfc} {phone} {email}: {:?}", p.validate(2026));
        }
        p.institution.legal_rfc = Some("AHE2001".into());
        p.institution.contact_phone = Some("5555".into());
        p.institution.contact_email = Some("contacto.asilo.org".into());
        let issues = p.validate(2026);
        assert!(issues.iter().all(|i| !i.blocking));
        let codes: Vec<_> = issues.iter().map(|i| i.code).collect();
        assert_eq!(codes, vec!["rfc_format", "phone_format", "email_format"]);
    }

    #[test]
    fn contributions_grants_and_outside_staff_are_not_payroll() {
        let mut p = base();
        p.staff = vec![
            StaffGroupInput { role: "Pastoral".into(), count: 2, paid: false, monthly_salary_mxn: Some(1_000), relation: Some("religious".into()), ..Default::default() },
            StaffGroupInput { role: "Psicología".into(), count: 1, paid: false, monthly_salary_mxn: Some(2_000), relation: Some("trainee".into()), ..Default::default() },
            StaffGroupInput { role: "Vigilancia".into(), count: 2, paid: false, monthly_salary_mxn: Some(8_000), relation: Some("external".into()), ..Default::default() },
            StaffGroupInput { role: "Acompañamiento".into(), count: 4, paid: false, monthly_salary_mxn: Some(500), relation: Some("volunteer".into()), ..Default::default() },
        ];
        let t = p.totals(2026);
        assert_eq!((t.payroll_cost_annual_mxn, t.staff_support_annual_mxn, t.external_staff_annual_mxn), (0, 48_000, 192_000));
        p.expenses = vec![ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(1_000), period: Period::Annual }];
        let f = p.finances(2026);
        let kinds: Vec<_> = f.expenses.iter().map(|l| (l.kind, l.annual_mxn, l.counted)).collect();
        assert_eq!(kinds, vec![("staff_support", Some(48_000), true), ("external_staff", Some(192_000), true), ("expense", Some(1_000), true)]);
        assert_eq!(f.expenses_annual_mxn, Some(241_000));
    }

    #[test]
    fn text_visitor_skips_institutional_contact() {
        let mut p = base();
        p.institution.contact_phone = Some("55 1234 5678".into());
        p.institution.legal_rep_name = Some("Rosa Hernández López".into());
        p.institution.mission = Some("Cuidar".into());
        let mut seen = Vec::new();
        p.for_each_text_mut(&mut |path, _| seen.push(path.to_string()));
        assert!(seen.contains(&"institution.mission".to_string()));
        assert!(seen.contains(&"population[0].label".to_string()));
        assert!(!seen.iter().any(|s| s.contains("contact") || s.contains("legal_rep") || s.contains("name")));
    }
}
