//! Income by kind, expenses and the balance (ADR-026). Every figure is made here, by code; the screen and the AI
//! only read it.

use super::lines::FinanceInput;
use super::money::{annual, IncomeKind};
use serde::Serialize;

/// What the core hands over from the other modules, already added up and anonymous: the stay fees the people served
/// pay, and what the staff costs (ADR-027, ADR-029). This module never sees a person.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Derived {
    /// Stay fees of the people served with a record, in a year.
    pub fees_annual_mxn: i64,
    /// The payroll with the benefits the law requires, in a year.
    pub payroll_cost_annual_mxn: i64,
    /// Contributions to the congregation and grants of social service, in a year.
    pub staff_support_annual_mxn: i64,
    /// Staff of outside companies, in a year.
    pub external_staff_annual_mxn: i64,
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
    /// Income: an `IncomeKind` (`fee_estimate`, `recurring_donor`, …) or `beneficiary_fees` (from the records).
    /// Expenses: `expense`, `payroll`, `staff_support` or `external_staff` (from the records).
    pub kind: &'static str,
    /// Position in `income` / `expenses` of the input; `None` for a line the app computes and nobody edits.
    pub index: Option<usize>,
    pub annual_mxn: Option<i64>,
    /// Whether it is part of the total. A fee estimate is left out once the records have the real fees, and the
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
    /// Counted income by kind, in the order of `IncomeKind` (the fees of the records first).
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
pub const STAFF_SUPPORT: &str = "staff_support";
pub const EXTERNAL_STAFF: &str = "external_staff";
pub const EXPENSE: &str = "expense";

