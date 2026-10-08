use super::*;
use std::sync::{Arc, Mutex};
use crate::ai::mock::MockProvider;
use crate::ai::{AiError, AiResponse};
use crate::conversation_service::{conversation_view, send_message, start_conversation, AnswerOutcome};
use crate::core::profile::domain::*;
use crate::storage::open_encrypted;
use crate::core::profile::storage as profile;
use crate::test_support::*;

fn setup() -> (tempfile::TempDir, SharedDb, String) {
    let (d, db) = profile_db();
    let pid = project_in_diagnosis(&db);
    (d, db, pid)
}

fn needs_reply() -> Result<AiResponse, AiError> {
    Ok(AiResponse {
        value: json!({"needs": [
            {"title": "Mantenimiento preventivo de la casa", "description": "Un sistema con responsable y fondo"},
            {"title": "Capacitación del personal", "description": "Quién revisa y cuándo"}
        ]}),
        model: "gemini-3.5-flash".into(),
        usage: usage(),
    })
}

#[tokio::test]
async fn an_internal_project_needs_a_confirmed_profile() {
    let dir = tempfile::tempdir().unwrap();
    let db: SharedDb = Arc::new(Mutex::new(open_encrypted(&dir.path().join("t.db"), KEY).unwrap()));
    assert!(matches!(
        create_project(&db, "x", None),
        Err(ServiceError::Stage(StageError::NotReady(Missing::ProfileNotConfirmed)))
    ));
    assert!(store::list_projects(&db.lock().unwrap()).unwrap().is_empty());
}

#[tokio::test]
async fn a_project_born_from_a_call_waits_until_the_person_confirms_it_and_then_has_to_finish_the_conversation() {
    let (_d, db) = profile_db();
    let pid = project_in_call_selection(&db);
    assert_eq!(store::get_project(&db.lock().unwrap(), &pid).unwrap().unwrap().stage, Stage::CallSelection);
    assert!(matches!(advance(&db, &pid), Err(ServiceError::Stage(StageError::NotReady(Missing::CallNotReady)))));
    let reading = store::get_project(&db.lock().unwrap(), &pid).unwrap().unwrap().call_reading_id.unwrap();
    assert!(crate::call_service::confirm(&db, &reading).unwrap());
    assert_eq!(advance(&db, &pid).unwrap().stage, Stage::Diagnosis);
    // the diagnosis ends when the root cause is confirmed and then the summary
    assert!(matches!(advance(&db, &pid), Err(ServiceError::Stage(StageError::NotReady(Missing::DiagnosisIncomplete)))));
    reach_confirmed_root(&db, &pid).await;
    assert!(matches!(advance(&db, &pid), Err(ServiceError::Stage(StageError::NotReady(Missing::SummaryNotConfirmed)))));
}

#[tokio::test]
async fn reading_the_call_again_takes_away_the_confirmation_and_the_project_cannot_move_on() {
    let (_d, db) = profile_db();
    let pid = project_in_call_selection(&db);
    let reading = store::get_project(&db.lock().unwrap(), &pid).unwrap().unwrap().call_reading_id.unwrap();
    // a call read in part can be read again (a complete one cannot: reading it again would only spend the allowance)
    crate::storage::calls::finish(&db.lock().unwrap(), &reading, crate::storage::calls::ReadingStatus::Partial, Some("unavailable"), Some(&canonical_doc()), &json!({})).unwrap();
    assert!(crate::call_service::confirm(&db, &reading).unwrap());
    assert!(store::facts(&db.lock().unwrap(), &pid).unwrap().call_confirmed);
    // «Leer otra vez»
    assert!(crate::call_service::prepare_retry(&db, &reading).unwrap());
    assert!(!store::facts(&db.lock().unwrap(), &pid).unwrap().call_confirmed);
    assert!(matches!(advance(&db, &pid), Err(ServiceError::Stage(StageError::NotReady(Missing::CallNotReady)))));
}

