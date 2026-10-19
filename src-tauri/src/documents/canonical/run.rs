//! The two shapes of the reading, to be measured against each other (ADR-015):
//!
//! * **one shot**: the whole package and the whole schema in one call;
//! * **by block**: one call per block of the schema, each with its own schema and only the pages that
//!   block needs (`BlockRetrieved`), or with every page as a control (`BlockAll`) to tell the effect of
//!   splitting from the effect of choosing pages.
//!
//! Calls run one after another: the per-minute allowance of the model is small, and `pipeline::pace`
//! waits it out. A block that cannot be read is left blank and listed, never invented.

use super::assemble::{assemble, Answers, Reading};
use super::contract::{canonical_schema, field_queries, model_schema, section_query, SECTIONS};
use super::package::Package;
use super::retrieve::{budget_chars, Embedder, Retrieval, RetrieverKind};
use crate::ai::pipeline::{self, AiCall, Custom, Ledger};
use crate::ai::{AiError, AiProvider, AiTask};
use crate::scanner::SensitiveScanner;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A whole schema answered in one call needs room: every field is a small object with its quotes.
const ONE_SHOT_OUTPUT_TOKENS: u32 = 30_000;
/// A block is a fraction of that.
const BLOCK_OUTPUT_TOKENS: u32 = 12_000;
/// The second reading sends the pages the first one never quoted, in groups no longer than this...
const RESIDUAL_GROUP_CHARS: usize = 40_000;
/// ...and at most this many groups per block, so a very long package cannot make the second reading
/// cost more than the first.
const RESIDUAL_MAX_GROUPS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    OneShot,
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    BlockAll,
    BlockRetrieved(RetrieverKind),
}

