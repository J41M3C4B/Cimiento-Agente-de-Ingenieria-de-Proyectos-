//! The projects (ADR-016 to ADR-024): a project is born from its call, the AI leads the conversation of the diagnosis,
//! the person chooses the need, the AI drafts and the code adds up, reviews and writes the guide in Word. Its
//! development is paused while the modules grow (ADR-032).
//!
//! It is the module that consumes: the AI of the projects reads the sheet of the institution the core writes. It
//! uses the base and `core::api`, nothing else of the app; `architecture_tests` checks it.

pub mod calls;
pub mod conversation;
pub mod diagnosis;
pub mod domain;
pub mod drafting;
pub mod guide;
pub mod jobs;
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
    #[error("the year of the call is not valid")]
    InvalidYear,
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
    #[error("the AI is already working on something for this project")]
    AlreadyRunning,
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
