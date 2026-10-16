//! The battery of the Capturist (IA4, C4), over `fixtures/capturista.json`: what a person tells in the chat of their
//! institution and what must be left suggested in the window. First dry (every field of the battery is one the
//! Capturist may suggest, every figure expected is in the person's words, and the code drops every trap), then a
//! capped real run with the cheapest model.

use super::*;
use crate::ai::mock::MockProvider;
use crate::ai::{AiError, AiRequest, AiResponse, Usage};
use crate::core::profile::forms::save;
use crate::core::profile::service::SaveProfileOutcome;
use std::sync::{Arc, Mutex};

/// Seconds between two cases of the real run.
const PACE_SECS: u64 = 4;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// One case of `capturista.json`.
#[derive(Debug, Clone, Deserialize)]
struct Case {
    id: String,
    ventana: String,
    dice: String,
    #[serde(default)]
    historial: Vec<Said>,
    campos: serde_json::Map<String, Value>,
    #[serde(default)]
    trampas: Vec<Value>,
    /// The person answered «No es dato personal» to the scanner (the institution's own phone or RFC): what they wrote
    /// goes as it is. Otherwise, covered.
    #[serde(default)]
    no_es_personal: bool,
}

/// A turn before what the person tells now: who said it (`persona` or `asistente`) and what.
#[derive(Debug, Clone, Deserialize)]
struct Said {
    de: String,
    texto: String,
}

fn cases() -> Vec<Case> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/capturista.json");
    let all: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    serde_json::from_value(all["casos"].clone()).unwrap()
}

/// An institution with only its name written.
fn db() -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let identity: Values = serde_json::from_value(json!({ "institution.name": "Casa Ficticia" })).unwrap();
    assert!(matches!(save(&mut c, "institution.identity", identity, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    (dir, Arc::new(Mutex::new(c)))
}

/// What the person decided before the scanner.
fn decision_of(c: &Case) -> Decision {
    if c.no_es_personal {
        Decision::NotPersonal
    } else {
        Decision::Redact
    }
}

fn capture_of(c: &Case) -> Capture {
    let history = c
        .historial
        .iter()
        .map(|s| Turn { from: if s.de == "asistente" { "assistant".into() } else { "person".into() }, text: s.texto.clone(), documents: vec![] })
        .collect();
    Capture { form: c.ventana.clone(), said: c.dice.clone(), history }
}

/// Everything the person said in a case: now and before.
fn told(c: &Case) -> String {
    c.historial.iter().filter(|s| s.de != "asistente").map(|s| s.texto.as_str()).chain([c.dice.as_str()]).collect::<Vec<_>>().join("\n")
}

/// Every field of the battery is one the Capturist may suggest in that window, every trap names a field of the
/// catalog, and every figure expected is in the person's words (so the code can find it).
#[test]
fn the_battery_speaks_of_fields_the_capturist_may_suggest() {
    let all = cases();
    assert!(all.len() >= 10, "{}", all.len());
    for c in &all {
        let spec = crate::core::institution::forms::form(&c.ventana).unwrap_or_else(|| panic!("{}: no window {}", c.id, c.ventana));
        assert!(fill_of_form(&c.ventana).is_some(), "{}: the Capturist does not fill {}", c.id, c.ventana);
        for (field, expected) in &c.campos {
            assert!(told_fields(spec).any(|f| f.id == field), "{}: {field} is not a field it may suggest", c.id);
            for n in expected["valores"].as_array().into_iter().flatten().filter_map(Value::as_i64) {
                assert!(crate::core::reader::fill::says_number(&told(c), n), "{}: {n} is not in the person's words", c.id);
            }
        }
        for t in &c.trampas {
            let field = t["field"].as_str().unwrap();
            assert!(crate::core::institution::forms::FORMS.iter().any(|f| f.field(field).is_some()), "{}: {field}", c.id);
        }
    }
    assert!(all.iter().any(|c| !c.historial.is_empty()), "one case with a conversation before");
    assert!(all.iter().any(|c| c.campos.values().all(|e| e["nada"].as_bool() == Some(true))), "one case with nothing to suggest");
}

fn reply(answer: Value) -> Result<AiResponse, AiError> {
    Ok(AiResponse { value: json!({ "tool": null, "args": null, "answer": answer }), model: "mock".into(), usage: Usage { input_tokens: 1_200, output_tokens: 150, ..Default::default() } })
}

/// The traps of the battery, answered by a model: a sum, a figure that is not in the words, words the person never
/// said, a protected field, a field of another window. The code drops them all and nothing waits.
#[tokio::test]
async fn the_code_drops_every_trap_of_the_battery() {
    let mut traps = 0;
    for c in cases().iter().filter(|c| !c.trampas.is_empty()) {
        let (_d, db) = db();
        let p = MockProvider::new(vec![reply(json!({ "proposals": c.trampas }))]);
        let out = capture(&db, Some(&p), capture_of(c), Some(decision_of(c))).await.unwrap();
        let CaptureOutcome::Proposed { fields, dropped, .. } = out else { panic!("{}: {out:?}", c.id) };
        assert_eq!((fields, dropped), (Vec::<String>::new(), c.trampas.len()), "{}", c.id);
        assert!(proposals::pending_counts(&db.lock().unwrap()).unwrap().is_empty(), "{}", c.id);
        traps += c.trampas.len();
    }
    assert!(traps >= 8, "{traps}");
}

/// How a suggestion (or its absence) compares with what the battery expects of its field.
fn grade(expected: &Value, got: Option<&Value>) -> &'static str {
    let plain_of = |v: &Value| plain(&v.as_str().map_or_else(|| v.to_string(), str::to_string));
    let Some(v) = got else {
        return if expected["nada"].as_bool() == Some(true) { "bien: nada" } else if expected["libre"].as_bool() == Some(true) { "libre: nada" } else { "FALTÓ" };
    };
    if expected["libre"].as_bool() == Some(true) {
        return "libre";
    }
    if expected["nada"].as_bool() == Some(true) {
        return "INVENTÓ";
    }
    let right = if let Some(values) = expected["valores"].as_array() {
        values.iter().any(|x| plain_of(x) == plain_of(v))
    } else if let Some(words) = expected["contiene"].as_array() {
        words.iter().all(|w| plain_of(v).contains(&plain_of(w)))
    } else if let Some(codes) = expected["incluye"].as_array() {
        codes.iter().all(|c| v.as_array().is_some_and(|a| a.contains(c)))
    } else {
        false
    };
    if right {
        "bien"
    } else {
        "MAL"
    }
}

/// The provider as it is, keeping what the model answered (before the code checks it).
struct Recording<'a> {
    inner: &'a dyn AiProvider,
    log: Mutex<Vec<Value>>,
}

