//! Which pages of the package a block of the schema should read.
//!
//! The model never receives 120 pages to fill one block. Each field of the block is a query made of the
//! schema's own words; the pages are ranked by meaning (embeddings) or by a plain lexical score, the
//! rankings of all the fields are fused, and the best pages are taken up to a budget, with their
//! neighbours and the first page of every document. No keyword is written by hand.

use super::package::Package;
use crate::documents::text::norm;
use crate::documents::chunking::chunk_text;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// A fragment too short to say anything is not indexed.
const MIN_CHUNK_CHARS: usize = 20;
/// Reciprocal rank fusion constant: the usual one.
const RRF_K: f32 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrieverKind {
    /// Cosine over embeddings of the fragments (a model of meaning).
    Embedding,
    /// TF-IDF over word stems: no model, no cost, the baseline the embeddings must beat.
    Lexical,
}

#[async_trait]
pub trait Embedder: Send + Sync {
    /// One vector per text. `query` says the texts are questions, not documents (some models embed them apart).
    async fn embed(&self, texts: &[String], query: bool) -> Result<Vec<Vec<f32>>, String>;
    fn label(&self) -> String;
    /// Requests and texts sent so far (for the cost report).
    fn usage(&self) -> (usize, usize) {
        (0, 0)
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub page: usize,
    pub text: String,
}

enum Index {
    Lexical { idf: HashMap<String, f32>, docs: Vec<HashMap<String, f32>> },
    Embedding { vecs: Vec<Vec<f32>>, embedder: Arc<dyn Embedder> },
}

pub struct Retrieval {
    pub chunks: Vec<Chunk>,
    index: Index,
}

/// The pages chosen for one block, and why.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageSelection {
    pub pages: Vec<usize>,
    pub chars: usize,
    pub total_chars: usize,
    /// The best pages by fused score, best first (for the report).
    pub top: Vec<(usize, f32)>,
}

/// How much text a block may read: a third of the package, between 8,000 and 40,000 characters. A short
/// package fits whole, and a long one never sends more than a bounded amount.
pub fn budget_chars(total: usize) -> usize {
    ((total as f64 * 0.35) as usize).clamp(8_000, 40_000)
}

/// Words of three or more letters or digits, cut to their first six characters: «organización»,
/// «organizaciones» and «organizar» meet. Language-light on purpose; a model of meaning does better.
fn tokens(text: &str) -> Vec<String> {
    norm(text)
        .split(' ')
        .filter(|w| w.chars().count() >= 3)
        .map(|w| fold(w).chars().take(6).collect())
        .collect()
}

