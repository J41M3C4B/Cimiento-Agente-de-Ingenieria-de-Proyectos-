//! The money of «Mi institución» (ADR-026): what comes in by kind, what goes out (a list by concept or, to start,
//! one approximate figure), the payroll with the benefits the law requires, and the balance. Every figure is
//! made here, by code; the screen and the AI only read it.

use super::profile::{ContractKind, IncomeKind, Period, ProfileInput};
use serde::Serialize;

/// The largest amount a person can write (one hundred thousand million pesos): more is a typing mistake, and it
/// keeps every sum far from overflowing.
pub const MAX_MXN: i64 = 100_000_000_000;

/// Reads an amount the way a person writes it: `1800000`, `1,800,000`, `$1 800 000` or `1.800.000`. Separators
/// are accepted only between groups of three digits; anything else (letters, cents, signs) is not an amount.
pub fn parse_pesos(s: &str) -> Option<i64> {
    let s = s.trim();
    let s = s.strip_prefix('$').unwrap_or(s).trim_start();
    if s.is_empty() {
        return None;
    }
    let groups: Vec<&str> = s.split([',', ' ', '.']).collect();
    let first = groups[0];
    let ok = !first.is_empty()
        && first.bytes().all(|b| b.is_ascii_digit())
        && (groups.len() == 1 || first.len() <= 3)
        && groups[1..].iter().all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit()));
    if !ok {
        return None;
    }
    groups.concat().parse().ok()
}

/// What an amount is worth in a year.
pub fn annual(amount: i64, period: Period) -> i64 {
    match period {
        Period::Monthly => amount * 12,
        Period::Annual => amount,
    }
}

/// Paid vacation days for a year of service (Ley Federal del Trabajo, art. 76, reformed in 2023): 12 the first
/// year, two more each year up to 20 in the fifth, then two more every five years. `years` below 1 counts as 1.
pub fn vacation_days(years: i64) -> i64 {
    let n = years.max(1);
    if n <= 5 {
        12 + 2 * (n - 1)
    } else {
        20 + 2 * ((n - 5 + 4) / 5)
    }
}

/// Christmas bonus by law: at least 15 days of pay.
pub const AGUINALDO_DAYS: i64 = 15;

/// What the benefits of one person with a monthly pay add in a year: the aguinaldo (15 days) and the vacation
/// premium (25 % of the vacation days). A day of pay is the month divided by 30; rounded to whole pesos.
pub fn annual_benefits(monthly_salary: i64, years_of_service: i64) -> i64 {
    let aguinaldo = (monthly_salary * AGUINALDO_DAYS + 15) / 30;
    let premium = (monthly_salary * vacation_days(years_of_service) + 60) / 120;
    aguinaldo + premium
}

/// Whether a contract carries the benefits of the law. Fees (honorarios) do not; an unknown contract is taken as
/// a job with benefits, which is the safe side for a budget.
pub fn has_benefits(contract: Option<ContractKind>) -> bool {
    !matches!(contract, Some(ContractKind::Fees))
}

/// How the total of expenses was reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseBasis {
    /// The list by concept, plus the payroll the app computes.
    List,
    /// The one approximate figure the person wrote (the quick way to start).
    Estimate,
    /// Nothing to go on yet.
    Unknown,
}

