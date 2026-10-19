//! The use cases of the money screens. What the other modules add up to (`Derived`) comes from the caller; the
//! scanner too (the core passes the labels through it before saving).

use super::domain::balance::{finances, Derived, Finances};
use super::domain::lines::{FinanceInput, FinanceIssue};
use super::storage as store;
use super::FinanceError;
use rusqlite::Connection;
use serde::Serialize;

/// The money as the screen shows it: what was written, the sums and the heads-ups.
#[derive(Debug, Clone, Serialize)]
pub struct FinanceView {
    pub input: FinanceInput,
    pub finances: Finances,
    /// Only heads-ups («algo no cuadra»); blocking problems stop the save.
    pub issues: Vec<FinanceIssue>,
}

pub enum SaveOutcome {
    Saved(FinanceView),
    Invalid(Vec<FinanceIssue>),
}

fn view_of(input: FinanceInput, d: &Derived) -> FinanceView {
    FinanceView { finances: finances(&input, d), issues: input.validate(d), input }
}

pub fn view(conn: &Connection, d: &Derived) -> Result<FinanceView, FinanceError> {
    Ok(view_of(store::load(conn)?, d))
}

/// Saves the whole money of the institution, unless something must be corrected first.
pub fn save(conn: &mut Connection, input: FinanceInput, d: &Derived) -> Result<SaveOutcome, FinanceError> {
    let blocking: Vec<FinanceIssue> = input.validate(d).into_iter().filter(|i| i.blocking).collect();
    if !blocking.is_empty() {
        return Ok(SaveOutcome::Invalid(blocking));
    }
    store::save(conn, &input)?;
    Ok(SaveOutcome::Saved(view(conn, d)?))
}
