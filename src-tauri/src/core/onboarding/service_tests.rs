//! The first start with the real database: from an empty start to a finished institution, step by step, with the
//! scanner, the checks, the records of the modules counting instead of the quick figures, and what the AI reads.

use super::*;
use crate::core::profile::domain::InstitutionKind;
use crate::modules::finance::domain::money::{IncomeKind, Period};
use crate::storage::open_encrypted;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn conn() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    (dir, c)
}

fn user(c: &Connection, id: &str, role: Role) -> CurrentUser {
    c.execute(
        "INSERT INTO app_user (id,username,display_name,role,password_hash,created_at,updated_at) VALUES (?1,?1,?1,?2,'x','t','t')",
        [id, role.as_db()],
    )
    .unwrap();
    CurrentUser { id: id.into(), username: id.into(), display_name: id.into(), role, must_change_password: false }
}

fn saved(out: OnboardingOutcome) -> OnboardingStatus {
    match out {
        OnboardingOutcome::Saved { onboarding } => onboarding,
        other => panic!("not saved: {other:?}"),
    }
}

fn missing(s: &OnboardingStatus) -> Vec<(&'static str, Vec<&'static str>)> {
    s.steps.iter().filter(|x| !x.complete).map(|x| (x.key, x.missing.clone())).collect()
}

#[test]
fn an_empty_start_shows_every_step_to_fill_and_the_setup_to_the_administrator() {
    let (_d, c) = conn();
    let admin = user(&c, "admin", Role::Admin);
    let s = status(&c, &admin).unwrap();
    assert!(!s.done && !s.ready && !s.welcomed && s.can_postpone);
    assert_eq!(s.steps.len(), 6);
    assert!(s.steps.iter().all(|x| !x.complete));
    let setup = s.setup.unwrap();
    assert_eq!(setup.managers, 0);
    assert_eq!(s.states.len(), 32);

    let director = user(&c, "rosa", Role::Manager);
    let s = status(&c, &director).unwrap();
    assert!(!s.can_postpone && s.setup.is_none(), "the direction cannot leave it for later and does not see the setup");
}

#[test]
fn step_by_step_to_a_finished_institution_that_the_ai_knows() {
    let (_d, mut c) = conn();
    let rosa = user(&c, "rosa", Role::Manager);
    assert!(matches!(finish(&mut c, &rosa), Err(ServiceError::OnboardingIncomplete)));

    // 1. the institution
    let mut d = OnboardingData::default();
    d.institution.name = "Asilo Ficticio".into();
    d.institution.kind = InstitutionKind::ElderlyHome;
    d.institution.mission = Some("Un hogar digno para adultos mayores.".into());
    let s = saved(save(&mut c, &rosa, d.clone(), None).unwrap());
    assert!(s.steps[0].complete);
    assert_eq!(s.data.institution.name, "Asilo Ficticio", "it comes back to go on where it was");

    // 2. where and what it is
    d.institution.state = Some("jal".into());
    d.institution.municipality = Some("Zapopan".into());
    d.institution.contact_email = Some("contacto@asilo-ficticio.org".into());
    d.institution.legal_form = Some("ac".into());
    d.institution.founded_year = Some(1987);
    d.institution.authorized_donee = Some("yes".into());
    d.institution.cluni = Some("in_progress".into());
    // 3, 4, 5. people, team and money
    d.capacity_total = Some(25);
    d.served_estimate = Some(22);
    d.staff_paid_estimate = Some(6);
    d.staff_volunteer_estimate = Some(0);
    d.annual_budget_mxn = Some(1_800_000);
    d.income = vec![IncomeSourceInput { label: "Donativos".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(500_000), period: Period::Annual }];
    let s = saved(save(&mut c, &rosa, d.clone(), None).unwrap());
    assert_eq!(missing(&s), vec![("building", vec!["floors", "tenure"])]);

    // 6. the house: it goes to the facilities module
    d.floors = Some(2);
    d.tenure = Some("loan".into());
    d.tenure_until = Some(2040);
    d.tenure_documented = Some(true);
    let s = saved(save(&mut c, &rosa, d.clone(), None).unwrap());
    assert!(s.ready && !s.done, "{:?}", missing(&s));
    let site = crate::modules::facilities::api::summaries(&c).unwrap().remove(0).site;
    assert_eq!((site.floors, site.tenure.as_deref(), site.tenure_until), (Some(2), Some("loan"), Some(2040)));

    let s = finish(&mut c, &rosa).unwrap();
    assert!(s.done);
    let p = profile_store::load_current(&c).unwrap().unwrap();
    assert!(p.confirmed_at.is_some(), "the first version is confirmed");
    let events: i64 = c.query_row("SELECT count(*) FROM audit_log WHERE event = 'institution.onboarded'", [], |r| r.get(0)).unwrap();
    assert_eq!(events, 1);
    assert!(finish(&mut c, &rosa).unwrap().done, "closing twice changes nothing");

    let sheet = crate::core::ai_sheet::profile_context(&c).unwrap();
    for fact in [
        "Ubicación: Zapopan, Jalisco.",
        "Fundada en 1987 (",
        "Figura jurídica: asociación civil (A.C.).",
        "Donataria autorizada por el SAT: sí.",
        "CLUNI (registro federal de organizaciones de la sociedad civil): en trámite.",
        "Personas atendidas (cifra aproximada que dio la persona; todavía sin registros): 22.",
        "Personal (cifra aproximada que dio la persona; todavía sin registros): 6 con sueldo y 0 de voluntariado.",
        "El inmueble está en comodato hasta 2040",
    ] {
        assert!(sheet.contains(fact), "missing «{fact}» in:\n{sheet}");
    }
    assert!(!sheet.contains("contacto@"), "the contact never reaches the AI:\n{sheet}");
    for gap in ["dónde está", "a quiénes atiende", "el personal", "la figura jurídica"] {
        assert!(!sheet.contains(gap), "«{gap}» is captured:\n{sheet}");
    }
}

