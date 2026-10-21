//! Shared scaffolding for the tests of the services: a database with a confirmed profile, and a project born from a
//! call (read and confirmed, or not yet), as far along as the test needs. No model: what the AI used to leave in the
//! base (the root cause, the summary) is written straight into it. Tests only.

use crate::core::api::PageMarks;
use crate::modules::projects::diagnosis::{advance, SharedDb};
use crate::core::profile::domain::*;
use crate::modules::projects::storage::calls::{self, CallMeta, FileRole, NewFile, ReadingStatus};
use crate::storage::open_encrypted;
use crate::core::profile::storage as profile;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

pub const ROOT: &str = "No hay un sistema de mantenimiento con responsable y fondo propio";

pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office").join(name)).unwrap()
}

/// A database with a confirmed profile of a fictional institution.
pub fn profile_db() -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let input = ProfileInput {
        institution: InstitutionInput { name: "Asilo Ficticio".into(), ..Default::default() },
        capacity_total: Some(25),
        population: vec![PopulationGroupInput { label: "Adultos mayores".into(), count: 18, ..Default::default() }],
        ..Default::default()
    };
    // the 18 people served live in their module (ADR-029): the sheet of the AI counts them from there
    for i in 0..18 {
        let d = crate::modules::care::domain::person::BeneficiaryData { first_names: format!("Persona {i}"), approx_age: Some(80), status: "active".into(), ..Default::default() };
        crate::modules::care::service::save_person(&mut conn, crate::modules::care::domain::catalog::Flavor::Other, None, d).unwrap();
    }
    // three bathrooms in poor state, in the facilities module (ADR-030)
    let bathrooms = crate::modules::facilities::domain::group::SpaceData {
        kind: "bathroom".into(),
        count: 3,
        states: crate::modules::facilities::domain::group::States::all(3, "poor"),
        ..Default::default()
    };
    crate::modules::facilities::service::save_space(&conn, None, bathrooms).unwrap();
    profile::save(&mut conn, &input).unwrap();
    profile::confirm(&mut conn).unwrap();
    (dir, Arc::new(Mutex::new(conn)))
}

/// Like `profile_db`, with a profile that has every kind of datum (see `core::ai_sheet::tests`).
pub fn rich_profile_db() -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    crate::core::ai_sheet::tests::seed_rich_staff(&mut conn);
    crate::core::ai_sheet::tests::seed_rich_people(&mut conn);
    crate::core::ai_sheet::tests::seed_rich_facilities(&conn);
    crate::core::ai_sheet::tests::seed_rich_board(&mut conn);
    profile::save(&mut conn, &crate::core::ai_sheet::tests::rich()).unwrap();
    crate::modules::finance::storage::save(&mut conn, &crate::core::ai_sheet::tests::rich_money()).unwrap();
    profile::confirm(&mut conn).unwrap();
    (dir, Arc::new(Mutex::new(conn)))
}

/// What the reading of the call understood, as the canonical document the summary is built from.
pub fn canonical_doc() -> Value {
    let ev = |cita: &str, pagina: u64| json!([{ "cita": cita, "pagina": pagina, "documento": "bases.pdf" }]);
    json!({
        "identidad": {
            "nombre": { "estado": "encontrado", "valor_texto": "Apoyos 2027", "evidencias": ev("APOYOS 2027", 1) },
            "convocante": { "estado": "encontrado", "valor_texto": "Fundación Ficticia", "evidencias": ev("Fundación Ficticia A.C.", 1) },
            "que_apoya": { "estado": "encontrado", "elementos": [{ "titulo": null, "evidencias": ev("Proyectos de alimentación y nutrición.", 2), "aplica_a": null, "obligatoriedad": "no_indicado" }] }
        },
        "elegibilidad": {
            "quienes_pueden_participar": { "estado": "encontrado", "elementos": [{ "titulo": "Casas hogar y asilos", "evidencias": ev("Podrán participar casas hogar y asilos.", 2), "aplica_a": null, "obligatoriedad": "obligatorio" }] }
        },
        "financiamiento": { "monto_maximo": { "estado": "encontrado", "valor_texto": "$250,000", "evidencias": ev("Monto máximo: $250,000 pesos", 2), "normalizado": { "cantidad": 250000.0, "moneda": "MXN" } } }
    })
}

pub fn summary_value(extra: &str) -> Value {
    json!({
        "problem_statement": format!("La casa no tiene un sistema de mantenimiento y por eso el agua sale con óxido {extra}"),
        "affected": {"group": "personas que viven en la casa", "count": 18, "description": "Usan a diario la cocina y los baños"},
        "current_consequences": ["El agua sale con óxido"],
        "root_causes": [ROOT],
        "reframed_need": "Alimentación segura en una casa con mantenimiento preventivo",
        "alternatives": [{"title": "Obra con plan de mantenimiento", "pros": ["Evita que se repita"], "cons": ["Requiere responsable"]}],
        "suggested_indicators": ["Instalaciones de agua conformes"],
        "open_questions": ["Cotizaciones", "Quién mantendrá la solución"]
    })
}