#[tokio::test]
async fn the_summary_waits_for_the_confirmed_root_cause_and_reads_the_whole_conversation() {
    let (_d, db, pid) = setup();
    assert!(matches!(generate_summary(&db, None, &pid).await, Err(ServiceError::WrongStage)));
    reach_confirmed_root(&db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    let out = generate_summary(&db, Some(&p), &pid).await.unwrap();
    assert_eq!(out.ai, AiStatus::Used);
    let ctx = p.requests()[0].context.join("\n");
    assert!(ctx.contains("Espacio: Baños, planta baja: 3 (3 mal)."), "the profile: {ctx}");
    assert!(ctx.contains("Quién convoca: Fundación Ficticia"), "the call");
    assert!(ctx.contains("Persona: No hay quien se encargue"), "what the person said");
    assert!(ctx.contains(&format!("Causa de fondo que la persona confirmó:\n{ROOT}")), "the root cause");
    let s = out.view.summary.unwrap();
    assert_eq!(s.origin, "ai_assumption");
    assert!(s.confirmed_at.is_none());
}

#[tokio::test]
async fn summary_with_an_invented_number_is_asked_again_once() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value("y 8 caídas")), summary_ok(summary_value(""))]);
    let out = generate_summary(&db, Some(&p), &pid).await.unwrap();
    assert_eq!(out.ai, AiStatus::Used);
    assert_eq!(p.requests().len(), 2);
    assert!(p.requests()[1].user.contains("Estas cifras no las dijo la persona: 8"));
    assert!(out.view.unsupported_figures.is_empty(), "{:?}", out.view.unsupported_figures);
}

#[tokio::test]
async fn invented_numbers_that_persist_are_flagged_for_the_person() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value("y 8 caídas")), summary_ok(summary_value("y 8 caídas"))]);
    let out = generate_summary(&db, Some(&p), &pid).await.unwrap();
    assert_eq!(out.view.unsupported_figures, vec!["8".to_string()]);
}

#[tokio::test]
async fn without_the_ai_there_is_no_summary_and_nothing_is_made_up() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    let out = generate_summary(&db, None, &pid).await.unwrap();
    assert_eq!(out.ai, AiStatus::NotConfigured);
    assert!(out.view.summary.is_none());
    // a failure of the service is the same: the person tries again
    let p = MockProvider::new(vec![Err(AiError::Auth)]);
    assert_eq!(generate_summary(&db, Some(&p), &pid).await.unwrap().ai, AiStatus::KeyRejected);
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    assert!(generate_summary(&db, Some(&p), &pid).await.unwrap().view.summary.is_some());
}

/// The institution does not fade as the stages go by: every AI call of the first steps (the conversation, the summary
/// and the objectives) reads the whole sheet of «Mi institución», and the objectives also read what the person said
/// in their own words (not only the summary that the AI wrote from it).
#[tokio::test]
async fn every_ai_call_of_the_first_steps_reads_the_whole_institution() {
    use crate::core::ai_sheet::tests::RICH_FACTS;
    let (_d, db) = rich_profile_db();
    let pid = project_in_diagnosis(&db);
    let conversation = reach_confirmed_root(&db, &pid).await;
    let mut said = summary_value("");
    said["affected"]["count"] = json!(12); // the people the profile says it serves
    let summary = MockProvider::new(vec![summary_ok(said)]);
    generate_summary(&db, Some(&summary), &pid).await.unwrap();
    confirm_summary(&db, &pid).unwrap();
    advance(&db, &pid).unwrap();
    let objectives = MockProvider::new(vec![needs_reply()]);
    propose_needs(&db, Some(&objectives), &pid).await.unwrap();

    let calls: Vec<(String, Vec<String>)> = [
        ("the conversation", conversation.requests()),
        ("the summary", summary.requests()),
        ("the objectives", objectives.requests()),
    ]
    .into_iter()
    .flat_map(|(stage, requests)| requests.into_iter().map(move |r| (stage.to_string(), r.context)))
    .collect();
    assert!(calls.len() >= 7, "the opening, the whys, the summary and the objectives were asked: {}", calls.len());
    for (stage, context) in &calls {
        let all = context.join("\n");
        for fact in RICH_FACTS {
            assert!(all.contains(fact), "{stage} did not read «{fact}»");
        }
        // the sheet says what it is: captured data, never contact data, pay or the fee of a person
        for secret in ["7777", "3333", "AFI200101", "Rosa Representante"] {
            assert!(!all.contains(secret), "{stage} read «{secret}»");
        }
    }

    let objectives_context = objectives.requests()[0].context.join("\n");
    for words in [OPENING_ANSWER, WHY1_ANSWER, WHY2_ANSWER, WHY3_ANSWER] {
        assert!(objectives_context.contains(words), "the objectives did not read what the person said: {words}");
    }
    assert!(objectives_context.contains("Borrador del diagnóstico (lo redactó la IA"), "the summary is labelled as the AI's draft");
    assert!(!objectives_context.contains("- Sí, es esa"), "a bare «yes» says nothing and is left out");
}

