//! The battery of the Assistant as the orchestrator of the core (C3, ADR-036 §8), over the fictitious Expediente of
//! `fixtures/expediente` and the questions of `asistente.json`: from which screen, which tool it must ask for (or not),
//! which document and page the answer must cite, and what it must say or offer. First dry (the expected page is among
//! the passages the Reader would read), then a capped real run with the cheapest model.

use super::*;
use crate::ai::mock::MockProvider;
use crate::ai::{AiResponse, Usage};
use crate::core::archive::search::tests::load_record;
use crate::core::profile::forms::save;
use crate::storage::fts::fold;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Mutex;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// One question of `asistente.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Case {
    pub q: String,
    pub pantalla: String,
    #[serde(default)]
    pub campo: Option<String>,
    #[serde(default)]
    pub usa: Vec<String>,
    #[serde(default)]
    pub no_usa: Vec<String>,
    /// (file, page); page 0 is any page of that file.
    #[serde(default)]
    pub paginas: Vec<(String, i64)>,
    #[serde(default)]
    pub con_cita: bool,
    #[serde(default)]
    pub nada: bool,
    #[serde(default)]
    pub contiene: Vec<String>,
    #[serde(default)]
    pub alguno: Vec<String>,
    #[serde(default)]
    pub no_contiene: Vec<String>,
    #[serde(default)]
    pub ir_a: Option<String>,
    #[serde(default)]
    pub llenar_con_documentos: Option<String>,
    #[serde(default)]
    pub fuentes: Vec<String>,
}

pub fn cases() -> Vec<Case> {
    #[derive(Deserialize)]
    struct File {
        preguntas: Vec<Case>,
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente/asistente.json");
    serde_json::from_str::<File>(&std::fs::read_to_string(path).unwrap()).unwrap().preguntas
}

/// An institution with only its name written and the fictitious Expediente, each file in its document: one that is
/// starting to fill «Mi institución» with its papers.
pub fn db() -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let identity: crate::common::forms::Values = serde_json::from_value(json!({ "institution.name": "Casa Ficticia" })).unwrap();
    save(&mut c, "institution.identity", identity, None).unwrap();
    load_record(&c);
    (dir, Arc::new(Mutex::new(c)))
}

pub fn ask_of(c: &Case) -> Ask {
    Ask { screen: c.pantalla.clone(), field: c.campo.clone(), question: c.q.clone(), history: vec![] }
}

/// What is wrong with an answer, against what its case expects; empty when it passes. `tools`: the tools the
/// Assistant asked for that ran, by name.
pub fn grade(c: &Case, out: &AssistantOutcome, tools: &BTreeMap<String, u64>) -> Vec<String> {
    let AssistantOutcome::Answered { text, go_to, sources, documents, from_documents, .. } = out else {
        return vec![format!("no respondió: {out:?}")];
    };
    let mut bad = Vec::new();
    for t in &c.usa {
        if !tools.contains_key(t) {
            bad.push(format!("no pidió `{t}`"));
        }
    }
    for t in &c.no_usa {
        if tools.contains_key(t) {
            bad.push(format!("pidió `{t}`"));
        }
    }
    let cited: Vec<(String, i64)> = documents.iter().map(|d| (d.document.clone(), d.page.unwrap_or(0))).collect();
    if !c.paginas.is_empty() && !c.paginas.iter().any(|(f, p)| cited.iter().any(|(cf, cp)| cf == f && (*p == 0 || cp == p))) {
        bad.push(format!("citó {cited:?}, se esperaba {:?}", c.paginas));
    }
    if c.con_cita && cited.is_empty() {
        bad.push("sin cita".into());
    }
    if c.nada && !cited.is_empty() {
        bad.push(format!("INVENTÓ: citó {cited:?} y ningún documento lo dice"));
    }
    let said = fold(text);
    for w in &c.contiene {
        if !said.contains(&fold(w)) {
            bad.push(format!("no dice «{w}»"));
        }
    }
    if !c.alguno.is_empty() && !c.alguno.iter().any(|w| said.contains(&fold(w))) {
        bad.push(format!("no dice ninguna de {:?}", c.alguno));
    }
    for w in &c.no_contiene {
        if said.contains(&fold(w)) {
            bad.push(format!("dice «{w}»"));
        }
    }
    if let Some(g) = &c.ir_a {
        if go_to.as_deref() != Some(g.as_str()) {
            bad.push(format!("ir a {go_to:?}, se esperaba {g}"));
        }
    }
    if let Some(w) = &c.llenar_con_documentos {
        if from_documents.as_ref().map(|f| f.place) != from_documents_of(w).map(|f| f.place) {
            bad.push(format!("ofreció llenar {from_documents:?}, se esperaba {w}"));
        }
    }
    for f in &c.fuentes {
        if !sources.iter().any(|s| &s.id == f) {
            bad.push(format!("sin la fuente {f}"));
        }
    }
    bad
}

