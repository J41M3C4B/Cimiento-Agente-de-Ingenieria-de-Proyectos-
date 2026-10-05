//! Institution profile: only aggregated data (how many, never who). Staff are one anonymous line per position
//! (role, pay, contract; never a name), and people served are grouped, never one by one.

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
pub enum Condition {
    Good,
    Fair,
    Poor,
    Critical,
}

impl Condition {
    pub fn as_db(self) -> &'static str {
        match self {
            Condition::Good => "good",
            Condition::Fair => "fair",
            Condition::Poor => "poor",
            Condition::Critical => "critical",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        Some(match s {
            "good" => Condition::Good,
            "fair" => Condition::Fair,
            "poor" => Condition::Poor,
            "critical" => Condition::Critical,
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FacilityInput {
    pub kind: String,
    pub count: i64,
    pub condition: Option<Condition>,
    pub accessible: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncomeSourceInput {
    pub label: String,
    pub annual_amount_mxn: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileInput {
    pub institution: InstitutionInput,
    pub capacity_total: Option<i64>,
    pub annual_budget_mxn: Option<i64>,
    pub notes: Option<String>,
    #[serde(default)]
    pub population: Vec<PopulationGroupInput>,
    #[serde(default)]
    pub staff: Vec<StaffGroupInput>,
    #[serde(default)]
    pub facilities: Vec<FacilityInput>,
    #[serde(default)]
    pub income: Vec<IncomeSourceInput>,
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
    pub payroll_annual_mxn: i64,
    /// People who pay a stay fee, and what they bring in.
    pub fee_payers: i64,
    pub fees_monthly_mxn: i64,
    pub fees_annual_mxn: i64,
}

impl ProfileInput {
    pub fn totals(&self) -> ProfileTotals {
        let payroll_monthly_mxn: i64 =
            self.staff.iter().filter(|s| s.paid).map(|s| s.count * s.monthly_salary_mxn.unwrap_or(0)).sum();
        let fees_monthly_mxn: i64 =
            self.population.iter().map(|g| g.paying_count.unwrap_or(0) * g.monthly_fee_mxn.unwrap_or(0)).sum();
        ProfileTotals {
            population: self.population.iter().map(|g| g.count).sum(),
            staff_paid: self.staff.iter().filter(|s| s.paid).map(|s| s.count).sum(),
            staff_volunteer: self.staff.iter().filter(|s| !s.paid).map(|s| s.count).sum(),
            income_annual_mxn: self.income.iter().filter_map(|i| i.annual_amount_mxn).sum(),
            payroll_monthly_mxn,
            payroll_annual_mxn: payroll_monthly_mxn * 12,
            fee_payers: self.population.iter().map(|g| g.paying_count.unwrap_or(0)).sum(),
            fees_monthly_mxn,
            fees_annual_mxn: fees_monthly_mxn * 12,
        }
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

    pub fn validate(&self) -> Vec<ProfileIssue> {
        let mut v = Vec::new();
        let mut add = |code, field: String, blocking| v.push(ProfileIssue { code, field, blocking });

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
        for (i, f) in self.facilities.iter().enumerate() {
            if f.kind.trim().is_empty() {
                add("label_missing", format!("facilities[{i}].kind"), true);
            }
            if f.count < 0 {
                add("negative_number", format!("facilities[{i}].count"), true);
            }
        }
        for (i, inc) in self.income.iter().enumerate() {
            if inc.label.trim().is_empty() {
                add("label_missing", format!("income[{i}].label"), true);
            }
            if neg(inc.annual_amount_mxn) {
                add("negative_number", format!("income[{i}].annual_amount_mxn"), true);
            }
        }
        // Heads-ups
        let t = self.totals();
        if let Some(cap) = self.capacity_total {
            if t.population > cap {
                add("population_over_capacity", "population".into(), false);
            }
        }
        if let Some(budget) = self.annual_budget_mxn {
            if t.income_annual_mxn > 0 && t.income_annual_mxn != budget {
                add("income_differs_from_budget", "income".into(), false);
            }
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
        for (i, fa) in self.facilities.iter_mut().enumerate() {
            f(&format!("facilities[{i}].kind"), &mut fa.kind);
            opt(&format!("facilities[{i}].notes"), &mut fa.notes, f);
        }
        for (i, inc) in self.income.iter_mut().enumerate() {
            f(&format!("income[{i}].label"), &mut inc.label);
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
                IncomeSourceInput { label: "Cuotas".into(), annual_amount_mxn: Some(600_000) },
                IncomeSourceInput { label: "Donativos".into(), annual_amount_mxn: Some(400_000) },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn totals_are_computed_in_code() {
        let t = base().totals();
        assert_eq!(
            t,
            ProfileTotals {
                population: 22,
                staff_paid: 3,
                staff_volunteer: 5,
                income_annual_mxn: 1_000_000,
                payroll_monthly_mxn: 0,
                payroll_annual_mxn: 0,
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
        let t = p.totals();
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
        let codes: Vec<_> = p.validate().into_iter().filter(|i| i.blocking).map(|i| (i.code, i.field)).collect();
        assert!(codes.contains(&("paying_over_count", "population[0].paying_count".into())));
        assert!(codes.contains(&("negative_number", "staff[0].monthly_salary_mxn".into())));
        assert!(codes.contains(&("year_invalid", "staff[0].start_year".into())));
    }

    #[test]
    fn valid_profile_has_no_issues() {
        assert!(base().validate().is_empty());
    }

    #[test]
    fn blocking_problems() {
        let mut p = base();
        p.institution.name = "  ".into();
        p.population[0].count = -1;
        p.population[1].age_min = Some(80);
        p.population[1].age_max = Some(60);
        p.staff[0].role.clear();
        let codes: Vec<_> = p.validate().into_iter().filter(|i| i.blocking).map(|i| (i.code, i.field)).collect();
        assert!(codes.contains(&("name_missing", "institution.name".into())));
        assert!(codes.contains(&("negative_number", "population[0].count".into())));
        assert!(codes.contains(&("age_range", "population[1].age_min".into())));
        assert!(codes.contains(&("label_missing", "staff[0].role".into())));
    }

    #[test]
    fn heads_up_when_numbers_do_not_add_up() {
        let mut p = base();
        p.capacity_total = Some(20); // 22 people, 20 beds
        p.income[1].annual_amount_mxn = Some(300_000);
        let issues = p.validate();
        assert!(issues.iter().all(|i| !i.blocking));
        let codes: Vec<_> = issues.iter().map(|i| i.code).collect();
        assert_eq!(codes, vec!["population_over_capacity", "income_differs_from_budget"]);
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
