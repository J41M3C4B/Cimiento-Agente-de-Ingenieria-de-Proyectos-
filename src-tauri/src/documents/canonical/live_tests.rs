//! Measuring the canonical reading on the real documents (ADR-015). Everything here is `#[ignore]`: it
//! runs on purpose, with the key saved in the app, and writes its results to a folder.
//!
//! The manifest (`CIMIENTO_CANON_MANIFEST`) names the packages and the strategies:
//! ```json
//! { "out": "D:\\...\\salida", "model": "gemini-3.5-flash-lite", "rpm": 4, "thinking": "",
//!   "embedding_model": "gemini-embedding-001", "inline_schema": false,
//!   "strategies": ["oneshot", "block_all", "block_lexical", "block_embedding"],
//!   "packages": [{ "name": "alsea", "docs": ["D:\\...\\a.pdf", "D:\\...\\b.docx"], "ref": "D:\\...\\alsea.json" }] }
//! ```
//! `canonical_sizes` spends nothing (no call to any service); `canonical_models_live` lists the models that
//! can embed (free); `canonical_live` makes the calls. A run whose result file already exists is skipped,
//! so a run that stops can be started again without paying twice.

use super::assemble::{assemble, evidence_text, Reading};
use super::contract::{canonical_schema, field_queries, model_schema, model_schema_of, section_query, SECTIONS};
use super::package::Package;
use super::retrieve::{budget_chars, Embedder, GeminiEmbedder, Retrieval, RetrieverKind};
use super::run::{read_canonical, Options, Strategy};
use crate::ai::gemini::GeminiProvider;
use crate::ai::{AiError, AiProvider, AiRequest, AiResponse, ModelTier};
use async_trait::async_trait;
use crate::ai::pipeline::SqliteLedger;
use crate::ai::settings::{self, RateLimit};
use crate::documents::text::norm;
use crate::scanner::{RegexScanner, ScannerConfig};
use crate::storage::open_encrypted;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

struct PackSpec {
    name: String,
    docs: Vec<String>,
    reference: Option<String>,
}

struct Manifest {
    out: PathBuf,
    model: String,
    rpm: u32,
    thinking: String,
    embedding_model: String,
    inline_schema: bool,
    /// Make the second reading over the pages the first one left uncited.
    residual: bool,
    /// Seconds one call may take before it is given up (0: the task's own limit).
    timeout_secs: u64,
    /// Real calls the whole run may make (0: no limit); the next one is refused without touching the service.
    max_calls: u32,
    strategies: Vec<Strategy>,
    packages: Vec<PackSpec>,
}

fn manifest() -> Manifest {
    let path = std::env::var("CIMIENTO_CANON_MANIFEST").expect("set CIMIENTO_CANON_MANIFEST to the manifest JSON");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("the manifest can be read")).expect("the manifest is JSON");
    let out = PathBuf::from(v["out"].as_str().expect("out"));
    std::fs::create_dir_all(&out).unwrap();
    Manifest {
        out,
        model: v["model"].as_str().unwrap_or("gemini-3.5-flash-lite").to_string(),
        rpm: v["rpm"].as_u64().unwrap_or(4) as u32,
        thinking: v["thinking"].as_str().unwrap_or("").to_string(),
        embedding_model: v["embedding_model"].as_str().unwrap_or("gemini-embedding-001").to_string(),
        inline_schema: v["inline_schema"].as_bool().unwrap_or(false),
        residual: v["residual"].as_bool().unwrap_or(false),
        timeout_secs: v["timeout_secs"].as_u64().unwrap_or(0),
        max_calls: v["max_calls"].as_u64().unwrap_or(0) as u32,
        strategies: v["strategies"].as_array().expect("strategies").iter().map(|s| Strategy::parse(s.as_str().unwrap()).expect("a known strategy")).collect(),
        packages: v["packages"]
            .as_array()
            .expect("packages")
            .iter()
            .map(|p| PackSpec {
                name: p["name"].as_str().unwrap().to_string(),
                docs: p["docs"].as_array().unwrap().iter().map(|d| d.as_str().unwrap().to_string()).collect(),
                reference: p["ref"].as_str().map(String::from),
            })
            .collect(),
    }
}

