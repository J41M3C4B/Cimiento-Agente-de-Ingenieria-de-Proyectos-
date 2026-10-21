//! The reading of a call into the canonical schema (ADR-015): the model fills each block of the schema with quotes
//! copied from the pages, and `documents::canonical` verifies them, with no AI. It lives in the platform because it
//! calls the model, and no deterministic service does (ADR-001).
//!
//! Stopped, not retired (leader, 2026-10-21): nothing in the program calls it since Proyectos left. It waits to be
//! connected again as an operator of the platform, over the capabilities of a module.

pub mod run;

#[cfg(test)]
mod live_tests;