fn fold(w: &str) -> String {
    w.chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

fn normalize_vec(v: &mut [f32]) {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        v.iter_mut().for_each(|x| *x /= n);
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn tf_idf(text: &str, idf: &HashMap<String, f32>) -> HashMap<String, f32> {
    let mut tf: HashMap<String, f32> = HashMap::new();
    for t in tokens(text) {
        *tf.entry(t).or_default() += 1.0;
    }
    let mut v: HashMap<String, f32> = tf
        .into_iter()
        .filter_map(|(t, c)| idf.get(&t).map(|w| (t, (1.0 + c.ln()) * w)))
        .collect();
    let n = v.values().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        v.values_mut().for_each(|x| *x /= n);
    }
    v
}

fn sparse_dot(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    let (small, big) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    small.iter().filter_map(|(t, x)| big.get(t).map(|y| x * y)).sum()
}

impl Retrieval {
    /// Indexes the package. With embeddings this is the only moment the embedding service is asked
    /// about the package's own text.
    pub async fn build(pkg: &Package, kind: RetrieverKind, embedder: Option<Arc<dyn Embedder>>) -> Result<Retrieval, String> {
        let chunks: Vec<Chunk> = pkg
            .pages
            .iter()
            .flat_map(|p| chunk_text(&p.text).into_iter().filter(|c| c.chars().count() >= MIN_CHUNK_CHARS).map(move |c| Chunk { page: p.number, text: c }))
            .collect();
        let index = match kind {
            RetrieverKind::Lexical => {
                let docs_tokens: Vec<Vec<String>> = chunks.iter().map(|c| tokens(&c.text)).collect();
                let mut df: HashMap<String, f32> = HashMap::new();
                for t in &docs_tokens {
                    let mut seen: Vec<&String> = t.iter().collect();
                    seen.sort();
                    seen.dedup();
                    for w in seen {
                        *df.entry(w.clone()).or_default() += 1.0;
                    }
                }
                let n = chunks.len() as f32;
                let idf: HashMap<String, f32> = df.into_iter().map(|(w, d)| (w, ((n + 1.0) / (d + 1.0)).ln() + 1.0)).collect();
                let docs = chunks.iter().map(|c| tf_idf(&c.text, &idf)).collect();
                Index::Lexical { idf, docs }
            }
            RetrieverKind::Embedding => {
                let embedder = embedder.ok_or("an embedding retriever needs an embedder")?;
                let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
                let mut vecs = embedder.embed(&texts, false).await?;
                if vecs.len() != chunks.len() {
                    return Err(format!("the embedder returned {} vectors for {} fragments", vecs.len(), chunks.len()));
                }
                vecs.iter_mut().for_each(|v| normalize_vec(v));
                Index::Embedding { vecs, embedder }
            }
        };
        Ok(Retrieval { chunks, index })
    }

    /// `[query][chunk]` similarities.
    async fn similarities(&self, queries: &[String]) -> Result<Vec<Vec<f32>>, String> {
        match &self.index {
            Index::Lexical { idf, docs } => Ok(queries
                .iter()
                .map(|q| {
                    let qv = tf_idf(q, idf);
                    docs.iter().map(|d| sparse_dot(&qv, d)).collect()
                })
                .collect()),
            Index::Embedding { vecs, embedder } => {
                let mut qs = embedder.embed(queries, true).await?;
                qs.iter_mut().for_each(|v| normalize_vec(v));
                Ok(qs.iter().map(|q| vecs.iter().map(|d| dot(q, d)).collect()).collect())
            }
        }
    }

    /// The pages a block should read: the best pages of every field fused, up to `budget` characters,
    /// each with its neighbours inside the same document, plus the first page of every document.
    pub async fn select(&self, pkg: &Package, queries: &[String], budget: usize) -> Result<PageSelection, String> {
        let total = pkg.total_chars();
        if total <= budget {
            return Ok(PageSelection { pages: pkg.all_numbers(), chars: total, total_chars: total, top: Vec::new() });
        }
        let sims = self.similarities(queries).await?;
        let mut fused: HashMap<usize, f32> = HashMap::new();
        for per_chunk in &sims {
            let mut best: HashMap<usize, f32> = HashMap::new();
            for (chunk, s) in self.chunks.iter().zip(per_chunk) {
                let e = best.entry(chunk.page).or_insert(f32::MIN);
                if *s > *e {
                    *e = *s;
                }
            }
            let mut ranked: Vec<(usize, f32)> = best.into_iter().filter(|(_, s)| *s > 0.0).collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            for (rank, (page, _)) in ranked.iter().enumerate() {
                *fused.entry(*page).or_default() += 1.0 / (RRF_K + rank as f32);
            }
        }
        let mut order: Vec<(usize, f32)> = fused.into_iter().collect();
        order.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));

        let len = |n: usize| pkg.page(n).map_or(0, |p| p.text.chars().count());
        let mut chosen: Vec<usize> = Vec::new();
        let mut chars = 0;
        for (page, _) in &order {
            if chars >= budget {
                break;
            }
            chosen.push(*page);
            chars += len(*page);
        }
        // a rule that spans a page break is read with the page next to it
        let cap = budget + budget * 3 / 10;
        for p in chosen.clone() {
            for n in [p.wrapping_sub(1), p + 1] {
                let same_doc = pkg.page(n).zip(pkg.page(p)).is_some_and(|(a, b)| a.document == b.document);
                if same_doc && !chosen.contains(&n) && chars + len(n) <= cap {
                    chosen.push(n);
                    chars += len(n);
                }
            }
        }
        // the first page of a document is its cover: it names the call, whatever the field asks
        let mut last_doc: Option<&str> = None;
        for p in &pkg.pages {
            if last_doc != Some(p.document.as_str()) {
                last_doc = Some(p.document.as_str());
                if !chosen.contains(&p.number) {
                    chosen.push(p.number);
                    chars += len(p.number);
                }
            }
        }
        chosen.sort_unstable();
        chosen.dedup();
        Ok(PageSelection { pages: chosen, chars, total_chars: total, top: order.into_iter().take(6).collect() })
    }
}

// ---------------------------------------------------------------- Gemini embeddings

pub struct GeminiEmbedder {
    http: reqwest::Client,
    base_url: String,
    key: String,
    pub model: String,
    dims: Option<u32>,
    /// The service does not take batches for this model: one text per request.
    single: AtomicBool,
    pub requests: AtomicUsize,
    pub texts: AtomicUsize,
}