#[async_trait::async_trait]
impl AiProvider for Recording<'_> {
    async fn complete(&self, req: &AiRequest) -> Result<AiResponse, AiError> {
        let out = self.inner.complete(req).await;
        self.log.lock().unwrap().push(match &out {
            Ok(r) => r.value["answer"].clone(),
            Err(e) => json!({ "error": e.kind() }),
        });
        out
    }
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn model_name(&self, tier: ModelTier) -> String {
        self.inner.model_name(tier)
    }
}

/// The real run of the Capturist (C4), opt-in and capped: every case of `capturista.json` with the provider and key
/// saved in the app (one call per case with the cheapest model), each in a new institution with only its name. For
/// each one: what the model answered, what the code kept and dropped, the verdict per field, what the program tells
/// the person, tokens, cost
/// and time. Only fictitious text goes out. Its whole output goes to a file:
///   cargo test capturist_battery_live -- --ignored --nocapture > capturist_battery_live.txt
/// With `CIMIENTO_ROUNDS` (1 to 5) it tells everything that many times, to see how steady the cheapest model is.
#[tokio::test]
#[ignore]
async fn capturist_battery_live() {
    let (cfg, key) = {
        let (_d, db) = db();
        let c = db.lock().unwrap();
        let cfg = crate::ai::settings::load(&c).unwrap();
        let key = crate::storage::get_api_key(cfg.provider.as_str()).unwrap().expect("save the key in the app first");
        println!("Proveedor: {}  modelo ligero: {}", cfg.provider.as_str(), cfg.model_for(ModelTier::Light));
        (cfg, key)
    };
    // at most 14 calls a minute, over all the cases (the free plan allows 15)
    let provider = crate::ai::paced::provider(crate::ai::build_provider(&cfg, key));
    let recording = Recording { inner: provider.as_ref(), log: Mutex::new(Vec::new()) };
    let rounds: usize = std::env::var("CIMIENTO_ROUNDS").ok().and_then(|r| r.parse().ok()).unwrap_or(1).clamp(1, 5);
    let all = cases();
    let mut right_per_case = vec![0usize; all.len()];
    let (mut passed, mut fields_right, mut fields_graded, mut invented, mut extra, mut dropped_all) = (0, 0, 0, 0, 0, 0);
    let (mut calls, mut input, mut output, mut cost) = (0i64, 0i64, 0i64, 0f64);
    for round in 1..=rounds {
        println!("\n######## Ronda {round} de {rounds}");
        for (i, c) in all.iter().enumerate() {
            // one case every few seconds: the cheapest model has a quota per minute, and a refusal for it is no verdict
            tokio::time::sleep(std::time::Duration::from_secs(PACE_SECS)).await;
            let (_d, db) = db();
            recording.log.lock().unwrap().clear();
            let started = std::time::Instant::now();
            let out = capture(&db, Some(&recording), capture_of(c), Some(decision_of(c))).await.unwrap();
            let secs = started.elapsed().as_secs_f64();
            let conn = db.lock().unwrap();
            let used: (i64, i64, i64, f64) = conn
                .query_row("SELECT count(*), coalesce(sum(input_tokens),0), coalesce(sum(output_tokens),0), coalesce(sum(estimated_cost_mxn),0) FROM ai_usage WHERE task = 'agent.capturist'", [], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                })
                .unwrap();
            (calls, input, output, cost) = (calls + used.0, input + used.1, output + used.2, cost + used.3);
            let waiting = proposals::pending(&conn, &c.ventana, None).unwrap();
            let mut bad: Vec<String> = Vec::new();
            let mut lines: Vec<String> = Vec::new();
            for (field, expected) in &c.campos {
                let got = waiting.iter().find(|p| &p.field_id == field);
                let verdict = grade(expected, got.map(|p| &p.value));
                if !verdict.starts_with("libre") {
                    fields_graded += 1;
                    fields_right += usize::from(verdict.starts_with("bien"));
                }
                if matches!(verdict, "FALTÓ" | "MAL" | "INVENTÓ") {
                    bad.push(format!("{field} {verdict}"));
                }
                invented += usize::from(verdict == "INVENTÓ");
                let shown = got.map_or("—".to_string(), |p| format!("{}  «{}»", p.value, p.said.as_deref().unwrap_or_default()));
                lines.push(format!("  {field:<40} {verdict:<12} {shown}"));
            }
            for p in waiting.iter().filter(|p| !c.campos.contains_key(&p.field_id)) {
                bad.push(format!("{} DE MÁS", p.field_id));
                extra += 1;
                lines.push(format!("  {:<40} {:<12} {}  «{}»", p.field_id, "DE MÁS", p.value, p.said.as_deref().unwrap_or_default()));
            }
            let (dropped, said) = match &out {
                CaptureOutcome::Proposed { dropped, left, still, .. } => (*dropped, format!("dejó {left:?}; puede contar {still:?}")),
                other => {
                    bad.push(format!("{other:?}"));
                    (0, String::new())
                }
            };
            dropped_all += dropped;
            right_per_case[i] += usize::from(bad.is_empty());
            passed += usize::from(bad.is_empty());
            println!("\n== {} ({})  {}", c.id, c.ventana, if bad.is_empty() { "BIEN".to_string() } else { format!("MAL: {}", bad.join("; ")) });
            println!("   «{}»", c.dice);
            println!("   llamadas {}  entrada {}  salida {}  costo ${:.4} MXN  {secs:.1} s  el código tiró {dropped}", used.0, used.1, used.2, used.3);
            for answer in recording.log.lock().unwrap().iter() {
                println!("   · el modelo contestó {answer}");
            }
            println!("{}", lines.join("\n"));
            println!("   el programa le dice: {said}");
        }
    }
    println!("\nPor caso (bien de {rounds}):");
    for (c, n) in all.iter().zip(&right_per_case) {
        println!("  {n}/{rounds}  {}", c.id);
    }
    println!(
        "\nTOTAL: {passed} de {} casos bien  campos bien {fields_right} de {fields_graded}  inventó {invented}  de más {extra}  el código tiró {dropped_all}",
        all.len() * rounds
    );
    println!("Llamadas {calls}  entrada {input}  salida {output}  costo ${cost:.4} MXN");
}
