//! Search in the documents of the institution (IA5, ADR-034 §3): by words with FTS5, always and without internet, and
//! by meaning when there are vectors; the two rankings fused by position. Each passage carries its document and page,
//! taken by the code from the fragment it came from: a citation nobody has to trust.

use crate::core::archive::storage::Drawer;
use crate::core::archive::vectors;
use crate::core::error::ServiceError;
use crate::documents::canonical::retrieve::Embedder;
use crate::scanner::SensitiveScanner;
use crate::storage::fts;
use crate::storage::{SharedDb, StorageError};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Reciprocal rank fusion constant: the usual one.
const RRF_K: f32 = 60.0;
/// The longest piece of a fragment shown to recognize it.
pub const SNIPPET_CHARS: usize = 240;
/// How much the ranking by meaning weighs against the one by words. Measured with the reference questions of
/// `fixtures/expediente` (`search_live`, 2026-10-12): 1:3 put the expected page among the first three in 100 % of
/// the questions, with the words of the document or with others; words alone, 94 % and 50 %; 1:1, 100 % and 93 %.
const MEANING_WEIGHT: f32 = 3.0;
/// Fragments each ranking brings before they are fused.
const CANDIDATES: usize = 20;
/// The places of the ranking by meaning that count without a word of the question (measured in `search_live`: the
/// expected page of a question with other words was never below the third place by meaning).
const MEANING_ONLY: usize = 3;
/// The longest question, in characters.
pub const MAX_QUERY_CHARS: usize = 300;

/// A piece of a document that answers a question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Passage {
    /// The fragment it comes from (`document_chunk.rowid`).
    pub chunk: i64,
    pub document_id: String,
    /// The name the person gave the document.
    pub document: String,
    pub page: Option<i64>,
    /// The words around what was looked for.
    pub snippet: String,
}

/// The fragments of the documents of one drawer that share words with the question, best first (ADR-036: there is no
/// search in everything). A document waiting to be deleted is not shown.
pub fn by_words(conn: &Connection, drawer: Drawer, query: &str, limit: usize) -> Result<Vec<i64>, StorageError> {
    let Some(terms) = fts::match_terms(query) else { return Ok(Vec::new()) };
    let mut stmt = conn.prepare(
        "SELECT f.rowid FROM document_chunk_fts f
           JOIN document_chunk c ON c.rowid = f.rowid
           JOIN document d ON d.id = c.document_id
         WHERE document_chunk_fts MATCH ?1 AND d.drawer = ?3 AND d.hidden = 0
         ORDER BY f.rank LIMIT ?2",
    )?;
    let ids = stmt.query_map(params![terms, limit as i64, drawer.as_str()], |r| r.get(0))?.collect::<Result<Vec<i64>, _>>()?;
    Ok(ids)
}

/// The fragments of one drawer closest in meaning to a question (its vector), best first.
pub fn by_meaning(conn: &Connection, drawer: Drawer, query: &[f32], limit: usize) -> Result<Vec<i64>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT v.rowid FROM (SELECT rowid, distance FROM document_chunk_vec WHERE embedding MATCH ?1 AND k = ?2) v
           JOIN document_chunk c ON c.rowid = v.rowid
           JOIN document d ON d.id = c.document_id
         WHERE d.drawer = ?3 AND d.hidden = 0
         ORDER BY v.distance",
    )?;
    let ids = stmt.query_map(params![vectors::vec_json(query), limit as i64, drawer.as_str()], |r| r.get(0))?.collect::<Result<Vec<i64>, _>>()?;
    Ok(ids)
}

/// The passages of one drawer that best answer a question: by words, and by meaning when its vector came; fused by
/// position.
pub fn search(conn: &Connection, drawer: Drawer, query: &str, vector: Option<&[f32]>, limit: usize, chars: usize) -> Result<Vec<Passage>, StorageError> {
    let words = by_words(conn, drawer, query, CANDIDATES)?;
    let meaning = match vector {
        Some(v) => by_meaning(conn, drawer, v, CANDIDATES)?,
        None => Vec::new(),
    };
    passages(conn, &fuse_words_and_meaning(&words, &meaning, limit), query, chars)
}

