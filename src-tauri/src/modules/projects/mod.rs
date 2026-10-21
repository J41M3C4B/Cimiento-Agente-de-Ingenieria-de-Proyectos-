//! The projects (ADR-016 to ADR-024): the person chooses the need, writes each section, adds up the budget and the
//! schedule, and the code reviews them and writes the guide in Word. It is frozen like the other modules (it is
//! hidden, and its development waits for the core) and it does not use the AI: what the conversation, the
//! diagnosis, the drafting and the reading of the calls did comes back as capabilities of the platform.
//!
//! It uses the base and `core::api`, nothing else of the app; `architecture_tests` checks it.

pub mod conversation;
pub mod diagnosis;
pub mod domain;
pub mod drafting;
pub mod guide;
pub mod review;
pub mod storage;

use crate::core::api::ServiceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectsError {
    /// What the core said; its storage, the scanner, the audit log and the database come through it.
    #[error(transparent)]
    Core(#[from] ServiceError),
    #[error("not found")]
    NotFound,
    #[error("empty text")]
    EmptyText,
    #[error("text too large")]
    TextTooLarge,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("a budget line needs a quantity above zero and a price that is not negative")]
    InvalidBudgetItem,
    #[error("some budget lines have no cost yet")]
    BudgetIncomplete,
    #[error("the months of an activity are not valid")]
    InvalidActivity,
    #[error("the guide carries data that identifies a person")]
    GuideHasPersonalData,
    #[error("not available at this stage")]
    WrongStage,
    #[error(transparent)]
    Stage(#[from] domain::stage::StageError),
    #[error("priority error: {0}")]
    Priority(String),
}

/// What the base can fail with reaches the projects as the core would say it.
macro_rules! through_the_core {
    ($($t:ty),*) => {$(
        impl From<$t> for ProjectsError {
            fn from(e: $t) -> Self {
                ProjectsError::Core(e.into())
            }
        }
    )*};
}
through_the_core!(crate::storage::StorageError, rusqlite::Error, crate::scanner::guard::GuardError, crate::scanner::guard::ScreenError, crate::audit::AuditError);
