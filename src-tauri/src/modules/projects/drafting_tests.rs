use super::*;
use crate::test_support::*;
use serde_json::json;

async fn setup() -> (tempfile::TempDir, SharedDb, String) {
    let (d, db) = profile_db();
    let pid = project_in_drafting(&db).await;
    (d, db, pid)
}

fn view(db: &SharedDb, pid: &str) -> DraftingView {
    drafting_view(&db.lock().unwrap(), pid).unwrap()
}

fn item(description: &str, qty: f64, price: f64, funded_by: Funder) -> BudgetItemInput {
    BudgetItemInput { id: None, category: "material".into(), description: description.into(), quantity: qty, unit: Some("pieza".into()), unit_price_mxn: price, vat_included: false, funded_by, administrative: false }
}

fn saved(out: EditOutcome) -> DraftingView {
    match out {
        EditOutcome::Saved { view } => view,
        EditOutcome::Quarantine { .. } => panic!("unexpected quarantine"),
    }
}

/// Gives the call a list of what the proposal must include.
fn call_asks_for_a_proposal(db: &SharedDb, pid: &str) {
    let conn = db.lock().unwrap();
    let reading = projects::get_project(&conn, pid).unwrap().unwrap().call_reading_id.unwrap();
    let mut doc = canonical_doc();
    doc["proyecto"] = json!({ "requisitos_del_proyecto": { "estado": "encontrado", "elementos": [
        { "titulo": "Justificación del problema", "evidencias": [{ "cita": "Justificación del problema", "pagina": 7, "documento": "bases.pdf" }], "aplica_a": null, "obligatoriedad": "obligatorio" },
        { "titulo": "Plan de trabajo con responsables", "evidencias": [{ "cita": "Plan de trabajo con responsables", "pagina": 7, "documento": "bases.pdf" }], "aplica_a": null, "obligatoriedad": "obligatorio" }
    ] } });
    calls::finish(&conn, &reading, crate::modules::projects::storage::calls::ReadingStatus::Ready, None, Some(&doc), &json!({})).unwrap();
}

#[tokio::test]
async fn the_plan_follows_the_call_when_it_asks_for_a_proposal_and_the_person_can_say_otherwise() {
    let (_d, db, pid) = setup().await;
    // this call gives no structure: the base one, and the person has not been asked yet
    let v = view(&db, &pid);
    assert_eq!((v.asks_for_proposal, v.asks_confirmed), (false, false));
    assert_eq!(v.sections.iter().map(|s| s.spec.key.as_str()).collect::<Vec<_>>(), vec!["call_data", "deliverables", "what", "why", "who", "where", "when", "how", "how_much", "results", "sustainability", "budget", "schedule"]);
    assert_eq!(v.requirements.max_amount_mxn.as_ref().map(|a| a.value), Some(250_000.0));

    call_asks_for_a_proposal(&db, &pid);
    let v = view(&db, &pid);
    assert_eq!((v.asks_for_proposal, v.asks_confirmed), (true, false), "the code proposes it");
    let titles: Vec<_> = v.sections.iter().filter(|s| s.spec.source == crate::modules::projects::domain::sections::Source::Call).map(|s| s.spec.title.as_str()).collect();
    assert_eq!(titles, vec!["Justificación del problema", "Plan de trabajo con responsables"]);

    // the person says it is not so: the base structure again, and the answer is kept
    let v = set_asks_for_proposal(&db, &pid, false).unwrap();
    assert_eq!((v.asks_for_proposal, v.asks_confirmed), (false, true));
    assert!(v.sections.iter().all(|s| s.spec.source != crate::modules::projects::domain::sections::Source::Call));
    let v = set_asks_for_proposal(&db, &pid, true).unwrap();
    assert!(v.sections.iter().any(|s| s.spec.source == crate::modules::projects::domain::sections::Source::Call));
}