/// A project born from a call that was read (`ready`) but that the person has not confirmed yet.
pub fn project_in_call_selection(db: &SharedDb) -> String {
    let files = vec![NewFile {
        name: "bases.pdf".into(),
        mime: "application/pdf",
        pages: vec!["APOYOS 2027".into(), "Monto máximo: $250,000 pesos".into()],
        redactions: 0,
        role: FileRole::Main,
        marks: PageMarks::default(),
    }];
    let meta = CallMeta { name: "Apoyos 2027".into(), funder: Some("Fundación Ficticia".into()), year: Some(2027) };
    let mut conn = db.lock().unwrap();
    let (pid, reading) = calls::create_project_with_call(&mut conn, "Apoyos 2027", "Arreglar la cocina y los baños de la casa", &meta, &files).unwrap();
    calls::finish(&conn, &reading, ReadingStatus::Ready, None, Some(&canonical_doc()), &json!({})).unwrap();
    crate::modules::projects::storage::projects::set_stage(&mut conn, &pid, crate::modules::projects::domain::stage::Stage::CallSelection, false).unwrap();
    pid
}

/// A project in the diagnosis: its call was read and the person confirmed it.
pub fn project_in_diagnosis(db: &SharedDb) -> String {
    let pid = project_in_call_selection(db);
    {
        let conn = db.lock().unwrap();
        let reading = crate::modules::projects::storage::projects::get_project(&conn, &pid).unwrap().unwrap().call_reading_id.unwrap();
        assert!(calls::confirm(&conn, &reading).unwrap());
    }
    advance(db, &pid).unwrap();
    pid
}

/// What an earlier diagnosis left in the base, written straight into it: a confirmed root cause and a confirmed
/// summary.
pub fn confirm_the_diagnosis(db: &SharedDb, pid: &str) {
    use crate::modules::projects::storage::projects as store;
    let conn = db.lock().unwrap();
    store::set_root(&conn, pid, ROOT, None).unwrap();
    assert!(store::confirm_root(&conn, pid).unwrap());
    store::save_summary(&conn, pid, &summary_value(""), "user", None).unwrap();
    assert!(store::confirm_summary(&conn, pid).unwrap());
}

/// A project in the drafting stage: its call is confirmed, the diagnosis is confirmed and a goal was chosen.
pub async fn project_in_drafting(db: &SharedDb) -> String {
    use crate::modules::projects::diagnosis::{add_need, select_need, AddNeedOutcome};
    let pid = project_in_diagnosis(db);
    confirm_the_diagnosis(db, &pid);
    advance(db, &pid).unwrap();
    let AddNeedOutcome::Saved { view } = add_need(db, &pid, "Un sistema de mantenimiento preventivo", "Con responsable, agenda y fondo propio", None).unwrap() else { panic!() };
    select_need(db, &pid, &view.needs[0].id).unwrap();
    assert_eq!(advance(db, &pid).unwrap().stage, crate::modules::projects::domain::stage::Stage::Drafting);
    pid
}

/// A project with everything drafted and confirmed (every required text, the budget and the schedule), waiting in
/// the drafting stage to move on.
pub async fn project_fully_drafted(db: &SharedDb) -> String {
    use crate::modules::projects::domain::budget::Funder;
    use crate::modules::projects::domain::sections::SectionKind;
    use crate::modules::projects::drafting::{confirm_budget, confirm_schedule, confirm_text, drafting_view, save_activity, save_budget_item, save_text, BudgetItemInput};
    let pid = project_in_drafting(db).await;
    save_budget_item(
        db,
        &pid,
        BudgetItemInput { id: None, category: "material".into(), description: "Tubería de cobre".into(), quantity: 2.0, unit: Some("pieza".into()), unit_price_mxn: 1000.0, vat_included: false, funded_by: Funder::Requested, administrative: false },
        None,
    )
    .unwrap();
    save_budget_item(
        db,
        &pid,
        BudgetItemInput { id: None, category: "mano de obra".into(), description: "Instalación".into(), quantity: 1.0, unit: None, unit_price_mxn: 1160.0, vat_included: true, funded_by: Funder::Institution, administrative: false },
        None,
    )
    .unwrap();
    save_activity(db, &pid, None, "Cambio de tuberías", 1, 3, None).unwrap();
    save_activity(db, &pid, None, "Revisión mensual", 4, 12, None).unwrap();
    let required: Vec<(String, String)> = {
        let conn = db.lock().unwrap();
        drafting_view(&conn, &pid).unwrap().sections.into_iter().filter(|s| s.spec.required && s.spec.kind == SectionKind::Text).map(|s| (s.spec.key, s.spec.title)).collect()
    };
    for (key, title) in required {
        save_text(db, &pid, &key, &format!("Texto de la sección «{title}» escrito por la persona."), None).unwrap();
        confirm_text(db, &pid, &key).unwrap();
    }
    confirm_budget(db, &pid).unwrap();
    confirm_schedule(db, &pid).unwrap();
    pid
}

/// A project that already passed to the review stage.
pub async fn project_in_review(db: &SharedDb) -> String {
    let pid = project_fully_drafted(db).await;
    assert_eq!(advance(db, &pid).unwrap().stage, crate::modules::projects::domain::stage::Stage::Review);
    pid
}

/// A project whose review is clean and that is ready to give its guide.
pub async fn project_ready(db: &SharedDb) -> String {
    let pid = project_in_review(db).await;
    assert_eq!(advance(db, &pid).unwrap().stage, crate::modules::projects::domain::stage::Stage::Ready);
    pid
}