#[tokio::test]
async fn a_late_or_repeated_request_never_writes_over_a_summary_the_person_confirmed_or_corrected() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    generate_summary(&db, Some(&p), &pid).await.unwrap();
    // an unconfirmed suggestion can be asked for again
    let again = MockProvider::new(vec![summary_ok(summary_value(""))]);
    assert_eq!(generate_summary(&db, Some(&again), &pid).await.unwrap().ai, AiStatus::Used);

    confirm_summary(&db, &pid).unwrap();
    let late = MockProvider::new(vec![summary_ok(summary_value("y otra cosa"))]);
    let out = generate_summary(&db, Some(&late), &pid).await.unwrap();
    assert_eq!(out.ai, AiStatus::Skipped);
    assert!(late.requests().is_empty(), "no call is made, nothing is spent");
    assert!(out.view.summary.unwrap().confirmed_at.is_some(), "it stays confirmed");
}

#[tokio::test]
async fn editing_the_summary_is_scanned_and_returns_it_to_draft() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    generate_summary(&db, Some(&p), &pid).await.unwrap();
    confirm_summary(&db, &pid).unwrap();
    let edit = |t: &str| SummaryEdit {
        problem_statement: t.into(),
        reframed_need: "Alimentación segura".into(),
        affected_description: "18 personas".into(),
        current_consequences: vec!["Agua con óxido".into()],
        root_causes: vec![],
        suggested_indicators: vec!["Instalaciones conformes".into()],
        open_questions: vec![],
    };
    let EditOutcome::Quarantine { report } = edit_summary(&db, &pid, edit("Habló la señora de CURP LOPM800101MDFRZN09"), None).unwrap() else { panic!() };
    assert!(report.has_blocking);
    let EditOutcome::Saved { view } = edit_summary(&db, &pid, edit("Problema corregido por la directora"), None).unwrap() else { panic!() };
    let s = view.summary.unwrap();
    assert_eq!(s.origin, "user");
    assert!(s.confirmed_at.is_none(), "an edited summary must be confirmed again");
    assert_eq!(s.summary["problem_statement"], "Problema corregido por la directora");
    assert_eq!(s.summary["alternatives"][0]["title"], "Obra con plan de mantenimiento", "alternatives are kept");
}