/// The tools the last errand of the Assistant used, from its line in the audit log (counts, never content).
pub fn last_tools(db: &SharedDb) -> BTreeMap<String, u64> {
    let runs: Vec<String> = db.lock().unwrap().prepare("SELECT details_json FROM audit_log WHERE event='ai.run' ORDER BY rowid DESC").unwrap().query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    runs.iter()
        .filter_map(|d| serde_json::from_str::<Value>(d).ok())
        .find(|d| d["agent"] == "assistant")
        .and_then(|d| serde_json::from_value(d["tools"].clone()).ok())
        .unwrap_or_default()
}

/// Dry, with no AI: for every question the Reader must answer, the expected page is among the passages it would read
/// (by words, with the question of the person as it is). What it does not find, no model can cite.
#[tokio::test]
async fn the_page_of_each_question_is_among_the_passages_of_the_reader() {
    let (_d, db) = db();
    let (mut asked, mut found) = (0, 0);
    for c in cases().iter().filter(|c| !c.paginas.is_empty()) {
        let map = crate::core::records::map(&db.lock().unwrap()).unwrap();
        let only = crate::core::records::named_in(&map, &c.q).and_then(|d| d.document_id.clone());
        let shown = crate::core::reader::ask::passages_for(&db, None, &c.q, only.as_deref()).await.unwrap();
        let hit = c.paginas.iter().any(|(f, p)| shown.iter().any(|s| &s.passage.document == f && (*p == 0 || s.passage.page == Some(*p))));
        println!("{:<60} {} {:?}", c.q, if hit { "sí" } else { "NO" }, shown.iter().map(|s| (s.passage.document.as_str(), s.passage.page)).collect::<Vec<_>>());
        asked += 1;
        found += usize::from(hit);
    }
    println!("{found} de {asked} preguntas con su página entre los pasajes del Lector");
    assert_eq!(found, asked);
}

/// Dry: the grading itself. A simulated run that asks the Reader and cites the expected page passes; one that cites a
/// document no one asked for where nothing is said, or offers the wrong window, fails.
#[tokio::test]
async fn the_battery_grades_what_it_should() {
    let (_d, db) = db();
    let all = cases();
    let predial = all.iter().find(|c| c.q.contains("predial")).unwrap();
    let shown = crate::core::reader::ask::passages_for(&db, None, &predial.q, None).await.unwrap();
    let p = shown.iter().find(|s| s.passage.snippet.contains("impuesto predial")).unwrap();
    let source = crate::core::agents::tools::passage_source(&p.passage);
    let reply = |v: Value| Ok(AiResponse { value: v, model: "mock".into(), usage: Usage::default() });
    let m = MockProvider::new(vec![
        reply(json!({ "tool": "record_reader", "args": { "question": predial.q, "document": null }, "answer": null })),
        reply(json!({ "tool": null, "args": null, "answer": { "found": true, "text": "La comodataria paga el predial.", "citations": [{ "passage": p.id, "quote": "del impuesto predial" }] } })),
        reply(json!({ "tool": null, "args": null, "answer": { "text": "Lo paga la institución, como comodataria.", "go_to": null, "sources": [source], "fill": null, "from_documents": null } })),
    ]);
    let out = ask(&db, Some(&m), None, Some(Role::Manager), ask_of(predial), None).await.unwrap();
    let tools = last_tools(&db);
    assert_eq!(grade(predial, &out, &tools), Vec::<String>::new(), "{out:?}");

    let nothing = all.iter().find(|c| c.nada).unwrap();
    assert!(grade(nothing, &out, &tools).iter().any(|b| b.starts_with("INVENTÓ")));
    let legal = all.iter().find(|c| c.llenar_con_documentos.is_some()).unwrap();
    assert!(!grade(legal, &out, &tools).is_empty());
    let docs = all.iter().find(|c| c.q == "¿Qué documentos tengo?").unwrap();
    assert!(grade(docs, &out, &tools).iter().any(|b| b == "pidió `record_reader`"));
}