/// The two rankings fused, the meaning weighing `MEANING_WEIGHT`. What only the meaning found comes only from its first
/// `MEANING_ONLY` places: a vector always has a nearest fragment, even when nothing is about the question.
pub fn fuse_words_and_meaning<K: Clone + Eq + Hash>(words: &[K], meaning: &[K], limit: usize) -> Vec<K> {
    let fused = fuse_weighted(&[(words, 1.0), (meaning, MEANING_WEIGHT)], usize::MAX);
    let close = &meaning[..meaning.len().min(MEANING_ONLY)];
    fused.into_iter().filter(|k| words.contains(k) || close.contains(k)).take(limit).collect()
}

/// What a search found, and whether it also looked by meaning (with internet, Gemini and the vectors made).
#[derive(Debug, Clone, Serialize)]
pub struct Found {
    pub passages: Vec<Passage>,
    pub by_meaning: bool,
}

/// Looks for a question in the documents of one drawer. Its vector is asked for only when there are vectors to compare
/// it with and the question carries no data of a person (the scanner says so); otherwise, or if the service does not
/// answer in a few seconds, the search goes by words alone. Nothing of it is kept.
pub async fn search_documents(db: &SharedDb, embedder: Option<Arc<dyn Embedder>>, drawer: Drawer, query: &str, limit: usize, chars: usize) -> Result<Found, ServiceError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(ServiceError::EmptyText);
    }
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(ServiceError::TextTooLarge);
    }
    let may_send = {
        let conn = db.lock().map_err(|_| ServiceError::Internal("lock".into()))?;
        vectors::any(&conn)? && crate::core::screen::scanner_for(&conn)?.scan(query).is_clean()
    };
    let mut vector = None;
    if let Some(e) = embedder.filter(|_| may_send) {
        let started = std::time::Instant::now();
        let answer = e.embed(&[query.to_string()], true).await;
        let ok = matches!(&answer, Ok(v) if v.len() == 1);
        vectors::record_usage(db, e.as_ref(), query.chars().count(), ok, started.elapsed().as_millis() as u64);
        vector = answer.ok().filter(|_| ok).and_then(|mut v| v.pop());
    }
    let conn = db.lock().map_err(|_| ServiceError::Internal("lock".into()))?;
    let passages = search(&conn, drawer, query, vector.as_deref(), limit, chars)?;
    Ok(Found { passages, by_meaning: vector.is_some() })
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

/// The fragments as passages, in the same order, with their document, page and about `chars` characters around the
/// question.
pub fn passages(conn: &Connection, chunks: &[i64], query: &str, chars: usize) -> Result<Vec<Passage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT c.document_id, d.display_name, c.page, c.text FROM document_chunk c JOIN document d ON d.id = c.document_id WHERE c.rowid = ?1",
    )?;
    let mut out = Vec::with_capacity(chunks.len());
    for &chunk in chunks {
        let row = stmt.query_row([chunk], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<i64>>(2)?, r.get::<_, String>(3)?)));
        if let Ok((document_id, document, page, text)) = row {
            out.push(Passage { chunk, document_id, document, page, snippet: excerpt(&text, query, chars) });
        }
    }
    Ok(out)
}