#[tokio::test]
async fn full_path_to_prioritization_with_scores_computed_by_code() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    // cannot move on before the summary is confirmed
    assert!(matches!(advance(&db, &pid), Err(ServiceError::Stage(StageError::NotReady(Missing::SummaryNotConfirmed)))));
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    generate_summary(&db, Some(&p), &pid).await.unwrap();
    confirm_summary(&db, &pid).unwrap();
    assert_eq!(advance(&db, &pid).unwrap().stage, Stage::Prioritization);

    let p = MockProvider::new(vec![needs_reply()]);
    let (view, ai) = propose_needs(&db, Some(&p), &pid).await.unwrap();
    assert_eq!(ai, AiStatus::Used);
    // the goals are proposed from the root cause the person confirmed and from what the call funds
    let ctx = p.requests()[0].context.join("\n");
    assert!(ctx.contains(ROOT) && ctx.contains("Quién convoca: Fundación Ficticia"), "{ctx}");
    assert_eq!(view.needs.len(), 2);
    assert!(view.needs.iter().all(|n| n.origin == "ai_assumption" && !n.confirmed));
    // 18 affected of 18 people -> suggestion computed by code
    assert_eq!(view.beneficiaries_suggestion, Some(5));

    let ids: Vec<String> = view.needs.iter().map(|n| n.id.clone()).collect();
    let high = Scores { beneficiaries: 5, severity: 5, mission: 5, feasibility: 3, sustainability: 4 };
    let low = Scores { beneficiaries: 2, severity: 3, mission: 3, feasibility: 4, sustainability: 3 };
    rate_need(&db, &pid, &ids[1], low).unwrap();
    let view = rate_need(&db, &pid, &ids[0], high).unwrap();
    assert_eq!(view.ranking, vec![ids[0].clone(), ids[1].clone()]);
    // (5*25 + 5*25 + 5*20 + 3*15 + 4*15) / 500 = 91 %
    assert_eq!(view.needs[0].total_score, Some(91.0));
    // bad rating is refused
    assert!(rate_need(&db, &pid, &ids[0], Scores { beneficiaries: 9, ..high }).is_err());

    assert!(matches!(advance(&db, &pid), Err(ServiceError::Stage(StageError::NotReady(Missing::NoNeedSelected)))));
    select_need(&db, &pid, &ids[0]).unwrap();
    assert_eq!(advance(&db, &pid).unwrap().stage, Stage::Drafting);
    // next stages are not ready (later phases)
    assert!(matches!(advance(&db, &pid), Err(ServiceError::Stage(StageError::NotReady(Missing::SectionsNotConfirmed)))));
}

#[tokio::test]
async fn only_three_objectives_are_kept_in_the_models_order_and_one_click_chooses_without_rating() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    generate_summary(&db, Some(&p), &pid).await.unwrap();
    confirm_summary(&db, &pid).unwrap();
    advance(&db, &pid).unwrap();

    let four = Ok(AiResponse {
        value: json!({"needs": [
            {"title": "Primero", "description": "El más importante"},
            {"title": "Segundo", "description": "Sigue"},
            {"title": "Tercero", "description": "Sigue"},
            {"title": "Cuarto", "description": "Sobra"}
        ]}),
        model: "gemini-3.5-flash".into(),
        usage: usage(),
    });
    let p = MockProvider::new(vec![four]);
    let (view, ai) = propose_needs(&db, Some(&p), &pid).await.unwrap();
    assert_eq!(ai, AiStatus::Used);
    let titles: Vec<&str> = view.needs.iter().map(|n| n.title.as_str()).collect();
    assert_eq!(titles, ["Primero", "Segundo", "Tercero"], "the model's order is the order shown, and no more than three");

    // choosing needs no rating: one click and the project can move on
    select_need(&db, &pid, &view.needs[1].id).unwrap();
    assert_eq!(advance(&db, &pid).unwrap().stage, Stage::Drafting);
}

#[tokio::test]
async fn going_back_keeps_the_work_and_marks_it_for_review() {
    let (_d, db, pid) = setup();
    reach_confirmed_root(&db, &pid).await;
    let p = MockProvider::new(vec![summary_ok(summary_value(""))]);
    generate_summary(&db, Some(&p), &pid).await.unwrap();
    confirm_summary(&db, &pid).unwrap();
    advance(&db, &pid).unwrap();
    let back = go_back(&db, &pid, Stage::Diagnosis).unwrap();
    assert_eq!(back.stage, Stage::Diagnosis);
    assert!(back.needs_review);
    let v = conversation_view(&db.lock().unwrap(), &pid).unwrap();
    assert!(v.summary.is_some() && v.root.is_some_and(|r| r.confirmed), "nothing was deleted");
    assert_eq!(v.turns.len(), 10, "the conversation is still there");
    assert!(matches!(go_back(&db, &pid, Stage::Prioritization), Err(ServiceError::Stage(StageError::NotEarlier))));
    // going back to the call is allowed too, and the confirmation of the call is still there
    assert_eq!(go_back(&db, &pid, Stage::CallSelection).unwrap().stage, Stage::CallSelection);
    assert!(store::facts(&db.lock().unwrap(), &pid).unwrap().call_confirmed);
}

