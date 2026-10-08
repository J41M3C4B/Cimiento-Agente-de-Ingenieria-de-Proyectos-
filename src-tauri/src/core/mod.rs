//! The core of the app (ADR-032): «Inicio» and «Mi institución». It knows the institution, opens the workspace (the
//! first start), keeps the accounts, puts together what the modules say and writes the sheet the AI reads. The
//! modules never reach into it: it hands them what they need (the kind of institution) and reads their `api`.
//!

pub mod access;
pub mod ai_sheet;
pub mod api;
pub mod archive;
pub mod bridge;
pub mod error;
pub mod insights;
pub mod institution;
pub mod onboarding;
pub mod profile;
pub mod screen;
pub mod security;