#[test]
fn what_does_not_add_up_is_not_saved_and_free_text_goes_through_the_scanner() {
    let (_d, mut c) = conn();
    let rosa = user(&c, "rosa", Role::Manager);
    let mut d = OnboardingData::default();
    d.institution.name = "Casa Ficticia".into();
    d.institution.founded_year = Some(3000);
    d.institution.state = Some("atlantis".into());
    let OnboardingOutcome::Invalid { issues } = save(&mut c, &rosa, d.clone(), None).unwrap() else { panic!("a year to come") };
    let fields: Vec<_> = issues.iter().map(|i| i.field.as_str()).collect();
    assert_eq!(fields, vec!["institution.founded_year", "institution.state"]);

    d.institution.founded_year = Some(1990);
    d.institution.state = Some("pue".into());
    d.institution.mission = Some("Cuidamos niñas. Informes con la directora al 55 1234 5678.".into());
    assert!(matches!(save(&mut c, &rosa, d.clone(), None).unwrap(), OnboardingOutcome::Quarantine { .. }));
    let s = saved(save(&mut c, &rosa, d, Some(Decision::Redact)).unwrap());
    assert!(!s.data.institution.mission.as_deref().unwrap().contains("5678"));

    // a floor count out of range is the site's to say
    let mut d = s.data.clone();
    d.floors = Some(40);
    let OnboardingOutcome::Invalid { issues } = save(&mut c, &rosa, d, None).unwrap() else { panic!("forty floors") };
    assert_eq!(issues[0].field, "site.floors");
}

#[test]
fn the_records_of_the_modules_count_instead_of_the_quick_figures() {
    let (_d, mut c) = conn();
    let rosa = user(&c, "rosa", Role::Manager);
    let mut d = OnboardingData::default();
    d.institution.name = "Asilo Ficticio".into();
    saved(save(&mut c, &rosa, d, None).unwrap());
    for i in 0..3 {
        let p = crate::modules::care::domain::person::BeneficiaryData { first_names: format!("Persona {i}"), approx_age: Some(80), status: "active".into(), ..Default::default() };
        crate::modules::care::service::save_person(&mut c, crate::modules::care::domain::catalog::Flavor::ElderlyHome, None, p).unwrap();
    }
    let s = status(&c, &rosa).unwrap();
    assert_eq!(s.records.served, 3);
    assert_eq!(missing(&s).iter().find(|(k, _)| *k == "people").unwrap().1, vec!["capacity_total"], "served comes from the records");
}

#[test]
fn the_welcome_is_once_per_person() {
    let (_d, c) = conn();
    let rosa = user(&c, "rosa", Role::Manager);
    let lupe = user(&c, "lupe", Role::Manager);
    welcome_done(&c, &rosa).unwrap();
    assert!(status(&c, &rosa).unwrap().welcomed);
    assert!(!status(&c, &lupe).unwrap().welcomed);
}

#[test]
fn what_mi_institucion_saves_is_confirmed_once_the_first_start_is_finished_and_not_before() {
    let (_d, mut c) = conn();
    let rosa = user(&c, "rosa", Role::Manager);
    let mut d = OnboardingData::default();
    d.institution.name = "Asilo Ficticio".into();
    d.institution.mission = Some("Un hogar digno.".into());
    saved(save(&mut c, &rosa, d, None).unwrap());

    // before the first start is finished the profile is a draft, and saving from the page keeps it so
    let mut input = profile_store::load_current(&c).unwrap().unwrap().input;
    input.institution.mission = Some("Un hogar digno para adultos mayores.".into());
    match save_profile_confirmed(&mut c, input, None).unwrap() {
        SaveProfileOutcome::Saved { profile } => assert!(profile.is_draft, "not finished yet: still a draft"),
        other => panic!("not saved: {other:?}"),
    }

    // the institution finishes (everything the steps ask for) and from then on every save is confirmed
    c.execute("UPDATE institution SET onboarded_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')", []).unwrap();
    let mut input = profile_store::load_current(&c).unwrap().unwrap().input;
    input.institution.mission = Some("Un hogar digno y seguro para adultos mayores.".into());
    match save_profile_confirmed(&mut c, input, None).unwrap() {
        SaveProfileOutcome::Saved { profile } => {
            assert!(!profile.is_draft && profile.confirmed_at.is_some(), "after the first start a save is confirmed at once");
            assert_eq!(profile.input.institution.mission.as_deref(), Some("Un hogar digno y seguro para adultos mayores."));
        }
        other => panic!("not saved: {other:?}"),
    }
    let sheet = crate::core::ai_sheet::profile_context(&c).unwrap();
    assert!(!sheet.contains("BORRADOR"), "the AI reads it as confirmed:\n{sheet}");

    // what is invalid is still not saved, and nothing is confirmed
    let mut bad = profile_store::load_current(&c).unwrap().unwrap().input;
    bad.institution.founded_year = Some(3000);
    assert!(matches!(save_profile_confirmed(&mut c, bad, None).unwrap(), SaveProfileOutcome::Invalid { .. }));
}