#[tokio::test]
async fn user_added_needs_work_without_ai() {
    let (_d, db, pid) = setup();
    let AddNeedOutcome::Saved { view } = add_need(&db, &pid, "Cambiar la caldera", "Está vieja", None).unwrap() else { panic!() };
    assert_eq!(view.needs[0].origin, "user");
    assert!(view.needs[0].confirmed);
    let out = add_need(&db, &pid, "Hablar con la señora de CURP LOPM800101MDFRZN09", "", None).unwrap();
    assert!(matches!(out, AddNeedOutcome::Quarantine { .. }));
}

/// The saved settings and the key of the provider they choose (the same ones the app uses).
/// Live tests use a throwaway database, so the person's real data is never touched.
fn live_provider(conn: &rusqlite::Connection) -> (crate::ai::settings::AiSettings, Box<dyn AiProvider>) {
    let cfg = crate::ai::settings::load(conn).unwrap();
    let key = crate::storage::get_api_key(cfg.provider.as_str())
        .unwrap()
        .unwrap_or_else(|| panic!("no key saved for {}: save it in the app first", cfg.provider.as_str()));
    let provider = crate::ai::build_provider(&cfg, key);
    (cfg, provider)
}

/// Smallest possible first contact with the real service: ONE call to the light model, asking for the opening
/// question of the conversation. It confirms the request shape written from the documentation (ADR-007) while
/// using one call of the daily allowance. Run it before the golden case:
///   cargo test gemini_smoke_live -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn gemini_smoke_live() {
    use crate::ai::{metrics, pipeline};
    use crate::scanner::{RegexScanner, ScannerConfig};

    let dir = tempfile::tempdir().unwrap();
    let conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let (cfg, provider) = live_provider(&conn);
    assert_eq!(provider.name(), "gemini", "choose Gemini in the app first");
    let db: SharedDb = Arc::new(Mutex::new(conn));

    // 1. free check: the key works and both models exist (no generation call)
    let checks = provider.check().await.expect("the key or the connection failed");
    for c in &checks {
        println!("modelo {}: existe={} puede_responder={}", c.model, c.exists, c.can_generate);
    }
    assert!(checks.iter().all(|c| c.exists && c.can_generate), "a configured model is not available: {checks:?}");

    // 2. one real call through the whole pipeline
    let ledger = pipeline::SqliteLedger(db.clone());
    let scanner = RegexScanner::new(ScannerConfig::default());
    let answer = pipeline::run(
        provider.as_ref(), &scanner, &ledger,
        pipeline::AiCall {
            task: crate::ai::AiTask::ConversationTurn,
            context: vec![
                "Perfil de la institución:\nInstitución: Asilo Ficticio (asilo).\nPoblación: Adultos mayores — 18 personas.".into(),
                "Convocatoria que la persona confirmó:\nConvocatoria: Apoyos 2027\nQuién convoca: Fundación Ficticia\nQué apoya:\n- Proyectos de alimentación y nutrición.".into(),
                "Conversación hasta ahora:\n(todavía no empieza)".into(),
            ],
            user: "Paso: apertura. Escribe la pregunta de apertura con sus tres partes.".into(),
            project_id: None,
        },
    )
    .await
    .expect("the real call failed: read the error, it says what the service did not accept");
    println!("\nRespuesta real: {answer}");

    let report = metrics::usage_report(&db.lock().unwrap(), &cfg).unwrap();
    println!("\n{}", metrics::format_report(&report));
}

