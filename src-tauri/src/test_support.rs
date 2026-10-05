//! Shared scaffolding for the tests of the services: a database with a confirmed profile, a project born from a
//! call (read and confirmed, or not yet) and a scripted model for the conversation. Tests only.

use crate::ai::mock::MockProvider;
use crate::ai::{AiError, AiProvider, AiResponse, Usage};
use crate::call_service::{create_project_from_call, NewProjectOutcome, PackageFile, UploadedFile};
use crate::conversation_service::{send_message, start_conversation, AnswerOutcome, ConversationView};
use crate::diagnosis_service::{advance, SharedDb};
use crate::domain::profile::*;
use crate::storage::calls::{self, FileRole, ReadingStatus};
use crate::storage::{open_encrypted, profile};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// What the person writes in the scripted conversation. Every cause quoted below is a literal piece of these.
pub const OPENING_ANSWER: &str = "Queremos arreglar la cocina y los baños de la casa para que las niñas coman y se bañen seguras, porque el agua sale con óxido y no hemos tenido dinero; así mejoraría su alimentación y bajarían los malestares.";
pub const WHY1_ANSWER: &str = "No hay quien se encargue del mantenimiento de la casa, solo se arregla cuando algo falla.";
pub const WHY2_ANSWER: &str = "Nadie ha hecho un plan de mantenimiento porque la directora atiende todo sola.";
pub const WHY3_ANSWER: &str = "No existe un fondo para mantenimiento, el dinero se usa en lo urgente de cada día.";
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
        facilities: vec![FacilityInput { kind: "Baño".into(), count: 3, condition: Some(Condition::Poor), ..Default::default() }],
        ..Default::default()
    };
    profile::save(&mut conn, &input).unwrap();
    profile::confirm(&mut conn).unwrap();
    (dir, Arc::new(Mutex::new(conn)))
}

/// Like `profile_db`, with a profile that has every kind of datum (see `institution_context::tests`).
pub fn rich_profile_db() -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    profile::save(&mut conn, &crate::institution_context::tests::rich()).unwrap();
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

/// A project born from a call that was read (`ready`) but that the person has not confirmed yet.
pub fn project_in_call_selection(db: &SharedDb) -> String {
    let files = vec![PackageFile { file: UploadedFile { name: "bases.pdf".into(), bytes: fixture("convocatoria-con-tablas.pdf") }, role: FileRole::Main }];
    let NewProjectOutcome::Created { project, reading } = create_project_from_call(db, files, "Apoyos 2027", Some("Fundación Ficticia"), Some(2027), None).unwrap() else {
        panic!("the project was not created")
    };
    let conn = db.lock().unwrap();
    calls::finish(&conn, &reading.id, ReadingStatus::Ready, None, Some(&canonical_doc()), &json!({})).unwrap();
    project.id
}

/// A project in the diagnosis: its call was read and the person confirmed it.
pub fn project_in_diagnosis(db: &SharedDb) -> String {
    let pid = project_in_call_selection(db);
    {
        let conn = db.lock().unwrap();
        let reading = crate::storage::projects::get_project(&conn, &pid).unwrap().unwrap().call_reading_id.unwrap();
        assert!(calls::confirm(&conn, &reading).unwrap());
    }
    advance(db, &pid).unwrap();
    pid
}

pub fn usage() -> Usage {
    Usage { input_tokens: 800, output_tokens: 120, ..Default::default() }
}