fn plain(s: &str) -> String {
    s.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

fn facts(path: &str) -> Vec<(String, Vec<regex::Regex>)> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    v["facts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["name"].as_str().unwrap().to_string(), f["all"].as_array().unwrap().iter().map(|p| regex::Regex::new(&format!("(?i){}", p.as_str().unwrap())).unwrap()).collect()))
        .collect()
}

fn load(spec: &PackSpec) -> Package {
    Package::from_paths(&spec.name, &spec.docs).unwrap_or_else(|e| panic!("package {}: {e}", spec.name))
}

/// Without calling any service: how big each package is, what each strategy would send per block with the
/// lexical retriever (the one that needs nothing), and how big the schema is. Run it first.
///   cargo test canonical_sizes -- --ignored --nocapture
#[test]
#[ignore]
fn canonical_sizes() {
    let m = manifest();
    let canon = canonical_schema();
    println!("\nEsquema que ve el modelo (bytes): completo con $ref {}, completo en línea {}", model_schema(&canon, None, false).to_string().len(), model_schema(&canon, None, true).to_string().len());
    for s in SECTIONS {
        println!("  {s:<15} con $ref {:>6}  en línea {:>6}", model_schema(&canon, Some(s), false).to_string().len(), model_schema(&canon, Some(s), true).to_string().len());
    }
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let mut md = String::from("# Tamaños en seco (sin llamadas)\n\n");
    for spec in &m.packages {
        let pkg = load(spec);
        let total = pkg.total_chars();
        let budget = budget_chars(total);
        println!("\n=== {} · {} archivos · {} páginas · {} caracteres (≈ {} tokens) · presupuesto por bloque {}", spec.name, pkg.documents().len(), pkg.pages.len(), total, total / 4, budget);
        md.push_str(&format!("## {}\n\n{} páginas, {} caracteres (≈ {} tokens); presupuesto por bloque {} caracteres.\n\n| bloque | páginas enviadas | caracteres | % del paquete |\n|---|---|---|---|\n", spec.name, pkg.pages.len(), total, total / 4, budget));
        let retrieval = rt.block_on(Retrieval::build(&pkg, RetrieverKind::Lexical, None)).unwrap();
        let mut sum = 0;
        for s in SECTIONS {
            let mut q: Vec<String> = field_queries(&canon, s).into_iter().map(|(_, q)| q).collect();
            q.push(section_query(&canon, s));
            let sel = rt.block_on(retrieval.select(&pkg, &q, budget)).unwrap();
            sum += sel.chars;
            println!("  {s:<15} {:>3} de {} páginas · {:>6} caracteres ({:>3.0} %) · primeras por puntaje {:?}", sel.pages.len(), pkg.pages.len(), sel.chars, 100.0 * sel.chars as f64 / total as f64, sel.top.iter().take(3).map(|(p, _)| *p).collect::<Vec<_>>());
            md.push_str(&format!("| {s} | {} de {} | {} | {:.0} % |\n", sel.pages.len(), pkg.pages.len(), sel.chars, 100.0 * sel.chars as f64 / total as f64));
        }
        println!("  suma de los 8 bloques: {sum} caracteres = {:.1} veces el paquete (el paquete entero en una llamada = 1.0; los 8 bloques con todo = 8.0)", sum as f64 / total as f64);
        md.push_str(&format!("\nSuma de los 8 bloques: {:.1} veces el paquete.\n\n", sum as f64 / total as f64));
        println!("  mapa de páginas (primeras 8):\n{}", pkg.render_map().lines().take(8).map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n"));
    }
    std::fs::write(m.out.join("tamanos.md"), md).unwrap();
}

/// Models of the service that can embed (free; generates nothing).
///   cargo test canonical_models_live -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn canonical_models_live() {
    let key = crate::storage::get_api_key("gemini").unwrap().expect("no Gemini key saved: save it in the app first");
    let e = GeminiEmbedder::new(key, "x");
    for (name, methods) in e.embedding_models().await.expect("the list of models") {
        println!("{name}: {methods:?}");
    }
}

fn usage_of(db: &Arc<Mutex<rusqlite::Connection>>, label: &str) -> Value {
    let c = db.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*), COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0), COALESCE(SUM(cached_tokens),0), COALESCE(SUM(thought_tokens),0),
                COALESCE(SUM(estimated_cost_mxn),0), COALESCE(SUM(CASE WHEN success=0 THEN 1 ELSE 0 END),0), COALESCE(SUM(latency_ms),0)
         FROM ai_usage WHERE project_id = ?1",
        [label],
        |r| {
            Ok(json!({
                "registros": r.get::<_, i64>(0)?, "entrada": r.get::<_, i64>(1)?, "salida": r.get::<_, i64>(2)?, "cache": r.get::<_, i64>(3)?,
                "pensamiento": r.get::<_, i64>(4)?, "costo_mxn": r.get::<_, f64>(5)?, "fallidos": r.get::<_, i64>(6)?, "latencia_ms": r.get::<_, i64>(7)?,
            }))
        },
    )
    .unwrap()
}