/// Golden case against the REAL service (docs: fixtures/caso-dorado-bano.md).
/// It uses the provider and key saved in the app. With Gemini's free plan it costs no money but uses allowance
/// (up to 7 light calls and 1 strong one); with Claude it costs a few pesos. Run on purpose:
///   cargo test golden_case_live -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn golden_case_live() {
    use crate::ai::{golden, metrics};
    use crate::domain::conversation::Phase;

    let dir = tempfile::tempdir().unwrap();
    let mut conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let input: ProfileInput = serde_json::from_str(include_str!("../../fixtures/institucion-asilo.json")).unwrap();
    profile::save(&mut conn, &input).unwrap();
    profile::confirm(&mut conn).unwrap();
    let (cfg, provider) = live_provider(&conn);
    println!("Proveedor: {}  modelos: {} / {}", provider.name(),
        cfg.model_for(crate::ai::ModelTier::Light), cfg.model_for(crate::ai::ModelTier::Strong));
    let db: SharedDb = Arc::new(Mutex::new(conn));
    let provider = provider.as_ref();
    let pid = project_in_diagnosis(&db);

    let mut whys = golden::WHY_ANSWERS.iter();
    let mut out = start_conversation(&db, Some(provider), &pid).await.unwrap();
    for _ in 0..20 {
        let AnswerOutcome::Saved { view, ai } = out else { panic!("quarantine on a golden answer") };
        // a status other than Used or Skipped means the real service failed: say it now instead of finishing a hollow run
        assert!(matches!(ai, AiStatus::Used | AiStatus::Skipped), "the AI part failed: {ai:?}");
        if let Some(t) = view.turns.last() {
            println!("[{:?}{}] {}", t.kind, t.level.map(|l| format!(" {l}")).unwrap_or_default(), t.text);
        }
        out = match view.phase {
            Phase::AwaitingAnswer => {
                let text = if view.turns.len() == 1 { golden::OPENING_ANSWER } else { whys.next().expect("the conversation did not reach a root cause") };
                println!("      R: {text}");
                send_message(&db, Some(provider), &pid, text, false, None).await.unwrap()
            }
            Phase::RootProposed => {
                println!("      R: (confirma la causa de fondo)");
                send_message(&db, Some(provider), &pid, "", true, None).await.unwrap()
            }
            Phase::Closed => break,
            other => panic!("unexpected phase {other:?}"),
        };
    }
    let view = conversation_view(&db.lock().unwrap(), &pid).unwrap();
    println!("\nCausa de fondo: {:?}  encaje: {:?}", view.root, view.fit);
    assert_eq!(view.phase, Phase::Closed, "the conversation did not close");

    let out = generate_summary(&db, Some(provider), &pid).await.unwrap();
    println!("\nai status: {:?}", out.ai);
    let summary = out.view.summary.clone().expect("the AI did not write the summary").summary;
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());

    let criteria = golden::criteria(&summary, &out.view.unsupported_figures);
    golden::print(&criteria);

    let report = metrics::usage_report(&db.lock().unwrap(), &cfg).unwrap();
    println!("\n{}", metrics::format_report(&report));
    let (n, cost): (i64, f64) = db
        .lock().unwrap()
        .query_row("SELECT count(*), COALESCE(SUM(estimated_cost_mxn),0) FROM ai_usage", [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    println!("Costo del diagnóstico (a precio de pago): {n} llamadas = ${cost:.4} MXN");
    assert_eq!(out.ai, AiStatus::Used, "el resumen NO lo escribió la IA; mire «Últimos intentos» arriba para ver por qué falló");
    // hard requirements: no invented figures and no personal data
    assert!(golden::hard_requirements_met(&criteria));
}

/// Sends the strong summary request to real models and prints, for each probe, what the service
/// answered (status, time, body) with no interpretation. It separates "the model is overloaded"
/// from "my request has a problem": the same request goes first to Flash-Lite (500 calls a day)
/// with and without the thinking setting, and then to the model named in PROBE_MODEL (default
/// gemini-3.5-flash; one call of its 20 a day).
///   cargo test gemini_probe_live -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn gemini_probe_live() {
    use crate::ai::gemini::GeminiProvider;
    use crate::ai::{golden, prompts, AiRequest, AiTask, ModelTier};

    let dir = tempfile::tempdir().unwrap();
    let mut conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let input: ProfileInput = serde_json::from_str(include_str!("../../fixtures/institucion-asilo.json")).unwrap();
    profile::save(&mut conn, &input).unwrap();
    profile::confirm(&mut conn).unwrap();
    let cfg = crate::ai::settings::load(&conn).unwrap();
    let key = crate::storage::get_api_key("gemini").unwrap().expect("no Gemini key saved: save it in the app first");
    let db: SharedDb = Arc::new(Mutex::new(conn));

    // a conversation written by hand from the golden case (no calls): the opening, the whys and the root cause
    let pid = project_in_diagnosis(&db);
    let (profile_ctx, call_ctx) = {
        let c = db.lock().unwrap();
        let project = store::get_project(&c, &pid).unwrap().unwrap();
        (profile_context(&c).unwrap(), call_context(&c, &project).unwrap())
    };
    let mut conversation = format!("Analista: Hola, ¿qué proyecto tienen en mente?\nPersona: {}\n", golden::OPENING_ANSWER);
    for why in &golden::WHY_ANSWERS[..3] {
        conversation.push_str(&format!("Analista: ¿Y por qué?\nPersona: {why}\n"));
    }
    let task = AiTask::DiagnosisSummary;
    let req = AiRequest {
        task,
        tier: ModelTier::Strong,
        system: prompts::system_prompt(task),
        context: vec![
            format!("Perfil de la institución:\n{profile_ctx}"),
            format!("Convocatoria que la persona confirmó:\n{call_ctx}"),
            format!("Conversación con la persona:\n{conversation}"),
            "Causa de fondo que la persona confirmó:\nNo tenemos un plan ni un fondo para adecuar la casa".into(),
        ],
        user: "Escriba el resumen del diagnóstico.".into(),
        output_schema: prompts::schema(task),
        max_output_tokens: task.max_output_tokens(),
    };
    let model = std::env::var("PROBE_MODEL").unwrap_or_else(|_| "gemini-3.5-flash".into());
    let body = GeminiProvider::new(key.clone(), cfg).build_body(&req);
    let mut without_thinking = body.clone();
    without_thinking["generationConfig"].as_object_mut().unwrap().remove("thinkingConfig");

    if std::env::var("PROBE_MODEL").is_ok() {
        // another model (e.g. Gemma): find which field of the request it rejects, one call each
        let mut without_schema = without_thinking.clone();
        let g = without_schema["generationConfig"].as_object_mut().unwrap();
        g.remove("responseJsonSchema");
        g.remove("responseMimeType");
        probe(&format!("{model}, petición fuerte completa"), &model, &body, &key).await;
        probe(&format!("{model}, sin razonamiento"), &model, &without_thinking, &key).await;
        probe(&format!("{model}, sin razonamiento ni esquema"), &model, &without_schema, &key).await;
        return;
    }
    probe("Flash-Lite, petición fuerte completa", "gemini-3.5-flash-lite", &body, &key).await;
    probe("Flash-Lite, sin el campo de razonamiento", "gemini-3.5-flash-lite", &without_thinking, &key).await;
    probe(&format!("{model}, petición fuerte completa"), &model, &body, &key).await;
}

