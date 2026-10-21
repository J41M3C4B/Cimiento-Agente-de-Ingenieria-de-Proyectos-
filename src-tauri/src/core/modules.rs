//! Which modules of the ERP the program shows (ADR-043 §3). Personal, Beneficiarios, Instalaciones and Finanzas are
//! frozen until the core (Mi institución and Documentos, with the AI platform) is done: their code still builds and
//! is tested, but nothing shows them. Proyectos is not here: it left the program (ERP-6) and comes back, if it does,
//! from the tag `proyectos-rust-congelado`. Everything else
//! follows from this list: the map of places (and so the bar, the Assistant and its buttons), the figures of the
//! institution and what the core asks of them. Turning a module back on is taking its page out of `HIDDEN`.

/// The pages of the modules the program does not show.
pub const HIDDEN: &[&str] = &["staff", "people", "facilities", "finance"];

/// Whether a page is shown: every page is, except the hidden modules.
pub fn shown(page: &str) -> bool {
    !HIDDEN.contains(&page)
}

/// What the program does not have yet, for the Assistant: each hidden module by the name of its screen in the manual.
/// Only the name: what each one holds («su equipo») drew other questions to it («¿cuántos extintores tenemos?»).
pub fn not_yet() -> Vec<String> {
    HIDDEN
        .iter()
        .filter_map(|page| crate::core::manual::MANUAL.iter().find(|e| e.id == format!("screen.{page}")))
        .map(|e| e.title.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::manual::MANUAL;
    use crate::core::places::{listed, place};

    /// A hidden module is nowhere the person or the Assistant could reach it: no place, no line in the Assistant's
    /// list, no answer of the manual; its screen stays in the manual, to be turned back on.
    #[test]
    fn a_hidden_module_is_no_place_and_the_assistant_does_not_know_it() {
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::storage::open_encrypted(&dir.path().join("t.db"), "k").unwrap();
        let list = listed();
        for page in HIDDEN {
            assert!(!shown(page));
            assert!(place(page).is_none(), "{page} is a place");
            assert!(!list.contains(&format!("`{page}`")), "the Assistant's list names {page}");
            let screen = MANUAL.iter().find(|e| e.id == format!("screen.{page}")).unwrap_or_else(|| panic!("{page} lost its screen"));
            let hits = crate::core::manual::search(&conn, &screen.title, 10).unwrap();
            assert!(hits.iter().all(|h| h.id != screen.id), "the manual answers with {}", screen.id);
        }
        for page in ["home", "profile", "documents"] {
            assert!(shown(page) && place(page).is_some(), "{page}");
        }
        // the Assistant is told what does not exist yet, so it does not send the person somewhere else
        let missing = not_yet();
        assert_eq!(missing.len(), HIDDEN.len());
        assert!(missing.contains(&"Finanzas".to_string()) && missing.contains(&"Personal".to_string()), "{missing:?}");
    }
}
