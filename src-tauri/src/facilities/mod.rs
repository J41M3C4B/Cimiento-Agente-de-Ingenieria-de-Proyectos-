//! The facilities module (ADR-030): the building, its services, its safety and civil protection, and its spaces and
//! equipment in groups that count how many are in each state. The base of a future specialized module.
//!
//! Like the staff and the people served, it is kept apart so it can become a crate of its own: its own tables
//! (`fac_*`), errors and rules, and it uses nothing of the rest of the app but the audit log and `common`. The app
//! talks to it only through `api` (summaries and indicators) and, for the screens, through `service`. A test checks
//! the border.

pub mod api;
pub mod domain;
pub mod legacy;
pub mod service;
pub mod storage;

pub use domain::catalog::Flavor;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FacilitiesError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    /// The module uses nothing of the rest of the app but the audit log and `common`.
    #[test]
    fn the_module_does_not_reach_into_the_rest_of_the_app() {
        fn visit(dir: &std::path::Path, out: &mut Vec<String>) {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    visit(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    for (n, line) in std::fs::read_to_string(&p).unwrap().lines().enumerate() {
                        if let Some(at) = line.find(concat!("crate", "::")) {
                            let rest = &line[at + 7..];
                            if !(rest.starts_with("facilities") || rest.starts_with("audit") || rest.starts_with("common")) {
                                out.push(format!("{}:{}: {}", p.display(), n + 1, line.trim()));
                            }
                        }
                    }
                }
            }
        }
        let mut found = Vec::new();
        visit(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/facilities"), &mut found);
        assert!(found.is_empty(), "the facilities module reaches into the app: {found:#?}");
    }
}