impl GeminiEmbedder {
    pub fn new(key: String, model: &str) -> Self {
        GeminiEmbedder {
            http: reqwest::Client::builder().timeout(Duration::from_secs(60)).build().expect("http client"),
            base_url: "https://generativelanguage.googleapis.com".into(),
            key,
            model: model.trim_start_matches("models/").to_string(),
            dims: Some(768),
            single: AtomicBool::new(false),
            requests: AtomicUsize::new(0),
            texts: AtomicUsize::new(0),
        }
    }

    /// Models of the service that can embed: `(name, methods)`. Free: it generates nothing.
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub async fn embedding_models(&self) -> Result<Vec<(String, Vec<String>)>, String> {
        let mut out = Vec::new();
        let mut page_token = String::new();
        loop {
            let url = format!("{}/v1beta/models?pageSize=200{}", self.base_url, if page_token.is_empty() { String::new() } else { format!("&pageToken={page_token}") });
            let body: Value = self.http.get(url).header("x-goog-api-key", &self.key).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
            for m in body["models"].as_array().into_iter().flatten() {
                let methods: Vec<String> = m["supportedGenerationMethods"].as_array().into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).collect();
                if methods.iter().any(|x| x.contains("mbed")) {
                    out.push((m["name"].as_str().unwrap_or("").to_string(), methods));
                }
            }
            match body["nextPageToken"].as_str() {
                Some(t) if !t.is_empty() => page_token = t.to_string(),
                _ => return Ok(out),
            }
        }
    }
}

#[async_trait]
impl Embedder for GeminiEmbedder {
    fn label(&self) -> String {
        format!("gemini:{}", self.model)
    }

    fn usage(&self) -> (usize, usize) {
        (self.requests.load(Ordering::Relaxed), self.texts.load(Ordering::Relaxed))
    }

    async fn embed(&self, texts: &[String], query: bool) -> Result<Vec<Vec<f32>>, String> {
        let mut out: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
        let request_of = |t: &String| {
            let mut r = json!({
                "model": format!("models/{}", self.model),
                "content": { "parts": [{ "text": t }] },
                "taskType": if query { "RETRIEVAL_QUERY" } else { "RETRIEVAL_DOCUMENT" },
            });
            if let Some(d) = self.dims {
                r["outputDimensionality"] = json!(d);
            }
            r
        };
        let vector = |e: &Value| -> Result<Vec<f32>, String> {
            Ok(e["values"].as_array().ok_or("an embedding without values")?.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect())
        };
        // POST with the waits a real service asks for; `Ok(None)` = the method is not offered for this model
        let post = |method: &'static str, body: Value| async move {
            let mut tries = 0;
            loop {
                tries += 1;
                self.requests.fetch_add(1, Ordering::Relaxed);
                let resp = self
                    .http
                    .post(format!("{}/v1beta/models/{}:{method}", self.base_url, self.model))
                    .header("x-goog-api-key", &self.key)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                let status = resp.status().as_u16();
                let v: Value = resp.json().await.unwrap_or(Value::Null);
                match status {
                    200 => return Ok(Some(v)),
                    404 => return Ok(None),
                    429 if tries < 8 => tokio::time::sleep(Duration::from_secs(30)).await,
                    s if s >= 500 && tries < 4 => tokio::time::sleep(Duration::from_secs(10)).await,
                    s => return Err(format!("embedding service answered {s}: {}", v["error"]["message"].as_str().unwrap_or("").chars().take(300).collect::<String>())),
                }
            }
        };
        let mut rest = texts;
        while !rest.is_empty() {
            if !self.single.load(Ordering::Relaxed) {
                let batch = &rest[..rest.len().min(50)];
                let body = json!({ "requests": batch.iter().map(&request_of).collect::<Vec<_>>() });
                match post("batchEmbedContents", body).await? {
                    Some(answer) => {
                        let embeddings = answer["embeddings"].as_array().ok_or("no embeddings in the answer")?;
                        if embeddings.len() != batch.len() {
                            return Err(format!("{} embeddings for {} texts", embeddings.len(), batch.len()));
                        }
                        for e in embeddings {
                            out.push(vector(e)?);
                        }
                        self.texts.fetch_add(batch.len(), Ordering::Relaxed);
                        rest = &rest[batch.len()..];
                        continue;
                    }
                    None => self.single.store(true, Ordering::Relaxed),
                }
            }
            // one text per request, a little slower than the allowance of a free key
            let answer = post("embedContent", request_of(&rest[0])).await?.ok_or("the model cannot embed")?;
            out.push(vector(&answer["embedding"])?);
            self.texts.fetch_add(1, Ordering::Relaxed);
            rest = &rest[1..];
            tokio::time::sleep(Duration::from_millis(650)).await;
        }
        Ok(out)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Words hashed into 128 slots: a stand-in for a model of meaning that needs no network.
    pub struct FakeEmbedder;

