//! The finance module (ADR-026, ADR-032): what comes in by kind, what goes out (a list by concept or, to start, one
//! approximate figure) and the balance. The base of a future specialized module (movements, budgets, reports).
//!
//! Like the staff, the people served and the facilities, it is kept apart so it can become a crate of its own: its
//! own tables (`fin_*`), errors and rules, and it uses nothing of the rest of the app but the audit log and `common`.
//! It never sees a person: the stay fees and what the staff costs reach it already added up, from the core. The app
//! talks to it only through `api` and, for the screens, through `service`. A test checks the border.

pub mod api;
pub mod domain;
pub mod service;
pub mod storage;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FinanceError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
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
                            if !(rest.starts_with("modules::finance") || rest.starts_with("audit") || rest.starts_with("common")) {
                                out.push(format!("{}:{}: {}", p.display(), n + 1, line.trim()));
                            }
                        }
                    }
                }
            }
        }
        let mut found = Vec::new();
        visit(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/modules/finance"), &mut found);
        assert!(found.is_empty(), "the finance module reaches into the app: {found:#?}");
    }
}
