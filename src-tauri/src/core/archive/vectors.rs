//! The vectors of the documents of the institution (IA5, ADR-034 §3): what lets the search find by meaning what a
//! person asks with other words (measured in `search_live`: from 50 % to 100 % of those questions among the first
//! three). They are made in the background, a batch at a time, without holding the base while the service answers.
//!
//! - **Only with Gemini and its key saved,** like the pages of a call: the text is not sent to a service the person
//!   did not choose. Without them, the search goes by words and nothing leaves the computer.
//! - **Only the clean text,** already through the scanner, of the documents of the institution; never a call.
//! - **Each request counts** in `ai_usage` (`embed.records`) and against the monthly cap; at the cap, nothing is made.
//! - **The model is written down** (`vec_model`): if it changes, the vectors are made again.

use crate::ai::pipeline::{Ledger, SqliteLedger, UsageRecord};
use crate::ai::settings::{self, ProviderKind};
use crate::ai::{AiTask, Usage};
use crate::core::archive::search::{self, Passage};
use crate::core::archive::storage::Drawer;
use crate::core::error::ServiceError;
use crate::scanner::SensitiveScanner;
use crate::documents::canonical::retrieve::{Embedder, GeminiEmbedder, EMBEDDING_DIMS, EMBEDDING_MODEL};
use crate::storage::{SharedDb, StorageError};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The kind of vectors in `vec_model`.
const SCOPE: &str = "records";
/// Fragments sent in one request.
const BATCH: usize = 50;
/// A fragment shorter than this says nothing: it gets no vector.
const MIN_CHARS: i64 = 20;

/// Who makes the vectors, if the person chose Gemini and saved its key. `quick` for a question somebody waits for.
pub fn embedder(db: &SharedDb, quick: bool) -> Option<Arc<dyn Embedder>> {
    let provider = settings::load(&*db.lock().ok()?).ok()?.provider;
    if provider != ProviderKind::Gemini {
        return None;
    }
    let key = crate::storage::get_api_key(provider.as_str()).ok().flatten()?;
    Some(if quick { Arc::new(GeminiEmbedder::quick(key, EMBEDDING_MODEL)) } else { Arc::new(GeminiEmbedder::new(key, EMBEDDING_MODEL)) })
}

/// The model of an embedder, as the price table names it (`gemini:gemini-embedding-2` → `gemini-embedding-2`).
fn model_of(e: &dyn Embedder) -> String {
    let label = e.label();
    label.split_once(':').map_or(label.clone(), |x| x.1.to_string())
}

/// Writes down the model of the vectors; if the stored ones were made with another, they go. Says if they went.
pub fn ensure_model(conn: &Connection, label: &str) -> Result<bool, StorageError> {
    let stored: Option<String> = conn.query_row("SELECT model FROM vec_model WHERE scope = ?1", [SCOPE], |r| r.get(0)).optional()?;
    if stored.as_deref() == Some(label) {
        return Ok(false);
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM document_chunk_vec", [])?;
    tx.execute(
        "INSERT INTO vec_model (scope, model, dims, set_at) VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         ON CONFLICT(scope) DO UPDATE SET model = excluded.model, dims = excluded.dims, set_at = excluded.set_at",
        params![SCOPE, label, EMBEDDING_DIMS as i64],
    )?;
    tx.commit()?;
    Ok(stored.is_some())
}

/// Fragments that still have no vector, oldest first: only of the drawers the AI may read (ADR-036 §3); today the
/// Expediente. Never Personal nor Beneficiarios.
pub fn pending(conn: &Connection, limit: usize) -> Result<Vec<(i64, String)>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT c.rowid, c.text FROM document_chunk c JOIN document d ON d.id = c.document_id
         WHERE d.drawer IN ('record') AND d.hidden = 0 AND length(trim(c.text)) >= ?1
           AND c.rowid NOT IN (SELECT rowid FROM document_chunk_vec)
         ORDER BY c.rowid LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![MIN_CHARS, limit as i64], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// A vector as `sqlite-vec` reads it.
pub fn vec_json(v: &[f32]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "[]".into())
}

