use super::*;
use crate::ai::mock::MockProvider;
use crate::ai::AiError;
use crate::test_support::*;

fn setup() -> (tempfile::TempDir, SharedDb, String) {
    let (d, db) = profile_db();
    let pid = project_in_diagnosis(&db);
    (d, db, pid)
}

fn opening() -> Result<crate::ai::AiResponse, AiError> {
    turn_reply("Hola, ¿qué proyecto tienen en mente, por qué no se ha podido y cómo cambiaría la vida de las niñas?", None, None, &[], None)
}

fn stored(db: &SharedDb, pid: &str) -> Vec<TurnRow> {
    store::turns(&db.lock().unwrap(), pid).unwrap()
}

#[tokio::test]
async fn the_opening_is_asked_once_with_the_profile_and_the_confirmed_call_as_context() {
    let (_d, db, pid) = setup();
    let p = MockProvider::new(vec![opening()]);
    let v = view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    assert_eq!((v.phase, v.turns.len(), v.turns[0].kind, v.turns[0].role), (Phase::AwaitingAnswer, 1, Kind::Opening, Role::Assistant));
    let call = v.call.clone().unwrap();
    assert_eq!((call.name.as_str(), call.funder.as_deref(), call.year), ("Apoyos 2027", Some("Fundación Ficticia"), Some(2027)));
    let req = &p.requests()[0];
    let ctx = req.context.join("\n");
    assert!(ctx.contains("Espacio: Baños, planta baja: 3 (3 mal)."), "what the profile already says is sent: {ctx}");
    assert!(ctx.contains("Quién convoca: Fundación Ficticia") && ctx.contains("Proyectos de alimentación y nutrición."), "what the call funds is sent: {ctx}");
    assert!(!ctx.contains("contact"), "no contact data");
    assert_eq!((req.task, req.user.contains("Paso: apertura")), (AiTask::ConversationTurn, true));
    // asking again changes nothing and asks nobody (nothing is billed twice)
    let again = view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    assert_eq!((again.turns.len(), p.requests().len()), (1, 1));
}

#[tokio::test]
async fn it_only_starts_in_the_diagnosis_and_only_with_the_ai() {
    let (_d, db) = profile_db();
    let pid = project_in_call_selection(&db); // the call is not confirmed yet
    let p = MockProvider::new(vec![opening()]);
    assert!(matches!(start_conversation(&db, Some(&p), &pid).await, Err(ProjectsError::WrongStage)));
    assert!(p.requests().is_empty());

    let (_d2, db, pid) = setup();
    // no key: nothing is made up in place of the AI, and nothing is saved
    let AnswerOutcome::Saved { view, ai } = start_conversation(&db, None, &pid).await.unwrap() else { panic!() };
    assert_eq!((ai, view.phase, view.turns.len()), (AiStatus::NotConfigured, Phase::NeedsOpening, 0));
    // with the AI back, the same call goes through
    let p = MockProvider::new(vec![opening()]);
    let v = view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    assert_eq!(v.phase, Phase::AwaitingAnswer);
}

#[tokio::test]
async fn the_happy_path_finds_the_root_in_the_third_why_and_only_the_person_confirms_it() {
    let (_d, db, pid) = setup();
    let p = reach_confirmed_root(&db, &pid).await;
    // opening + first why + two more + the proposal: five calls, the confirmation costs none
    assert_eq!(p.requests().len(), 5);
    assert!(p.requests().iter().all(|r| r.task == AiTask::ConversationTurn));
    let v = conversation_view(&db.lock().unwrap(), &pid).unwrap();
    assert_eq!(v.phase, Phase::Closed);
    let root = v.root.clone().unwrap();
    assert_eq!((root.text.as_str(), root.confirmed), (ROOT, true));
    assert_eq!(v.fit.as_ref().map(|f| f.fit.as_str()), Some("fits"));
    assert_eq!(v.turns.iter().map(|t| (t.role, t.kind)).collect::<Vec<_>>(), vec![
        (Role::Assistant, Kind::Opening), (Role::Person, Kind::Opening),
        (Role::Assistant, Kind::Why), (Role::Person, Kind::Why),
        (Role::Assistant, Kind::Why), (Role::Person, Kind::Why),
        (Role::Assistant, Kind::Why), (Role::Person, Kind::Why),
        (Role::Assistant, Kind::RootProposal), (Role::Person, Kind::RootReply),
    ]);
    assert_eq!(v.turns[8].options, vec![CONFIRM_ROOT_OPTION.to_string(), REJECT_ROOT_OPTION.to_string()]);
    // each call tells the AI where it is and carries what was said so far
    let users: Vec<String> = p.requests().iter().map(|r| r.user.clone()).collect();
    assert!(users[1].contains("primer porqué") && users[2].contains("porqué número 1") && users[4].contains("porqué número 3"));
    let last_ctx = p.requests()[4].context.join("\n");
    assert!(last_ctx.contains("Persona: No existe un fondo para mantenimiento") && last_ctx.contains("Analista: ¿Por qué nadie hizo un plan?"));
    // with the root confirmed the diagnosis waits only for the summary
    let facts = store::facts(&db.lock().unwrap(), &pid).unwrap();
    assert!(facts.root_cause_confirmed && !facts.diagnosis_summary_confirmed);
}