/// The real run of C3 (ADR-036 §8), opt-in and capped: every question of `asistente.json` with the provider and key
/// saved in the app (the cheapest model; at most 5 steps of the Assistant and one call of the Reader per request it
/// makes) and, with Gemini, the vectors of the fictitious fragments. For each one: the verdict, the tools, what it
/// cited, the answer, calls, tokens, cost and time. Only fictitious text goes out. Its whole output goes to a file:
///   cargo test assistant_battery_live -- --ignored --nocapture > assistant_battery_live.txt
/// With `CIMIENTO_ROUNDS` (1 to 5) it asks everything that many times, to see how steady the cheapest model is.
#[tokio::test]
#[ignore]
async fn assistant_battery_live() {
    let (_d, db) = db();
    let provider = {
        let c = db.lock().unwrap();
        let cfg = crate::ai::settings::load(&c).unwrap();
        let key = crate::storage::get_api_key(cfg.provider.as_str()).unwrap().expect("save the key in the app first");
        println!("Proveedor: {}  modelo ligero: {}", cfg.provider.as_str(), cfg.model_for(ModelTier::Light));
        crate::ai::build_provider(&cfg, key)
    };
    let embedder = crate::core::archive::vectors::embedder(&db, false);
    if let Some(e) = &embedder {
        let made = crate::core::archive::vectors::index_pending(&db, e.as_ref()).await.unwrap();
        println!("Vectores de los fragmentos: {made}");
    } else {
        println!("Sin vectores: solo por palabras");
    }
    let usage = |db: &SharedDb, task: &str| -> (i64, i64, i64, f64) {
        db.lock()
            .unwrap()
            .query_row(
                "SELECT count(*), coalesce(sum(input_tokens),0), coalesce(sum(output_tokens),0), coalesce(sum(estimated_cost_mxn),0) FROM ai_usage WHERE task LIKE ?1",
                [task],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap()
    };
    let rounds: usize = std::env::var("CIMIENTO_ROUNDS").ok().and_then(|r| r.parse().ok()).unwrap_or(1).clamp(1, 5);
    let recording = Recording { inner: provider.as_ref(), log: Mutex::new(Vec::new()) };
    let all = cases();
    let mut right_per_case = vec![0usize; all.len()];
    let (mut passed, mut tools_ok, mut cites_ok, mut cites_asked, mut invented) = (0, 0, 0, 0, 0);
    for round in 1..=rounds {
        println!("\n######## Ronda {round} de {rounds}");
        for (i, c) in all.iter().enumerate() {
            let before = (usage(&db, "agent.assistant"), usage(&db, "agent.reader_answer"));
            recording.log.lock().unwrap().clear();
            let started = std::time::Instant::now();
            let out = ask(&db, Some(&recording), embedder.clone(), Some(Role::Manager), ask_of(c), None).await.unwrap();
            let secs = started.elapsed().as_secs_f64();
            let after = (usage(&db, "agent.assistant"), usage(&db, "agent.reader_answer"));
            let tools = last_tools(&db);
            let bad = grade(c, &out, &tools);
            right_per_case[i] += usize::from(bad.is_empty());
            passed += usize::from(bad.is_empty());
            tools_ok += usize::from(!bad.iter().any(|b| b.contains('`')));
            if !c.paginas.is_empty() || c.con_cita {
                cites_asked += 1;
                cites_ok += usize::from(!bad.iter().any(|b| b.starts_with("citó") || b == "sin cita"));
            }
            invented += usize::from(bad.iter().any(|b| b.starts_with("INVENTÓ")));
            let calls = (after.0 .0 - before.0 .0, after.1 .0 - before.1 .0);
            let cost = (after.0 .3 - before.0 .3) + (after.1 .3 - before.1 .3);
            let tokens = ((after.0 .1 - before.0 .1) + (after.1 .1 - before.1 .1), (after.0 .2 - before.0 .2) + (after.1 .2 - before.1 .2));
            println!("\n== «{}» ({})  {}", c.q, c.pantalla, if bad.is_empty() { "BIEN".to_string() } else { format!("MAL: {}", bad.join("; ")) });
            println!("   herramientas {tools:?}  llamadas asistente {} lector {}  entrada {} salida {}  costo ${cost:.4} MXN  {secs:.1} s", calls.0, calls.1, tokens.0, tokens.1);
            // each step, as it went and came back (fictitious text only): what the Assistant asked for and what the
            // Reader was asked and answered
            for (task, user, value) in recording.log.lock().unwrap().iter() {
                if task == "agent.reader_answer" {
                    println!("   · Lector: {}\n     contestó {value}", user.lines().next().unwrap_or_default());
                } else if !value["tool"].is_null() {
                    println!("   · Asistente pidió `{}` con {}", value["tool"].as_str().unwrap_or("?"), value["args"]);
                }
            }
            println!("   {out:?}");
        }
    }
    let a = usage(&db, "agent.assistant");
    let r = usage(&db, "agent.reader_answer");
    let e = usage(&db, "embed.%");
    println!("\nPor pregunta (bien de {rounds}):");
    for (c, n) in all.iter().zip(&right_per_case) {
        println!("  {n}/{rounds}  {}", c.q);
    }
    println!("\nTOTAL: {passed} de {} bien  herramienta bien {tools_ok}  cita bien {cites_ok} de {cites_asked}  inventó {invented}", all.len() * rounds);
    println!("Asistente: {} llamadas, {} entrada, {} salida, ${:.4} MXN", a.0, a.1, a.2, a.3);
    println!("Lector: {} llamadas, {} entrada, {} salida, ${:.4} MXN", r.0, r.1, r.2, r.3);
    println!("Vectores: {} solicitudes, ${:.4} MXN", e.0, e.3);
    println!("Todo: ${:.4} MXN", a.3 + r.3 + e.3);
}

/// A provider that writes down what each step asked and answered, for the real run to show it (fictitious text only).
struct Recording<'a> {
    inner: &'a dyn crate::ai::AiProvider,
    log: Mutex<Vec<(String, String, Value)>>,
}