/// Keeps the vector of a fragment, if the fragment is still there (its document may have been deleted meanwhile)
/// and the vector has the size of the table. Says if it was kept.
pub fn save(conn: &Connection, chunk: i64, v: &[f32]) -> Result<bool, StorageError> {
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM document_chunk WHERE rowid = ?1)", [chunk], |r| r.get(0))?;
    if !exists || v.len() != EMBEDDING_DIMS {
        return Ok(false);
    }
    conn.execute("INSERT OR REPLACE INTO document_chunk_vec (rowid, embedding) VALUES (?1, ?2)", params![chunk, vec_json(v)])?;
    Ok(true)
}

/// Are there vectors to look in?
pub fn any(conn: &Connection) -> Result<bool, StorageError> {
    Ok(conn.query_row("SELECT EXISTS(SELECT 1 FROM document_chunk_vec)", [], |r| r.get(0))?)
}

/// Writes one request to the service in `ai_usage`, as every call to the AI: the text it took (estimated, four
/// letters a token; the service does not say) and what it cost.
pub fn record_usage(db: &SharedDb, e: &dyn Embedder, chars: usize, ok: bool, latency_ms: u64) {
    let ledger = SqliteLedger(db.clone());
    let Ok(s) = ledger.settings() else { return };
    let model = model_of(e);
    let input_tokens = (chars as u64).div_ceil(4);
    let cost_mxn = if ok { s.cost_mxn(&model, &Usage { input_tokens, ..Default::default() }) } else { 0.0 };
    let _ = ledger.record_usage(&UsageRecord {
        task: AiTask::Embed,
        provider: "gemini".into(),
        model,
        input_tokens: if ok { input_tokens } else { 0 },
        output_tokens: 0,
        cached_tokens: 0,
        thought_tokens: 0,
        cost_mxn,
        project_id: None,
        success: ok,
        latency_ms: Some(latency_ms),
        error_kind: (!ok).then(|| "embed".to_string()),
    });
}

fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("lock".into()))
}

/// The monthly cap is reached: no more requests this month.
fn at_cap(db: &SharedDb) -> Result<bool, ServiceError> {
    let conn = lock(db)?;
    let s = settings::load(&conn)?;
    Ok(settings::month_spend_mxn(&conn)? >= s.monthly_cap_mxn)
}

/// Makes the vectors that are missing, a batch at a time; the base is free while the service answers. Stops at the
/// first failure (no internet, a busy service): what is missing is made the next time. Says how many it made.
pub async fn index_pending(db: &SharedDb, e: &dyn Embedder) -> Result<usize, ServiceError> {
    ensure_model(&*lock(db)?, &e.label())?;
    let mut made = 0;
    loop {
        if at_cap(db)? {
            return Ok(made);
        }
        let batch = pending(&*lock(db)?, BATCH)?;
        if batch.is_empty() {
            return Ok(made);
        }
        let texts: Vec<String> = batch.iter().map(|b| b.1.clone()).collect();
        let chars: usize = texts.iter().map(|t| t.chars().count()).sum();
        let started = std::time::Instant::now();
        let answer = e.embed(&texts, false).await;
        let ms = started.elapsed().as_millis() as u64;
        let vecs = match answer {
            Ok(v) if v.len() == batch.len() => v,
            _ => {
                record_usage(db, e, chars, false, ms);
                return Ok(made);
            }
        };
        record_usage(db, e, chars, true, ms);
        let conn = lock(db)?;
        let tx = conn.unchecked_transaction()?;
        let mut kept = 0;
        for ((chunk, _), v) in batch.iter().zip(&vecs) {
            kept += usize::from(save(&tx, *chunk, v)?);
        }
        tx.commit()?;
        made += kept;
        if kept == 0 {
            // nothing of the batch could be kept (a vector of another size): do not ask for it again and again
            return Ok(made);
        }
    }
}

/// One indexing at a time in the whole app.
static INDEXING: AtomicBool = AtomicBool::new(false);

/// Makes the missing vectors if the person chose Gemini and saved its key; otherwise does nothing. For the
/// background, after a document is saved and when the app opens.
pub async fn index_in_background(db: SharedDb) {
    let Some(e) = embedder(&db, false) else { return };
    if INDEXING.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = index_pending(&db, e.as_ref()).await;
    INDEXING.store(false, Ordering::SeqCst);
}

// ------------------------------------------------------------------ the search by meaning (it asks a model)