#[tokio::test]
async fn a_hypothesis_before_the_third_why_or_about_a_cause_the_person_did_not_say_is_not_proposed() {
    let (_d, db, pid) = setup();
    // too early: at the first why the AI already wants to propose a root cause
    let p = MockProvider::new(vec![
        opening(),
        turn_reply("Entiendo. ¿Por qué no se ha podido?", None, None, &[], Some(("fits", ""))),
        turn_reply("¿Y por qué no hay quien se encargue?", Some(("No hay quien dé mantenimiento", "solo se arregla cuando algo falla")), Some(ROOT), &[], None),
    ]);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    say(&db, &p, &pid, OPENING_ANSWER).await;
    let v = say(&db, &p, &pid, WHY1_ANSWER).await;
    assert_eq!((v.phase, v.root.is_none(), v.why_level), (Phase::AwaitingAnswer, true, 2), "the code asks the next why instead");

    // at the third why, but the quote is not in what the person wrote: the cause does not count
    let p = MockProvider::new(happy_replies().into_iter().take(4).chain([turn_reply(
        "¿Y por qué eso?",
        Some(("El techo se está cayendo", "el techo se está cayendo")),
        Some(ROOT),
        &[],
        None,
    )]).collect());
    let (_d2, db, pid) = setup();
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    for text in [OPENING_ANSWER, WHY1_ANSWER, WHY2_ANSWER] {
        say(&db, &p, &pid, text).await;
    }
    let v = say(&db, &p, &pid, WHY3_ANSWER).await;
    assert_eq!((v.phase, v.why_level), (Phase::AwaitingAnswer, 4));
    let last = stored(&db, &pid).into_iter().rev().find(|t| t.role == Role::Assistant).unwrap();
    assert_eq!(last.record["cause"]["verified"], json!(false));
}

#[tokio::test]
async fn at_the_fifth_why_the_code_proposes_even_if_the_ai_never_does() {
    let (_d, db, pid) = setup();
    let answers = [
        "Porque no hay mantenimiento regular en la casa de las niñas",
        "Porque nadie lleva un registro de lo que se descompone",
        "Porque la directora atiende todo sola cada día",
        "Porque el patronato solo aprueba gastos urgentes",
        "Porque nunca se ha pedido un fondo para eso",
    ];
    let quotes = ["no hay mantenimiento regular", "nadie lleva un registro", "la directora atiende todo sola", "el patronato solo aprueba gastos", "nunca se ha pedido un fondo"];
    let mut replies = vec![opening(), turn_reply("¿Por qué no se ha podido?", None, None, &[], Some(("partial", "Solo en parte encaja con la convocatoria.")))];
    for q in quotes {
        replies.push(turn_reply("¿Y por qué?", Some((q, q)), None, &[], None));
    }
    let p = MockProvider::new(replies);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    say(&db, &p, &pid, OPENING_ANSWER).await;
    let mut view = None;
    for a in answers {
        view = Some(say(&db, &p, &pid, a).await);
    }
    let v = view.unwrap();
    assert_eq!(v.phase, Phase::RootProposed);
    assert_eq!(p.requests().len(), 7, "the opening, the first why and one call per answer: no sixth why exists");
    assert_eq!(v.root.unwrap().text, "nunca se ha pedido un fondo", "the best verified cause is what is proposed");
    let proposal = stored(&db, &pid).into_iter().rev().find(|t| t.kind == Kind::RootProposal).unwrap();
    assert_eq!(proposal.record["forced"], json!(true));
    assert_eq!(v.fit.unwrap().fit, "partial");
}