impl Recording<'_> {
    fn keep(&self, req: &crate::ai::AiRequest, out: &Result<AiResponse, crate::ai::AiError>) {
        let value = match out {
            Ok(r) => r.value.clone(),
            Err(e) => json!({ "error": e.kind() }),
        };
        let answer = if value["answer"].is_null() { value.clone() } else { value["answer"].clone() };
        let shown = if req.task.as_str() == "agent.reader_answer" { answer } else { value };
        let user = req.user.lines().find(|l| l.starts_with("Pregunta:")).unwrap_or(&req.user).to_string();
        self.log.lock().unwrap().push((req.task.as_str().to_string(), user, shown));
    }
}

#[async_trait::async_trait]
impl crate::ai::AiProvider for Recording<'_> {
    async fn complete(&self, req: &crate::ai::AiRequest) -> Result<AiResponse, crate::ai::AiError> {
        let out = self.inner.complete(req).await;
        self.keep(req, &out);
        out
    }
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn model_name(&self, tier: ModelTier) -> String {
        self.inner.model_name(tier)
    }
    fn model_chain(&self, tier: ModelTier) -> Vec<String> {
        self.inner.model_chain(tier)
    }
    async fn complete_with_model(&self, req: &crate::ai::AiRequest, model: &str) -> Result<AiResponse, crate::ai::AiError> {
        let out = self.inner.complete_with_model(req, model).await;
        self.keep(req, &out);
        out
    }
}