#[tokio::test]
async fn what_the_person_writes_is_scanned_and_an_empty_text_cannot_be_confirmed() {
    let (_d, db, pid) = setup().await;
    assert!(matches!(confirm_text(&db, &pid, "what"), Err(ProjectsError::Core(crate::core::api::ServiceError::Storage(StorageError::NothingToConfirm)))));
    assert!(matches!(save_text(&db, &pid, "what", "   ", None), Err(ProjectsError::EmptyText)));
    let risky = "La señora con CURP LOPM800101MDFRZN09 pidió la obra";
    assert!(matches!(save_text(&db, &pid, "what", risky, None).unwrap(), EditOutcome::Quarantine { .. }));
    assert_eq!(view(&db, &pid).sections.iter().find(|s| s.spec.key == "what").unwrap().status, SectionStatus::Empty);
    let v = saved(save_text(&db, &pid, "what", risky, Some(Decision::Redact)).unwrap());
    assert!(v.sections.iter().find(|s| s.spec.key == "what").unwrap().content.contains("[CURP OCULTA]"));
}

#[tokio::test]
async fn the_code_adds_up_the_budget_and_a_change_undoes_its_confirmation_and_marks_the_cost_text() {
    let (_d, db, pid) = setup().await;
    assert!(matches!(confirm_budget(&db, &pid), Err(ProjectsError::Core(crate::core::api::ServiceError::Storage(StorageError::NothingToConfirm)))));
    saved(save_budget_item(&db, &pid, item("Tubería", 2.0, 1000.0, Funder::Requested), None).unwrap());
    let v = saved(save_budget_item(&db, &pid, BudgetItemInput { vat_included: true, ..item("Mano de obra", 1.0, 5800.0, Funder::Institution) }, None).unwrap());
    assert_eq!(v.budget.items.len(), 2);
    assert_eq!((v.budget.totals.requested, v.budget.totals.institution, v.budget.totals.total), (2320.0, 5800.0, 8120.0));
    assert_eq!(v.budget.totals.counterpart_percent, 71.43);
    assert_eq!(v.budget.items[0].line.total, 2320.0);

    // the text about the cost was confirmed before the budget changed
    saved(save_text(&db, &pid, "how_much", "Cuesta ocho mil pesos.", None).unwrap());
    confirm_text(&db, &pid, "how_much").unwrap();
    let v = confirm_budget(&db, &pid).unwrap();
    assert!(v.budget.confirmed);
    let id = v.budget.items[0].row.id.clone();
    let v = saved(save_budget_item(&db, &pid, BudgetItemInput { id: Some(id.clone()), ..item("Tubería", 3.0, 1000.0, Funder::Requested) }, None).unwrap());
    assert!(!v.budget.confirmed, "a change takes the confirmation away");
    assert_eq!(v.sections.iter().find(|s| s.spec.key == "how_much").unwrap().status, SectionStatus::NeedsReview);
    let v = delete_budget_item(&db, &pid, &id).unwrap();
    assert_eq!(v.budget.items.len(), 1);
    assert!(matches!(delete_budget_item(&db, &pid, &id), Err(ProjectsError::NotFound)));
}

#[tokio::test]
async fn a_budget_line_with_bad_numbers_or_personal_data_is_not_saved() {
    let (_d, db, pid) = setup().await;
    for bad in [item("Algo", 0.0, 10.0, Funder::Requested), item("Algo", 1.0, -1.0, Funder::Requested), item("Algo", f64::NAN, 1.0, Funder::Requested)] {
        assert!(matches!(save_budget_item(&db, &pid, bad, None), Err(ProjectsError::InvalidBudgetItem)));
    }
    assert!(matches!(save_budget_item(&db, &pid, item("  ", 1.0, 1.0, Funder::Requested), None), Err(ProjectsError::EmptyText)));
    assert!(matches!(save_budget_item(&db, &pid, item("Pago a LOPM800101MDFRZN09", 1.0, 1.0, Funder::Requested), None).unwrap(), EditOutcome::Quarantine { .. }));
    assert!(matches!(save_budget_item(&db, &pid, BudgetItemInput { id: Some("bud_x".into()), ..item("Algo", 1.0, 1.0, Funder::Requested) }, None), Err(ProjectsError::NotFound)));
    assert!(view(&db, &pid).budget.items.is_empty());
}