/// Spends a limited daily allowance with care. The pipeline retries a slow or failed call (and a
/// read retries its calls again), and the service counts every attempt, a hung one included. Here the
/// first failure of a reading ends its calls without touching the service, and the whole run has a
/// ceiling of real calls.
struct Careful<'a> {
    inner: &'a GeminiProvider,
    spent: &'a AtomicU32,
    max_calls: u32,
    failed: Mutex<Option<AiError>>,
}

impl Careful<'_> {
    fn forget_failure(&self) {
        *self.failed.lock().unwrap() = None;
    }
}

#[async_trait]
impl AiProvider for Careful<'_> {
    async fn complete(&self, req: &AiRequest) -> Result<AiResponse, AiError> {
        self.complete_with_model(req, &self.inner.model_name(req.tier)).await
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
    async fn complete_with_model(&self, req: &AiRequest, model: &str) -> Result<AiResponse, AiError> {
        if let Some(e) = self.failed.lock().unwrap().clone() {
            println!("    (sin llamar al servicio: la lectura ya falló con «{e}»)");
            return Err(e);
        }
        let n = self.spent.fetch_add(1, Ordering::SeqCst) + 1;
        if self.max_calls > 0 && n > self.max_calls {
            self.spent.fetch_sub(1, Ordering::SeqCst);
            println!("    (tope de {} llamadas reales alcanzado: no se llama)", self.max_calls);
            return Err(AiError::QuotaReached);
        }
        println!("    llamada real {n}{}", if self.max_calls > 0 { format!(" de {}", self.max_calls) } else { String::new() });
        let result = self.inner.complete_with_model(req, model).await;
        if let Err(e) = &result {
            println!("    la llamada {n} falló: {e}");
            *self.failed.lock().unwrap() = Some(e.clone());
        }
        result
    }
}

