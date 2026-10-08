//! The error of the core (ADR-032). The modules have their own; the commands turn all of them into a friendly
//! message (`error.rs` of the commands).

use crate::audit;
use crate::core::profile::domain::ProfileIssue;
use crate::scanner::guard::{GuardError, ScreenError};
use crate::storage::StorageError;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Guard(#[from] GuardError),
    #[error(transparent)]
    Audit(#[from] audit::AuditError),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("the profile has problems that must be fixed first")]
    ProfileInvalid(Vec<ProfileIssue>),
    #[error("a record of the roster has a missing or wrong value: {0}")]
    InvalidRoster(&'static str),
    #[error("empty text")]
    EmptyText,
    #[error("the PIN is not four to eight digits")]
    InvalidPin,
    #[error("the PIN is not the right one")]
    WrongPin,
    #[error("text too large")]
    TextTooLarge,
    #[error("unknown document kind")]
    UnknownKind,
    #[error("not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("staff module: {0}")]
    Hr(#[from] crate::modules::hr::HrError),
    #[error("people served module: {0}")]
    Care(#[from] crate::modules::care::CareError),
    #[error("facilities module: {0}")]
    Facilities(#[from] crate::modules::facilities::FacilitiesError),
    #[error("finance module: {0}")]
    Finance(#[from] crate::modules::finance::FinanceError),
    /// The onboarding cannot be closed while a required datum is missing (ADR-031).
    #[error("the onboarding is not complete")]
    OnboardingIncomplete,
    #[error("the staff is kept in its own module now")]
    StaffMoved,
    /// An account or session rule (ADR-028); the code says which.
    #[error("access: {0}")]
    Access(&'static str),
}

impl From<ScreenError> for ServiceError {
    fn from(e: ScreenError) -> Self {
        match e {
            ScreenError::Guard(e) => ServiceError::Guard(e),
            ScreenError::Audit(e) => ServiceError::Audit(e),
        }
    }
}