/// One real POST, reported as it came back.
async fn probe(label: &str, model: &str, body: &serde_json::Value, key: &str) {
    println!("\n=== {label} ({model}) ===");
    println!("Campos de generación enviados: {}", {
        let mut g = body["generationConfig"].clone();
        g["responseJsonSchema"] = serde_json::json!("…");
        g
    });
    let started = std::time::Instant::now();
    let resp = reqwest::Client::new()
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"))
        .header("x-goog-api-key", key)
        .json(body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await;
    println!("Tiempo hasta la respuesta: {:?}", started.elapsed());
    let resp = match resp {
        Ok(r) => r,
        Err(e) => return println!("Sin respuesta del servicio (conexión): {e}"),
    };
    let status = resp.status().as_u16();
    println!("Estado HTTP: {status}");
    let text = resp.text().await.unwrap_or_default();
    if status != 200 {
        println!("Cuerpo: {}", text.chars().take(1500).collect::<String>());
        return;
    }
    let v: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
    match crate::ai::gemini::parse_response(&v, model) {
        Ok(r) => {
            println!("Interpretado bien. Uso: {:?}", r.usage);
            println!("{}", serde_json::to_string_pretty(&r.value).unwrap().chars().take(2500).collect::<String>());
        }
        Err(e) => println!("Llegó 200 pero el código no pudo usarla: {e}\nCuerpo: {}", text.chars().take(1500).collect::<String>()),
    }
}
