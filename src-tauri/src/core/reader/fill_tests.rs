use super::*;
use crate::core::agents::capturist::plain;
use crate::ai::mock::MockProvider;
use crate::ai::{AiError, AiResponse, Usage};
use crate::core::archive::search::tests::load_record;
use crate::core::profile::forms::save;
use crate::core::profile::service::SaveProfileOutcome;
use std::sync::Mutex;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// An institution with only its name written, and the fictitious Expediente of `fixtures/expediente` in its drawer.
fn db(with_record: bool) -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let identity: Values = serde_json::from_value(json!({ "institution.name": "Casa Ficticia" })).unwrap();
    assert!(matches!(save(&mut c, "institution.identity", identity, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    if with_record {
        load_record(&c);
    }
    (dir, Arc::new(Mutex::new(c)))
}

fn reply(answer: Value) -> Result<AiResponse, AiError> {
    Ok(AiResponse { value: json!({ "tool": null, "args": null, "answer": answer }), model: "mock".into(), usage: Usage { input_tokens: 3_000, output_tokens: 300, ..Default::default() } })
}

/// The reference of the Reader (`fixtures/expediente/lector.json`): by window, what each field should get and where.
pub fn reference() -> serde_json::Map<String, Value> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente/lector.json");
    let mut all: serde_json::Map<String, Value> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    all.remove("_nota");
    all
}

#[test]
fn every_field_the_ai_may_see_has_its_search_words_and_its_line_in_the_reference() {
    let reference = reference();
    for (form, _, _) in FILLABLE {
        let spec = crate::core::institution::forms::form(form).unwrap();
        for f in open_fields(spec) {
            assert!(SEARCH.iter().any(|(id, w)| *id == f.id && !w.is_empty()), "{} has no search words", f.id);
            assert!(reference[*form].get(f.id).is_some(), "{} has no line in lector.json", f.id);
        }
    }
}

/// The search per field, by words alone (no AI, no internet): the page where each field is written is among the
/// passages shown for it. What it does not find, no model can read.
#[test]
fn the_page_of_each_field_is_among_its_passages() {
    let (_d, db) = db(true);
    let conn = db.lock().unwrap();
    let (mut asked, mut found) = (0, 0);
    for (form, fields) in reference() {
        let spec = crate::core::institution::forms::form(&form).unwrap();
        for (field, expected) in fields.as_object().unwrap() {
            let Some(pages) = expected["paginas"].as_array() else { continue };
            let f = spec.field(field).unwrap();
            let shown = passages_for(&conn, &[f], &[None]).unwrap();
            let hit = shown.iter().any(|p| pages.iter().any(|e| e[0] == p.document.as_str() && e[1].as_i64() == p.page));
            println!("{field:<40} {} {:?}", if hit { "sí" } else { "NO" }, shown.iter().map(|p| (p.document.as_str(), p.page)).collect::<Vec<_>>());
            asked += 1;
            found += usize::from(hit);
        }
    }
    println!("{found} de {asked} campos con su página entre sus pasajes");
    assert!(found * 10 >= asked * 9, "{found} de {asked}");
}

/// The passage that says something, by the words it has.
fn passage_with<'a>(passages: &'a [Passage], words: &str) -> &'a Passage {
    passages.iter().find(|p| plain(&p.text).contains(&plain(words))).unwrap_or_else(|| panic!("no passage says «{words}»"))
}

