use super::*;
use crate::ai::mock::MockProvider;
use crate::ai::{AiError, AiResponse};
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

fn section_reply(content: &str, open: &[&str]) -> Result<AiResponse, AiError> {
    Ok(AiResponse { value: json!({ "content": content, "open_points": open }), model: "gemini-3.5-flash".into(), usage: usage() })
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
async fn the_ai_writes_a_section_from_what_is_confirmed_and_it_is_a_draft_until_the_person_confirms() {
    let (_d, db, pid) = setup().await;
    saved(save_budget_item(&db, &pid, item("Tubería de cobre", 2.0, 1000.0, Funder::Requested), None).unwrap());
    saved(save_activity(&db, &pid, None, "Cambio de tuberías", 1, 3, None).unwrap());
    let p = MockProvider::new(vec![section_reply("La institución realizará el cambio de tuberías para tener agua segura.", &["Quién hará la obra"])]);
    let out = draft_section(&db, Some(&p), &pid, "what").await.unwrap();
    assert_eq!(out.ai, AiStatus::Used);
    let s = out.view.sections.iter().find(|s| s.spec.key == "what").unwrap();
    assert_eq!((s.status, s.open_points.clone()), (SectionStatus::DraftAi, vec!["Quién hará la obra".to_string()]));
    // the AI was told which section, and had the confirmed context and the numbers the code added up
    let req = &p.requests()[0];
    assert_eq!(req.task, AiTask::DraftingSection);
    let ctx = req.context.join("\n");
    assert!(ctx.contains("Sección que se redacta: Qué se va a hacer"), "{ctx}");
    assert!(ctx.contains("Causa de fondo confirmada") && ctx.contains(ROOT), "the root cause");
    assert!(ctx.contains("Objetivo del proyecto:\nUn sistema de mantenimiento preventivo"), "the goal");
    assert!(ctx.contains("Total del proyecto: $2,320.00") && ctx.contains("Cambio de tuberías: del mes 1 al mes 3"), "{ctx}");
    assert!(ctx.contains("Quién convoca: Fundación Ficticia"), "the call");

    let v = confirm_text(&db, &pid, "what").unwrap();
    assert_eq!(v.sections.iter().find(|s| s.spec.key == "what").unwrap().status, SectionStatus::Confirmed);
    // editing it goes back to a draft that is the person's
    let v = saved(save_text(&db, &pid, "what", "La institución cambiará las tuberías de la cocina.", None).unwrap());
    let s = v.sections.iter().find(|s| s.spec.key == "what").unwrap();
    assert_eq!((s.status, s.open_points.is_empty()), (SectionStatus::DraftUser, true));
}

#[tokio::test]
async fn a_figure_nobody_gave_is_asked_again_once_and_flagged_if_it_stays() {
    let (_d, db, pid) = setup().await;
    let p = MockProvider::new(vec![section_reply("Se atenderá a 412 personas.", &[]), section_reply("Se atenderá a 18 personas.", &[])]);
    let out = draft_section(&db, Some(&p), &pid, "who").await.unwrap();
    assert_eq!(p.requests().len(), 2);
    assert!(p.requests()[1].user.contains("Estas cifras no constan en lo que se le dio: 412"));
    let s = out.view.sections.iter().find(|s| s.spec.key == "who").unwrap();
    assert_eq!((s.content.as_str(), s.unsupported_figures.is_empty()), ("Se atenderá a 18 personas.", true), "18 is in the profile");

    let p = MockProvider::new(vec![section_reply("Se atenderá a 412 personas.", &[]), section_reply("Se atenderá a 412 personas.", &[])]);
    let out = draft_section(&db, Some(&p), &pid, "who").await.unwrap();
    let s = out.view.sections.iter().find(|s| s.spec.key == "who").unwrap();
    assert_eq!(s.unsupported_figures, vec!["412".to_string()], "the person is warned before confirming");
    let v = saved(save_text(&db, &pid, "who", "Se atenderá a 18 personas.", None).unwrap());
    assert!(v.sections.iter().find(|s| s.spec.key == "who").unwrap().unsupported_figures.is_empty());
}

#[tokio::test]
async fn if_the_ai_fails_nothing_is_written_and_only_text_sections_can_be_drafted() {
    let (_d, db, pid) = setup().await;
    let out = draft_section(&db, None, &pid, "what").await.unwrap();
    assert_eq!(out.ai, AiStatus::NotConfigured);
    assert_eq!(out.view.sections.iter().find(|s| s.spec.key == "what").unwrap().status, SectionStatus::Empty);
    let p = MockProvider::new(vec![Err(AiError::Auth)]);
    assert_eq!(draft_section(&db, Some(&p), &pid, "what").await.unwrap().ai, AiStatus::KeyRejected);
    // the facts, the budget and the schedule are not written by the AI
    for key in ["call_data", "deliverables", "budget", "schedule", "nope"] {
        assert!(matches!(draft_section(&db, Some(&p), &pid, key).await, Err(ProjectsError::NotFound)), "{key}");
    }
    // an answer with no text is not a section
    let p = MockProvider::new(vec![section_reply("   ", &[])]);
    assert_eq!(draft_section(&db, Some(&p), &pid, "what").await.unwrap().ai, AiStatus::Unavailable);
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
    assert!(matches!(draft_section(&db, None, &pid, "what").await, Err(ProjectsError::WrongStage)));
}

// ------------------------------------------------------------------ the assistant prepares the draft (ADR-021)

fn plan_reply(sections: Value, budget_lines: Value, activities: Value) -> Result<AiResponse, AiError> {
    Ok(AiResponse { value: json!({ "sections": sections, "budget_lines": budget_lines, "activities": activities }), model: "gemini-3.5-flash".into(), usage: usage() })
}

fn all_reply(sections: Value) -> Result<AiResponse, AiError> {
    Ok(AiResponse { value: json!({ "sections": sections }), model: "gemini-3.5-flash".into(), usage: usage() })
}

fn line(description: &str, quantity: Value, funded_by: &str) -> Value {
    json!({ "description": description, "category": "material", "quantity": quantity, "unit": "pieza", "funded_by": funded_by, "administrative": false })
}

#[tokio::test]
async fn the_assistant_prepares_the_explanations_the_budget_lines_and_the_schedule_once_and_the_person_only_writes_costs() {
    let (_d, db, pid) = setup().await;
    call_asks_for_a_proposal(&db, &pid);
    let call_sections: Vec<String> = view(&db, &pid).sections.iter().filter(|s| s.spec.source == crate::modules::projects::domain::sections::Source::Call).map(|s| s.spec.key.clone()).collect();
    assert_eq!(call_sections.len(), 2);
    assert!(!view(&db, &pid).plan_ready);

    let p = MockProvider::new(vec![plan_reply(
        json!([
            { "key": call_sections[0], "title": "Justificación del problema", "plain": "Cuente por qué hace falta el proyecto y qué pasa hoy." },
            { "key": call_sections[1], "title": "Plan de trabajo", "plain": "Explique qué actividades harán y quién se encarga." },
            { "key": "no_existe", "title": "Ignorada", "plain": "No es una sección del proyecto." }
        ]),
        json!([
            line("Tubería de cobre", json!(18), "requested"),   // 18 is in the profile: the quantity stays
            line("Llaves y conexiones", json!(9999), "requested"), // nobody said it: the quantity is 1
            line("Mano de obra de plomería", Value::Null, "institution"),
            { "description": " ", "category": "material", "quantity": null, "unit": null, "funded_by": "requested", "administrative": false }
        ]),
        json!([
            { "title": "Compra de material", "start_month": 1, "end_month": 2 },
            { "title": "Cambio de tuberías", "start_month": 3, "end_month": 6 },
            { "title": "Fuera de plazo", "start_month": 20, "end_month": 40 },
            { "title": "Al revés", "start_month": 5, "end_month": 2 }
        ]),
    )]);
    let out = prepare_plan(&db, Some(&p), &pid).await.unwrap();
    assert_eq!(out.ai, AiStatus::Used);
    let v = out.view;
    assert!(v.plan_ready);

    // a clear title and plain words for the sections of the call; what is not a section is ignored
    let first = v.sections.iter().find(|s| s.spec.key == call_sections[0]).unwrap();
    assert_eq!((first.spec.title.as_str(), first.plain.as_deref()), ("Justificación del problema", Some("Cuente por qué hace falta el proyecto y qué pasa hoy.")));
    assert_eq!(first.spec.guidance, "Justificación del problema", "what the call says is kept as it was");
    assert!(v.sections.iter().all(|s| s.spec.key != "no_existe"));

    // budget lines with no price, marked as proposed; only a quantity somebody said is kept
    let items = &v.budget.items;
    assert_eq!(
        items.iter().map(|i| (i.row.description.as_str(), i.row.quantity, i.row.origin.as_str(), i.row.unit_price_mxn)).collect::<Vec<_>>(),
        vec![("Tubería de cobre", 18.0, "ai_assumption", 0.0), ("Llaves y conexiones", 1.0, "ai_assumption", 0.0), ("Mano de obra de plomería", 1.0, "ai_assumption", 0.0)]
    );
    assert_eq!((items[2].row.funded_by, v.budget.missing_prices, v.budget.confirmed), (Funder::Institution, 3, false));
    // the schedule it proposed, without what did not fit
    assert_eq!(
        v.schedule.activities.iter().map(|a| (a.title.as_str(), a.start_month, a.end_month, a.origin.as_str())).collect::<Vec<_>>(),
        vec![("Compra de material", 1, 2, "ai_assumption"), ("Cambio de tuberías", 3, 6, "ai_assumption")]
    );

    // it was told which sections there are, what the goal is and the longest the project may last
    let ctx = p.requests()[0].context.join("\n");
    assert_eq!(p.requests()[0].task, AiTask::DraftingPlan);
    assert!(ctx.contains(&format!("- {} | Justificación del problema |", call_sections[0])) && ctx.contains("Objetivo del proyecto:\nUn sistema de mantenimiento preventivo"), "{ctx}");

    // it runs once: coming back does not ask again
    let again = prepare_plan(&db, Some(&p), &pid).await.unwrap();
    assert_eq!((again.ai, p.requests().len()), (AiStatus::Skipped, 1));

    // the budget cannot be confirmed with lines that have no cost; the person writes them and then it can
    assert!(matches!(confirm_budget(&db, &pid), Err(ProjectsError::BudgetIncomplete)));
    for (row, price) in v.budget.items.iter().map(|i| &i.row).zip([85.0, 120.0, 4000.0]) {
        let input = BudgetItemInput {
            id: Some(row.id.clone()),
            category: row.category.clone(),
            description: row.description.clone(),
            quantity: row.quantity,
            unit: row.unit.clone(),
            unit_price_mxn: price,
            vat_included: false,
            funded_by: row.funded_by,
            administrative: false,
        };
        saved(save_budget_item(&db, &pid, input, None).unwrap());
    }
    let v = view(&db, &pid);
    assert_eq!((v.budget.missing_prices, v.budget.items.iter().all(|i| i.row.origin == "user")), (0, true), "touched by the person: it is theirs now");
    assert!(confirm_budget(&db, &pid).unwrap().budget.confirmed);
}

#[tokio::test]
async fn if_the_assistant_is_not_there_nothing_is_prepared_and_the_person_can_try_again_or_go_on_by_hand() {
    let (_d, db, pid) = setup().await;
    assert_eq!(prepare_plan(&db, None, &pid).await.unwrap().ai, AiStatus::NotConfigured);
    assert!(!view(&db, &pid).plan_ready);
    let p = MockProvider::new(vec![Err(AiError::Auth)]);
    assert_eq!(prepare_plan(&db, Some(&p), &pid).await.unwrap().ai, AiStatus::KeyRejected);
    let v = view(&db, &pid);
    assert!(!v.plan_ready && v.budget.items.is_empty() && v.schedule.activities.is_empty());
    // a person's own lines are never replaced by a proposal
    saved(save_budget_item(&db, &pid, item("Tubería propia", 1.0, 100.0, Funder::Requested), None).unwrap());
    let p = MockProvider::new(vec![plan_reply(json!([]), json!([line("Otra cosa", Value::Null, "requested")]), json!([]))]);
    let v = prepare_plan(&db, Some(&p), &pid).await.unwrap().view;
    assert_eq!(v.budget.items.iter().map(|i| i.row.description.as_str()).collect::<Vec<_>>(), vec!["Tubería propia"]);
    assert!(v.plan_ready);
}

#[tokio::test]
async fn the_assistant_writes_every_pending_section_in_one_call_and_leaves_what_the_person_wrote() {
    let (_d, db, pid) = setup().await;
    saved(save_text(&db, &pid, "why", "Lo escribí yo.", None).unwrap());
    saved(save_text(&db, &pid, "who", "Se atenderá a 18 personas.", None).unwrap());
    confirm_text(&db, &pid, "who").unwrap();
    let pending: Vec<String> = view(&db, &pid).sections.iter().filter(|s| s.spec.kind == SectionKind::Text && s.status == SectionStatus::Empty).map(|s| s.spec.key.clone()).collect();
    assert_eq!(pending.len(), 7);
    let sections = pending
        .iter()
        .map(|k| json!({ "key": k, "content": format!("Texto de {k}."), "open_points": [] }))
        .chain([json!({ "key": "why", "content": "No debe pisar lo del usuario.", "open_points": [] })])
        .collect::<Vec<_>>();
    let p = MockProvider::new(vec![all_reply(Value::Array(sections))]);
    let out = draft_all(&db, Some(&p), &pid, DraftMode::Full).await.unwrap();
    assert_eq!(out.ai, AiStatus::Used);
    assert_eq!(p.requests().len(), 1, "one call for all of them");
    let req = &p.requests()[0];
    assert_eq!(req.task, AiTask::DraftingAll);
    assert!(req.user.contains("Modo: borrador completo"));
    let ctx = req.context.join("\n");
    assert!(ctx.contains("Secciones que hay que cubrir") && ctx.contains("- what |") && !ctx.contains("- why |") && !ctx.contains("- who |"), "{ctx}");
    let v = out.view;
    for s in v.sections.iter().filter(|s| pending.contains(&s.spec.key)) {
        assert_eq!((s.status, s.content.clone()), (SectionStatus::DraftAi, format!("Texto de {}.", s.spec.key)));
    }
    assert_eq!(v.sections.iter().find(|s| s.spec.key == "why").unwrap().content, "Lo escribí yo.");
    assert_eq!(v.sections.iter().find(|s| s.spec.key == "who").unwrap().status, SectionStatus::Confirmed);

    // the drafts of the assistant can be written again; the person's texts never
    let p2 = MockProvider::new(vec![all_reply(json!([{ "key": "what", "content": "Otra versión.", "open_points": [] }]))]);
    let out = draft_all(&db, Some(&p2), &pid, DraftMode::Guide).await.unwrap();
    assert_eq!(out.ai, AiStatus::Used);
    assert!(!p2.requests()[0].context.join("\n").contains("- why |"));
}

#[tokio::test]
async fn the_short_guide_asks_for_a_guide_and_a_figure_nobody_gave_is_asked_again_once_and_flagged_on_its_section() {
    let (_d, db, pid) = setup().await;
    let all = |content_who: &str| {
        let keys = ["what", "why", "who", "where", "when", "how", "how_much", "results", "sustainability"];
        all_reply(Value::Array(keys.iter().map(|k| json!({ "key": k, "content": if *k == "who" { content_who.to_string() } else { format!("- Punto de {k}") }, "open_points": [] })).collect()))
    };
    let p = MockProvider::new(vec![all("- Atenderá a 412 personas"), all("- Atenderá a 412 personas")]);
    let out = draft_all(&db, Some(&p), &pid, DraftMode::Guide).await.unwrap();
    assert_eq!(p.requests().len(), 2);
    assert!(p.requests()[0].user.contains("Modo: guía breve"));
    assert!(p.requests()[1].user.contains("Estas cifras no constan en lo que se le dio (who: 412)"));
    let who = out.view.sections.iter().find(|s| s.spec.key == "who").unwrap();
    assert_eq!(who.unsupported_figures, vec!["412".to_string()]);
    assert!(out.view.sections.iter().filter(|s| s.spec.key != "who" && s.spec.kind == SectionKind::Text).all(|s| s.unsupported_figures.is_empty()));

    // «confirm everything that is ready» leaves out the flagged one, and the rest is confirmed at once
    let v = confirm_all_texts(&db, &pid).unwrap();
    assert_eq!(v.sections.iter().find(|s| s.spec.key == "who").unwrap().status, SectionStatus::DraftAi);
    assert!(v.sections.iter().filter(|s| s.spec.kind == SectionKind::Text && s.spec.key != "who").all(|s| s.status == SectionStatus::Confirmed));
}

#[tokio::test]
async fn writing_everything_needs_the_assistant_and_the_drafting_stage() {
    let (_d, db, pid) = setup().await;
    assert_eq!(draft_all(&db, None, &pid, DraftMode::Full).await.unwrap().ai, AiStatus::NotConfigured);
    let p = MockProvider::new(vec![all_reply(json!([]))]);
    assert_eq!(draft_all(&db, Some(&p), &pid, DraftMode::Full).await.unwrap().ai, AiStatus::Unavailable, "an answer with no text is not a draft");
    let (_d2, db2) = profile_db();
    let early = project_in_diagnosis(&db2);
    assert!(matches!(draft_all(&db2, None, &early, DraftMode::Full).await, Err(ProjectsError::WrongStage)));
    assert!(matches!(prepare_plan(&db2, None, &early).await, Err(ProjectsError::WrongStage)));
}