/// About `chars` characters of a fragment around the first word of the question it has (from its start when it has
/// none, as a passage found by meaning), on one line and cut between words.
pub fn excerpt(text: &str, query: &str, chars: usize) -> String {
    let terms: Vec<String> = fts::words(query).iter().map(|w| fts::fold(w)).collect();
    let flat: Vec<char> = text.split_whitespace().collect::<Vec<_>>().join(" ").chars().collect();
    // where each word starts, and whether it begins with a term of the question
    let mut hit: Option<usize> = None;
    let mut i = 0;
    while i < flat.len() && hit.is_none() {
        if flat[i].is_alphanumeric() && (i == 0 || !flat[i - 1].is_alphanumeric()) {
            let word: String = flat[i..].iter().take_while(|c| c.is_alphanumeric()).collect();
            let folded = fts::fold(&word);
            if terms.iter().any(|t| folded.starts_with(t.as_str())) {
                hit = Some(i);
            }
        }
        i += 1;
    }
    let mut start = hit.map_or(0, |h| h.saturating_sub(chars / 4));
    while start > 0 && flat[start - 1] != ' ' {
        start -= 1;
    }
    let mut end = (start + chars).min(flat.len());
    while end < flat.len() && flat[end] != ' ' {
        end += 1;
    }
    let body: String = flat[start..end].iter().collect();
    format!("{}{}{}", if start > 0 { "…" } else { "" }, body.trim(), if end < flat.len() { "…" } else { "" })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::core::archive::storage::{add_paged_document, Drawer, PageMarks};
    use serde::Deserialize;
    use std::path::{Path, PathBuf};

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente")
    }

    /// The pages of a fixture file (`=== PAGINA n ===`).
    pub fn fixture_pages(text: &str) -> Vec<String> {
        text.split("=== PAGINA").skip(1).map(|p| p.split_once("===").map_or("", |x| x.1).trim().to_string()).collect()
    }

    /// The fictitious record of `fixtures/expediente`, saved as documents of the institution; what each fragment is
    /// (file, page).
    pub fn load_record(conn: &Connection) -> HashMap<i64, (String, i64)> {
        let mut files: Vec<PathBuf> = std::fs::read_dir(fixtures()).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "txt")).collect();
        files.sort();
        for f in &files {
            let name = f.file_name().unwrap().to_string_lossy().to_string();
            add_paged_document(conn, Drawer::Record, &name, "application/pdf", &fixture_pages(&std::fs::read_to_string(f).unwrap()), 0, &PageMarks::default()).unwrap();
        }
        let mut stmt = conn.prepare("SELECT c.rowid, d.display_name, c.page FROM document_chunk c JOIN document d ON d.id = c.document_id").unwrap();
        let v = stmt.query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?)))).unwrap().collect::<Result<HashMap<_, _>, _>>().unwrap();
        v
    }

    #[derive(Deserialize)]
    pub struct RecordQuestion {
        pub q: String,
        pub tipo: String,
        pub esperado: Vec<(String, i64)>,
    }

    #[derive(Deserialize)]
    pub struct ManualQuestion {
        pub q: String,
        pub tipo: String,
        pub esperado: Vec<String>,
    }

    #[derive(Deserialize)]
    pub struct Reference {
        pub expediente: Vec<RecordQuestion>,
        pub manual: Vec<ManualQuestion>,
    }

    pub fn reference() -> Reference {
        serde_json::from_str(&std::fs::read_to_string(fixtures().join("preguntas.json")).unwrap()).unwrap()
    }

    /// Hits at 1 and at 3 and the reciprocal rank (up to the fifth) of a ranking, against what is expected.
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Score {
        pub n: usize,
        pub at1: usize,
        pub at3: usize,
        pub rr: f32,
    }

    impl Score {
        pub fn add<K: PartialEq>(&mut self, ranking: &[K], expected: &[K]) -> Option<usize> {
            self.n += 1;
            let pos = ranking.iter().take(5).position(|k| expected.contains(k));
            if let Some(p) = pos {
                self.at1 += usize::from(p == 0);
                self.at3 += usize::from(p < 3);
                self.rr += 1.0 / (p as f32 + 1.0);
            }
            pos
        }
        pub fn line(&self, label: &str) -> String {
            let pct = |x: usize| if self.n == 0 { 0.0 } else { 100.0 * x as f32 / self.n as f32 };
            format!("{label:<28} {:>3} preguntas  al 1.º: {:>5.1} %  entre 3: {:>5.1} %  MRR: {:.3}", self.n, pct(self.at1), pct(self.at3), self.rr / self.n.max(1) as f32)
        }
    }

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, c)
    }

    /// The baseline of IA5 (ADR-034): the reference questions of the fictitious record, by words alone (no AI, no
    /// internet). A question that uses the words of the document finds its page; the measurement against vectors is
    /// `search_live`.
    #[test]
    fn the_reference_questions_find_their_page_by_words() {
        let (_d, c) = conn();
        let of = load_record(&c);
        let (mut words, mut meaning) = (Score::default(), Score::default());
        for q in reference().expediente {
            let ranking: Vec<(String, i64)> = by_words(&c, Drawer::Record, &q.q, 5).unwrap().iter().map(|r| of[r].clone()).collect();
            let score = if q.tipo == "palabras" { &mut words } else { &mut meaning };
            let pos = score.add(&ranking, &q.esperado);
            println!("{:<8} {:<75} {}", q.tipo, q.q, pos.map_or("—".to_string(), |p| format!("{}.º", p + 1)));
        }
        println!("{}\n{}", words.line("Con las palabras del documento"), meaning.line("Con otras palabras"));
        assert!(words.at3 * 10 >= words.n * 9, "{}", words.line("palabras"));
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
    /// a handful of requests). With `CIMIENTO_VEC_CACHE` the vectors are kept in that file and read from it the next
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

        let cache = std::env::var("CIMIENTO_VEC_CACHE").ok().map(PathBuf::from);
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


    #[test]
    fn calls_and_documents_waiting_to_be_deleted_are_not_searched() {
        let (_d, c) = conn();
        add_paged_document(&c, Drawer::Projects, "Convocatoria", "application/pdf", &["Los extintores del fondo ficticio.".into()], 0, &PageMarks::default()).unwrap();
        let hidden = add_paged_document(&c, Drawer::Record, "Por borrar", "application/pdf", &["Extintores revisados en agosto.".into()], 0, &PageMarks::default()).unwrap();
        c.execute("UPDATE document SET hidden = 1 WHERE id = ?1", [&hidden]).unwrap();
        let kept = add_paged_document(&c, Drawer::Record, "Protección civil", "application/pdf", &["Hay doce extintores.".into()], 0, &PageMarks::default()).unwrap();
        let found = by_words(&c, Drawer::Record, "¿Cuántos extintores tenemos?", 5).unwrap();
        let p = passages(&c, &found, "extintores", SNIPPET_CHARS).unwrap();
        assert_eq!(p.iter().map(|p| (p.document_id.as_str(), p.page)).collect::<Vec<_>>(), vec![(kept.as_str(), Some(1))]);
        assert!(by_words(&c, Drawer::Record, "\" OR * NEAR( -", 5).unwrap().is_empty(), "what the person writes never reaches FTS5 as syntax");
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
        let found = search_documents(&db, Some(e.clone()), Drawer::Record, "¿Cuántos extintores tenemos?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(!found.by_meaning && found.passages[0].document == "programa-proteccion-civil.txt");
        assert_eq!(e.texts.load(Ordering::Relaxed), 0);

        index_pending(&db, e.as_ref()).await.unwrap();
        let made = e.texts.load(Ordering::Relaxed);
        let found = search_documents(&db, Some(e.clone()), Drawer::Record, "¿Cuántos extintores tenemos?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(found.by_meaning);
        assert_eq!((found.passages[0].document.as_str(), found.passages[0].page), ("programa-proteccion-civil.txt", Some(1)));
        assert!(found.passages[0].snippet.contains("extintores"), "{:?}", found.passages[0]);
        assert_eq!(e.texts.load(Ordering::Relaxed), made + 1, "one question, one text");

        let found = search_documents(&db, Some(e.clone()), Drawer::Record, "¿Dónde está la CURP LOPM800101MDFRZN09 del poder?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(!found.by_meaning && !found.passages.is_empty(), "a datum of a person never leaves; the words still answer");
        assert_eq!(e.texts.load(Ordering::Relaxed), made + 1);

        let offline = Arc::new(WordsEmbedder { fail: true, ..WordsEmbedder::new() });
        let found = search_documents(&db, Some(offline), Drawer::Record, "¿Quién paga el predial?", 5, SNIPPET_CHARS).await.unwrap();
        assert!(!found.by_meaning && found.passages[0].document == "contrato-comodato.txt");
        assert!(matches!(search_documents(&db, None, Drawer::Record, "  ", 5, SNIPPET_CHARS).await, Err(ServiceError::EmptyText)));
        assert!(matches!(search_documents(&db, None, Drawer::Record, &"x".repeat(MAX_QUERY_CHARS + 1), 5, SNIPPET_CHARS).await, Err(ServiceError::TextTooLarge)));
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

    #[test]
    fn the_snippet_shows_the_words_around_what_was_asked() {
        let text = format!("{} El régimen es de Personas Morales con Fines no Lucrativos.\n\nOtra cosa. {}", "Relleno largo. ".repeat(20), "Más relleno. ".repeat(30));
        let s = excerpt(&text, "¿Cuál es el regimen fiscal?", SNIPPET_CHARS);
        assert!(s.starts_with('…') && s.ends_with('…') && s.contains("régimen es de Personas Morales"), "{s}");
        assert!(s.chars().count() <= SNIPPET_CHARS + 30 && !s.contains('\n'), "{s}");
        // nothing of the question in it (found by meaning): its beginning
        assert!(excerpt("Uno dos tres.", "zzz", SNIPPET_CHARS).starts_with("Uno dos tres."));
    }
}