/// C2 (ADR-036 §4): the code keeps only what a passage shown says, in its words, with a figure that is in them and a
/// value that fits; a protected field or one not asked is never kept. What passes waits with its document and page.
#[tokio::test]
async fn the_code_keeps_only_what_a_passage_says() {
    let (_d, db) = db(true);
    let spec = crate::core::institution::forms::form("institution.legal").unwrap();
    let passages = {
        let conn = db.lock().unwrap();
        let saved = crate::core::profile::forms::saved_values(&conn, "institution.legal").unwrap();
        passages_for(&conn, &asked_fields(spec, &saved), &[]).unwrap()
    };
    let capital = passage_with(&passages, "Régimen capital: INSTITUCION DE ASISTENCIA PRIVADA");
    let deed = passage_with(&passages, "dos mil quince");
    let answer = json!({
        "proposals": [
            { "field": "institution.legal_form", "value": "iap", "passage": capital.id, "quote": "Régimen capital: INSTITUCION DE ASISTENCIA PRIVADA" },
            { "field": "institution.founded_year", "value": 2015, "passage": deed.id, "quote": "a los diez días del mes de marzo de dos mil quince" },
            // words that are not in the passage it cites
            { "field": "institution.tax_regime", "value": "non_profit", "passage": deed.id, "quote": "Personas Morales con Fines no Lucrativos" },
            // a passage that was never shown
            { "field": "institution.authorized_donee", "value": "yes", "passage": "P99", "quote": "se autoriza" },
            // a figure that is not in its words: the AI counted
            { "field": "institution.founded_year", "value": 2016, "passage": deed.id, "quote": "dos mil quince" },
            // a protected field, never asked
            { "field": "institution.legal_rfc", "value": "CRF150310KX4", "passage": capital.id, "quote": "Régimen capital" },
            // a value out of the options of its field
            { "field": "institution.legal_form", "value": "sociedad", "passage": capital.id, "quote": "Régimen capital" }
        ],
        "note": "Dejé dos sugerencias."
    });
    let provider = MockProvider::new(vec![reply(answer)]);
    let out = fill_window(&db, Some(&provider), None, "institution.legal").await.unwrap();
    let FillOutcome::Proposed { fields, dropped, note, .. } = out else { panic!("{out:?}") };
    assert_eq!(fields, vec!["institution.legal_form", "institution.founded_year"]);
    assert_eq!(dropped, 5);
    assert_eq!(note, "Dejé dos sugerencias.");
    let conn = db.lock().unwrap();
    let waiting = proposals::pending(&conn, "institution.legal", None).unwrap();
    let year = waiting.iter().find(|p| p.field_id == "institution.founded_year").unwrap();
    assert_eq!((year.value.clone(), year.origin, year.agent.as_str()), (json!(2015), Origin::Document, "reader"));
    assert_eq!((year.document.as_deref(), year.page), (Some("acta-constitutiva.txt"), Some(1)));
    let source: Value = serde_json::from_str(year.source_ref.as_deref().unwrap()).unwrap();
    assert!(source["rank"].is_i64() && source["sha"].is_string() && source["quote"].as_str().unwrap().contains("dos mil quince"));
    // nothing of the institution changed
    assert!(crate::core::profile::forms::saved_values(&conn, "institution.legal").unwrap().is_empty());
    // what the AI was shown: the empty fields it may see, never a protected one, and the passages with their page
    let sent: String = provider.requests().iter().map(|r| format!("{}\n{}", r.context.join("\n"), r.user)).collect();
    assert!(sent.contains("institution.purpose") && !sent.contains("`institution.legal_rfc`"), "{sent}");
}

/// ADR-038: a written field is not asked, and a window with every field written asks nothing (no call).
#[tokio::test]
async fn a_written_field_is_not_asked_and_a_full_window_asks_nothing() {
    let (_d, db) = db(true);
    {
        let mut conn = db.lock().unwrap();
        let capacity: Values = serde_json::from_value(json!({
            "institution.capacity_total": 50, "institution.served_estimate": 48, "institution.staff_paid_estimate": 31,
            "institution.staff_volunteer_estimate": 18, "institution.notes": "Lista de espera de catorce personas."
        }))
        .unwrap();
        assert!(matches!(save(&mut conn, "institution.capacity", capacity, None).unwrap(), SaveProfileOutcome::Saved { .. }));
        let spec = crate::core::institution::forms::form("institution.identity").unwrap();
        let saved = crate::core::profile::forms::saved_values(&conn, "institution.identity").unwrap();
        assert!(asked_fields(spec, &saved).iter().all(|f| f.id != "institution.name"), "the name is written");
    }
    let provider = MockProvider::new(vec![]);
    let out = fill_window(&db, Some(&provider), None, "institution.capacity").await.unwrap();
    assert!(matches!(out, FillOutcome::Nothing { reason: "all_written", .. }), "{out:?}");
    assert!(provider.requests().is_empty(), "no call");
}

/// Without documents nothing is asked; without the AI it says so; a window that is not of «Mi institución» is none.
#[tokio::test]
async fn without_documents_or_the_ai_it_says_why() {
    let (_d, empty) = db(false);
    let out = fill_window(&empty, None, None, "institution.legal").await.unwrap();
    assert!(matches!(out, FillOutcome::Nothing { reason: "no_documents", .. }), "{out:?}");
    let (_d2, db) = db(true);
    let out = fill_window(&db, None, None, "institution.legal").await.unwrap();
    assert!(matches!(out, FillOutcome::Unavailable { reason: "not_configured", .. }), "{out:?}");
    assert!(matches!(fill_window(&db, None, None, "institution.board_member").await, Err(ServiceError::NotFound)));
}