/// Reciprocal rank fusion constant: the usual one.
const RRF_K: f32 = 60.0;
/// How much the ranking by meaning weighs against the one by words. Measured with the reference questions of
/// `fixtures/expediente` (`search_live`, 2026-10-12): 1:3 put the expected page among the first three in 100 % of
/// the questions, with the words of the document or with others; words alone, 94 % and 50 %; 1:1, 100 % and 93 %.
const MEANING_WEIGHT: f32 = 3.0;
/// Fragments each ranking brings before they are fused.
const CANDIDATES: usize = 20;
/// The places of the ranking by meaning that count without a word of the question (measured in `search_live`: the
/// expected page of a question with other words was never below the third place by meaning).
const MEANING_ONLY: usize = 3;

/// The fragments of one drawer closest in meaning to a question (its vector), best first.
pub fn by_meaning(conn: &Connection, drawer: Drawer, query: &[f32], limit: usize) -> Result<Vec<i64>, StorageError> {
    meaning_in(conn, drawer, None, query, limit)
}

/// The nearest fragments are looked for before they are filtered: in one document, among this many.
const NEAREST_IN_ONE: usize = 400;

fn meaning_in(conn: &Connection, drawer: Drawer, document: Option<&str>, query: &[f32], limit: usize) -> Result<Vec<i64>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT v.rowid FROM (SELECT rowid, distance FROM document_chunk_vec WHERE embedding MATCH ?1 AND k = ?2) v
           JOIN document_chunk c ON c.rowid = v.rowid
           JOIN document d ON d.id = c.document_id
         WHERE d.drawer = ?3 AND d.hidden = 0 AND (?4 IS NULL OR d.id = ?4)
         ORDER BY v.distance LIMIT ?5",
    )?;
    let k = if document.is_some() { NEAREST_IN_ONE.max(limit) } else { limit };
    let ids = stmt.query_map(params![vec_json(query), k as i64, drawer.as_str(), document, limit as i64], |r| r.get(0))?.collect::<Result<Vec<i64>, _>>()?;
    Ok(ids)
}

/// The two rankings fused, the meaning weighing `MEANING_WEIGHT`. What only the meaning found comes only from its first
/// `MEANING_ONLY` places: a vector always has a nearest fragment, even when nothing is about the question.
pub fn fuse_words_and_meaning<K: Clone + Eq + Hash>(words: &[K], meaning: &[K], limit: usize) -> Vec<K> {
    let fused = fuse_weighted(&[(words, 1.0), (meaning, MEANING_WEIGHT)], usize::MAX);
    let close = &meaning[..meaning.len().min(MEANING_ONLY)];
    fused.into_iter().filter(|k| words.contains(k) || close.contains(k)).take(limit).collect()
}

/// Rankings fused by position (reciprocal rank fusion): what is high in both comes first, and a ranking that knows
/// nothing of the question (empty) changes nothing.
pub fn fuse<K: Clone + Eq + Hash>(rankings: &[Vec<K>], limit: usize) -> Vec<K> {
    let weighted: Vec<(&[K], f32)> = rankings.iter().map(|r| (r.as_slice(), 1.0)).collect();
    fuse_weighted(&weighted, limit)
}

/// The same, with a weight for each ranking.
pub fn fuse_weighted<K: Clone + Eq + Hash>(rankings: &[(&[K], f32)], limit: usize) -> Vec<K> {
    let mut score: HashMap<K, f32> = HashMap::new();
    let mut first_seen: Vec<K> = Vec::new();
    for (ranking, weight) in rankings {
        for (rank, k) in ranking.iter().enumerate() {
            let s = score.entry(k.clone()).or_insert_with(|| {
                first_seen.push(k.clone());
                0.0
            });
            *s += weight / (RRF_K + rank as f32);
        }
    }
    // ties keep the order in which they first appeared (the words first)
    let mut order: Vec<(usize, K)> = first_seen.into_iter().enumerate().collect();
    order.sort_by(|(ia, a), (ib, b)| score[b].total_cmp(&score[a]).then(ia.cmp(ib)));
    order.into_iter().take(limit).map(|(_, k)| k).collect()
}