    #[async_trait]
    impl Embedder for FakeEmbedder {
        fn label(&self) -> String {
            "fake".into()
        }
        async fn embed(&self, texts: &[String], _query: bool) -> Result<Vec<Vec<f32>>, String> {
            Ok(texts
                .iter()
                .map(|t| {
                    let mut v = vec![0.0f32; 128];
                    for w in tokens(t) {
                        let h = w.bytes().fold(7usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize));
                        v[h % 128] += 1.0;
                    }
                    v
                })
                .collect())
        }
    }

    fn filler(topic: &str, n: usize) -> String {
        (0..n).map(|i| format!("{topic} renglón {i} de relleno sobre este tema en particular.")).collect::<Vec<_>>().join("\n\n")
    }

    /// Ten pages, one topic each, in two documents.
    fn package() -> Package {
        let topics = ["presentación", "objetivo", "población atendida", "requisitos de organizaciones", "monto máximo pesos proyecto", "calendario fechas cierre postulación", "evaluación criterios puntaje", "documentos anexos entrega", "firmas", "referencias"];
        let pages: Vec<String> = topics.iter().map(|t| format!("{t}\n\n{}", filler(t, 30))).collect();
        Package::from_documents("t", vec![("bases.pdf".into(), pages[..8].to_vec()), ("anexo.pdf".into(), pages[8..].to_vec())])
    }

    #[test]
    fn the_budget_is_a_third_of_the_package_within_bounds() {
        assert_eq!(budget_chars(5_000), 8_000);
        assert_eq!(budget_chars(60_000), 21_000);
        assert_eq!(budget_chars(1_000_000), 40_000);
    }

    #[tokio::test]
    async fn the_page_that_talks_about_the_field_is_chosen_with_either_retriever() {
        let pkg = package();
        let cases = [
            ("monto máximo por proyecto en pesos", 5usize),
            ("fechas del calendario, cierre de la postulación", 6),
            ("criterios de evaluación y puntaje", 7),
        ];
        for kind in [RetrieverKind::Lexical, RetrieverKind::Embedding] {
            let r = Retrieval::build(&pkg, kind, Some(Arc::new(FakeEmbedder))).await.unwrap();
            for (query, page) in cases {
                let s = r.select(&pkg, &[query.to_string()], 3_000).await.unwrap();
                assert!(s.pages.contains(&page), "{kind:?} {query}: {:?}", s.pages);
                assert!(s.chars < s.total_chars, "a long package is not sent whole");
                assert_eq!(s.top[0].0, page, "{kind:?} {query}: {:?}", s.top);
            }
        }
    }

    #[tokio::test]
    async fn neighbours_and_the_cover_of_every_document_come_along_and_nothing_crosses_documents() {
        let pkg = package();
        let r = Retrieval::build(&pkg, RetrieverKind::Lexical, None).await.unwrap();
        let page = pkg.page(5).unwrap().text.chars().count();
        // only page 5 talks about the amount, and the budget has room for three pages
        let s = r.select(&pkg, &["monto máximo pesos proyecto".into()], page * 3).await.unwrap();
        assert!(s.pages.contains(&5) && s.pages.contains(&4) && s.pages.contains(&6), "the neighbours of the page: {:?}", s.pages);
        assert!(s.pages.contains(&1) && s.pages.contains(&9), "the first page of each document: {:?}", s.pages);
        assert!(s.pages.windows(2).all(|w| w[0] < w[1]), "in package order");
        // page 8 closes the first document: the next one is only there as a cover, never as its neighbour
        let s = r.select(&pkg, &["documentos anexos entrega".into()], page * 3).await.unwrap();
        assert!(s.pages.contains(&8) && s.pages.contains(&7) && s.pages.contains(&9), "{:?}", s.pages);
        assert!(!s.pages.contains(&10), "a page of another document is not a neighbour: {:?}", s.pages);
    }

    #[tokio::test]
    async fn a_package_that_fits_the_budget_is_sent_whole() {
        let pkg = Package::from_documents("c", vec![("a.pdf".into(), vec!["Una página corta con algo de texto.".into(), "Otra página corta con más texto.".into()])]);
        let r = Retrieval::build(&pkg, RetrieverKind::Lexical, None).await.unwrap();
        let s = r.select(&pkg, &["lo que sea".into()], budget_chars(pkg.total_chars())).await.unwrap();
        assert_eq!(s.pages, vec![1, 2]);
    }

    #[tokio::test]
    async fn embeddings_need_an_embedder() {
        assert!(Retrieval::build(&package(), RetrieverKind::Embedding, None).await.is_err());
    }
}