#[test]
fn a_number_is_in_words_or_in_figures() {
    assert!(says_number("Atendimos a 48 personas residentes", 48));
    assert!(says_number("a los diez días del mes de marzo de dos mil quince", 2015));
    assert!(says_number("treinta y una personas contratadas", 31));
    assert!(!says_number("Atendimos a 48 personas y a 22 más", 70), "a sum is not in the words");
    // an article is not a count (the Capturist took «voluntarios de una parroquia» as 1 volunteer)
    assert!(!says_number("vienen voluntarios de una parroquia", 1));
    assert!(!says_number("un grupo de voluntarios", 1));
    assert!(says_number("solo nos ayuda uno", 1));
    assert!(says_number("Tenemos 1 voluntario", 1));
    assert!(says_number("cuarenta y un lugares", 41));
    assert!(!says_number("el primer piso", 1));
    assert!(says_number("cero voluntarios", 0));
}

/// How a suggestion (or its absence) compares with what `lector.json` expects of its field.
fn grade(expected: &Value, got: Option<&proposals::Proposal>) -> (&'static str, bool) {
    let plain_of = |v: &Value| plain(&v.as_str().map_or_else(|| v.to_string(), str::to_string));
    let page_ok = |p: &proposals::Proposal| {
        expected["paginas"].as_array().is_none_or(|pages| pages.iter().any(|e| Some(e[0].as_str().unwrap_or_default()) == p.document.as_deref() && e[1].as_i64() == p.page))
    };
    let Some(p) = got else {
        return if expected["nada"].as_bool() == Some(true) { ("bien: nada", true) } else if expected["libre"].as_bool() == Some(true) { ("libre: nada", true) } else { ("FALTÓ", true) };
    };
    if expected["libre"].as_bool() == Some(true) {
        return ("libre", page_ok(p));
    }
    if expected["nada"].as_bool() == Some(true) {
        let fine = expected["aceptables"].as_array().is_some_and(|a| a.iter().any(|x| plain_of(x) == plain_of(&p.value)));
        return (if fine { "aceptable" } else { "INVENTÓ" }, page_ok(p));
    }
    let right = if let Some(values) = expected["valores"].as_array() {
        values.iter().any(|x| plain_of(x) == plain_of(&p.value))
    } else if let Some(words) = expected["contiene"].as_array() {
        words.iter().all(|w| plain_of(&p.value).contains(&plain_of(w)))
    } else if let Some(words) = expected["alguno"].as_array() {
        words.iter().any(|w| plain_of(&p.value).contains(&plain_of(w)))
    } else if let Some(codes) = expected["incluye"].as_array() {
        codes.iter().all(|c| p.value.as_array().is_some_and(|a| a.contains(c)))
    } else {
        false
    };
    (if right { "bien" } else { "MAL" }, page_ok(p))
}

/// The real run of C2 (ADR-036 §8), opt-in and capped: the four windows of «Mi institución» over the fictitious
/// Expediente, with the provider and key saved in the app (4 calls with the cheapest model) and, with Gemini, the
/// vectors of the fictitious fragments (a couple of requests). For every field of `lector.json`: the value, whether
/// it is right, and whether its page is one expected; then calls, tokens, cost and time per window. Only fictitious
/// text goes out. With `CIMIENTO_ROUNDS` (1 to 5) it fills everything that many times, each round over a new
/// Expediente, to see how steady the cheapest model is. Its whole output goes to a file:
///   cargo test reader_live -- --ignored --nocapture > reader_live.txt
#[tokio::test]
#[ignore]
async fn reader_live() {
    let rounds: usize = std::env::var("CIMIENTO_ROUNDS").ok().and_then(|r| r.parse().ok()).unwrap_or(1).clamp(1, 5);
    let (mut right, mut wrong, mut missed, mut invented, mut pages_ok, mut graded) = (0, 0, 0, 0, 0, 0);
    let mut spent = (0, 0, 0, 0.0);
    for round in 1..=rounds {
        println!("\n######## Ronda {round} de {rounds}");
        let (r, s) = reader_round().await;
        (right, wrong, missed, invented, pages_ok, graded) = (right + r.0, wrong + r.1, missed + r.2, invented + r.3, pages_ok + r.4, graded + r.5);
        spent = (spent.0 + s.0, spent.1 + s.1, spent.2 + s.2, spent.3 + s.3);
    }
    println!("\nTOTAL ({rounds} rondas): bien {right}  mal {wrong}  faltó {missed}  inventó {invented}  páginas bien {pages_ok} de {graded}");
    println!("Llamadas {}  entrada {}  salida {}  costo ${:.4} MXN", spent.0, spent.1, spent.2, spent.3);
}