/// The money of the institution: what the person wrote, with what the other modules add up to.
pub fn finances(input: &FinanceInput, d: &Derived) -> Finances {
    let roster_fees = d.fees_annual_mxn > 0;

    let mut income = Vec::new();
    if roster_fees {
        income.push(FinanceLine {
            label: "Cuotas de los beneficiarios (del padrón)".into(),
            kind: BENEFICIARY_FEES,
            index: None,
            annual_mxn: Some(d.fees_annual_mxn),
            counted: true,
        });
    }
    for (i, inc) in input.income.iter().enumerate() {
        income.push(FinanceLine {
            label: inc.label.clone(),
            kind: inc.kind.as_db(),
            index: Some(i),
            annual_mxn: inc.amount_mxn.map(|a| annual(a, inc.period)),
            counted: !(roster_fees && inc.kind == IncomeKind::FeeEstimate),
        });
    }
    let counted = |l: &&FinanceLine| l.counted;
    let sum = |kind: &str| -> i64 { income.iter().filter(counted).filter(|l| l.kind == kind).filter_map(|l| l.annual_mxn).sum() };
    let mut income_by_kind = Vec::new();
    for kind in std::iter::once(BENEFICIARY_FEES).chain(IncomeKind::ALL.iter().map(|k| k.as_db())) {
        if income.iter().filter(counted).any(|l| l.kind == kind && l.annual_mxn.is_some()) {
            income_by_kind.push(KindTotal { kind, annual_mxn: sum(kind) });
        }
    }
    let fixed = sum(BENEFICIARY_FEES) + sum(IncomeKind::FeeEstimate.as_db()) + sum(IncomeKind::RecurringDonor.as_db());
    let income_annual_mxn: i64 = income.iter().filter(counted).filter_map(|l| l.annual_mxn).sum();
    let income_known = income.iter().filter(counted).any(|l| l.annual_mxn.is_some());

    let basis = if !input.expenses.is_empty() {
        ExpenseBasis::List
    } else if input.annual_budget_mxn.is_some() {
        ExpenseBasis::Estimate
    } else {
        ExpenseBasis::Unknown
    };
    let mut expenses = Vec::new();
    for (kind, label, amount) in [
        (PAYROLL, "Nómina del personal con prestaciones (del padrón)", d.payroll_cost_annual_mxn),
        (STAFF_SUPPORT, "Aportaciones a la congregación y apoyos de servicio social (del padrón)", d.staff_support_annual_mxn),
        (EXTERNAL_STAFF, "Personal de empresas externas (del padrón)", d.external_staff_annual_mxn),
    ] {
        if amount > 0 {
            expenses.push(FinanceLine { label: label.into(), kind, index: None, annual_mxn: Some(amount), counted: basis == ExpenseBasis::List });
        }
    }
    for (i, e) in input.expenses.iter().enumerate() {
        expenses.push(FinanceLine { label: e.label.clone(), kind: EXPENSE, index: Some(i), annual_mxn: e.amount_mxn.map(|a| annual(a, e.period)), counted: true });
    }
    let expenses_annual_mxn = match basis {
        ExpenseBasis::List => Some(expenses.iter().filter(counted).filter_map(|l| l.annual_mxn).sum()),
        ExpenseBasis::Estimate => input.annual_budget_mxn,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::finance::domain::lines::{ExpenseItemInput, IncomeSourceInput};
    use crate::modules::finance::domain::money::Period;

    fn money() -> FinanceInput {
        FinanceInput {
            income: vec![
                IncomeSourceInput { label: "Padrinos".into(), kind: IncomeKind::RecurringDonor, amount_mxn: Some(5_000), period: Period::Monthly },
                IncomeSourceInput { label: "Colecta".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(40_000), period: Period::Annual },
                IncomeSourceInput { label: "Fundación X 2026".into(), kind: IncomeKind::ProjectGrant, amount_mxn: Some(200_000), period: Period::Annual },
            ],
            ..Default::default()
        }
    }

    /// Two girls paying 1,500 a month; a cook with pay and benefits and a psychologist by fees (186,150 a year).
    fn records() -> Derived {
        Derived { fees_annual_mxn: 36_000, payroll_cost_annual_mxn: 186_150, ..Default::default() }
    }

    #[test]
    fn income_by_kind_with_the_fees_of_the_records_and_monthly_amounts_turned_into_a_year() {
        let f = finances(&money(), &records());
        // donors 5,000 x 12 = 60,000
        let kinds: Vec<(&str, i64)> = f.income_by_kind.iter().map(|k| (k.kind, k.annual_mxn)).collect();
        assert_eq!(kinds, vec![("beneficiary_fees", 36_000), ("recurring_donor", 60_000), ("occasional_donation", 40_000), ("project_grant", 200_000)]);
        assert_eq!((f.income_fixed_annual_mxn, f.income_variable_annual_mxn, f.income_annual_mxn), (96_000, 240_000, 336_000));
        assert_eq!(f.income[0].index, None, "the fees of the records are computed, nobody edits them");
        assert_eq!(f.income[1].index, Some(0));
    }

    #[test]
    fn a_fee_estimate_counts_only_while_the_records_have_no_fees() {
        let mut p = money();
        p.income.push(IncomeSourceInput { label: "Cuotas aprox.".into(), kind: IncomeKind::FeeEstimate, amount_mxn: Some(50_000), period: Period::Annual });
        let f = finances(&p, &records());
        let est = f.income.iter().find(|l| l.kind == "fee_estimate").unwrap();
        assert!(!est.counted, "the records have the real fees: the estimate would count them twice");
        assert_eq!(f.income_annual_mxn, 336_000);

        let f = finances(&p, &Derived { fees_annual_mxn: 0, ..records() });
        assert!(f.income.iter().find(|l| l.kind == "fee_estimate").unwrap().counted);
        assert!(f.income.iter().all(|l| l.kind != "beneficiary_fees"));
        assert_eq!(f.income_annual_mxn, 350_000);
        assert_eq!(f.income_fixed_annual_mxn, 110_000);
    }

    #[test]
    fn expenses_start_with_one_approximate_figure_and_then_the_list_takes_over() {
        let mut p = money();
        let f = finances(&p, &records());
        assert_eq!((f.expenses_basis, f.expenses_annual_mxn, f.balance_annual_mxn), (ExpenseBasis::Unknown, None, None));
        assert_eq!(f.expenses.len(), 1, "the payroll is shown even before any expense is known");

        p.annual_budget_mxn = Some(400_000);
        let f = finances(&p, &records());
        assert_eq!((f.expenses_basis, f.expenses_annual_mxn, f.balance_annual_mxn), (ExpenseBasis::Estimate, Some(400_000), Some(-64_000)));
        assert!(!f.expenses[0].counted, "the approximate figure already includes the payroll");

        p.expenses = vec![
            ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(8_000), period: Period::Monthly },
            ExpenseItemInput { label: "Luz y agua".into(), amount_mxn: Some(30_000), period: Period::Annual },
            ExpenseItemInput { label: "Mantenimiento".into(), amount_mxn: None, period: Period::Annual },
        ];
        let f = finances(&p, &records());
        // 96,000 + 30,000 + payroll 186,150
        assert_eq!((f.expenses_basis, f.expenses_annual_mxn), (ExpenseBasis::List, Some(312_150)));
        assert!(f.expenses[0].counted && f.expenses[0].kind == "payroll");
        assert_eq!(f.balance_annual_mxn, Some(336_000 - 312_150));
    }

    #[test]
    fn contributions_grants_and_outside_staff_are_expenses_but_not_payroll() {
        let p = FinanceInput { expenses: vec![ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(1_000), period: Period::Annual }], ..Default::default() };
        let d = Derived { staff_support_annual_mxn: 48_000, external_staff_annual_mxn: 192_000, ..Default::default() };
        let f = finances(&p, &d);
        let kinds: Vec<_> = f.expenses.iter().map(|l| (l.kind, l.annual_mxn, l.counted)).collect();
        assert_eq!(kinds, vec![("staff_support", Some(48_000), true), ("external_staff", Some(192_000), true), ("expense", Some(1_000), true)]);
        assert_eq!(f.expenses_annual_mxn, Some(241_000));
    }

    #[test]
    fn without_any_income_amount_there_is_no_balance() {
        let p = FinanceInput { annual_budget_mxn: Some(100), ..Default::default() };
        let f = finances(&p, &Derived::default());
        assert!(!f.income_known);
        assert_eq!(f.balance_annual_mxn, None, "unknown income is not zero income");
    }
}
