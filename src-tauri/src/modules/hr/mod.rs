//! The staff module (ADR-027): the first base of a future human-resources module.
//!
//! It is kept apart so it can become a crate of its own: it has its own tables (`hr_*`), its own errors and its
//! own rules, and it uses nothing of the rest of the app but the audit log and `common` (shared validators). The rest of the app talks to it only
//! through `api` (anonymous aggregates) and, for the screens, through `service`. A test checks the border.

pub mod api;
pub mod domain;
pub mod legacy;
pub mod service;
pub mod storage;

pub use domain::catalog::Flavor;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HrError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("audit error: {0}")]
    Audit(#[from] crate::audit::AuditError),
    #[error("not found")]
    NotFound,
    #[error("a position with that title already exists")]
    DuplicateTitle,
    #[error("the position has people in it")]
    PositionInUse,
    #[error("the title is empty")]
    EmptyTitle,
    #[error("unknown modality")]
    UnknownModality,
    #[error("unknown field")]
    UnknownField,
}

#[cfg(test)]
mod tests {
    /// The module uses nothing of the rest of the app but the audit log: it can leave as a crate of its own.
    #[test]
    fn the_module_does_not_reach_into_the_rest_of_the_app() {
        fn visit(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    visit(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    let text = std::fs::read_to_string(&p).unwrap();
                    for (n, line) in text.lines().enumerate() {
                        if let Some(at) = line.find(concat!("crate", "::")) {
                            let rest = &line[at + 7..];
                            if !(rest.starts_with("modules::hr") || rest.starts_with("audit") || rest.starts_with("common")) {
                                out.push((format!("{}:{}", p.display(), n + 1), line.trim().to_string()));
                            }
                        }
                    }
                }
            }
        }
        let mut found = Vec::new();
        visit(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/modules/hr"), &mut found);
        assert!(found.is_empty(), "the staff module reaches into the app: {found:#?}");
    }
}