/// One round of `reader_live` over a new Expediente: the verdicts (right, wrong, missed, invented, pages right, pages
/// graded) and what it spent (calls, tokens in and out, cost).
async fn reader_round() -> ((usize, usize, usize, usize, usize, usize), (i64, i64, i64, f64)) {
    let (_d, db) = db(true);
    let provider = {
        let c = db.lock().unwrap();
        let cfg = crate::ai::settings::load(&c).unwrap();
        let key = crate::storage::get_api_key(cfg.provider.as_str()).unwrap().expect("save the key in the app first");
        println!("Proveedor: {}  modelo ligero: {}", cfg.provider.as_str(), cfg.model_for(ModelTier::Light));
        // at most 14 calls a minute to each model, over all the rounds (the free plan allows 15)
        crate::ai::paced::provider(crate::ai::build_provider(&cfg, key))
    };
    let embedder = vectors::embedder(&db, false).map(crate::ai::paced::embedder);
    if let Some(e) = &embedder {
        let made = vectors::index_pending(&db, e.as_ref()).await.unwrap();
        println!("Vectores de los fragmentos: {made}");
    } else {
        println!("Sin vectores: solo por palabras");
    }
    let usage = |db: &SharedDb| -> (i64, i64, i64, f64) {
        db.lock()
            .unwrap()
            .query_row(
                "SELECT count(*), coalesce(sum(input_tokens),0), coalesce(sum(output_tokens),0), coalesce(sum(estimated_cost_mxn),0) FROM ai_usage WHERE task = 'agent.reader'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap()
    };
    let reference = reference();
    let (mut right, mut wrong, mut missed, mut invented, mut pages_ok, mut graded) = (0, 0, 0, 0, 0, 0);
    for (form, _, name) in FILLABLE {
        let before = usage(&db);
        let started = std::time::Instant::now();
        let out = fill_window(&db, Some(provider.as_ref()), embedder.clone(), form).await.unwrap();
        let after = usage(&db);
        println!("\n== {name}: {:.1} s  llamadas {}  entrada {}  salida {}  costo ${:.4} MXN", started.elapsed().as_secs_f64(), after.0 - before.0, after.1 - before.1, after.2 - before.2, after.3 - before.3);
        println!("{out:?}");
        let waiting = proposals::pending(&db.lock().unwrap(), form, None).unwrap();
        let saved = crate::core::profile::forms::saved_values(&db.lock().unwrap(), form).unwrap();
        for (field, expected) in reference[*form].as_object().unwrap() {
            if is_filled(saved.get(field.as_str())) {
                println!("  {field:<40} ya escrito: no se pidió");
                continue;
            }
            let got = waiting.iter().find(|p| &p.field_id == field);
            let (verdict, page) = grade(expected, got);
            let shown = got.map_or("—".to_string(), |p| format!("{}  ({}, p{})", p.value, p.document.as_deref().unwrap_or("?"), p.page.unwrap_or(0)));
            let quote = got.and_then(|p| p.source_ref.as_deref()).and_then(|s| serde_json::from_str::<Value>(s).ok()).and_then(|v| v["quote"].as_str().map(str::to_string)).unwrap_or_default();
            println!("  {field:<40} {verdict:<12} {}  {shown}\n      «{quote}»", if got.is_some() { if page { "página bien" } else { "PÁGINA MAL" } } else { "" });
            match verdict {
                "bien" => right += 1,
                "MAL" => wrong += 1,
                "FALTÓ" => missed += 1,
                "INVENTÓ" => invented += 1,
                _ => {}
            }
            if got.is_some() && expected["libre"].as_bool() != Some(true) {
                graded += 1;
                pages_ok += usize::from(page);
            }
        }
    }
    let total = usage(&db);
    println!("\nRonda: bien {right}  mal {wrong}  faltó {missed}  inventó {invented}  páginas bien {pages_ok} de {graded}");
    println!("Llamadas {}  entrada {}  salida {}  costo ${:.4} MXN", total.0, total.1, total.2, total.3);
    ((right, wrong, missed, invented, pages_ok, graded), total)
}