#[tokio::test]
async fn the_schedule_is_validated_by_the_code_and_its_duration_is_the_last_month() {
    let (_d, db, pid) = setup().await;
    for (s, e) in [(0, 2), (5, 4), (1, 700)] {
        assert!(matches!(save_activity(&db, &pid, None, "Obra", s, e, None), Err(ProjectsError::InvalidActivity)), "{s}-{e}");
    }
    assert!(matches!(save_activity(&db, &pid, None, " ", 1, 2, None), Err(ProjectsError::EmptyText)));
    saved(save_activity(&db, &pid, None, "Compra de material", 1, 2, None).unwrap());
    let v = saved(save_activity(&db, &pid, None, "Obra", 3, 8, None).unwrap());
    assert_eq!((v.schedule.activities.len(), v.schedule.duration_months, v.schedule.confirmed), (2, 8, false));
    assert!(confirm_schedule(&db, &pid).unwrap().schedule.confirmed);
    let id = view(&db, &pid).schedule.activities[1].id.clone();
    let v = delete_activity(&db, &pid, &id).unwrap();
    assert_eq!((v.schedule.duration_months, v.schedule.confirmed), (2, false));
}

#[tokio::test]
async fn the_stage_condition_is_every_required_section_plus_the_budget_and_the_schedule() {
    let (_d, db, pid) = setup().await;
    assert!(!sections_confirmed(&db.lock().unwrap(), &pid).unwrap());
    saved(save_budget_item(&db, &pid, item("Tubería", 1.0, 1000.0, Funder::Requested), None).unwrap());
    saved(save_activity(&db, &pid, None, "Obra", 1, 4, None).unwrap());
    let required: Vec<_> = view(&db, &pid).sections.iter().filter(|s| s.spec.required && s.spec.kind == SectionKind::Text).map(|s| s.spec.key.clone()).collect();
    for key in &required {
        saved(save_text(&db, &pid, key, "Texto de la sección con lo que la persona quiso decir.", None).unwrap());
        confirm_text(&db, &pid, key).unwrap();
    }
    assert!(!sections_confirmed(&db.lock().unwrap(), &pid).unwrap(), "the budget and the schedule are still unconfirmed");
    confirm_budget(&db, &pid).unwrap();
    assert!(!sections_confirmed(&db.lock().unwrap(), &pid).unwrap());
    confirm_schedule(&db, &pid).unwrap();
    assert!(sections_confirmed(&db.lock().unwrap(), &pid).unwrap());
    // anything that changes after confirming puts the project back in review: the schedule and the text about it
    saved(save_activity(&db, &pid, None, "Otra", 5, 6, None).unwrap());
    assert!(!sections_confirmed(&db.lock().unwrap(), &pid).unwrap());
    let v = view(&db, &pid);
    assert_eq!(v.sections.iter().find(|s| s.spec.key == "when").unwrap().status, SectionStatus::NeedsReview);
    assert!(!v.schedule.confirmed);
}

#[tokio::test]
async fn nothing_of_the_drafting_can_be_touched_outside_its_stage() {
    let (_d, db) = profile_db();
    let pid = project_in_diagnosis(&db);
    assert!(matches!(set_asks_for_proposal(&db, &pid, true), Err(ProjectsError::WrongStage)));
    assert!(matches!(save_text(&db, &pid, "what", "x", None), Err(ProjectsError::WrongStage)));
    assert!(matches!(save_budget_item(&db, &pid, item("a", 1.0, 1.0, Funder::Requested), None), Err(ProjectsError::WrongStage)));
    assert!(matches!(save_activity(&db, &pid, None, "a", 1, 2, None), Err(ProjectsError::WrongStage)));
}