/// The measured run: every package with every strategy, one call after another within the allowance.
///   $env:CIMIENTO_CANON_MANIFEST="D:\...\manifiesto.json"; cargo test canonical_live -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn canonical_live() {
    let m = manifest();
    let key = crate::storage::get_api_key("gemini").unwrap().expect("no Gemini key saved: save it in the app first");

    // a throwaway database: the person's data is never touched, and the allowance is counted from zero
    let dir = tempfile::tempdir().unwrap();
    let conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let mut cfg = settings::load(&conn).unwrap();
    cfg.gemini_model_light = m.model.clone();
    cfg.gemini_model_strong = m.model.clone();
    cfg.gemini_light_fallbacks.clear();
    cfg.gemini_strong_fallbacks.clear();
    cfg.disabled_models.clear();
    cfg.effort_strong = m.thinking.clone();
    cfg.monthly_cap_mxn = 100_000.0;
    // the person's rule: at most 5 calls a minute; the run stops at 4 and lets the minute pass
    cfg.rate_limits.insert(m.model.clone(), RateLimit { per_minute: m.rpm, tokens_per_minute: 200_000, per_day: 400 });
    settings::save(&conn, &cfg).unwrap();
    let db = Arc::new(Mutex::new(conn));
    let ledger = SqliteLedger(db.clone());
    let timed = |p: GeminiProvider| if m.timeout_secs > 0 { p.with_timeout(Duration::from_secs(m.timeout_secs)) } else { p };
    let provider = timed(GeminiProvider::new(key.clone(), cfg.clone()));
    // the service refuses the whole schema as too complex: the one-shot reading describes it in the instructions
    let prompt_provider = timed(GeminiProvider::new(key.clone(), cfg.clone()).with_schema_in_prompt());
    let spent = AtomicU32::new(0);
    let careful = Careful { inner: &provider, spent: &spent, max_calls: m.max_calls, failed: Mutex::new(None) };
    let careful_prompt = Careful { inner: &prompt_provider, spent: &spent, max_calls: m.max_calls, failed: Mutex::new(None) };
    let scanner = RegexScanner::new(ScannerConfig::default());
    let embedder: Arc<dyn Embedder> = Arc::new(GeminiEmbedder::new(key, &m.embedding_model));
    println!("modelo {} · {} llamadas por minuto · pensamiento «{}» · embeddings {} · esquema {}", m.model, m.rpm, m.thinking, m.embedding_model, if m.inline_schema { "en línea" } else { "con $ref" });

    for spec in &m.packages {
        let pkg = load(spec);
        let ref_facts = spec.reference.as_deref().map(facts);
        println!("\n=== {} · {} páginas · {} caracteres", spec.name, pkg.pages.len(), pkg.total_chars());
        for strategy in &m.strategies {
            let stem = format!("{}__{}{}", spec.name, strategy.label(), if m.residual { "_seg" } else { "" });
            let report_path = m.out.join(format!("{stem}.informe.json"));
            // a finished run is not paid twice; one with a failed call is done again
            let finished = std::fs::read_to_string(&report_path)
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                .is_some_and(|r| r["llamadas"].as_array().is_some_and(|c| !c.is_empty() && c.iter().all(|x| x["ok"] == true)));
            if finished {
                println!("  {} ya está hecho: se salta", strategy.label());
                continue;
            }
            let label = format!("canon:{stem}");
            let opts = Options { strategy: *strategy, inline_schema: m.inline_schema, budget: None, residual: m.residual, label: label.clone() };
            println!("  → {}…", strategy.label());
            let started = std::time::Instant::now();
            let used: &Careful = if *strategy == Strategy::OneShot { &careful_prompt } else { &careful };
            used.forget_failure();
            let run = read_canonical(used, &scanner, &ledger, Some(embedder.clone()), &pkg, &opts).await;
            let wall = started.elapsed().as_secs_f64();
            let usage = usage_of(&db, &label);
            let reading = Reading {
                model: m.model.clone(),
                at: now_iso(),
                input_tokens: usage["entrada"].as_u64().unwrap_or(0) + usage["cache"].as_u64().unwrap_or(0),
                output_tokens: usage["salida"].as_u64().unwrap_or(0),
                cost_mxn: usage["costo_mxn"].as_f64().unwrap_or(0.0),
            };
            let (doc, report) = assemble(&pkg, &run.answers, &reading);
            let scored = ref_facts.as_ref().map(|f| {
                let hay = plain(&evidence_text(&doc));
                let missing: Vec<String> = f.iter().filter(|(_, all)| !all.iter().all(|r| r.is_match(&hay))).map(|(n, _)| n.clone()).collect();
                json!({ "encontrados": f.len() - missing.len(), "total": f.len(), "faltan": missing })
            });
            println!(
                "    {}/{} llamadas · {:.0} s · {} tokens de entrada, {} de salida · ${:.4} MXN · citas verificadas {:.0} % ({} reubicadas) · campos {}/{}/{} (encontrado/ambiguo/no aparece){}",
                run.calls_ok(), run.calls.len(), wall, usage["entrada"], usage["salida"], usage["costo_mxn"].as_f64().unwrap_or(0.0),
                100.0 * report.verified_rate(), report.citations_relocated, report.fields.encontrado, report.fields.ambiguo, report.fields.no_aparece,
                scored.as_ref().map(|s| format!(" · hechos {}/{}", s["encontrados"], s["total"])).unwrap_or_default()
            );
            println!(
                "    cobertura: {:.0} % de las páginas con alguna cita verificada{} · {:.0} % del texto citado · sin cita: {:?}",
                100.0 * report.page_coverage(),
                run.uncited_before.as_ref().map(|u| format!(" (antes de la segunda lectura: {:.0} %)", 100.0 * (1.0 - u.len() as f64 / pkg.pages.len().max(1) as f64))).unwrap_or_default(),
                100.0 * report.text_coverage,
                report.pages_uncited
            );
            std::fs::write(m.out.join(format!("{stem}.canonico.json")), serde_json::to_string_pretty(&doc).unwrap()).unwrap();
            let informe = json!({
                "paquete": spec.name, "estrategia": strategy.label(), "modelo": m.model, "paginas": pkg.pages.len(), "caracteres": pkg.total_chars(),
                "segundos_reloj": wall, "esquema_en_prompt": *strategy == Strategy::OneShot, "llamadas": run.calls, "caracteres_enviados": run.chars_sent(), "uso": usage, "embeddings": run.embedding,
                "ensamblado": report, "hechos": scored, "paginas_por_bloque": run.selections, "paginas_sin_cita_antes": run.uncited_before, "paginas_poco_citadas_antes": run.thin_before, "cobertura_de_texto": report.text_coverage, "respaldo_lexico": run.lexical_fallback, "cobertura_de_paginas": report.page_coverage(),
            });
            std::fs::write(&report_path, serde_json::to_string_pretty(&informe).unwrap()).unwrap();
        }
    }
    summarize(&m);
}

