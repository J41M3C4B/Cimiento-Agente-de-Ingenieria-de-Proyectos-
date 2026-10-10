//! The fictitious institutions of `fixtures/` (development only): an asilo and a casa hogar, both I.A.P. like the
//! institutions the app is made for (audit D8). Loading one replaces the records with the example: the institution
//! with every datum of «Mi institución», its people, its building, its money, its board and its bank account.

use crate::core::error::ServiceError;
use crate::core::profile::domain::ProfileInput;
use crate::core::profile::service::{self, ProfileView, SaveProfileOutcome};
use crate::core::protected::{self, ListKind, ListOutcome};
use crate::common::forms::Values;
use rusqlite::Connection;

/// The files of an example: the institution (with its money, board and accounts), its people and its building.
fn files(name: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match name {
        "asilo" => (
            include_str!("../../../fixtures/institucion-asilo.json"),
            include_str!("../../../fixtures/padron-asilo.json"),
            include_str!("../../../fixtures/instalaciones-asilo.json"),
        ),
        "casa-hogar" => (
            include_str!("../../../fixtures/institucion-casa-hogar.json"),
            include_str!("../../../fixtures/padron-casa-hogar.json"),
            include_str!("../../../fixtures/instalaciones-casa-hogar.json"),
        ),
        _ => return None,
    })
}

/// The board and the accounts of an example, as their windows would send them.
#[derive(serde::Deserialize)]
struct Lists {
    #[serde(default)]
    board: Vec<Values>,
    #[serde(default)]
    bank_accounts: Vec<Values>,
}

fn bad(what: &str) -> ServiceError {
    ServiceError::Internal(format!("the example has a wrong {what}"))
}

/// Puts the board and the accounts of the example in place of the ones there are.
fn seed_lists(conn: &mut Connection, raw: &str) -> Result<(), ServiceError> {
    let lists: Lists = serde_json::from_str(raw).map_err(|_| bad("list"))?;
    let now = protected::lists(conn)?;
    for m in now.board {
        protected::delete(conn, ListKind::BoardMember, &m.id)?;
    }
    for a in now.accounts {
        protected::delete(conn, ListKind::BankAccount, &a.id)?;
    }
    for (kind, rows) in [(ListKind::BoardMember, lists.board), (ListKind::BankAccount, lists.bank_accounts)] {
        for values in rows {
            if let ListOutcome::Invalid { .. } = protected::save(conn, kind, None, values)? {
                return Err(bad("row of a list"));
            }
        }
    }
    Ok(())
}

/// Loads an example by name (`asilo`, `casa-hogar`). `None` when there is no such example.
pub fn load(conn: &mut Connection, name: &str) -> Result<Option<ProfileView>, ServiceError> {
    let Some((raw, padron, facilities)) = files(name) else { return Ok(None) };
    let input: ProfileInput = serde_json::from_str(raw).map_err(|_| bad("profile"))?;
    // the institution first, so the forms of the roster start with its kind; then the people of the example,
    // and the profile adds them up again
    service::save_profile(conn, input.clone(), None)?;
    crate::core::profile::sync::seed_examples(conn, padron)?;
    crate::core::bridge::facilities::seed_example(conn, facilities)?;
    // an example is a finished institution: it does not go through the first start (ADR-031)
    crate::core::onboarding::service::mark_done(conn)?;
    let money: crate::modules::finance::domain::lines::FinanceInput = serde_json::from_str(raw).map_err(|_| bad("money"))?;
    if !matches!(crate::core::bridge::finance::save(conn, money, None)?, crate::core::bridge::finance::FinanceOutcome::Saved { .. }) {
        return Err(bad("money"));
    }
    seed_lists(conn, raw)?;
    match service::save_profile(conn, input, None)? {
        SaveProfileOutcome::Saved { profile } => Ok(Some(profile)),
        _ => Err(bad("profile")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = crate::storage::open_encrypted(&dir.path().join("t.db"), "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff").unwrap();
        (dir, c)
    }

    /// Audit D8: both examples are I.A.P. and come complete, so the real case (Junta, board) is what is tried.
    #[test]
    fn both_examples_are_complete_iap_institutions() {
        for name in ["asilo", "casa-hogar"] {
            let (_d, mut c) = conn();
            let profile = load(&mut c, name).unwrap().unwrap();
            assert_eq!(profile.input.institution.legal_form.as_deref(), Some("iap"), "{name}");
            let o = crate::core::overview::institution_overview(&c).unwrap();
            assert!(o.completion.gaps.is_empty(), "{name}: still missing {:?}", o.completion.gaps);
            assert_eq!(o.completion.percent, 100, "{name}");
            let lists = protected::lists(&c).unwrap();
            assert!(lists.board.len() >= 4 && lists.accounts.len() == 1, "{name}");

            // the AI learns it is registered before its Junta and how the board is made, never a name nor a folio
            let sheet = crate::core::ai_sheet::profile_context(&c).unwrap();
            assert!(sheet.contains("Registrada ante la Junta de Asistencia Privada") && sheet.contains("Patronato u órgano de gobierno"), "{sheet}");
            for secret in ["Ejemplo Ruiz", "Ejemplo Luna", "JAP-", "CLUNI-EJEMPLO", "OFICIO-EJEMPLO", "002010", "012180", "Calle Ejemplo", "Avenida Ejemplo"] {
                assert!(!sheet.contains(secret), "{name}: «{secret}» reached the AI:\n{sheet}");
            }

            // loading it again replaces it: nothing doubles
            load(&mut c, name).unwrap().unwrap();
            let again = protected::lists(&c).unwrap();
            let names = |l: &protected::Lists| l.board.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
            assert_eq!((names(&again), again.accounts.len()), (names(&lists), 1), "{name}");
        }
        let (_d, mut c) = conn();
        assert!(load(&mut c, "nope").unwrap().is_none());
    }

    /// The casa hogar is read as one: girls and adolescents, so its forms ask what fits them.
    #[test]
    fn the_casa_hogar_is_read_as_a_home_for_girls() {
        let (_d, mut c) = conn();
        let p = load(&mut c, "casa-hogar").unwrap().unwrap();
        assert_eq!(crate::core::institution::kind(&c).as_deref(), Some("children_home"));
        assert_eq!(p.input.institution.attention.unwrap().sex_served.as_deref(), Some("women"));
    }
}