/// One line of money as the screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FinanceLine {
    pub label: String,
    /// Income: an `IncomeKind` (`fee_estimate`, `recurring_donor`, …) or `beneficiary_fees` (from the roster).
    /// Expenses: `expense` or `payroll` (from the roster).
    pub kind: &'static str,
    /// Position in `input.income` / `input.expenses`; `None` for a line the app computes and nobody edits.
    pub index: Option<usize>,
    pub annual_mxn: Option<i64>,
    /// Whether it is part of the total. A fee estimate is left out once the roster has the real fees, and the
    /// payroll is left out while the total is the approximate figure (that figure already includes it).
    pub counted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KindTotal {
    pub kind: &'static str,
    pub annual_mxn: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finances {
    pub income: Vec<FinanceLine>,
    /// Counted income by kind, in the order of `IncomeKind` (the roster fees first).
    pub income_by_kind: Vec<KindTotal>,
    /// What comes every month: the stay fees and the regular donors.
    pub income_fixed_annual_mxn: i64,
    /// What comes when it comes: occasional donations, projects won and the rest.
    pub income_variable_annual_mxn: i64,
    pub income_annual_mxn: i64,
    /// Whether anything with an amount is known about the income.
    pub income_known: bool,
    pub expenses: Vec<FinanceLine>,
    pub expenses_basis: ExpenseBasis,
    pub expenses_annual_mxn: Option<i64>,
    /// Income minus expenses in a year; `None` while either side is unknown.
    pub balance_annual_mxn: Option<i64>,
}

pub const BENEFICIARY_FEES: &str = "beneficiary_fees";
pub const PAYROLL: &str = "payroll";
pub const EXPENSE: &str = "expense";

impl ProfileInput {
    /// The money of the institution in `year` (the year sets how long each person has worked).
    pub fn finances(&self, year: i64) -> Finances {
        let t = self.totals(year);
        let roster_fees = t.fees_annual_mxn > 0;

        let mut income = Vec::new();
        if roster_fees {
            income.push(FinanceLine {
                label: "Cuotas de los beneficiarios (del padrón)".into(),
                kind: BENEFICIARY_FEES,
                index: None,
                annual_mxn: Some(t.fees_annual_mxn),
                counted: true,
            });
        }
        for (i, inc) in self.income.iter().enumerate() {
            income.push(FinanceLine {
                label: inc.label.clone(),
                kind: inc.kind.as_db(),
                index: Some(i),
                annual_mxn: inc.amount_mxn.map(|a| annual(a, inc.period)),
                counted: !(roster_fees && inc.kind == IncomeKind::FeeEstimate),
            });
        }
        let counted = |l: &&FinanceLine| l.counted;
        let sum = |kind: &str| -> i64 {
            income.iter().filter(counted).filter(|l| l.kind == kind).filter_map(|l| l.annual_mxn).sum()
        };
        let mut income_by_kind = Vec::new();
        for kind in std::iter::once(BENEFICIARY_FEES).chain(IncomeKind::ALL.iter().map(|k| k.as_db())) {
            if income.iter().filter(counted).any(|l| l.kind == kind && l.annual_mxn.is_some()) {
                income_by_kind.push(KindTotal { kind, annual_mxn: sum(kind) });
            }
        }
        let fixed = sum(BENEFICIARY_FEES) + sum(IncomeKind::FeeEstimate.as_db()) + sum(IncomeKind::RecurringDonor.as_db());
        let income_annual_mxn: i64 = income.iter().filter(counted).filter_map(|l| l.annual_mxn).sum();
        let income_known = income.iter().filter(counted).any(|l| l.annual_mxn.is_some());

        let listed = !self.expenses.is_empty();
        let basis = if listed {
            ExpenseBasis::List
        } else if self.annual_budget_mxn.is_some() {
            ExpenseBasis::Estimate
        } else {
            ExpenseBasis::Unknown
        };
        let mut expenses = Vec::new();
        if t.payroll_cost_annual_mxn > 0 {
            expenses.push(FinanceLine {
                label: "Nómina del personal con prestaciones (del padrón)".into(),
                kind: PAYROLL,
                index: None,
                annual_mxn: Some(t.payroll_cost_annual_mxn),
                counted: basis == ExpenseBasis::List,
            });
        }
        for (i, e) in self.expenses.iter().enumerate() {
            expenses.push(FinanceLine {
                label: e.label.clone(),
                kind: EXPENSE,
                index: Some(i),
                annual_mxn: e.amount_mxn.map(|a| annual(a, e.period)),
                counted: true,
            });
        }
        let expenses_annual_mxn = match basis {
            ExpenseBasis::List => Some(expenses.iter().filter(counted).filter_map(|l| l.annual_mxn).sum()),
            ExpenseBasis::Estimate => self.annual_budget_mxn,
            ExpenseBasis::Unknown => None,
        };
        let balance_annual_mxn = match (income_known, expenses_annual_mxn) {
            (true, Some(e)) => Some(income_annual_mxn - e),
            _ => None,
        };

        Finances {
            income,
            income_by_kind,
            income_fixed_annual_mxn: fixed,
            income_variable_annual_mxn: income_annual_mxn - fixed,
            income_annual_mxn,
            income_known,
            expenses,
            expenses_basis: basis,
            expenses_annual_mxn,
            balance_annual_mxn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::*;

    #[test]
    fn amounts_are_read_as_people_write_them() {
        for (text, n) in [("1800000", 1_800_000), ("1,800,000", 1_800_000), ("$1 800 000", 1_800_000), ("$ 950", 950), ("1.800.000", 1_800_000), (" 0 ", 0)] {
            assert_eq!(parse_pesos(text), Some(n), "{text}");
        }
        for text in ["", "abc", "9 mil", "1,80,000", "1800,000", "-5", "12.50", "1,8000", "$", "1,800,"] {
            assert_eq!(parse_pesos(text), None, "{text}");
        }
    }

    #[test]
    fn vacation_days_follow_the_2023_law() {
        let days: Vec<i64> = [0, 1, 2, 3, 4, 5, 6, 10, 11, 15, 16, 20, 21, 30].iter().map(|&y| vacation_days(y)).collect();
        assert_eq!(days, vec![12, 12, 14, 16, 18, 20, 22, 22, 24, 24, 26, 26, 28, 30]);
    }

    #[test]
    fn benefits_are_aguinaldo_and_vacation_premium() {
        // 9,000 a month: a day is 300; aguinaldo 15 days = 4,500; first year 12 days, premium 25 % = 900
        assert_eq!(annual_benefits(9_000, 1), 5_400);
        // eighth year: 22 days, premium 1,650
        assert_eq!(annual_benefits(9_000, 8), 6_150);
        assert!(has_benefits(Some(ContractKind::Permanent)) && has_benefits(Some(ContractKind::Temporary)) && has_benefits(None));
        assert!(!has_benefits(Some(ContractKind::Fees)));
    }

    fn money() -> ProfileInput {
        ProfileInput {
            institution: InstitutionInput { name: "Casa Ficticia".into(), ..Default::default() },
            population: vec![PopulationGroupInput { label: "Niñas".into(), count: 4, paying_count: Some(2), monthly_fee_mxn: Some(1_500), ..Default::default() }],
            staff: vec![
                StaffGroupInput { role: "Cocina".into(), count: 1, paid: true, monthly_salary_mxn: Some(9_000), contract: Some(ContractKind::Permanent), start_year: Some(2018), ..Default::default() },
                StaffGroupInput { role: "Psicología".into(), count: 1, paid: true, monthly_salary_mxn: Some(6_000), contract: Some(ContractKind::Fees), ..Default::default() },
                StaffGroupInput { role: "Voluntariado".into(), count: 2, paid: false, ..Default::default() },
            ],
            income: vec![
                IncomeSourceInput { label: "Padrinos".into(), kind: IncomeKind::RecurringDonor, amount_mxn: Some(5_000), period: Period::Monthly },
                IncomeSourceInput { label: "Colecta".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(40_000), period: Period::Annual },
                IncomeSourceInput { label: "Fundación X 2026".into(), kind: IncomeKind::ProjectGrant, amount_mxn: Some(200_000), period: Period::Annual },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn payroll_cost_adds_the_benefits_only_to_jobs_that_carry_them() {
        let t = money().totals(2026);
        assert_eq!(t.payroll_monthly_mxn, 15_000);
        assert_eq!(t.payroll_annual_mxn, 180_000);
        // cook: 8 years in 2026 -> 6,150; the psychologist works for fees: nothing
        assert_eq!(t.payroll_benefits_annual_mxn, 6_150);
        assert_eq!(t.payroll_cost_annual_mxn, 186_150);
        assert_eq!(t.benefits_assumed, 0);
    }

    #[test]
    fn missing_contract_or_start_year_is_counted_with_benefits_and_flagged_as_assumed() {
        let mut p = money();
        p.staff[0].contract = None;
        p.staff[0].start_year = None;
        let t = p.totals(2026);
        assert_eq!(t.payroll_benefits_annual_mxn, 5_400, "first year, with benefits");
        assert_eq!(t.benefits_assumed, 1);
        // a group of several people with the same line counts once per person
        p.staff[0].count = 3;
        assert_eq!(p.totals(2026).payroll_benefits_annual_mxn, 16_200);
        assert_eq!(p.totals(2026).benefits_assumed, 3);
    }

    #[test]
    fn income_by_kind_with_the_roster_fees_and_monthly_amounts_turned_into_a_year() {
        let f = money().finances(2026);
        // fees 2 x 1,500 x 12 = 36,000; donors 5,000 x 12 = 60,000
        let kinds: Vec<(&str, i64)> = f.income_by_kind.iter().map(|k| (k.kind, k.annual_mxn)).collect();
        assert_eq!(kinds, vec![("beneficiary_fees", 36_000), ("recurring_donor", 60_000), ("occasional_donation", 40_000), ("project_grant", 200_000)]);
        assert_eq!((f.income_fixed_annual_mxn, f.income_variable_annual_mxn, f.income_annual_mxn), (96_000, 240_000, 336_000));
        assert_eq!(f.income[0].index, None, "the fees of the roster are computed, nobody edits them");
        assert_eq!(f.income[1].index, Some(0));
    }

    #[test]
    fn a_fee_estimate_counts_only_while_the_roster_has_no_fees() {
        let mut p = money();
        p.income.push(IncomeSourceInput { label: "Cuotas aprox.".into(), kind: IncomeKind::FeeEstimate, amount_mxn: Some(50_000), period: Period::Annual });
        let f = p.finances(2026);
        let est = f.income.iter().find(|l| l.kind == "fee_estimate").unwrap();
        assert!(!est.counted, "the roster has the real fees: the estimate would count them twice");
        assert_eq!(f.income_annual_mxn, 336_000);

        p.population[0].paying_count = None;
        p.population[0].monthly_fee_mxn = None;
        let f = p.finances(2026);
        assert!(f.income.iter().find(|l| l.kind == "fee_estimate").unwrap().counted);
        assert!(f.income.iter().all(|l| l.kind != "beneficiary_fees"));
        assert_eq!(f.income_annual_mxn, 350_000);
        assert_eq!(f.income_fixed_annual_mxn, 110_000);
    }

    #[test]
    fn expenses_start_with_one_approximate_figure_and_then_the_list_takes_over() {
        let mut p = money();
        let f = p.finances(2026);
        assert_eq!((f.expenses_basis, f.expenses_annual_mxn, f.balance_annual_mxn), (ExpenseBasis::Unknown, None, None));
        assert_eq!(f.expenses.len(), 1, "the payroll is shown even before any expense is known");

        p.annual_budget_mxn = Some(400_000);
        let f = p.finances(2026);
        assert_eq!((f.expenses_basis, f.expenses_annual_mxn, f.balance_annual_mxn), (ExpenseBasis::Estimate, Some(400_000), Some(-64_000)));
        assert!(!f.expenses[0].counted, "the approximate figure already includes the payroll");

        p.expenses = vec![
            ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(8_000), period: Period::Monthly },
            ExpenseItemInput { label: "Luz y agua".into(), amount_mxn: Some(30_000), period: Period::Annual },
            ExpenseItemInput { label: "Mantenimiento".into(), amount_mxn: None, period: Period::Annual },
        ];
        let f = p.finances(2026);
        // 96,000 + 30,000 + payroll 186,150
        assert_eq!((f.expenses_basis, f.expenses_annual_mxn), (ExpenseBasis::List, Some(312_150)));
        assert!(f.expenses[0].counted && f.expenses[0].kind == "payroll");
        assert_eq!(f.balance_annual_mxn, Some(336_000 - 312_150));
    }

    #[test]
    fn without_any_income_amount_there_is_no_balance() {
        let p = ProfileInput { annual_budget_mxn: Some(100), ..Default::default() };
        let f = p.finances(2026);
        assert!(!f.income_known);
        assert_eq!(f.balance_annual_mxn, None, "unknown income is not zero income");
    }
}