impl Strategy {
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn label(self) -> &'static str {
        match self {
            Strategy::OneShot => "oneshot",
            Strategy::BlockAll => "block_all",
            Strategy::BlockRetrieved(RetrieverKind::Embedding) => "block_embedding",
            Strategy::BlockRetrieved(RetrieverKind::Lexical) => "block_lexical",
        }
    }

    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn parse(s: &str) -> Option<Strategy> {
        [Strategy::OneShot, Strategy::BlockAll, Strategy::BlockRetrieved(RetrieverKind::Embedding), Strategy::BlockRetrieved(RetrieverKind::Lexical)]
            .into_iter()
            .find(|x| x.label() == s)
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub strategy: Strategy,
    /// No `$ref` in the schema sent to the model: larger, accepted by any provider.
    pub inline_schema: bool,
    /// Characters a block may read; `None` = `budget_chars` of the package.
    pub budget: Option<usize>,
    /// A second reading of each block over the pages the first one left without a single verified quote.
    /// Only by block: the code counts what was covered, the model decides what the rest means.
    pub residual: bool,
    /// Label stored as the project of every call, so the usage log can be added up per run.
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallReport {
    pub block: String,
    pub pages_sent: usize,
    pub pages_total: usize,
    pub chars_sent: usize,
    pub ok: bool,
    pub error: Option<String>,
    pub secs: f64,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingUse {
    pub model: String,
    pub requests: usize,
    pub texts: usize,
}

/// What the model said, block by block, and what each call took. `assemble::assemble` turns it into the document.
#[derive(Debug, Clone)]
pub struct CanonicalRun {
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub strategy: Strategy,
    pub answers: Answers,
    pub calls: Vec<CallReport>,
    pub embedding: Option<EmbeddingUse>,
    /// Pages chosen per block, for the report.
    pub selections: Vec<(String, Vec<usize>)>,
    /// Pages with no verified quote after the first reading (only when a second one was made).
    pub uncited_before: Option<Vec<usize>>,
    /// Pages whose quotes cover little of their text after the first reading: what the second one reads.
    pub thin_before: Option<Vec<usize>>,
    /// Blocks whose pages were chosen with the lexical score because the embedding service failed
    /// (or «índice» when it failed before any block).
    pub lexical_fallback: Vec<String>,
}

impl CanonicalRun {
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn secs(&self) -> f64 {
        self.calls.iter().map(|c| c.secs).sum()
    }

    pub fn calls_ok(&self) -> usize {
        self.calls.iter().filter(|c| c.ok).count()
    }

    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn chars_sent(&self) -> usize {
        self.calls.iter().map(|c| c.chars_sent).sum()
    }
}

/// One call, with the waits a real service asks for: a minute when it says «too many», a few seconds
/// when it is overloaded. A daily allowance used up, a refusal or a bad key are not retried.
async fn call_with_retries(
    provider: &dyn AiProvider,
    scanner: &dyn SensitiveScanner,
    ledger: &dyn Ledger,
    label: &str,
    context: Vec<String>,
    user: String,
    schema: &Value,
    max_output_tokens: u32,
) -> (Result<Value, AiError>, u32) {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let call = AiCall { task: AiTask::CallCanonical, context: context.clone(), user: user.clone(), project_id: Some(label.to_string()) };
        let result = pipeline::run_with(provider, scanner, ledger, call, Some(Custom { schema: schema.clone(), max_output_tokens, system: None })).await;
        let wait = match &result {
            Err(AiError::RateLimited) if attempts < 4 => 65,
            Err(AiError::Http { status, .. }) if *status >= 500 && attempts < 3 => 20 * u64::from(attempts),
            Err(AiError::Timeout | AiError::Truncated | AiError::BadOutput(_)) if attempts < 2 => 5,
            _ => return (result, attempts),
        };
        tokio::time::sleep(Duration::from_secs(wait)).await;
    }
}

/// Reads the package with the chosen strategy. Never fails: what could not be read is missing from
/// `answers` and the calls say why.
pub async fn read_canonical(
    provider: &dyn AiProvider,
    scanner: &dyn SensitiveScanner,
    ledger: &dyn Ledger,
    embedder: Option<Arc<dyn Embedder>>,
    pkg: &Package,
    opts: &Options,
) -> CanonicalRun {
    let canon = canonical_schema();
    let mut run = CanonicalRun { strategy: opts.strategy, answers: Answers::default(), calls: Vec::new(), embedding: None, selections: Vec::new(), uncited_before: None, thin_before: None, lexical_fallback: Vec::new() };
    let total_pages = pkg.pages.len();

    if opts.strategy == Strategy::OneShot {
        let schema = model_schema(&canon, None, opts.inline_schema);
        let context = pkg.render(&pkg.all_numbers());
        let started = Instant::now();
        let chars = context.chars().count();
        let (result, attempts) = call_with_retries(
            provider,
            scanner,
            ledger,
            &opts.label,
            vec![context],
            "Lea toda la convocatoria y llene la ficha completa.".into(),
            &schema,
            ONE_SHOT_OUTPUT_TOKENS,
        )
        .await;
        let ok = result.is_ok();
        let error = result.as_ref().err().map(|e| format!("{}: {e}", e.kind()));
        if let Ok(answer) = &result {
            run.answers.add(answer);
        }
        run.calls.push(CallReport { block: "todo".into(), pages_sent: total_pages, pages_total: total_pages, chars_sent: chars, ok, error, secs: started.elapsed().as_secs_f64(), attempts });
        return run;
    }

    // by block: the index is built once for the package
    let retrieval = match opts.strategy {
        Strategy::BlockRetrieved(kind) => match Retrieval::build(pkg, kind, embedder.clone()).await {
            Ok(r) => Some(r),
            // the embedding service is down or out of allowance: the lexical score needs no service, and a
            // reading with it is better than no reading
            Err(e) if kind == RetrieverKind::Embedding && Retrieval::build(pkg, RetrieverKind::Lexical, None).await.is_ok() => {
                run.lexical_fallback.push(format!("índice: {e}"));
                Retrieval::build(pkg, RetrieverKind::Lexical, None).await.ok()
            }
            Err(e) => {
                run.calls.push(CallReport { block: "índice".into(), pages_sent: 0, pages_total: total_pages, chars_sent: 0, ok: false, error: Some(e), secs: 0.0, attempts: 1 });
                return run;
            }
        },
        _ => None,
    };
    let budget = opts.budget.unwrap_or_else(|| budget_chars(pkg.total_chars()));
    let map = pkg.render_map();

    let mut stopped = false;
    let mut lexical: Option<Retrieval> = None;
    for section in SECTIONS {
        let pages: Vec<usize> = match &retrieval {
            Some(r) => {
                let mut queries: Vec<String> = field_queries(&canon, section).into_iter().map(|(_, q)| q).collect();
                queries.push(section_query(&canon, section));
                let selected = match r.select(pkg, &queries, budget).await {
                    Ok(s) => Ok(s.pages),
                    Err(e) if opts.strategy == Strategy::BlockRetrieved(RetrieverKind::Embedding) => {
                        if lexical.is_none() {
                            lexical = Retrieval::build(pkg, RetrieverKind::Lexical, None).await.ok();
                        }
                        match &lexical {
                            Some(l) => l.select(pkg, &queries, budget).await.map(|s| s.pages).map_err(|_| e.clone()).inspect(|_| run.lexical_fallback.push(format!("{section}: {e}"))),
                            None => Err(e),
                        }
                    }
                    Err(e) => Err(e),
                };
                match selected {
                    Ok(pages) => pages,
                    Err(e) => {
                        run.calls.push(CallReport { block: section.into(), pages_sent: 0, pages_total: total_pages, chars_sent: 0, ok: false, error: Some(e), secs: 0.0, attempts: 1 });
                        continue;
                    }
                }
            }
            None => pkg.all_numbers(),
        };
        run.selections.push((section.to_string(), pages.clone()));
        let mut context = vec![pkg.render(&pages)];
        if pages.len() < total_pages {
            context.push(format!("Mapa de todas las páginas del paquete (recibes solo algunas):\n{map}"));
        }
        let chars: usize = context.iter().map(|c| c.chars().count()).sum();
        let schema = model_schema(&canon, Some(section), opts.inline_schema);
        let user = format!("Llene el bloque «{section}» de la ficha. Recibes {} de {total_pages} páginas del paquete.", pages.len());
        let started = Instant::now();
        let (result, attempts) = call_with_retries(provider, scanner, ledger, &opts.label, context, user, &schema, BLOCK_OUTPUT_TOKENS).await;
        let ok = result.is_ok();
        let error = result.as_ref().err().map(|e| format!("{}: {e}", e.kind()));
        let stop = matches!(result, Err(AiError::QuotaReached | AiError::Auth | AiError::BudgetExhausted));
        if let Ok(answer) = &result {
            run.answers.add(answer);
        }
        run.calls.push(CallReport { block: section.into(), pages_sent: pages.len(), pages_total: total_pages, chars_sent: chars, ok, error, secs: started.elapsed().as_secs_f64(), attempts });
        if stop {
            stopped = true;
            break;
        }
    }
    if opts.residual && !stopped {
        residual_pass(provider, scanner, ledger, pkg, opts, &canon, &mut run).await;
    }
    run.embedding = embedder.filter(|_| matches!(opts.strategy, Strategy::BlockRetrieved(RetrieverKind::Embedding))).map(|e| {
        let (requests, texts) = e.usage();
        EmbeddingUse { model: e.label(), requests, texts }
    });
    run
}

/// Marks what the first reading already took from a page.
const TAKEN: &str = "[…ya extraído…]";

/// Every verified quote of the document with the page it came from.
fn quotes_by_page(doc: &Value) -> BTreeMap<usize, Vec<String>> {
    fn walk(v: &Value, out: &mut BTreeMap<usize, Vec<String>>) {
        match v {
            Value::Object(m) => {
                if let (Some(q), Some(p)) = (m.get("cita").and_then(Value::as_str), m.get("pagina").and_then(Value::as_u64)) {
                    out.entry(p as usize).or_default().push(q.to_string());
                }
                m.values().for_each(|x| walk(x, out));
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = BTreeMap::new();
    walk(doc, &mut out);
    out
}

/// The page with the passages the first reading quoted replaced by a marker, so the second reads what
/// nobody claimed. A quote is looked for word by word, whatever the spacing between the words; one that is
/// not found in the page as written is left out of the masking (the page then simply shows more).
fn mask(text: &str, quotes: &[String]) -> String {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for q in quotes {
        let words: Vec<String> = q.split_whitespace().map(regex::escape).collect();
        if words.is_empty() {
            continue;
        }
        if let Ok(re) = regex::Regex::new(&words.join(r"\s+")) {
            spans.extend(re.find_iter(text).map(|m| (m.start(), m.end())));
        }
    }
    spans.sort_unstable();
    let mut out = String::new();
    let mut at = 0;
    for (s, e) in spans {
        if e <= at {
            continue;
        }
        if s > at {
            out.push_str(&text[at..s]);
        }
        // two taken passages with only blanks between them are one
        if !out.trim_end().ends_with(TAKEN) {
            out.push_str(TAKEN);
        }
        at = e;
    }
    out.push_str(&text[at..]);
    out
}

/// Pages ready to send, each as `(number, text as the model reads it)`, grouped in runs within `limit`
/// characters (a page longer than the limit goes alone), at most `max_groups` of them.
fn group_rendered(pages: Vec<(usize, String)>, limit: usize, max_groups: usize) -> Vec<Vec<(usize, String)>> {
    let mut groups: Vec<Vec<(usize, String)>> = Vec::new();
    let mut chars = 0;
    for page in pages {
        let len = page.1.chars().count();
        match groups.last_mut() {
            Some(g) if chars + len <= limit => {
                g.push(page);
                chars += len;
            }
            _ => {
                groups.push(vec![page]);
                chars = len;
            }
        }
    }
    groups.truncate(max_groups);
    groups
}

/// What the first reading already put in a block, field by field, so the second does not repeat it.
fn digest(doc: &Value, section: &str) -> String {
    let Some(fields) = doc[section].as_object() else { return String::new() };
    fields
        .iter()
        .map(|(name, v)| {
            let items: usize = ["elementos", "hitos", "criterios", "modalidades"].iter().map(|k| v[*k].as_array().map_or(0, Vec::len)).sum();
            match (v["estado"].as_str().unwrap_or("no_aparece"), items) {
                ("no_aparece", _) => format!("- {name}: vacío"),
                (_, 0) => format!("- {name}: ya tiene dato"),
                (_, n) => format!("- {name}: ya tiene {n} elemento(s)"),
            }
        })
        .collect::<Vec<_>>()
        .join("
")
}

/// The second reading. The code counts which pages carry no verified quote after the first one; each block
/// is read again over those pages only, and what it finds is added to the first answers. Nothing here
/// knows what a funder writes: a page nobody quoted is simply where what was missed has to be.
async fn residual_pass(provider: &dyn AiProvider, scanner: &dyn SensitiveScanner, ledger: &dyn Ledger, pkg: &Package, opts: &Options, canon: &Value, run: &mut CanonicalRun) {
    let (first, report) = assemble(pkg, &run.answers, &Reading::default());
    run.uncited_before = Some(report.pages_uncited.clone());
    run.thin_before = Some(report.pages_thin.clone());
    if report.pages_thin.is_empty() {
        return;
    }
    let total_pages = pkg.pages.len();
    let taken = quotes_by_page(&first);
    let rendered: Vec<(usize, String)> = report
        .pages_thin
        .iter()
        .filter_map(|n| pkg.page(*n))
        .map(|p| (p.number, format!("[Página {} | {}]\n{}", p.number, p.document, mask(&p.text, taken.get(&p.number).map_or(&[][..], Vec::as_slice)))))
        .collect();
    let groups = group_rendered(rendered, RESIDUAL_GROUP_CHARS, RESIDUAL_MAX_GROUPS);
    for section in SECTIONS {
        // a block the first reading could not do is not made up for here: that would be a first reading in disguise
        if !run.answers.sections.contains_key(section) {
            continue;
        }
        let schema = model_schema(canon, Some(section), opts.inline_schema);
        let known = digest(&first, section);
        for (i, group) in groups.iter().enumerate() {
            let context = vec![group.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>().join("\n\n")];
            let chars: usize = context.iter().map(|c| c.chars().count()).sum();
            let user = format!(
                "Llene el bloque «{section}» de la ficha. Es una segunda lectura: la primera ya llenó este bloque y en estas {} de {total_pages} páginas dejó pasajes sin tomar \
                 (grupo {} de {}). Lo que la primera lectura ya tomó está tapado con {TAKEN}: no lo repitas, léelo solo como contexto. \
                 Lee lo que queda a la vista de principio a fin; si hay listas con incisos o números, revisa uno por uno sus elementos y toma cada uno que corresponda a este bloque. \
                 Deja `no_aparece` lo que estas páginas no digan. Lo que la primera lectura ya tiene en este bloque:\n{known}",
                group.len(),
                i + 1,
                groups.len()
            );
            let started = Instant::now();
            let (result, attempts) = call_with_retries(provider, scanner, ledger, &opts.label, context, user, &schema, BLOCK_OUTPUT_TOKENS).await;
            let ok = result.is_ok();
            let error = result.as_ref().err().map(|e| format!("{}: {e}", e.kind()));
            let stop = matches!(result, Err(AiError::QuotaReached | AiError::Auth | AiError::BudgetExhausted));
            if let Ok(answer) = &result {
                run.answers.merge(answer);
            }
            run.calls.push(CallReport { block: format!("{section}+segunda"), pages_sent: group.len(), pages_total: total_pages, chars_sent: chars, ok, error, secs: started.elapsed().as_secs_f64(), attempts });
            if stop {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::mock::MockProvider;
    use crate::ai::pipeline::SqliteLedger;
    use crate::ai::{AiResponse, Usage};
    use crate::documents::canonical::assemble::{assemble, Reading};
    use crate::documents::canonical::contract::{blank_section, EXTRAS};
    use crate::documents::canonical::package::tests::sample;
    use crate::documents::canonical::retrieve::tests::FakeEmbedder;
    use crate::scanner::{RegexScanner, ScannerConfig};
    use crate::storage::open_encrypted;
    use serde_json::{json, Map};
    use std::sync::Mutex;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn ledger() -> (tempfile::TempDir, SqliteLedger) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(open_encrypted(&dir.path().join("t.db"), KEY).unwrap()));
        (dir, SqliteLedger(conn))
    }

    fn strip(v: &mut Value) {
        match v {
            Value::Object(m) => {
                m.remove("normalizado");
                m.values_mut().for_each(strip);
            }
            Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }

    /// A model that answers «no_aparece» to everything, in the shape it is asked for.
    fn blank(sections: &[&str]) -> Value {
        let canon = canonical_schema();
        let mut m = Map::new();
        for s in sections {
            let mut b = blank_section(&canon, s);
            strip(&mut b);
            m.insert(s.to_string(), b);
        }
        for e in EXTRAS {
            m.insert(e.to_string(), json!([]));
        }
        Value::Object(m)
    }

    fn reply(v: Value) -> Result<AiResponse, AiError> {
        Ok(AiResponse { value: v, model: "gemini-3.5-flash-lite".into(), usage: Usage { input_tokens: 5000, output_tokens: 800, ..Default::default() } })
    }

    fn options(strategy: Strategy) -> Options {
        Options { strategy, inline_schema: false, budget: None, residual: false, label: "test".into() }
    }

    fn long_package() -> Package {
        let topics = ["presentación", "objetivo", "población atendida", "requisitos de organizaciones", "monto máximo pesos proyecto", "calendario fechas cierre postulación", "evaluación criterios puntaje", "documentos anexos entrega", "firmas", "referencias"];
        let pages: Vec<String> = topics.iter().map(|t| format!("{t}\n\n{}", (0..80).map(|i| format!("{t} renglón {i} de relleno sobre este tema en particular.")).collect::<Vec<_>>().join("\n\n"))).collect();
        Package::from_documents("largo", vec![("bases.pdf".into(), pages)])
    }

    #[tokio::test]
    async fn one_shot_sends_every_page_and_the_whole_schema_in_one_call() {
        let (_d, l) = ledger();
        let p = MockProvider::new(vec![reply(blank(&SECTIONS))]);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let pkg = sample();
        let run = read_canonical(&p, &scanner, &l, None, &pkg, &options(Strategy::OneShot)).await;
        assert_eq!(p.requests().len(), 1);
        let req = &p.requests()[0];
        assert_eq!((req.task, req.max_output_tokens), (AiTask::CallCanonical, ONE_SHOT_OUTPUT_TOKENS));
        assert!(req.context[0].contains("[Página 1 | bases.pdf]") && req.context[0].contains("[Página 4 | formato.docx]"));
        assert_eq!(req.output_schema["required"].as_array().unwrap().len(), SECTIONS.len() + EXTRAS.len());
        assert_eq!((run.calls.len(), run.calls_ok()), (1, 1));
        let (doc, r) = assemble(&pkg, &run.answers, &Reading::default());
        assert!(r.sections_missing.is_empty() && r.schema_errors.is_empty(), "{r:?}");
        assert_eq!(doc["identidad"]["nombre"]["estado"], "no_aparece");
    }

    #[tokio::test]
    async fn by_block_makes_one_call_per_block_each_with_its_own_schema() {
        let (_d, l) = ledger();
        let replies = SECTIONS.iter().map(|s| reply(blank(&[s]))).collect();
        let p = MockProvider::new(replies);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let pkg = sample();
        let run = read_canonical(&p, &scanner, &l, None, &pkg, &options(Strategy::BlockAll)).await;
        let reqs = p.requests();
        assert_eq!(reqs.len(), SECTIONS.len());
        for (req, section) in reqs.iter().zip(SECTIONS) {
            let names: Vec<&str> = req.output_schema["properties"].as_object().unwrap().keys().map(String::as_str).collect();
            assert!(names.contains(&section) && names.len() == 1 + EXTRAS.len(), "{section}: {names:?}");
            assert_eq!(req.max_output_tokens, BLOCK_OUTPUT_TOKENS);
            assert!(req.user.contains(section));
            // every page was sent: no map needed
            assert_eq!(req.context.len(), 1);
        }
        let (_, r) = assemble(&pkg, &run.answers, &Reading::default());
        assert!(r.sections_missing.is_empty() && r.schema_errors.is_empty(), "{r:?}");
    }

    #[tokio::test]
    async fn by_block_with_retrieval_sends_only_the_pages_a_block_needs_and_the_map_of_the_rest() {
        for kind in [RetrieverKind::Lexical, RetrieverKind::Embedding] {
            let (_d, l) = ledger();
            let p = MockProvider::new(SECTIONS.iter().map(|s| reply(blank(&[s]))).collect());
            let scanner = RegexScanner::new(ScannerConfig::default());
            let pkg = long_package();
            let embedder: Option<Arc<dyn Embedder>> = Some(Arc::new(FakeEmbedder));
            let run = read_canonical(&p, &scanner, &l, embedder, &pkg, &options(Strategy::BlockRetrieved(kind))).await;
            let reqs = p.requests();
            assert_eq!(reqs.len(), SECTIONS.len(), "{kind:?}");
            for (req, call) in reqs.iter().zip(&run.calls) {
                assert!(call.pages_sent < call.pages_total, "{kind:?} {}: {} of {}", call.block, call.pages_sent, call.pages_total);
                assert!(req.context[1].starts_with("Mapa de todas las páginas") && req.context[1].contains("P010 | bases.pdf"), "{kind:?}");
                assert!(!req.context[0].contains("[Página 10 |") || call.pages_sent > 1);
            }
            // the financing block is sent the page about the amount (the stand-in embedder is too crude to promise it)
            let money = &run.selections.iter().find(|(s, _)| s == "financiamiento").unwrap().1;
            assert!(kind == RetrieverKind::Embedding || money.contains(&5), "{kind:?}: {money:?}");
            assert_eq!(run.embedding.is_some(), kind == RetrieverKind::Embedding);
        }
    }

    #[tokio::test]
    async fn a_block_that_fails_is_left_blank_and_listed_and_the_rest_goes_on() {
        let (_d, l) = ledger();
        let mut replies: Vec<Result<AiResponse, AiError>> = SECTIONS.iter().map(|s| reply(blank(&[s]))).collect();
        replies[2] = Err(AiError::Refused);
        let p = MockProvider::new(replies);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let pkg = sample();
        let run = read_canonical(&p, &scanner, &l, None, &pkg, &options(Strategy::BlockAll)).await;
        assert_eq!((run.calls.len(), run.calls_ok()), (SECTIONS.len(), SECTIONS.len() - 1));
        assert!(run.calls[2].error.as_deref().is_some_and(|e| e.starts_with("refused")));
        let (doc, r) = assemble(&pkg, &run.answers, &Reading::default());
        assert_eq!(r.sections_missing, vec!["elegibilidad"]);
        assert!(r.schema_errors.is_empty());
        assert_eq!(doc["elegibilidad"]["regiones"]["estado"], "no_aparece");
    }

    #[tokio::test]
    async fn a_daily_allowance_used_up_stops_the_run() {
        let (_d, l) = ledger();
        let mut replies: Vec<Result<AiResponse, AiError>> = vec![reply(blank(&["identidad"]))];
        replies.push(Err(AiError::QuotaReached));
        let p = MockProvider::new(replies);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let run = read_canonical(&p, &scanner, &l, None, &sample(), &options(Strategy::BlockAll)).await;
        assert_eq!(run.calls.len(), 2, "it did not go on asking");
    }

    #[tokio::test]
    async fn the_second_reading_goes_over_the_pages_nobody_quoted_and_adds_what_it_finds() {
        let (_d, l) = ledger();
        let mut replies: Vec<Result<AiResponse, AiError>> = Vec::new();
        for s in SECTIONS {
            let mut a = blank(&[s]);
            if s == "identidad" {
                a["identidad"]["nombre"] = json!({ "estado": "encontrado", "valor_texto": "CONVOCATORIA EJEMPLO 2027", "evidencias": [{ "cita": "CONVOCATORIA EJEMPLO 2027 Fundación Ficticia Presentación del programa.", "pagina": 1 }] });
            }
            replies.push(reply(a));
        }
        // second reading: the financing block finds the amount on page 2, in a page the first reading never quoted
        for s in SECTIONS {
            let mut a = blank(&[s]);
            if s == "financiamiento" {
                a["financiamiento"]["monto_maximo"] = json!({ "estado": "encontrado", "valor_texto": "$250,000 pesos", "evidencias": [{ "cita": "Monto máximo: $250,000 pesos por proyecto.", "pagina": 2 }] });
            }
            replies.push(reply(a));
        }
        let p = MockProvider::new(replies);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let pkg = sample();
        let opts = Options { residual: true, ..options(Strategy::BlockAll) };
        let run = read_canonical(&p, &scanner, &l, None, &pkg, &opts).await;

        // page 1 is quoted from end to end; the rest of the pages have no quote at all
        assert_eq!((run.uncited_before.clone(), run.thin_before.clone()), (Some(vec![2, 3, 4]), Some(vec![2, 3, 4])));
        let reqs = p.requests();
        assert_eq!(reqs.len(), 2 * SECTIONS.len());
        for req in &reqs[SECTIONS.len()..] {
            let ctx = &req.context[0];
            assert!(!ctx.contains("[Página 1 |") && ctx.contains("[Página 2 |") && ctx.contains("[Página 3 |") && ctx.contains("[Página 4 |"), "{ctx}");
            assert!(req.user.contains("segunda lectura"), "{}", req.user);
        }
        // the second reading is told what the first already has in that block
        assert!(reqs[SECTIONS.len()].user.contains("- nombre: ya tiene dato") && reqs[SECTIONS.len() + 3].user.contains("- monto_maximo: vacío"), "{}", reqs[SECTIONS.len()].user);
        assert!(run.calls.iter().skip(SECTIONS.len()).all(|c| c.block.ends_with("+segunda") && c.ok));

        let (doc, r) = assemble(&pkg, &run.answers, &Reading::default());
        assert!(r.schema_errors.is_empty(), "{r:?}");
        assert_eq!(doc["identidad"]["nombre"]["estado"], "encontrado");
        assert_eq!(doc["financiamiento"]["monto_maximo"]["estado"], "encontrado");
        assert_eq!(r.pages_uncited, vec![3, 4]);
        assert!(r.page_coverage() > 0.4);
    }

    #[tokio::test]
    async fn a_page_with_one_quote_out_of_a_lot_of_text_is_read_again() {
        let (_d, l) = ledger();
        let mut replies: Vec<Result<AiResponse, AiError>> = Vec::new();
        for s in SECTIONS {
            let mut a = blank(&[s]);
            if s == "identidad" {
                // one short quote from page 2, which has two paragraphs
                a["identidad"]["nombre"] = json!({ "estado": "encontrado", "valor_texto": "las organizaciones registradas", "evidencias": [{ "cita": "las organizaciones registradas", "pagina": 2 }] });
            }
            replies.push(reply(a));
        }
        for s in SECTIONS {
            replies.push(reply(blank(&[s])));
        }
        let p = MockProvider::new(replies);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let opts = Options { residual: true, ..options(Strategy::BlockAll) };
        let run = read_canonical(&p, &scanner, &l, None, &sample(), &opts).await;
        // page 2 is cited, and still thin: the second reading goes over it
        assert!(run.uncited_before.as_ref().is_some_and(|u| !u.contains(&2)));
        assert!(run.thin_before.as_ref().is_some_and(|t| t.contains(&2)), "{:?}", run.thin_before);
        assert!(p.requests()[SECTIONS.len()].context[0].contains("[Página 2 |"));
    }

    #[test]
    fn what_the_first_reading_took_is_covered_and_the_rest_of_the_page_stays_in_view() {
        let page = "Obligaciones de las beneficiarias

c) Proporcionar las evidencias
(fotografías, recibos)
y listas;
d) Presentar los CFDI que comprueben las compras;
e) Dar facilidades para las visitas.";
        // a quote copied with its line breaks as spaces still finds its place; a quote that is not on the page changes nothing
        let masked = mask(page, &["d) Presentar los CFDI que comprueben las compras;".to_string(), "una cita que no está".to_string()]);
        assert!(!masked.contains("CFDI") && masked.contains(TAKEN));
        assert!(masked.contains("evidencias") && masked.contains("visitas") && masked.starts_with("Obligaciones"));
        let both = mask(page, &["Proporcionar las evidencias (fotografías, recibos) y listas;".to_string(), "d) Presentar los CFDI que comprueben las compras;".to_string()]);
        assert_eq!(both.matches(TAKEN).count(), 1, "two taken passages with only blanks between them are one: {both}");
        assert!(both.contains("visitas") && !both.contains("evidencias"));
        assert_eq!(mask("sin nada tomado", &[]), "sin nada tomado");
    }

    #[tokio::test]
    async fn the_second_reading_gets_the_page_with_what_was_taken_covered() {
        let (_d, l) = ledger();
        let mut replies: Vec<Result<AiResponse, AiError>> = Vec::new();
        for s in SECTIONS {
            let mut a = blank(&[s]);
            if s == "elegibilidad" {
                a["elegibilidad"]["quienes_pueden_participar"] = json!({ "estado": "encontrado", "elementos": [
                    { "titulo": null, "aplica_a": null, "obligatoriedad": "obligatorio", "nota": null, "evidencias": [{ "cita": "Podrán participar las organizaciones registradas.", "pagina": 2 }] }
                ]});
            }
            replies.push(reply(a));
        }
        for s in SECTIONS {
            replies.push(reply(blank(&[s])));
        }
        let p = MockProvider::new(replies);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let opts = Options { residual: true, ..options(Strategy::BlockAll) };
        let _ = read_canonical(&p, &scanner, &l, None, &sample(), &opts).await;
        let ctx = &p.requests()[SECTIONS.len()].context[0];
        let page2 = &ctx[ctx.find("[Página 2 |").unwrap()..ctx.find("[Página 3 |").unwrap()];
        assert!(page2.contains(TAKEN) && !page2.contains("Podrán participar") && page2.contains("Monto máximo"), "{page2}");
        assert!(p.requests()[SECTIONS.len()].user.contains("tapado"));
    }

    #[tokio::test]
    async fn the_second_reading_skips_a_block_the_first_could_not_read_and_stops_when_the_allowance_ends() {
        let (_d, l) = ledger();
        let mut replies: Vec<Result<AiResponse, AiError>> = SECTIONS.iter().map(|s| reply(blank(&[s]))).collect();
        replies[0] = Err(AiError::Refused);
        replies.push(reply(blank(&["temporalidad"])));
        replies.push(Err(AiError::QuotaReached));
        let p = MockProvider::new(replies);
        let scanner = RegexScanner::new(ScannerConfig::default());
        let opts = Options { residual: true, ..options(Strategy::BlockAll) };
        let run = read_canonical(&p, &scanner, &l, None, &sample(), &opts).await;
        // identidad failed at first: it is not made up for; temporalidad answered; elegibilidad hit the allowance and the pass ended
        let blocks: Vec<&str> = run.calls.iter().skip(SECTIONS.len()).map(|c| c.block.as_str()).collect();
        assert_eq!(blocks, vec!["temporalidad+segunda", "elegibilidad+segunda"]);
    }

    struct DownEmbedder {
        fail_after: usize,
        calls: std::sync::atomic::AtomicUsize,
    }

    #[async_trait::async_trait]
    impl Embedder for DownEmbedder {
        fn label(&self) -> String {
            "down".into()
        }
        async fn embed(&self, texts: &[String], query: bool) -> Result<Vec<Vec<f32>>, String> {
            let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if n >= self.fail_after {
                return Err("embedding service answered 429: You exceeded your current quota".into());
            }
            FakeEmbedder.embed(texts, query).await
        }
    }

    #[tokio::test]
    async fn when_the_embedding_service_fails_the_blocks_are_chosen_with_the_lexical_score_and_still_read() {
        // down from the start (the index cannot be built) and down after the index (the queries fail)
        for fail_after in [0, 1] {
            let (_d, l) = ledger();
            let p = MockProvider::new(SECTIONS.iter().map(|s| reply(blank(&[s]))).collect());
            let scanner = RegexScanner::new(ScannerConfig::default());
            let pkg = long_package();
            let embedder: Option<Arc<dyn Embedder>> = Some(Arc::new(DownEmbedder { fail_after, calls: Default::default() }));
            let run = read_canonical(&p, &scanner, &l, embedder, &pkg, &options(Strategy::BlockRetrieved(RetrieverKind::Embedding))).await;
            assert_eq!(run.calls_ok(), SECTIONS.len(), "fail_after {fail_after}: {:?}", run.calls);
            assert_eq!(p.requests().len(), SECTIONS.len());
            assert!(run.calls.iter().all(|c| c.pages_sent < c.pages_total), "still chose pages, did not send everything");
            assert_eq!(run.lexical_fallback.len(), if fail_after == 0 { 1 } else { SECTIONS.len() }, "{:?}", run.lexical_fallback);
        }
    }

    #[test]
    fn the_strategies_have_stable_names() {
        for s in ["oneshot", "block_all", "block_embedding", "block_lexical"] {
            assert_eq!(Strategy::parse(s).unwrap().label(), s);
        }
        assert!(Strategy::parse("otra").is_none());
    }
}