/// The reply of the model for one turn of the conversation.
pub fn turn_reply(message: &str, cause: Option<(&str, &str)>, hypothesis: Option<&str>, options: &[&str], fit: Option<(&str, &str)>) -> Result<AiResponse, AiError> {
    Ok(AiResponse {
        value: json!({
            "message": message,
            "cause": cause.map(|(text, quote)| json!({ "text": text, "quote": quote })),
            "root_hypothesis": hypothesis,
            "options": options,
            "fit": fit.map(|f| f.0),
            "fit_note": fit.map_or("", |f| f.1),
        }),
        model: "gemini-3.5-flash-lite".into(),
        usage: usage(),
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

pub fn summary_ok(v: Value) -> Result<AiResponse, AiError> {
    Ok(AiResponse { value: v, model: "gemini-3.5-flash".into(), usage: Usage { input_tokens: 3000, output_tokens: 900, ..Default::default() } })
}

pub fn view_of(out: AnswerOutcome) -> ConversationView {
    match out {
        AnswerOutcome::Saved { view, .. } => view,
        AnswerOutcome::Quarantine { .. } => panic!("unexpected quarantine"),
    }
}

/// The person writes a message and the model answers.
pub async fn say(db: &SharedDb, p: &dyn AiProvider, pid: &str, text: &str) -> ConversationView {
    view_of(send_message(db, Some(p), pid, text, false, None).await.unwrap())
}

/// The replies of the happy path: opening, first «why», second, third and the root cause proposed.
pub fn happy_replies() -> Vec<Result<AiResponse, AiError>> {
    vec![
        turn_reply("Hola, para la convocatoria «Apoyos 2027»: ¿qué proyecto tienen en mente, por qué no se ha podido y cómo cambiaría la vida de las niñas?", None, None, &[], None),
        turn_reply("Entiendo. ¿Por qué no se ha podido arreglar?", None, None, &[], Some(("fits", ""))),
        turn_reply("¿Y por qué no hay quien se encargue?", Some(("No hay quien dé mantenimiento", "solo se arregla cuando algo falla")), None, &[], None),
        turn_reply("¿Por qué nadie hizo un plan?", Some(("Nadie hizo un plan", "la directora atiende todo sola")), None, &[], None),
        turn_reply("Entonces la causa parece ser la falta de un sistema.", Some(("No hay fondo para mantenimiento", "No existe un fondo para mantenimiento")), Some(ROOT), &[], None),
    ]
}

/// Runs the happy path with the scripted model until the person confirms the root cause. Returns the provider so
/// the test can look at what was asked.
pub async fn reach_confirmed_root(db: &SharedDb, pid: &str) -> MockProvider {
    let p = MockProvider::new(happy_replies());
    view_of(start_conversation(db, Some(&p), pid).await.unwrap());
    say(db, &p, pid, OPENING_ANSWER).await;
    say(db, &p, pid, WHY1_ANSWER).await;
    say(db, &p, pid, WHY2_ANSWER).await;
    let v = say(db, &p, pid, WHY3_ANSWER).await;
    assert_eq!(v.phase, crate::domain::conversation::Phase::RootProposed);
    let v = view_of(send_message(db, Some(&p), pid, "", true, None).await.unwrap());
    assert_eq!(v.phase, crate::domain::conversation::Phase::Closed);
    p
}

/// A project in the drafting stage: its call is confirmed, the conversation ended in a confirmed root cause, the
/// summary is confirmed and a goal was chosen.
pub async fn project_in_drafting(db: &SharedDb) -> String {
    use crate::diagnosis_service::{add_need, confirm_summary, generate_summary, select_need, AddNeedOutcome};
    let pid = project_in_diagnosis(db);
    reach_confirmed_root(db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    generate_summary(db, Some(&p), &pid).await.unwrap();
    confirm_summary(db, &pid).unwrap();
    advance(db, &pid).unwrap();
    let AddNeedOutcome::Saved { view } = add_need(db, &pid, "Un sistema de mantenimiento preventivo", "Con responsable, agenda y fondo propio", None).unwrap() else { panic!() };
    select_need(db, &pid, &view.needs[0].id).unwrap();
    assert_eq!(advance(db, &pid).unwrap().stage, crate::domain::stage::Stage::Drafting);
    pid
}

/// A project with everything drafted and confirmed (every required text, the budget and the schedule), waiting in
/// the drafting stage to move on.
pub async fn project_fully_drafted(db: &SharedDb) -> String {
    use crate::domain::budget::Funder;
    use crate::domain::sections::SectionKind;
    use crate::drafting_service::{confirm_budget, confirm_schedule, confirm_text, drafting_view, save_activity, save_budget_item, save_text, BudgetItemInput};
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
    assert_eq!(advance(db, &pid).unwrap().stage, crate::domain::stage::Stage::Review);
    pid
}

/// A project whose review is clean and that is ready to give its guide.
pub async fn project_ready(db: &SharedDb) -> String {
    let pid = project_in_review(db).await;
    assert_eq!(advance(db, &pid).unwrap().stage, crate::domain::stage::Stage::Ready);
    pid
}
