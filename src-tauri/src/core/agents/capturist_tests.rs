use super::*;
use crate::ai::mock::MockProvider;
use crate::ai::{AiError, AiResponse, Usage};
use crate::core::profile::forms::{get, save, save_using};
use crate::core::profile::service::SaveProfileOutcome;
use std::sync::{Arc, Mutex};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// An institution with its name and its contact written, nothing else.
fn db() -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let identity: Values = serde_json::from_value(json!({ "institution.name": "Asilo Ficticio" })).unwrap();
    assert!(matches!(save(&mut c, "institution.identity", identity, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    let legal: Values = serde_json::from_value(json!({ "institution.legal_rfc": "AFI200101AB1" })).unwrap();
    assert!(matches!(save(&mut c, "institution.legal", legal, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    (dir, Arc::new(Mutex::new(c)))
}

fn reply(answer: Value) -> Result<AiResponse, AiError> {
    Ok(AiResponse {
        value: json!({ "tool": null, "args": null, "answer": answer }),
        model: "mock".into(),
        usage: Usage { input_tokens: 1_200, output_tokens: 150, ..Default::default() },
    })
}

const TOLD: &str = "Somos un asilo: atendemos a 38 señoras de 60 años en adelante, viven aquí y les damos comida y atención médica.";

fn told(form: &str, said: &str) -> Capture {
    Capture { form: form.into(), said: said.into(), history: vec![] }
}

/// What the AI suggests for «Su institución» from `TOLD`.
fn suggestions() -> Value {
    json!({
        "proposals": [
            { "field": "institution.populations", "value": ["older_adults"], "quote": "atendemos a 38 señoras" },
            { "field": "institution.sex_served", "value": "women", "quote": "38 señoras" },
            { "field": "institution.age_min", "value": "60", "quote": "de 60 años en adelante" },
            { "field": "institution.modalities", "value": "residential", "quote": "viven aquí" },
            { "field": "institution.care_areas", "value": ["food", "health"], "quote": "les damos comida y atención médica" }
        ]
    })
}

fn proposed(out: CaptureOutcome) -> (Vec<String>, usize, Vec<String>) {
    match out {
        CaptureOutcome::Proposed { fields, dropped, still, .. } => (fields, dropped, still),
        other => panic!("nothing proposed: {other:?}"),
    }
}

/// The rule of IA4 (ADR-034): a suggestion waits in its window and nothing of the institution changes until a
/// person saves it.
#[tokio::test]
async fn a_suggestion_is_never_saved_without_a_click() {
    let (_d, db) = db();
    let before = get(&db.lock().unwrap(), "institution.identity").unwrap().values;
    let p = MockProvider::new(vec![reply(suggestions())]);
    let (fields, dropped, still) = proposed(capture(&db, Some(&p), told("institution.identity", TOLD), None).await.unwrap());
    assert_eq!(fields, ["institution.populations", "institution.sex_served", "institution.age_min", "institution.modalities", "institution.care_areas"]);
    assert_eq!(dropped, 0);
    // what the person may still tell, by its title: never what is written (the name) nor what waits
    assert_eq!(still, ["¿A qué se dedica?", "¿Qué servicios o programas ofrecen?", "Edad máxima que reciben", "¿Qué piden para que alguien ingrese?"]);

    let c = db.lock().unwrap();
    let view = get(&c, "institution.identity").unwrap();
    assert_eq!(view.values, before, "a suggestion is not a save");
    let waiting: Vec<(&str, &Value)> = view.proposals.iter().map(|p| (p.field_id.as_str(), &p.value)).collect();
    assert!(waiting.contains(&("institution.age_min", &json!(60))), "a number told as text is a number: {waiting:?}");
    assert!(waiting.contains(&("institution.modalities", &json!(["residential"]))), "one option alone is a list: {waiting:?}");
    assert_eq!(view.proposals[0].said.as_deref(), Some("atendemos a 38 señoras"), "it says where it comes from");
    assert_eq!(c.query_row("SELECT count(*) FROM core_change WHERE origin = 'ai_assumption'", [], |r| r.get::<_, i64>(0)).unwrap(), 0);

    // what the window carries for the AI: its fields, never what is protected
    let r = &p.requests()[0];
    assert!(r.system.contains("Usted es el Capturista"));
    assert!(r.context.iter().any(|x| x.contains("`institution.populations`") && x.contains("older_adults")), "{:?}", r.context);
    assert!(r.context.iter().any(|x| x.contains("Guardado: «Asilo Ficticio»")));
    assert_eq!(r.user.matches(TOLD).count(), 1);
}

/// The code keeps only what the person said, in a field of a window it fills that the AI may see, with a value that
/// fits it.
#[tokio::test]
async fn the_code_drops_what_the_person_did_not_say_what_is_protected_and_what_does_not_fit() {
    let (_d, db) = db();
    let answer = json!({
        "proposals": [
            // invented: the person never said it
            { "field": "institution.mission", "value": "Dar amor a las personas mayores.", "quote": "damos amor y cariño" },
            // an own datum that is not in the person's words (the saved one would be «already so», ADR-042 §6)
            { "field": "institution.legal_rfc", "value": "XYZ200101AB1", "quote": "atendemos a 38 señoras" },
            // a field of a window it never fills (the board), though the person said it
            { "field": "institution.board.role", "value": "president", "quote": "atendemos a 38 señoras" },
            // a code out of its list
            { "field": "institution.populations", "value": ["ancianas"], "quote": "38 señoras" },
            // what is saved already: nothing to suggest, nothing dropped
            { "field": "institution.name", "value": "Asilo Ficticio", "quote": "Somos un asilo" },
            // said with other capitals and accents: still the person's words
            { "field": "institution.sex_served", "value": "women", "quote": "38 SENORAS" },
            // a figure the person never said: the AI counted
            { "field": "institution.age_min", "value": 65, "quote": "de 60 años en adelante" }
        ]
    });
    let p = MockProvider::new(vec![reply(answer)]);
    let (fields, dropped, _) = proposed(capture(&db, Some(&p), told("institution.identity", TOLD), None).await.unwrap());
    assert_eq!((fields, dropped), (vec!["institution.sex_served".to_string()], 5));

    // a window the Capturist does not fill, or one it does not know
    assert!(matches!(capture(&db, Some(&p), told("institution.board_member", TOLD), None).await, Err(ServiceError::NotFound)));
    assert!(matches!(capture(&db, Some(&p), told("institution.contact", "  "), None).await, Err(ServiceError::EmptyText)));
}

/// What the person takes goes the usual way: the checks of the window, the scanner and the history, with the origin
/// of each datum; a suggestion is accepted in the same transaction, never without its datum.
#[tokio::test]
async fn what_is_taken_goes_through_the_usual_save_with_its_origin() {
    let (_d, db) = db();
    let p = MockProvider::new(vec![reply(suggestions())]);
    proposed(capture(&db, Some(&p), told("institution.identity", TOLD), None).await.unwrap());
    let mut c = db.lock().unwrap();
    let view = get(&c, "institution.identity").unwrap();
    let id_of = |field: &str| view.proposals.iter().find(|p| p.field_id == field).unwrap().id.clone();

    // a value out of its list stops the save: nothing is accepted
    let mut values = view.values.clone();
    values.insert("institution.populations".into(), json!(["older_adults"]));
    values.insert("institution.modalities".into(), json!(["nope"]));
    let used = vec![id_of("institution.populations"), id_of("institution.modalities")];
    assert!(matches!(save_using(&mut c, "institution.identity", values.clone(), None, &used).unwrap(), SaveProfileOutcome::Invalid { .. }));
    assert_eq!(get(&c, "institution.identity").unwrap().proposals.len(), 5, "still waiting");

    // taken as it came (populations) and corrected by the person (modalities: day care, not residential)
    values.insert("institution.modalities".into(), json!(["day_care"]));
    assert!(matches!(save_using(&mut c, "institution.identity", values, None, &used).unwrap(), SaveProfileOutcome::Saved { .. }));
    let view = get(&c, "institution.identity").unwrap();
    assert_eq!(view.origins["institution.populations"].origin, "ai_assumption");
    assert!(view.origins["institution.populations"].source_ref.as_deref().unwrap().contains("atendemos a 38 señoras"));
    assert_eq!(view.origins["institution.modalities"].origin, "user", "corrected: it is the person's");
    assert!(view.origins["institution.populations"].confirmed_at.is_some(), "a person confirmed it");
    let status = |id: &str| c.query_row("SELECT status FROM ai_proposal WHERE id = ?1", [id], |r| r.get::<_, String>(0)).unwrap();
    assert_eq!((status(&used[0]), status(&used[1])), ("accepted".into(), "accepted".into()));
    // the rest wait for the person; a used id of another window is not taken
    assert_eq!(view.proposals.iter().map(|p| p.field_id.as_str()).collect::<Vec<_>>(), ["institution.sex_served", "institution.age_min", "institution.care_areas"]);
    let changes: i64 = c.query_row("SELECT count(*) FROM core_change WHERE field = 'institution.populations' AND origin = 'ai_assumption'", [], |r| r.get(0)).unwrap();
    assert_eq!(changes, 1);

    // discarding changes nothing of the institution
    let rest: Vec<String> = view.proposals.iter().map(|p| p.id.clone()).collect();
    assert_eq!(proposals::reject(&c, &rest, Some("usr_1")).unwrap(), 3);
    let after = get(&c, "institution.identity").unwrap();
    assert!(after.proposals.is_empty());
    assert_eq!(after.values, view.values);
}

/// A suggested text with data of a person is covered before it is kept, and scanned again when saved.
#[tokio::test]
async fn a_suggested_text_is_scanned() {
    let (_d, db) = db();
    let said = "A qué nos dedicamos: cuidamos personas mayores; informes con maria.lopez@example.com";
    let answer = json!({
        "proposals": [{ "field": "institution.mission", "value": "Cuidamos personas mayores; informes con maria.lopez@example.com", "quote": "cuidamos personas mayores" }]
    });
    let p = MockProvider::new(vec![reply(answer)]);
    // the person's words are held first: covered, they go on without the email
    assert!(matches!(capture(&db, Some(&p), told("institution.identity", said), None).await.unwrap(), CaptureOutcome::Quarantine { .. }));
    assert!(p.requests().is_empty(), "nothing went to the AI");
    proposed(capture(&db, Some(&p), told("institution.identity", said), Some(Decision::Redact)).await.unwrap());
    assert!(!p.requests()[0].user.contains("maria.lopez"));
    let view = get(&db.lock().unwrap(), "institution.identity").unwrap();
    let mission = view.proposals[0].value.as_str().unwrap();
    assert!(!mission.contains("maria.lopez") && mission.starts_with("Cuidamos personas mayores"), "{mission}");
}

/// Without the AI nothing is suggested and the reason is said as in the Assistant.
#[tokio::test]
async fn without_the_ai_nothing_is_suggested_and_the_reason_is_said() {
    let (_d, db) = db();
    let out = capture(&db, None, told("institution.capacity", "Tenemos 40 camas."), None).await.unwrap();
    assert!(matches!(out, CaptureOutcome::Unavailable { reason: "not_configured", .. }), "{out:?}");
    let p = MockProvider::new(vec![Err(AiError::Offline), Err(AiError::Offline)]);
    let out = capture(&db, Some(&p), told("institution.capacity", "Tenemos 40 camas."), None).await.unwrap();
    assert!(matches!(out, CaptureOutcome::Unavailable { reason: "offline", .. }), "{out:?}");
    assert!(proposals::pending_counts(&db.lock().unwrap()).unwrap().is_empty());
}

/// What is saved in the four windows.
fn saved_all(c: &rusqlite::Connection) -> Saved {
    FILLABLE.iter().map(|(f, _, _)| (*f, crate::core::profile::forms::saved_values(c, f).unwrap())).collect()
}

/// The window where the person is first, then the others, each with its fields and what is saved, and the last turns;
/// never a protected value, even if it is saved: of the institution's own data the AI only learns whether they are
/// filled (ADR-041).
#[test]
fn the_context_names_the_fields_of_every_window_and_never_a_protected_value() {
    let (_d, db) = db();
    let c = db.lock().unwrap();
    let saved = saved_all(&c);
    assert!(saved.iter().any(|(_, v)| v.contains_key("institution.legal_rfc")));
    let history = vec![Turn { from: "person".into(), text: "Somos I.A.P. desde 1950".into(), documents: vec![] }];
    let ctx = context("institution.legal", &saved, &history).join("\n");
    assert!(ctx.starts_with("La persona está en la ventana «Datos legales y fiscales»"), "{ctx}");
    assert!(ctx.contains("`institution.legal_form`") && ctx.contains("iap") && ctx.contains("Solo aplica si `institution.authorized_donee` es yes"), "{ctx}");
    assert!(ctx.contains("Persona: Somos I.A.P. desde 1950"), "{ctx}");
    assert!(ctx.contains("`institution.legal_rfc`") && ctx.contains("Guardado: lleno") && !ctx.contains("AFI200101AB1"), "{ctx}");
    // the other windows come after, each with its fields
    let legal_at = ctx.find("Ventana «Datos legales y fiscales»").unwrap();
    for name in ["Su institución", "Contacto y ubicación", "Capacidad y cifras rápidas"] {
        assert!(ctx.find(&format!("Ventana «{name}»")).is_some_and(|at| at > legal_at), "{name}: {ctx}");
    }
    assert!(ctx.contains("cdmx (Ciudad de México)"), "{ctx}");
}

/// ADR-042 §6: what the person tells goes to the window of its field, whatever window the person is in; what is saved
/// already the same (read as a person reads it) is not suggested and the outcome says it was there.
#[tokio::test]
async fn each_datum_goes_to_its_window_and_the_same_is_not_suggested() {
    let (_d, db) = db();
    let said = "Nos representa la directora, Rosa Ficticia del Ejemplo, y tenemos 50 camas.";
    let answer = json!({ "proposals": [
        { "field": "institution.legal_rep_name", "value": "Rosa Ficticia del Ejemplo", "quote": "la directora, Rosa Ficticia del Ejemplo" },
        { "field": "institution.capacity_total", "value": 50, "quote": "tenemos 50 camas" }
    ] });
    let saved_capacity = {
        let c = db.lock().unwrap();
        crate::core::profile::forms::saved_values(&c, "institution.capacity").unwrap().get("institution.capacity_total").cloned()
    };
    let p = MockProvider::new(vec![reply(answer)]);
    let out = capture(&db, Some(&p), told("institution.contact", said), Some(Decision::NotPersonal)).await.unwrap();
    let CaptureOutcome::Proposed { fields, left, others, same, .. } = out else { panic!("{out:?}") };
    let c = db.lock().unwrap();
    let rep = proposals::pending(&c, "institution.legal", None).unwrap();
    assert!(rep.iter().any(|p| p.field_id == "institution.legal_rep_name"), "in «Datos legales», not in «Contacto»");
    assert!(left.is_empty(), "nothing for the window the person is in: {left:?}");
    assert!(others.iter().any(|o| o.place == "profile.legal" && o.left == vec![title("institution.legal_rep_name")]), "{others:?}");
    if saved_capacity == Some(json!(50)) {
        assert_eq!(same, vec![title("institution.capacity_total")]);
        assert!(!fields.contains(&"institution.capacity_total".to_string()));
    } else {
        assert!(fields.contains(&"institution.capacity_total".to_string()), "{fields:?}");
    }
}

#[test]
fn the_same_is_read_as_a_person_reads_it() {
    let f = |id: &str| crate::core::institution::forms::field(id).unwrap();
    assert!(same(f("institution.mission"), Some(&json!("Cuidar a personas mayores")), &json!("cuidar a  personas  mayores.")));
    assert!(same(f("institution.contact_phone"), Some(&json!("33 1234 5678")), &json!("3312345678")), "an own datum by its digits");
    assert!(same(f("institution.modalities"), Some(&json!(["residential", "day_care"])), &json!(["day_care", "residential"])));
    assert!(!same(f("institution.capacity_total"), Some(&json!(40)), &json!(41)));
    assert!(!same(f("institution.mission"), None, &json!("algo")), "nothing saved is never the same");
}

/// The windows it fills are windows of «Mi institución» with fields the AI may see; the board and the accounts are
/// never among them.
#[test]
fn it_fills_only_the_windows_of_the_catalog() {
    for (form, place, _) in FILLABLE {
        let spec = crate::core::institution::forms::form(form).unwrap();
        assert!(open_fields(spec).count() > 0, "{form}");
        assert_eq!(fill_of_place(place).unwrap().form, *form);
    }
    assert!(fill_of_form("institution.board_member").is_none() && fill_of_form("institution.bank_account").is_none());
}

/// ADR-041: the institution's own data the person tells are suggested copied from their words, letter by letter and
/// digit by digit; a date is one they wrote. A changed digit, a completed value or a date not said is dropped, and no
/// other agent of the AI may propose them.
#[test]
fn the_institutions_own_data_are_copied_never_made() {
    let (_d, db) = db();
    let c = db.lock().unwrap();
    let keep_one = |_form: &str, told: &str, field: &str, value: Value, quote: &str| {
        let answer = json!({ "proposals": [{ "field": field, "value": value, "quote": quote }] });
        keep(&c, &Saved::new(), told, &answer).unwrap().fields.into_iter().map(|(_, f)| f).collect::<Vec<_>>()
    };
    let contact = "Estamos en Calle Volcanes #412, C.P. 55060. Nuestro teléfono es 33 1234 5678.";
    assert_eq!(keep_one("institution.contact", contact, "institution.contact_phone", json!("3312345678"), "Nuestro teléfono es 33 1234 5678"), vec!["institution.contact_phone"]);
    assert_eq!(keep_one("institution.contact", contact, "institution.postal_code", json!("55060"), "C.P. 55060"), vec!["institution.postal_code"]);
    assert_eq!(keep_one("institution.contact", contact, "institution.street", json!("Calle Volcanes"), "Calle Volcanes #412"), vec!["institution.street"]);
    // a digit changed, a value completed, a quote without it
    assert!(keep_one("institution.contact", contact, "institution.contact_phone", json!("3312345679"), "Nuestro teléfono es 33 1234 5678").is_empty());
    assert!(keep_one("institution.contact", contact, "institution.postal_code", json!("55060"), "Estamos en Calle Volcanes").is_empty());
    assert!(keep_one("institution.contact", contact, "institution.street", json!("Calle Volcanes Norte"), "Calle Volcanes #412").is_empty());
    let legal = "Nuestro RFC es CSE870512AB3 y el poder vale hasta el 18 de enero de 2027.";
    assert_eq!(keep_one("institution.legal", legal, "institution.legal_rfc", json!("CSE870512AB3"), "Nuestro RFC es CSE870512AB3"), vec!["institution.legal_rfc"]);
    assert!(keep_one("institution.legal", legal, "institution.legal_rfc", json!("CSE870512AB8"), "Nuestro RFC es CSE870512AB3").is_empty());
    assert_eq!(keep_one("institution.legal", legal, "institution.legal_rep_valid_until", json!("2027-01-18"), "vale hasta el 18 de enero de 2027"), vec!["institution.legal_rep_valid_until"]);
    assert!(keep_one("institution.legal", legal, "institution.legal_rep_valid_until", json!("2027-01-31"), "vale hasta el 18 de enero de 2027").is_empty());
    // its value is kept as told (not covered by the scanner), and only the Capturist may propose it
    let rfc: String = c.query_row("SELECT value_json FROM ai_proposal WHERE field_id = 'institution.legal_rfc' AND status = 'pending'", [], |r| r.get(0)).unwrap();
    assert_eq!(rfc, "\"CSE870512AB3\"");
    let other = NewProposal {
        form_id: "institution.legal".into(),
        record_id: None,
        field_id: "institution.legal_rfc".into(),
        value: json!("CSE870512AB3"),
        origin: Origin::AiAssumption,
        source_ref: None,
        agent: "assistant",
    };
    assert!(matches!(proposals::propose(&c, other).unwrap(), ProposeOutcome::Refused { code: "not_for_the_ai" }));
}

/// ADR-041 with the scanner: the person said «No es dato personal» for the institution's RFC, so it reaches the
/// Capturist as it is and is suggested; covered, the AI never sees it and nothing is suggested for it.
#[tokio::test]
async fn what_the_person_cleared_reaches_the_capturist() {
    let said = "Nuestro RFC es CSE870512AB3.";
    let answer = json!({ "proposals": [{ "field": "institution.legal_rfc", "value": "CSE870512AB3", "quote": "Nuestro RFC es CSE870512AB3" }] });
    let (_d, db) = db();
    let held = capture(&db, None, told("institution.legal", said), None).await.unwrap();
    assert!(matches!(held, CaptureOutcome::Quarantine { .. }), "the scanner asks first: {held:?}");
    let p = MockProvider::new(vec![reply(answer.clone())]);
    let out = capture(&db, Some(&p), told("institution.legal", said), Some(Decision::NotPersonal)).await.unwrap();
    let CaptureOutcome::Proposed { fields, .. } = out else { panic!("{out:?}") };
    assert_eq!(fields, vec!["institution.legal_rfc"]);
    assert!(p.requests()[0].user.contains("CSE870512AB3"), "{}", p.requests()[0].user);

    let (_d2, covered) = self::db();
    let p = MockProvider::new(vec![reply(answer)]);
    let out = capture(&covered, Some(&p), told("institution.legal", said), Some(Decision::Redact)).await.unwrap();
    let CaptureOutcome::Proposed { fields, .. } = out else { panic!("{out:?}") };
    assert!(fields.is_empty() && !p.requests()[0].user.contains("CSE870512AB3"), "{}", p.requests()[0].user);
}
