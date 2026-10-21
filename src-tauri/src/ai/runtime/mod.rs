//! The runtime of the platform (ADR-003, the AI blueprint §3): the one door every request goes through, the task
//! that carries it, the one executor that invokes capabilities, the approval and the common delivery. It does not
//! think: it keeps the rules. What needs judgment (the intent, which capability, its values, the words) comes from
//! the model inside the steps, with IA-4; what is allowed, the limits and the approval are kept here, by code.
//!
//! Nothing here knows the screens, Tauri or a module: the app gives the runtime who asks, how to ask the permission,
//! the clock and where to deliver, and the capabilities come from the registry (ADR-001).

mod executor;
pub mod outcome;
pub mod request;
pub mod store;
pub mod task;
#[cfg(test)]
mod tests;

use crate::capability::{Permission, Registry};

pub use outcome::Outcome;
pub use request::Request;
pub use store::migrate;

/// Who asks: the account and the organization of the session, never something a screen sends.
pub struct Who<'a> {
    pub account: &'a str,
    pub organization: &'a str,
}

/// The question of authorization (K2): whether this account may use this permission now. The runtime asks it before
/// each invocation and again before handing an outcome over.
pub trait Authorize {
    fn may(&self, who: &Who, permission: Permission) -> bool;
}

pub trait Clock {
    /// Milliseconds since 1970 (UTC).
    fn now_ms(&self) -> i64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
    }
}

/// Where outcomes go besides the answer to the request: the window hears them, so whoever waits for a task does
/// not have to stay on the screen that asked.
pub trait Deliver {
    fn deliver(&self, outcome: &Outcome);
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
}

pub struct Runtime<'a> {
    pub registry: &'a Registry,
    pub clock: &'a dyn Clock,
    pub authorize: &'a dyn Authorize,
    pub deliver: &'a dyn Deliver,
}

impl Runtime<'_> {
    /// The last outcome of a task of this account, to ask for it later.
    pub fn task(&self, conn: &rusqlite::Connection, who: &Who, id: &str) -> Result<Option<Outcome>, RuntimeError> {
        match store::load_task(conn, id)? {
            Some(t) if t.account == who.account && t.organization == who.organization => Ok(store::last_outcome(conn, id)?),
            _ => Ok(None),
        }
    }
}