// ---------------------------------------------------------------- summary

fn states(doc: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for s in SECTIONS {
        for (f, v) in doc[s].as_object().into_iter().flatten() {
            out.insert(format!("{s}.{f}"), v["estado"].as_str().unwrap_or("").to_string());
        }
    }
    out
}

fn quotes(doc: &Value) -> BTreeSet<String> {
    fn walk(v: &Value, out: &mut BTreeSet<String>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    match (k.as_str(), x) {
                        ("metadatos", _) => {}
                        ("cita", Value::String(s)) => {
                            out.insert(norm(s).chars().take(60).collect());
                        }
                        _ => walk(x, out),
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    walk(doc, &mut out);
    out
}

/// Writes `resumen.md` from every result in the folder: one row per package and strategy, and how much
/// the strategies agree with one another (same state per field; same quotes).
fn summarize(m: &Manifest) {
    let mut md = String::from("# Resultados del pipeline canónico\n\n");
    md.push_str(&format!("Modelo: `{}` (temperatura 0, semilla 7). Generado {}.\n\n", m.model, now_iso()));
    md.push_str("| paquete | estrategia | llamadas ok | seg. (reloj) | tokens entrada | tokens salida | pensamiento | costo MXN | citas verificadas | campos enc./amb./no | elementos | hechos |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    let mut by_pkg: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for spec in &m.packages {
        for strategy in &m.strategies {
            let stem = format!("{}__{}{}", spec.name, strategy.label(), if m.residual { "_seg" } else { "" });
            let Ok(text) = std::fs::read_to_string(m.out.join(format!("{stem}.informe.json"))) else { continue };
            let r: Value = serde_json::from_str(&text).unwrap();
            let calls = r["llamadas"].as_array().cloned().unwrap_or_default();
            let ok = calls.iter().filter(|c| c["ok"] == true).count();
            let a = &r["ensamblado"];
            let facts = if r["hechos"].is_object() { format!("{}/{}", r["hechos"]["encontrados"], r["hechos"]["total"]) } else { "—".into() };
            md.push_str(&format!(
                "| {} | {} | {}/{} | {:.0} | {} | {} | {} | {:.4} | {:.0} % ({} de {}) | {}/{}/{} | {} | {} |\n",
                spec.name, strategy.label(), ok, calls.len(), r["segundos_reloj"].as_f64().unwrap_or(0.0),
                r["uso"]["entrada"], r["uso"]["salida"], r["uso"]["pensamiento"], r["uso"]["costo_mxn"].as_f64().unwrap_or(0.0),
                100.0 * a["citations_verified"].as_f64().unwrap_or(0.0) / a["citations_emitted"].as_f64().unwrap_or(1.0).max(1.0), a["citations_verified"], a["citations_emitted"],
                a["fields"]["encontrado"], a["fields"]["ambiguo"], a["fields"]["no_aparece"], a["fields"]["items"], facts
            ));
            by_pkg.entry(spec.name.clone()).or_default().insert(strategy.label().to_string(), r);
        }
    }
    md.push_str("\n## Acuerdo entre estrategias (mismo paquete)\n\nPor campo: mismo estado (encontrado/ambiguo/no aparece). Por cita: coinciden las citas (60 primeros caracteres normalizados), Jaccard.\n\n| paquete | A | B | mismo estado | citas en común |\n|---|---|---|---|---|\n");
    for (pkg, runs) in &by_pkg {
        let docs: Vec<(&String, Value)> = runs
            .keys()
            .filter_map(|s| std::fs::read_to_string(m.out.join(format!("{pkg}__{s}.canonico.json"))).ok().map(|t| (s, serde_json::from_str::<Value>(&t).unwrap())))
            .collect();
        for i in 0..docs.len() {
            for j in i + 1..docs.len() {
                let (sa, sb) = (states(&docs[i].1), states(&docs[j].1));
                let same = sa.iter().filter(|(k, v)| sb.get(*k) == Some(*v)).count();
                let (qa, qb) = (quotes(&docs[i].1), quotes(&docs[j].1));
                let inter = qa.intersection(&qb).count();
                let union = qa.union(&qb).count().max(1);
                md.push_str(&format!("| {pkg} | {} | {} | {:.0} % ({same}/{}) | {:.0} % ({inter}/{union}) |\n", docs[i].0, docs[j].0, 100.0 * same as f64 / sa.len().max(1) as f64, sa.len(), 100.0 * inter as f64 / union as f64));
            }
        }
    }
    md.push_str("\n## Hechos que faltan (paquetes con referencia)\n");
    for (pkg, runs) in &by_pkg {
        for (s, r) in runs {
            if let Some(f) = r["hechos"]["faltan"].as_array() {
                md.push_str(&format!("\n**{pkg} · {s}** ({} faltan): {}\n", f.len(), f.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("; ")));
            }
        }
    }
    std::fs::write(m.out.join("resumen.md"), md).unwrap();
    println!("\nResumen escrito en {}", m.out.join("resumen.md").display());
}

/// Rebuilds `resumen.md` from the results already in the folder (no calls).
///   cargo test canonical_summary -- --ignored --nocapture
#[test]
#[ignore]
fn canonical_summary() {
    summarize(&manifest());
}

/// What the service accepts of the schema: the same tiny request (one line of context) with the block
/// or the whole schema, with `$ref` or inline. A 400 says «invalid argument» and nothing more, so the
/// variants are what tells which part it does not take.
///   cargo test canonical_probe_live -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn canonical_probe_live() {
    use crate::ai::{AiRequest, AiTask, ModelTier};
    let key = crate::storage::get_api_key("gemini").unwrap().expect("no Gemini key saved: save it in the app first");
    let model = std::env::var("CIMIENTO_PROBE_MODEL").unwrap_or_else(|_| "gemini-3.5-flash-lite".into());
    let cfg = crate::ai::settings::AiSettings::default();
    let provider = GeminiProvider::new(key.clone(), cfg);
    let canon = canonical_schema();
    let http = reqwest::Client::new();
    let variants: Vec<(String, Value)> = vec![
        ("bloque entrega con $ref".into(), model_schema(&canon, Some("entrega"), false)),
        ("bloque entrega en línea".into(), model_schema(&canon, Some("entrega"), true)),
        ("bloque temporalidad con $ref".into(), model_schema(&canon, Some("temporalidad"), false)),
        ("4 bloques (identidad..financiamiento)".into(), model_schema_of(&canon, &SECTIONS[..4], false)),
        ("4 bloques (proyecto..entrega)".into(), model_schema_of(&canon, &SECTIONS[4..], false)),
        ("2 bloques (identidad, temporalidad)".into(), model_schema_of(&canon, &SECTIONS[..2], false)),
        ("3 bloques (identidad..elegibilidad)".into(), model_schema_of(&canon, &SECTIONS[..3], false)),
        ("6 bloques (identidad..documentacion)".into(), model_schema_of(&canon, &SECTIONS[..6], false)),
    ];
    for (name, schema) in variants {
        let req = AiRequest {
            task: AiTask::CallCanonical,
            tier: ModelTier::Strong,
            system: crate::ai::prompts::system_prompt(AiTask::CallCanonical),
            context: vec!["[Página 1 | a.pdf]\nEl cierre de la postulación es el 23 de mayo de 2027. Las solicitudes se entregan en la oficina central.".into()],
            user: "Llene el bloque pedido.".into(),
            output_schema: schema.clone(),
            max_output_tokens: 6000,
        };
        let body = provider.body_for(&req, &model);
        let resp = http
            .post(format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"))
            .header("x-goog-api-key", &key)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let v: Value = resp.json().await.unwrap_or(Value::Null);
        let msg = if status == 200 { format!("ok · {} tokens de salida", v["usageMetadata"]["candidatesTokenCount"]) } else { v["error"]["message"].as_str().unwrap_or("").chars().take(400).collect() };
        println!("{name:<32} esquema {:>6} bytes → {status} {msg}", schema.to_string().len());
        tokio::time::sleep(std::time::Duration::from_secs(16)).await;
    }
}

/// Spends nothing: prints, for every package of the manifest, the table rows rebuilt from the PDFs, to see
/// them before any model reads them.
#[test]
#[ignore]
fn canonical_rows() {
    let m = manifest();
    for spec in &m.packages {
        let pkg = load(spec);
        println!("\n=== {}", spec.name);
        for p in &pkg.pages {
            if let Some(i) = p.text.find(crate::documents::pdf_rows::BLOCK_TITLE) {
                println!("--- página {} ({})\n{}", p.number, p.document, &p.text[i..]);
            }
        }
    }
}

/// The start of the text of some pages of a package, as the model reads them (no cost). `CIMIENTO_CANON_PAGES` is
/// `paquete:3,4,5`.
///   $env:CIMIENTO_CANON_PAGES="nmp2026:13,14"; cargo test canonical_pages -- --ignored --nocapture
#[test]
#[ignore]
fn canonical_pages() {
    let m = manifest();
    let want = std::env::var("CIMIENTO_CANON_PAGES").expect("CIMIENTO_CANON_PAGES, as package:3,4,5");
    let (name, pages) = want.split_once(':').expect("package:3,4,5");
    let spec = m.packages.iter().find(|p| p.name == name).expect("a package of the manifest");
    let pkg = load(spec);
    for n in pages.split(',').filter_map(|n| n.trim().parse::<usize>().ok()) {
        match pkg.page(n) {
            Some(p) => println!("--- página {n} ({}, {} caracteres)
{}", p.document, p.text.chars().count(), p.text.chars().take(700).collect::<String>()),
            None => println!("--- página {n}: no existe"),
        }
    }
}

/// Two short texts to the embedding model of the manifest: tells at once whether the model takes what
/// the pipeline sends it (a task type and a size of vector), before a whole package depends on it.
#[tokio::test]
#[ignore]
async fn canonical_embed_probe() {
    let m = manifest();
    let key = crate::storage::get_api_key("gemini").unwrap().expect("no Gemini key saved: save it in the app first");
    let e = GeminiEmbedder::new(key, &m.embedding_model);
    let texts = vec!["fecha límite de entrega de solicitudes".to_string(), "monto máximo de apoyo por proyecto".to_string()];
    for query in [true, false] {
        match e.embed(&texts, query).await {
            Ok(v) => println!("{} · query={query}: {} vectores de {} dimensiones · {:?}", m.embedding_model, v.len(), v[0].len(), e.usage()),
            Err(err) => println!("{} · query={query}: ERROR {err}", m.embedding_model),
        }
    }
}
