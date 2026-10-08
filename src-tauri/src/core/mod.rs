//! The core of the app (ADR-032): «Inicio» and «Mi institución». It knows the institution, opens the workspace (the
//! first start), keeps the accounts, puts together what the modules say and writes the sheet the AI reads. The
//! modules never reach into it: it hands them what they need (the kind of institution) and reads their `api`.
//!
//! It is filled block by block (ADR-032); until then part of it still lives in the old places (`*_service.rs`).

pub mod institution;