/// The passages of one drawer that best answer a question: by words, and by meaning when its vector came; fused by
/// position.
pub fn search_in(conn: &Connection, drawer: Drawer, document: Option<&str>, query: &str, vector: Option<&[f32]>, limit: usize, chars: usize) -> Result<Vec<Passage>, StorageError> {
    let words = search::words_in(conn, drawer, document, query, CANDIDATES)?;
    let meaning = match vector {
        Some(v) => meaning_in(conn, drawer, document, v, CANDIDATES)?,
        None => Vec::new(),
    };
    search::passages(conn, &fuse_words_and_meaning(&words, &meaning, limit), query, chars)
}

/// What a search found, and whether it also looked by meaning (with internet, Gemini and the vectors made).
#[derive(Debug, Clone, serde::Serialize)]
pub struct MeaningFound {
    pub passages: Vec<Passage>,
    pub by_meaning: bool,
}

/// Looks for a question in the documents of one drawer, in one document when `document` says which. Its vector is
/// asked for only when there are vectors to compare it with and the question carries no data of a person (the scanner
/// says so); otherwise, or if the service does not answer in a few seconds, the search goes by words alone. Nothing of
/// it is kept.
pub async fn search_documents_in(
    db: &SharedDb,
    embedder: Option<Arc<dyn Embedder>>,
    drawer: Drawer,
    document: Option<&str>,
    query: &str,
    limit: usize,
    chars: usize,
) -> Result<MeaningFound, ServiceError> {
    let query = search::checked_query(query)?;
    let may_send = {
        let conn = db.lock().map_err(|_| ServiceError::Internal("lock".into()))?;
        any(&conn)? && crate::core::screen::scanner_for(&conn)?.scan(query).is_clean()
    };
    let mut vector = None;
    if let Some(e) = embedder.filter(|_| may_send) {
        let started = std::time::Instant::now();
        let answer = e.embed(&[query.to_string()], true).await;
        let ok = matches!(&answer, Ok(v) if v.len() == 1);
        record_usage(db, e.as_ref(), query.chars().count(), ok, started.elapsed().as_millis() as u64);
        vector = answer.ok().filter(|_| ok).and_then(|mut v| v.pop());
    }
    let conn = db.lock().map_err(|_| ServiceError::Internal("lock".into()))?;
    let passages = search_in(&conn, drawer, document, query, vector.as_deref(), limit, chars)?;
    Ok(MeaningFound { passages, by_meaning: vector.is_some() })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::core::archive::storage::{add_paged_document, emergency_delete_document, Drawer, PageMarks};
    use async_trait::async_trait;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    /// Words hashed into the slots of a vector of the real size: a stand-in for a model of meaning, with no network.
    /// It counts what it was asked, and can be made to fail.
    pub struct WordsEmbedder {
        pub label: &'static str,
        pub texts: AtomicUsize,
        pub fail: bool,
    }

    impl WordsEmbedder {
        pub fn new() -> Self {
            WordsEmbedder { label: "fake:words-1", texts: AtomicUsize::new(0), fail: false }
        }
    }

    pub fn words_vector(t: &str) -> Vec<f32> {
        let mut v = vec![0.0f32; EMBEDDING_DIMS];
        for w in crate::storage::fts::words(t) {
            let w = crate::storage::fts::fold(&w);
            let w: String = w.chars().take(6).collect();
            let h = w.bytes().fold(7usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize));
            v[h % EMBEDDING_DIMS] += 1.0;
        }
        v
    }

    #[async_trait]
    impl Embedder for WordsEmbedder {
        fn label(&self) -> String {
            self.label.into()
        }
        async fn embed(&self, texts: &[String], _query: bool) -> Result<Vec<Vec<f32>>, String> {
            if self.fail {
                return Err("offline".into());
            }
            self.texts.fetch_add(texts.len(), Ordering::Relaxed);
            Ok(texts.iter().map(|t| words_vector(t)).collect())
        }
    }

    fn db() -> (tempfile::TempDir, SharedDb) {
        let dir = tempfile::tempdir().unwrap();
        let c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, Arc::new(Mutex::new(c)))
    }

    fn q(db: &SharedDb, sql: &str) -> i64 {
        db.lock().unwrap().query_row(sql, [], |r| r.get(0)).unwrap()
    }

    /// Only the fragments of the documents of the institution get a vector, each request is written in `ai_usage`, and
    /// a second run asks for nothing.
    #[tokio::test]
    async fn the_vectors_of_the_institution_are_made_once_and_every_request_counts() {
        let (_d, db) = db();
        {
            let c = db.lock().unwrap();
            add_paged_document(&c, Drawer::Record, "Reglamento", "application/pdf", &["Las visitas son de martes a domingo.".into(), "Los medicamentos los da enfermería.".into(), "Corto".into()], 0, &PageMarks::default()).unwrap();
            add_paged_document(&c, Drawer::Projects, "Convocatoria", "application/pdf", &["El monto máximo es de cien mil pesos.".into()], 0, &PageMarks::default()).unwrap();
        }
        let e = WordsEmbedder::new();
        assert_eq!(index_pending(&db, &e).await.unwrap(), 2, "two pages; the short one and the call get none");
        assert_eq!(q(&db, "SELECT count(*) FROM document_chunk_vec"), 2);
        assert_eq!(q(&db, "SELECT count(*) FROM ai_usage WHERE task = 'embed.records' AND success = 1 AND input_tokens > 0"), 1);
        assert_eq!(index_pending(&db, &e).await.unwrap(), 0);
        assert_eq!(e.texts.load(Ordering::Relaxed), 2, "nothing asked twice");
        let model: String = db.lock().unwrap().query_row("SELECT model FROM vec_model WHERE scope = 'records'", [], |r| r.get(0)).unwrap();
        assert_eq!(model, "fake:words-1");
    }

    /// With another model the old vectors go and are made again; without internet nothing is lost and the next run
    /// goes on; at the monthly cap nothing is asked.
    #[tokio::test]
    async fn a_new_model_makes_them_again_and_a_failure_or_the_cap_ask_nothing_more() {
        let (_d, db) = db();
        add_paged_document(&db.lock().unwrap(), Drawer::Record, "Acta", "application/pdf", &["El patronato sesiona cada tres meses.".into()], 0, &PageMarks::default()).unwrap();
        let offline = WordsEmbedder { fail: true, ..WordsEmbedder::new() };
        assert_eq!(index_pending(&db, &offline).await.unwrap(), 0);
        assert_eq!(q(&db, "SELECT count(*) FROM ai_usage WHERE task = 'embed.records' AND success = 0"), 1, "the failure is written down too");
        assert_eq!(index_pending(&db, &WordsEmbedder::new()).await.unwrap(), 1);
        let other = WordsEmbedder { label: "fake:words-2", ..WordsEmbedder::new() };
        assert_eq!(index_pending(&db, &other).await.unwrap(), 1, "made again with the new model");
        assert_eq!(q(&db, "SELECT count(*) FROM document_chunk_vec"), 1);

        add_paged_document(&db.lock().unwrap(), Drawer::Record, "Informe", "application/pdf", &["Atendimos a cuarenta y ocho personas.".into()], 0, &PageMarks::default()).unwrap();
        let capped = {
            let c = db.lock().unwrap();
            let mut s = settings::load(&c).unwrap();
            s.monthly_cap_mxn = 0.0;
            settings::save(&c, &s).unwrap();
            WordsEmbedder { label: "fake:words-2", ..WordsEmbedder::new() }
        };
        assert_eq!(index_pending(&db, &capped).await.unwrap(), 0);
        assert_eq!(capped.texts.load(Ordering::Relaxed), 0, "at the cap nothing is sent");
    }

    /// A backup carries the vectors, and the restored base searches by meaning as before (ADR-019).
    #[tokio::test]
    async fn a_backup_carries_the_vectors() {
        let (dir, db) = db();
        add_paged_document(&db.lock().unwrap(), Drawer::Record, "Acta", "application/pdf", &["El patronato sesiona cada tres meses.".into()], 0, &PageMarks::default()).unwrap();
        index_pending(&db, &WordsEmbedder::new()).await.unwrap();
        let copy = dir.path().join("respaldo.cim");
        crate::storage::backup::create(&db.lock().unwrap(), &copy, "una clave larga").unwrap();
        let restored = dir.path().join("restaurada.db");
        crate::storage::backup::restore_to(&copy, "una clave larga", &restored, KEY).unwrap();
        let r = crate::storage::open_encrypted(&restored, KEY).unwrap();
        assert!(any(&r).unwrap());
        let found = by_meaning(&r, Drawer::Record, &words_vector("¿Cada cuánto sesiona el patronato?"), 3).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(pending(&r, 10).unwrap().len(), 0, "nothing to make again");
    }

    /// «Delete this and everything that came from it» takes its vectors too.
    #[tokio::test]
    async fn deleting_a_document_takes_its_vectors() {
        let (_d, db) = db();
        let id = add_paged_document(&db.lock().unwrap(), Drawer::Record, "Poder", "application/pdf", &["Poder general para actos de administración.".into()], 0, &PageMarks::default()).unwrap();
        index_pending(&db, &WordsEmbedder::new()).await.unwrap();
        assert_eq!(q(&db, "SELECT count(*) FROM document_chunk_vec"), 1);
        emergency_delete_document(&mut db.lock().unwrap(), &id).unwrap();
        assert_eq!(q(&db, "SELECT count(*) FROM document_chunk_vec"), 0);
        assert!(!save(&db.lock().unwrap(), 999, &vec![0.0; EMBEDDING_DIMS]).unwrap(), "a fragment that is gone gets no vector");
    }

    // ---------------------------------------------------------------- the search by meaning (moved from `search`)

    use crate::core::archive::search::tests::{load_record, reference, Score};
    use crate::core::archive::search::{by_words, MAX_QUERY_CHARS, SNIPPET_CHARS};
    use serde::Deserialize;
    use std::path::PathBuf;

    fn conn() -> (tempfile::TempDir, Connection) {
        crate::core::archive::search::tests::conn()
    }

    fn cosine_ranking<K: Clone>(query: &[f32], items: &[(K, Vec<f32>)], limit: usize) -> Vec<K> {
        let norm = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>().sqrt().max(f32::EPSILON);
        let qn = norm(query);
        let mut scored: Vec<(f32, &K)> = items.iter().map(|(k, v)| (query.iter().zip(v).map(|(a, b)| a * b).sum::<f32>() / (qn * norm(v)), k)).collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        scored.into_iter().take(limit).map(|(_, k)| k.clone()).collect()
    }

    /// The ways of putting the two rankings together that are compared: (name, weight of words, weight of meaning); a
    /// negative weight is the one of the app (`fuse_words_and_meaning`).
    const METHODS: &[(&str, f32, f32)] = &[("palabras", 1.0, 0.0), ("sentido", 0.0, 1.0), ("ambas 1:1", 1.0, 1.0), ("ambas 1:3", 1.0, 3.0), ("la del programa", -1.0, -1.0)];

    fn combine<K: Clone + Eq + Hash>(words: &[K], meaning: &[K], w: f32, m: f32) -> Vec<K> {
        if w < 0.0 {
            return fuse_words_and_meaning(words, meaning, 10);
        }
        let parts: Vec<(&[K], f32)> = [(words, w), (meaning, m)].into_iter().filter(|(_, x)| *x > 0.0).collect();
        fuse_weighted(&parts, 10)
    }
    /// The vectors of a measurement, kept in a file so other ways of fusing can be compared without asking again.
    #[derive(serde::Serialize, Deserialize)]
    struct Vectors {
        model: String,
        chunks: Vec<(i64, Vec<f32>)>,
        manual: Vec<(String, Vec<f32>)>,
        questions: Vec<Vec<f32>>,
    }

    /// The measurement of IA5 (ADR-034): the reference questions of the fictitious record and of the manual, by
    /// words alone, by meaning alone (vectors of the embedding model) and both fused with several weights. Only
    /// fictitious text goes out: the fragments of `fixtures/expediente`, the manual and the questions (about 170 texts,
    /// a handful of requests). With `SOCIAI_VEC_CACHE` the vectors are kept in that file and read from it the next
    /// time, with no request.
    ///   cargo test search_live -- --ignored --nocapture > salida.txt
    #[tokio::test]
    #[ignore]
    async fn search_live() {
        use crate::documents::canonical::retrieve::{Embedder, GeminiEmbedder, EMBEDDING_MODEL};
        let (_d, c) = conn();
        let of = load_record(&c);
        let reference = reference();
        let mut stmt = c.prepare("SELECT rowid, text FROM document_chunk ORDER BY rowid").unwrap();
        let chunks: Vec<(i64, String)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().collect::<Result<_, _>>().unwrap();
        drop(stmt);
        let manual: Vec<(String, String)> = crate::core::manual::MANUAL.iter().map(|m| (m.id.clone(), format!("{}\n{}", m.title, m.paragraphs.join("\n")))).collect();
        let questions: Vec<String> = reference.expediente.iter().map(|q| q.q.clone()).chain(reference.manual.iter().map(|q| q.q.clone())).collect();

        let cache = std::env::var("SOCIAI_VEC_CACHE").ok().map(PathBuf::from);
        let v: Vectors = match cache.as_ref().filter(|p| p.exists()) {
            Some(p) => {
                println!("Vectores leídos de {} (sin solicitudes)", p.display());
                serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
            }
            None => {
                let key = crate::storage::get_api_key("gemini").unwrap().expect("save the Gemini key in the app first");
                let e = GeminiEmbedder::new(key, EMBEDDING_MODEL);
                let started = std::time::Instant::now();
                let cv = e.embed(&chunks.iter().map(|x| x.1.clone()).collect::<Vec<_>>(), false).await.unwrap();
                let mv = e.embed(&manual.iter().map(|x| x.1.clone()).collect::<Vec<_>>(), false).await.unwrap();
                let index_secs = started.elapsed().as_secs_f64();
                let q_started = std::time::Instant::now();
                let qv = e.embed(&questions, true).await.unwrap();
                let q_secs = q_started.elapsed().as_secs_f64();
                let one = std::time::Instant::now();
                e.embed(&["¿Qué hacemos si tiembla?".to_string()], true).await.unwrap();
                println!("Calcular los vectores: {index_secs:.1} s  preguntas: {q_secs:.1} s  una sola pregunta: {:.2} s", one.elapsed().as_secs_f64());
                let (requests, texts) = e.usage();
                println!("Solicitudes al servicio: {requests}  textos: {texts}");
                let v = Vectors { model: e.label(), chunks: chunks.iter().map(|x| x.0).zip(cv).collect(), manual: manual.iter().map(|x| x.0.clone()).zip(mv).collect(), questions: qv };
                if let Some(p) = &cache {
                    std::fs::write(p, serde_json::to_string(&v).unwrap()).unwrap();
                }
                v
            }
        };
        println!("Modelo: {}  fragmentos: {}  entradas del manual: {}  preguntas: {}", v.model, v.chunks.len(), v.manual.len(), questions.len());
        let header = METHODS.iter().map(|m| m.0).collect::<Vec<_>>().join(" / ");

        println!("\n## Expediente: posición de la página esperada ({header})");
        let mut scores: HashMap<(&str, &str), Score> = HashMap::new();
        let mut shown: HashMap<&str, usize> = HashMap::new();
        for (i, q) in reference.expediente.iter().enumerate() {
            let words = by_words(&c, Drawer::Record, &q.q, 10).unwrap();
            let meaning = cosine_ranking(&v.questions[i], &v.chunks, 10);
            let mut cells = Vec::new();
            for &(label, w, m) in METHODS {
                let named: Vec<(String, i64)> = combine(&words, &meaning, w, m).iter().map(|r| of[r].clone()).collect();
                *shown.entry(label).or_default() += named.len();
                let pos = scores.entry((label, if q.tipo == "palabras" { "palabras" } else { "sentido" })).or_default().add(&named, &q.esperado);
                cells.push(pos.map_or("—".to_string(), |p| format!("{}.º", p + 1)));
            }
            println!("{:<8} {:<75} {}", q.tipo, q.q, cells.join(" / "));
        }
        for &(label, _, _) in METHODS {
            println!("Pasajes por pregunta, {label}: {:.1}", shown[label] as f32 / reference.expediente.len() as f32);
        }
        println!("\n## Manual: posición de la entrada esperada ({header})");
        let mut manual_scores: HashMap<(&str, &str), Score> = HashMap::new();
        for (i, q) in reference.manual.iter().enumerate() {
            let words: Vec<String> = crate::core::manual::search(&c, &q.q, 10).unwrap().into_iter().map(|h| h.id).collect();
            let meaning = cosine_ranking(&v.questions[reference.expediente.len() + i], &v.manual, 10);
            let mut cells = Vec::new();
            for &(label, w, m) in METHODS {
                let pos = manual_scores.entry((label, if q.tipo == "palabras" { "palabras" } else { "sentido" })).or_default().add(&combine(&words, &meaning, w, m), &q.esperado);
                cells.push(pos.map_or("—".to_string(), |p| format!("{}.º", p + 1)));
            }
            println!("{:<8} {:<75} {}", q.tipo, q.q, cells.join(" / "));
        }
        for (title, scores) in [("Expediente", &scores), ("Manual", &manual_scores)] {
            println!("\n## {title}: resumen");
            for kind in ["palabras", "sentido"] {
                for &(method, _, _) in METHODS {
                    let s = scores.get(&(method, kind)).copied().unwrap_or_default();
                    println!("{}", s.line(&format!("«{kind}», {method}")));
                }
            }
        }
    }
    /// With vectors the search also goes by meaning; a question with data of a person, or without the service, goes
    /// by words alone and nothing of it leaves.
    #[tokio::test]
    async fn the_question_goes_out_only_clean_and_without_the_service_the_words_answer() {
        use crate::core::archive::vectors::{index_pending, tests::WordsEmbedder};
        use std::sync::atomic::Ordering;
        let dir = tempfile::tempdir().unwrap();
        let db: SharedDb = Arc::new(std::sync::Mutex::new(crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap()));
        load_record(&db.lock().unwrap());
        let e = Arc::new(WordsEmbedder::new());
        // before the vectors are made, the question is not sent
        let found = search_documents_in(&db, Some(e.clone()), Drawer::Record, None, "¿Cuántos extintores tenemos?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(!found.by_meaning && found.passages[0].document == "programa-proteccion-civil.txt");
        assert_eq!(e.texts.load(Ordering::Relaxed), 0);

        index_pending(&db, e.as_ref()).await.unwrap();
        let made = e.texts.load(Ordering::Relaxed);
        let found = search_documents_in(&db, Some(e.clone()), Drawer::Record, None, "¿Cuántos extintores tenemos?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(found.by_meaning);
        assert_eq!((found.passages[0].document.as_str(), found.passages[0].page), ("programa-proteccion-civil.txt", Some(1)));
        assert!(found.passages[0].snippet.contains("extintores"), "{:?}", found.passages[0]);
        assert_eq!(e.texts.load(Ordering::Relaxed), made + 1, "one question, one text");

        let found = search_documents_in(&db, Some(e.clone()), Drawer::Record, None, "¿Dónde está la CURP LOPM800101MDFRZN09 del poder?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(!found.by_meaning && !found.passages.is_empty(), "a datum of a person never leaves; the words still answer");
        assert_eq!(e.texts.load(Ordering::Relaxed), made + 1);

        let offline = Arc::new(WordsEmbedder { fail: true, ..WordsEmbedder::new() });
        let found = search_documents_in(&db, Some(offline), Drawer::Record, None, "¿Quién paga el predial?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(!found.by_meaning && found.passages[0].document == "contrato-comodato.txt");
        assert!(matches!(search_documents_in(&db, None, Drawer::Record, None, "  ", 5, SNIPPET_CHARS).await, Err(ServiceError::EmptyText)));
        assert!(matches!(search_documents_in(&db, None, Drawer::Record, None, &"x".repeat(MAX_QUERY_CHARS + 1), 5, SNIPPET_CHARS).await, Err(ServiceError::TextTooLarge)));
        let usage: i64 = db.lock().unwrap().query_row("SELECT count(*) FROM ai_usage WHERE task = 'embed.records'", [], |r| r.get(0)).unwrap();
        assert_eq!(usage, 3, "the record, one question and the failed one");
    }

    #[test]
    fn fused_rankings_put_first_what_both_have_high() {
        let words = vec![1, 2, 3];
        let meaning = vec![3, 4, 1];
        assert_eq!(fuse(&[words.clone(), meaning], 3), vec![1, 3, 2]);
        assert_eq!(fuse(&[words.clone(), vec![]], 5), words, "an empty ranking changes nothing");
        assert!(fuse::<i64>(&[], 5).is_empty());
    }
}
