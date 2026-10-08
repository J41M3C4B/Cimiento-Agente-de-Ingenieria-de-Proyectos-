//! The money screens (ADR-026, ADR-032): the use cases of the finance module, plus what the core adds around them.
//! The stay fees and what the staff costs come from their modules, added up by the core (`derived`); the names of
//! the lines go through the scanner before they are saved, because they reach the AI.

use crate::core::profile::domain::ProfileTotals;
use crate::modules::finance::domain::balance::{Derived, Finances};
use crate::modules::finance::domain::lines::{FinanceInput, FinanceIssue};
use crate::modules::finance::service::{self as fin, FinanceView, SaveOutcome};
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::core::screen::guard_texts;
use crate::core::error::ServiceError;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FinanceOutcome {
    Saved { finance: FinanceView },
    Invalid { issues: Vec<FinanceIssue> },
    Quarantine { report: QuarantineReport },
}

/// What the other modules add up to, as the finance module needs it.
pub fn derived_from(t: &ProfileTotals) -> Derived {
    Derived {
        fees_annual_mxn: t.fees_annual_mxn,
        payroll_cost_annual_mxn: t.payroll_cost_annual_mxn,
        staff_support_annual_mxn: t.staff_support_annual_mxn,
        external_staff_annual_mxn: t.external_staff_annual_mxn,
    }
}

pub fn derived(conn: &Connection) -> Result<Derived, ServiceError> {
    Ok(derived_from(&crate::core::profile::sync::totals(conn)?))
}

pub fn view(conn: &Connection) -> Result<FinanceView, ServiceError> {
    Ok(fin::view(conn, &derived(conn)?)?)
}

/// Income by kind, expenses and the balance as they are now.
pub fn finances(conn: &Connection) -> Result<Finances, ServiceError> {
    Ok(crate::modules::finance::api::finances(conn, &derived(conn)?)?)
}

/// Saves the whole money of the institution: first what must be corrected, then the scanner over the names of the
/// lines, then the module.
pub fn save(conn: &mut Connection, mut input: FinanceInput, decision: Option<Decision>) -> Result<FinanceOutcome, ServiceError> {
    let d = derived(conn)?;
    let blocking: Vec<FinanceIssue> = input.validate(&d).into_iter().filter(|i| i.blocking).collect();
    if !blocking.is_empty() {
        return Ok(FinanceOutcome::Invalid { issues: blocking });
    }
    let mut fields: Vec<(String, String)> = Vec::new();
    input.for_each_text_mut(&mut |path, text| fields.push((path.to_string(), text.clone())));
    match guard_texts(conn, "finance", &fields, decision)? {
        Err(report) => return Ok(FinanceOutcome::Quarantine { report }),
        Ok(texts) => {
            let mut it = texts.into_iter();
            input.for_each_text_mut(&mut |_, text| *text = it.next().unwrap_or_default());
        }
    }
    Ok(match fin::save(conn, input, &d)? {
        SaveOutcome::Saved(finance) => FinanceOutcome::Saved { finance },
        SaveOutcome::Invalid(issues) => FinanceOutcome::Invalid { issues },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::finance::domain::lines::{ExpenseItemInput, IncomeSourceInput};
    use crate::modules::finance::domain::money::{IncomeKind, Period};
    use crate::storage::open_encrypted;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, c)
    }

    fn money() -> FinanceInput {
        FinanceInput {
            annual_budget_mxn: Some(900_000),
            income: vec![
                IncomeSourceInput { label: "Padrinos".into(), kind: IncomeKind::RecurringDonor, amount_mxn: Some(10_000), period: Period::Monthly },
                IncomeSourceInput { label: "Colecta".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(40_000), period: Period::Annual },
            ],
            expenses: vec![ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(8_000), period: Period::Monthly }],
        }
    }

    fn saved(out: FinanceOutcome) -> FinanceView {
        match out {
            FinanceOutcome::Saved { finance } => finance,
            other => panic!("not saved: {other:?}"),
        }
    }

    #[test]
    fn the_money_is_saved_whole_and_read_back_in_order() {
        let (_d, mut c) = conn();
        assert_eq!(view(&c).unwrap().input, FinanceInput::default());
        let f = saved(save(&mut c, money(), None).unwrap());
        assert_eq!(f.input, money());
        assert_eq!((f.finances.income_annual_mxn, f.finances.expenses_annual_mxn), (160_000, Some(96_000)));
        // saving again replaces what there was, it never adds
        let mut less = money();
        less.income.pop();
        less.annual_budget_mxn = None;
        saved(save(&mut c, less.clone(), None).unwrap());
        assert_eq!(view(&c).unwrap().input, less);
        let origin: String = c.query_row("SELECT origin FROM fin_income", [], |r| r.get(0)).unwrap();
        assert_eq!(origin, "user");
    }

    #[test]
    fn a_line_with_personal_data_waits_in_quarantine_and_is_covered_on_request() {
        let (_d, mut c) = conn();
        let mut m = money();
        m.expenses[0].label = "Pago a la cocinera, tel. 55 1234 5678".into();
        assert!(matches!(save(&mut c, m.clone(), None).unwrap(), FinanceOutcome::Quarantine { .. }));
        assert_eq!(view(&c).unwrap().input, FinanceInput::default(), "nothing is saved while the person decides");
        let f = saved(save(&mut c, m, Some(Decision::Redact)).unwrap());
        assert!(!f.input.expenses[0].label.contains("5678"), "{}", f.input.expenses[0].label);
    }

    #[test]
    fn what_must_be_corrected_stops_the_save() {
        let (_d, mut c) = conn();
        let mut m = money();
        m.income[0].label = "  ".into();
        match save(&mut c, m, None).unwrap() {
            FinanceOutcome::Invalid { issues } => assert_eq!(issues[0].field, "income[0].label"),
            other => panic!("saved: {other:?}"),
        }
    }

    #[test]
    fn the_payroll_comes_from_the_staff_module_already_added_up() {
        let (_d, mut c) = conn();
        crate::core::ai_sheet::tests::seed_rich_staff(&mut c);
        let d = derived(&c).unwrap();
        // 4 caregivers of 7,777 a month, with the benefits of 7 years of work
        assert!(d.payroll_cost_annual_mxn > 4 * 7_777 * 12, "{d:?}");
        assert_eq!(d.fees_annual_mxn, 0, "nobody pays a fee yet");
        let f = finances(&c).unwrap();
        assert_eq!((f.expenses[0].kind, f.expenses[0].annual_mxn), ("payroll", Some(d.payroll_cost_annual_mxn)));
        assert!(f.income.is_empty());
    }
}
