//! The border of the finance module: the only things the rest of the app may ask it for.

use super::domain::balance::{self, Derived, Finances};
use super::domain::lines::FinanceInput;
use super::storage as store;
use super::FinanceError;
use rusqlite::Connection;

/// What the person wrote: the AI reads the written lines (never the payroll nor the fees, ADR-026).
pub fn lines(conn: &Connection) -> Result<FinanceInput, FinanceError> {
    store::load(conn)
}

/// Income by kind, expenses and the balance, with what the other modules add up to.
pub fn finances(conn: &Connection, d: &Derived) -> Result<Finances, FinanceError> {
    Ok(balance::finances(&store::load(conn)?, d))
}
