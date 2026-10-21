//! The modules of the app (ADR-032): each one keeps a part of the work of the institution (staff, people served,
//! facilities) with its own tables, errors and rules. They do not know each other nor the core: the core hands them
//! what they need (the kind of institution) and reads their `api`. A test of each one, and `architecture_tests`,
//! check the border.

pub mod care;
pub mod facilities;
pub mod finance;
pub mod hr;
