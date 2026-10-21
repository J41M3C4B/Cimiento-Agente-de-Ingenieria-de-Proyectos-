use super::*;
use crate::modules::projects::storage::calls as call_store;
use crate::storage::open_encrypted;
use crate::test_support::*;
use std::sync::{Arc, Mutex};

fn setup() -> (tempfile::TempDir, SharedDb, String) {
    let (d, db) = profile_db();
    let pid = project_in_diagnosis(&db);
    (d, db, pid)
}

#[test]
fn an_internal_project_needs_a_confirmed_profile() {
    let dir = tempfile::tempdir().unwrap();
    let db: SharedDb = Arc::new(Mutex::new(open_encrypted(&dir.path().join("t.db"), KEY).unwrap()));
    assert!(matches!(
        create_project(&db, "x", None),
        Err(ProjectsError::Stage(StageError::NotReady(Missing::ProfileNotConfirmed)))
    ));
    assert!(store::list_projects(&db.lock().unwrap()).unwrap().is_empty());
}

#[test]
fn a_project_born_from_a_call_waits_until_the_person_confirms_it_and_then_the_diagnosis_has_to_be_confirmed() {
    let (_d, db) = profile_db();
    let pid = project_in_call_selection(&db);
    assert_eq!(store::get_project(&db.lock().unwrap(), &pid).unwrap().unwrap().stage, Stage::CallSelection);
    assert!(matches!(advance(&db, &pid), Err(ProjectsError::Stage(StageError::NotReady(Missing::CallNotReady)))));
    let reading = store::get_project(&db.lock().unwrap(), &pid).unwrap().unwrap().call_reading_id.unwrap();
    assert!(call_store::confirm(&db.lock().unwrap(), &reading).unwrap());
    assert_eq!(advance(&db, &pid).unwrap().stage, Stage::Diagnosis);
    // the diagnosis ends when the root cause is confirmed and then the summary
    assert!(matches!(advance(&db, &pid), Err(ProjectsError::Stage(StageError::NotReady(Missing::DiagnosisIncomplete)))));
    {
        let conn = db.lock().unwrap();
        store::set_root(&conn, &pid, ROOT, None).unwrap();
        assert!(store::confirm_root(&conn, &pid).unwrap());
    }
    assert!(matches!(advance(&db, &pid), Err(ProjectsError::Stage(StageError::NotReady(Missing::SummaryNotConfirmed)))));
}

#[test]
fn reading_the_call_again_takes_away_the_confirmation_and_the_project_cannot_move_on() {
    let (_d, db) = profile_db();
    let pid = project_in_call_selection(&db);
    let reading = store::get_project(&db.lock().unwrap(), &pid).unwrap().unwrap().call_reading_id.unwrap();
    assert!(call_store::confirm(&db.lock().unwrap(), &reading).unwrap());
    assert!(store::facts(&db.lock().unwrap(), &pid).unwrap().call_confirmed);
    // the reading starts again: the confirmation is gone
    call_store::set_status(&db.lock().unwrap(), &reading, call_store::ReadingStatus::Waiting, None).unwrap();
    assert!(!store::facts(&db.lock().unwrap(), &pid).unwrap().call_confirmed);
    assert!(matches!(advance(&db, &pid), Err(ProjectsError::Stage(StageError::NotReady(Missing::CallNotReady)))));
}

#[test]
fn editing_the_summary_is_scanned_and_returns_it_to_draft() {
    let (_d, db, pid) = setup();
    confirm_the_diagnosis(&db, &pid);
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

#[test]
fn what_an_earlier_conversation_left_is_still_read_and_nothing_is_asked_of_a_model() {
    let (_d, db, pid) = setup();
    confirm_the_diagnosis(&db, &pid);
    let v = conversation_view(&db.lock().unwrap(), &pid).unwrap();
    assert!(v.root.is_some_and(|r| r.confirmed) && v.summary.is_some_and(|s| s.confirmed_at.is_some()));
    assert!(v.unsupported_figures.is_empty(), "the check of the figures comes back with the platform");
}

#[test]
fn full_path_to_the_drafting_with_scores_computed_by_code() {
    let (_d, db, pid) = setup();
    // cannot move on before the diagnosis is confirmed
    assert!(matches!(advance(&db, &pid), Err(ProjectsError::Stage(StageError::NotReady(Missing::DiagnosisIncomplete)))));
    confirm_the_diagnosis(&db, &pid);
    assert_eq!(advance(&db, &pid).unwrap().stage, Stage::Prioritization);

    let AddNeedOutcome::Saved { .. } = add_need(&db, &pid, "Mantenimiento preventivo de la casa", "Un sistema con responsable y fondo", None).unwrap() else { panic!() };
    let AddNeedOutcome::Saved { view } = add_need(&db, &pid, "Capacitación del personal", "Quién revisa y cuándo", None).unwrap() else { panic!() };
    assert_eq!(view.needs.len(), 2);
    // 18 affected of 18 people -> suggestion computed by code, from the summary an earlier diagnosis left
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

    assert!(matches!(advance(&db, &pid), Err(ProjectsError::Stage(StageError::NotReady(Missing::NoNeedSelected)))));
    select_need(&db, &pid, &ids[0]).unwrap();
    assert_eq!(advance(&db, &pid).unwrap().stage, Stage::Drafting);
    // next stages are not ready (later phases)
    assert!(matches!(advance(&db, &pid), Err(ProjectsError::Stage(StageError::NotReady(Missing::SectionsNotConfirmed)))));
}

#[test]
fn going_back_keeps_the_work_and_marks_it_for_review() {
    let (_d, db, pid) = setup();
    confirm_the_diagnosis(&db, &pid);
    advance(&db, &pid).unwrap();
    let back = go_back(&db, &pid, Stage::Diagnosis).unwrap();
    assert_eq!(back.stage, Stage::Diagnosis);
    assert!(back.needs_review);
    let v = conversation_view(&db.lock().unwrap(), &pid).unwrap();
    assert!(v.summary.is_some() && v.root.is_some_and(|r| r.confirmed), "nothing was deleted");
    assert!(matches!(go_back(&db, &pid, Stage::Prioritization), Err(ProjectsError::Stage(StageError::NotEarlier))));
    // going back to the call is allowed too, and the confirmation of the call is still there
    assert_eq!(go_back(&db, &pid, Stage::CallSelection).unwrap().stage, Stage::CallSelection);
    assert!(store::facts(&db.lock().unwrap(), &pid).unwrap().call_confirmed);
}

#[test]
fn user_added_needs_work_without_ai() {
    let (_d, db, pid) = setup();
    let AddNeedOutcome::Saved { view } = add_need(&db, &pid, "Cambiar la caldera", "Está vieja", None).unwrap() else { panic!() };
    assert_eq!(view.needs[0].origin, "user");
    assert!(view.needs[0].confirmed);
    let out = add_need(&db, &pid, "Hablar con la señora de CURP LOPM800101MDFRZN09", "", None).unwrap();
    assert!(matches!(out, AddNeedOutcome::Quarantine { .. }));
}
