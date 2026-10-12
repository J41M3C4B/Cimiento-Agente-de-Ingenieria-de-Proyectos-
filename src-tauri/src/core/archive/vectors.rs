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
use crate::core::error::ServiceError;
use crate::documents::canonical::retrieve::{Embedder, GeminiEmbedder, EMBEDDING_DIMS, EMBEDDING_MODEL};
use crate::storage::{SharedDb, StorageError};
use rusqlite::{params, Connection, OptionalExtension};
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

/// Fragments of the documents of the institution that still have no vector, oldest first.
pub fn pending(conn: &Connection, limit: usize) -> Result<Vec<(i64, String)>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT c.rowid, c.text FROM document_chunk c JOIN document d ON d.id = c.document_id
         WHERE d.kind <> 'call' AND d.hidden = 0 AND length(trim(c.text)) >= ?1
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::core::archive::storage::{add_paged_document, emergency_delete_document, PageMarks};
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
            add_paged_document(&c, "internal", "Reglamento", "application/pdf", &["Las visitas son de martes a domingo.".into(), "Los medicamentos los da enfermería.".into(), "Corto".into()], 0, &PageMarks::default()).unwrap();
            add_paged_document(&c, "call", "Convocatoria", "application/pdf", &["El monto máximo es de cien mil pesos.".into()], 0, &PageMarks::default()).unwrap();
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
        add_paged_document(&db.lock().unwrap(), "internal", "Acta", "application/pdf", &["El patronato sesiona cada tres meses.".into()], 0, &PageMarks::default()).unwrap();
        let offline = WordsEmbedder { fail: true, ..WordsEmbedder::new() };
        assert_eq!(index_pending(&db, &offline).await.unwrap(), 0);
        assert_eq!(q(&db, "SELECT count(*) FROM ai_usage WHERE task = 'embed.records' AND success = 0"), 1, "the failure is written down too");
        assert_eq!(index_pending(&db, &WordsEmbedder::new()).await.unwrap(), 1);
        let other = WordsEmbedder { label: "fake:words-2", ..WordsEmbedder::new() };
        assert_eq!(index_pending(&db, &other).await.unwrap(), 1, "made again with the new model");
        assert_eq!(q(&db, "SELECT count(*) FROM document_chunk_vec"), 1);

        add_paged_document(&db.lock().unwrap(), "internal", "Informe", "application/pdf", &["Atendimos a cuarenta y ocho personas.".into()], 0, &PageMarks::default()).unwrap();
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
        add_paged_document(&db.lock().unwrap(), "internal", "Acta", "application/pdf", &["El patronato sesiona cada tres meses.".into()], 0, &PageMarks::default()).unwrap();
        index_pending(&db, &WordsEmbedder::new()).await.unwrap();
        let copy = dir.path().join("respaldo.cim");
        crate::storage::backup::create(&db.lock().unwrap(), &copy, "una clave larga").unwrap();
        let restored = dir.path().join("restaurada.db");
        crate::storage::backup::restore_to(&copy, "una clave larga", &restored, KEY).unwrap();
        let r = crate::storage::open_encrypted(&restored, KEY).unwrap();
        assert!(any(&r).unwrap());
        let found = crate::core::archive::search::by_meaning(&r, &words_vector("¿Cada cuánto sesiona el patronato?"), 3).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(pending(&r, 10).unwrap().len(), 0, "nothing to make again");
    }

    /// «Delete this and everything that came from it» takes its vectors too.
    #[tokio::test]
    async fn deleting_a_document_takes_its_vectors() {
        let (_d, db) = db();
        let id = add_paged_document(&db.lock().unwrap(), "internal", "Poder", "application/pdf", &["Poder general para actos de administración.".into()], 0, &PageMarks::default()).unwrap();
        index_pending(&db, &WordsEmbedder::new()).await.unwrap();
        assert_eq!(q(&db, "SELECT count(*) FROM document_chunk_vec"), 1);
        emergency_delete_document(&mut db.lock().unwrap(), &id).unwrap();
        assert_eq!(q(&db, "SELECT count(*) FROM document_chunk_vec"), 0);
        assert!(!save(&db.lock().unwrap(), 999, &vec![0.0; EMBEDDING_DIMS]).unwrap(), "a fragment that is gone gets no vector");
    }
}