#[tokio::test]
async fn turning_the_proposal_down_proposes_again_and_the_second_no_takes_the_persons_words() {
    let (_d, db, pid) = setup();
    let mut replies = happy_replies();
    replies.push(turn_reply("Entiendo.", None, Some("El patronato no autoriza gastos de mantenimiento"), &[], None));
    let p = MockProvider::new(replies);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    for text in [OPENING_ANSWER, WHY1_ANSWER, WHY2_ANSWER, WHY3_ANSWER] {
        say(&db, &p, &pid, text).await;
    }
    // first «no»: the AI proposes again with the correction
    let v = say(&db, &p, &pid, "No exactamente, el problema es que el patronato no autoriza gastos de mantenimiento").await;
    assert_eq!((v.phase, v.root.as_ref().map(|r| r.text.as_str())), (Phase::RootProposed, Some("El patronato no autoriza gastos de mantenimiento")));
    assert!(!v.root.unwrap().confirmed);
    assert_eq!(p.requests().len(), 6);
    assert!(p.requests()[5].user.contains("proponer de nuevo"));
    // second «no»: no more rounds; what the person says is the root cause, without calling the AI
    let v = say(&db, &p, &pid, "Tampoco, es que el patronato casi nunca se reúne y no decide nada").await;
    assert_eq!(p.requests().len(), 6, "no AI call");
    let root = v.root.unwrap();
    assert_eq!((v.phase, root.confirmed), (Phase::Closed, true));
    assert_eq!(root.text, "Tampoco, es que el patronato casi nunca se reúne y no decide nada");
    assert!(v.turns.last().unwrap().text.starts_with("Quedó como usted la dijo"));
}

#[tokio::test]
async fn two_vague_answers_in_a_row_bring_closed_options() {
    let (_d, db, pid) = setup();
    let p = MockProvider::new(vec![
        opening(),
        turn_reply("¿Por qué no se ha podido?", None, None, &[], Some(("fits", ""))),
        turn_reply("¿Y por qué?", None, None, &[], None),
        turn_reply("Elija lo que más se parezca:", None, None, &["Falta dinero", "Falta personal", "No lo sé todavía"], None),
    ]);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    say(&db, &p, &pid, OPENING_ANSWER).await;
    say(&db, &p, &pid, "no sé").await;
    assert!(p.requests()[2].user.contains("pregunta abierta"), "one vague answer is not enough");
    let v = say(&db, &p, &pid, "ni idea la verdad").await;
    assert!(p.requests()[3].user.contains("Táctica: options"));
    assert_eq!(v.turns.last().unwrap().options, vec!["Falta dinero", "Falta personal", "No lo sé todavía"]);
}

#[tokio::test]
async fn a_figure_nobody_said_is_asked_again_once_and_flagged_if_it_stays() {
    let (_d, db, pid) = setup();
    let p = MockProvider::new(vec![
        turn_reply("Hola, ¿cómo cambiaría la vida de las 37 niñas?", None, None, &[], None),
        opening(),
    ]);
    let v = view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    assert_eq!(p.requests().len(), 2);
    assert!(p.requests()[1].user.contains("Estas cifras no las dijo la persona: 37"));
    assert!(!v.turns[0].text.contains("37"));

    let (_d2, db, pid) = setup();
    let p = MockProvider::new(vec![
        turn_reply("Hola, ¿cómo cambiaría la vida de las 37 niñas?", None, None, &[], None),
        turn_reply("Hola, ¿y las 37 niñas?", None, None, &[], None),
    ]);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    assert_eq!(p.requests().len(), 2, "only one more try");
    assert_eq!(stored(&db, &pid)[0].record["unsupported_figures"], json!(["37"]));
    // a number the person or the profile gave is fine
    let (_d3, db, pid) = setup();
    let p = MockProvider::new(vec![turn_reply("Hola, el asilo tiene 25 lugares y 18 personas: ¿qué proyecto tienen en mente?", None, None, &[], None)]);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    assert_eq!(p.requests().len(), 1);
}

#[tokio::test]
async fn if_the_ai_fails_what_the_person_wrote_stays_and_trying_again_does_not_duplicate_anything() {
    let (_d, db, pid) = setup();
    let p = MockProvider::new(vec![
        opening(),
        Err(AiError::Offline),
        Err(AiError::Offline),
        turn_reply("¿Por qué no se ha podido?", None, None, &[], Some(("fits", ""))),
    ]);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    let AnswerOutcome::Saved { view, ai } = send_message(&db, Some(&p), &pid, OPENING_ANSWER, false, None).await.unwrap() else { panic!() };
    assert_eq!(ai, AiStatus::Offline);
    assert_eq!((view.phase, view.turns.len()), (Phase::AwaitingAi, 2), "the answer is saved and the AI owes the reply");
    // the person cannot write on top of a message the AI still owes
    assert!(matches!(send_message(&db, Some(&p), &pid, "otra cosa", false, None).await, Err(ProjectsError::WrongStage)));
    let v = view_of(retry(&db, Some(&p), &pid).await.unwrap());
    assert_eq!((v.phase, v.turns.len()), (Phase::AwaitingAnswer, 3));
    // when nothing is owed, trying again asks nobody
    let calls = p.requests().len();
    let v = view_of(retry(&db, Some(&p), &pid).await.unwrap());
    assert_eq!((v.turns.len(), p.requests().len()), (3, calls));
}

