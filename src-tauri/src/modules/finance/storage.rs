//! The tables of the money (`fin_*`, migration 0020). The lines are not versions of the profile any more: each one
//! keeps where it came from and when the person confirmed it.

use super::domain::lines::{ExpenseItemInput, FinanceInput, IncomeSourceInput};
use super::domain::money::{IncomeKind, Period};
use super::FinanceError;
use rusqlite::{params, Connection, OptionalExtension};
use ulid::Ulid;

fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

/// Everything the person wrote, in the order they wrote it.
pub fn load(conn: &Connection) -> Result<FinanceInput, FinanceError> {
    let income = conn
        .prepare("SELECT label, kind, amount_mxn, period FROM fin_income ORDER BY rowid")?
        .query_map([], |r| {
            Ok(IncomeSourceInput {
                label: r.get(0)?,
                kind: IncomeKind::from_db(&r.get::<_, String>(1)?),
                amount_mxn: r.get(2)?,
                period: Period::from_db(&r.get::<_, String>(3)?),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let expenses = conn
        .prepare("SELECT label, amount_mxn, period FROM fin_expense ORDER BY rowid")?
        .query_map([], |r| Ok(ExpenseItemInput { label: r.get(0)?, amount_mxn: r.get(1)?, period: Period::from_db(&r.get::<_, String>(2)?) }))?
        .collect::<Result<Vec<_>, _>>()?;
    let annual_budget_mxn = conn.query_row("SELECT annual_budget_mxn FROM fin_settings WHERE id = 1", [], |r| r.get(0)).optional()?.flatten();
    Ok(FinanceInput { annual_budget_mxn, income, expenses })
}

/// Writes what the person sent as the whole money of the institution, confirmed by them now. Already validated and
/// scanned by the caller.
pub fn save(conn: &mut Connection, input: &FinanceInput) -> Result<(), FinanceError> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM fin_income", [])?;
    tx.execute("DELETE FROM fin_expense", [])?;
    for i in &input.income {
        tx.execute(
            "INSERT INTO fin_income (id,label,kind,amount_mxn,period,origin,confirmed_at)
             VALUES (?1,?2,?3,?4,?5,'user',strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
            params![new_id("inc"), i.label.trim(), i.kind.as_db(), i.amount_mxn, i.period.as_db()],
        )?;
    }
    for e in &input.expenses {
        tx.execute(
            "INSERT INTO fin_expense (id,label,amount_mxn,period,origin,confirmed_at)
             VALUES (?1,?2,?3,?4,'user',strftime('%Y-%m-%dT%H:%M:%SZ','now'))",
            params![new_id("exp"), e.label.trim(), e.amount_mxn, e.period.as_db()],
        )?;
    }
    tx.execute(
        "INSERT INTO fin_settings (id,annual_budget_mxn,origin,confirmed_at) VALUES (1,?1,'user',strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         ON CONFLICT(id) DO UPDATE SET annual_budget_mxn = excluded.annual_budget_mxn, origin = 'user', confirmed_at = excluded.confirmed_at",
        [input.annual_budget_mxn],
    )?;
    tx.commit()?;
    Ok(())
}
