//! The money the person writes: income lines, expense lines and the approximate figure of what is spent in a year.
//! What stops a save, and the heads-ups when the numbers do not add up with what the other modules say.

use super::balance::Derived;
use super::money::{IncomeKind, Period, MAX_MXN};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncomeSourceInput {
    pub label: String,
    #[serde(default)]
    pub kind: IncomeKind,
    pub amount_mxn: Option<i64>,
    #[serde(default)]
    pub period: Period,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpenseItemInput {
    pub label: String,
    pub amount_mxn: Option<i64>,
    #[serde(default)]
    pub period: Period,
}

/// Everything the person writes about the money of the institution.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FinanceInput {
    /// «Gasto anual aproximado»: what the institution spends in a year, all included, as one approximate figure.
    /// It is the quick way to start; once the list of expenses has a line, the list is the total (ADR-026).
    pub annual_budget_mxn: Option<i64>,
    pub income: Vec<IncomeSourceInput>,
    pub expenses: Vec<ExpenseItemInput>,
}

/// Something the person should look at. The UI turns `code` into friendly text; `field` points into the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FinanceIssue {
    pub code: &'static str,
    pub field: String,
    pub blocking: bool,
}

fn normalized(s: &str) -> String {
    s.to_lowercase().replace(['á', 'à'], "a").replace('é', "e").replace('í', "i").replace('ó', "o").replace(['ú', 'ü'], "u")
}

impl FinanceInput {
    /// Problems that stop the save (`blocking`) and heads-ups, with what the other modules add up to (`d`).
    pub fn validate(&self, d: &Derived) -> Vec<FinanceIssue> {
        let mut v = Vec::new();
        let mut add = |code, field: String, blocking| v.push(FinanceIssue { code, field, blocking });
        let neg = |n: Option<i64>| n.is_some_and(|x| x < 0);
        let too_large = |n: Option<i64>| n.is_some_and(|x| x > MAX_MXN);

        if neg(self.annual_budget_mxn) {
            add("negative_number", "annual_budget_mxn".into(), true);
        }
        if too_large(self.annual_budget_mxn) {
            add("amount_too_large", "annual_budget_mxn".into(), true);
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
        if v.iter().any(|i| i.blocking) {
            // the heads-ups below would rest on numbers that are about to be corrected
            return v;
        }
        let mut add = |code, field: String| v.push(FinanceIssue { code, field, blocking: false });
        if d.fees_annual_mxn > 0 {
            if let Some(i) = self.income.iter().position(|i| i.kind == IncomeKind::FeeEstimate) {
                add("fee_estimate_ignored", format!("income[{i}]"));
            }
        }
        if self.expenses.is_empty() {
            if let Some(estimate) = self.annual_budget_mxn {
                if d.payroll_cost_annual_mxn > estimate {
                    add("payroll_over_estimate", "annual_budget_mxn".into());
                }
            }
        }
        if d.payroll_cost_annual_mxn > 0 {
            let payroll_words = ["nomina", "sueldo", "salario", "aguinaldo"];
            if let Some(i) = self.expenses.iter().position(|e| payroll_words.iter().any(|w| normalized(&e.label).contains(w))) {
                add("expense_looks_like_payroll", format!("expenses[{i}].label"));
            }
        }
        v
    }

    /// Visits every free text that must be scanned: the names of the lines (they reach the AI).
    pub fn for_each_text_mut(&mut self, f: &mut dyn FnMut(&str, &mut String)) {
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

    fn base() -> FinanceInput {
        FinanceInput {
            annual_budget_mxn: Some(1_000_000),
            income: vec![
                IncomeSourceInput { label: "Cuotas".into(), kind: IncomeKind::FeeEstimate, amount_mxn: Some(600_000), period: Period::Annual },
                IncomeSourceInput { label: "Donativos".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(400_000), period: Period::Annual },
            ],
            expenses: vec![],
        }
    }

    #[test]
    fn valid_money_has_no_issues() {
        assert!(base().validate(&Derived::default()).is_empty());
    }

    #[test]
    fn a_payroll_over_the_approximate_figure_is_a_heads_up() {
        let d = Derived { payroll_cost_annual_mxn: 1_100_000, ..Default::default() };
        let issues = base().validate(&d);
        assert_eq!(issues, vec![FinanceIssue { code: "payroll_over_estimate", field: "annual_budget_mxn".into(), blocking: false }]);
        // income that differs from what is spent is not «something wrong»: it is the balance
        let mut p = base();
        p.income[1].amount_mxn = Some(1);
        assert!(p.validate(&Derived::default()).is_empty());
    }

    #[test]
    fn heads_up_for_fees_counted_twice_and_payroll_written_as_an_expense() {
        let mut p = base();
        p.annual_budget_mxn = None;
        p.expenses = vec![
            ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(10_000), period: Period::Monthly },
            ExpenseItemInput { label: "Nómina y aguinaldos".into(), amount_mxn: Some(300_000), period: Period::Annual },
        ];
        let d = Derived { fees_annual_mxn: 240_000, payroll_cost_annual_mxn: 340_000, ..Default::default() };
        let found: Vec<_> = p.validate(&d).into_iter().map(|i| (i.code, i.field, i.blocking)).collect();
        assert_eq!(found, vec![("fee_estimate_ignored", "income[0]".into(), false), ("expense_looks_like_payroll", "expenses[1].label".into(), false)]);
    }

    #[test]
    fn amounts_out_of_reason_and_lines_without_name_are_blocked() {
        let mut p = base();
        p.income[0].amount_mxn = Some(MAX_MXN + 1);
        p.annual_budget_mxn = Some(-1);
        p.expenses = vec![ExpenseItemInput { label: " ".into(), amount_mxn: Some(-5), period: Period::Annual }];
        let codes: Vec<_> = p.validate(&Derived::default()).into_iter().filter(|i| i.blocking).map(|i| (i.code, i.field)).collect();
        assert!(codes.contains(&("amount_too_large", "income[0].amount_mxn".into())));
        assert!(codes.contains(&("negative_number", "annual_budget_mxn".into())));
        assert!(codes.contains(&("label_missing", "expenses[0].label".into())));
        assert!(codes.contains(&("negative_number", "expenses[0].amount_mxn".into())));
    }

    #[test]
    fn the_names_of_the_lines_are_scanned() {
        let mut p = base();
        p.expenses.push(ExpenseItemInput { label: "Luz".into(), ..Default::default() });
        let mut seen = Vec::new();
        p.for_each_text_mut(&mut |path, _| seen.push(path.to_string()));
        assert_eq!(seen, vec!["income[0].label", "income[1].label", "expenses[0].label"]);
    }
}