#[tokio::test]
async fn personal_data_goes_to_quarantine_and_only_the_covered_text_is_saved() {
    let (_d, db, pid) = setup();
    let p = MockProvider::new(vec![opening(), turn_reply("¿Por qué no se ha podido?", None, None, &[], Some(("fits", "")))]);
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    let risky = "La señora con CURP LOPM800101MDFRZN09 se cayó en la cocina y por eso queremos arreglarla";
    let AnswerOutcome::Quarantine { report } = send_message(&db, Some(&p), &pid, risky, false, None).await.unwrap() else { panic!() };
    assert!(report.has_blocking);
    assert_eq!((stored(&db, &pid).len(), p.requests().len()), (1, 1), "nothing is saved and nothing is sent");
    let v = view_of(send_message(&db, Some(&p), &pid, risky, false, Some(Decision::Redact)).await.unwrap());
    let said = &v.turns[1].text;
    assert!(said.contains("[CURP OCULTA]") && !said.contains("LOPM8"));
    assert!(!p.requests()[1].context.join("\n").contains("LOPM8"), "the AI only sees the covered text");
}

#[tokio::test]
async fn messages_out_of_turn_or_empty_are_refused() {
    let (_d, db, pid) = setup();
    let p = MockProvider::new(vec![opening()]);
    // nothing to answer yet
    assert!(matches!(send_message(&db, Some(&p), &pid, "hola", false, None).await, Err(ProjectsError::WrongStage)));
    view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
    assert!(matches!(send_message(&db, Some(&p), &pid, "   ", false, None).await, Err(ProjectsError::EmptyText)));
    assert!(matches!(send_message(&db, Some(&p), &pid, &"a ".repeat(4000), false, None).await, Err(ProjectsError::TextTooLarge)));
    // there is no proposal to confirm: the quick reply cannot close anything
    assert!(matches!(send_message(&db, Some(&p), &pid, "", true, None).await, Err(ProjectsError::WrongStage)));
    assert!(store::get_root(&db.lock().unwrap(), &pid).unwrap().is_none());
}

/// A battery of simulated people, the worst cases for a conversation that has to end: whatever they write, it
/// closes within the bound, the root cause ends up confirmed and no figure is invented. (The model here never
/// proposes by itself, which is the worst case for the bound; a real person is the only real judge.)
#[tokio::test]
async fn whoever_the_person_is_the_conversation_ends_within_the_bound() {
    type Writes = fn(usize, &ConversationView) -> (String, bool);
    let personas: Vec<(&str, Writes)> = vec![
        ("parca", |_, v| ("no sé".into(), v.phase == Phase::RootProposed)),
        ("divagante", |_, v| {
            ("Pues mire, la casa es muy grande y tiene muchos cuartos y cada día pasan cosas distintas con las niñas y con la comida".into(), v.phase == Phase::RootProposed)
        }),
        ("evasiva", |i, v| (if i % 2 == 0 { "puede ser" } else { "ni idea la verdad" }.into(), v.phase == Phase::RootProposed)),
        ("que rechaza la causa", |i, _| (format!("No es eso, lo que pasa es otra cosa distinta número {i} que nadie ha visto"), false)),
    ];
    for (name, writes) in personas {
        let (_d, db, pid) = setup();
        let replies = (0..14).map(|_| turn_reply("¿Y por qué?", None, None, &["Falta dinero", "No lo sé todavía"], None)).collect();
        let p = MockProvider::new(replies);
        let mut view = view_of(start_conversation(&db, Some(&p), &pid).await.unwrap());
        let mut written = 0;
        while view.phase != Phase::Closed {
            assert!(written < 14, "{name}: it did not end");
            let (text, confirm) = writes(written, &view);
            view = view_of(send_message(&db, Some(&p), &pid, &text, confirm, None).await.unwrap());
            written += 1;
        }
        let root = view.root.clone().unwrap();
        assert!(root.confirmed && !root.text.trim().is_empty(), "{name}");
        // opening + first why + up to five whys + one new proposal after a «no»
        assert!(p.requests().len() <= 8, "{name}: {} calls", p.requests().len());
        assert!(stored(&db, &pid).iter().all(|t| t.record["unsupported_figures"].as_array().map_or(true, Vec::is_empty)), "{name}");
    }
}
