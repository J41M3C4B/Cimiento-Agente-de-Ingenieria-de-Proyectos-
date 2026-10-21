//! Search in the documents of the institution (IA5, ADR-034 §3): by words with FTS5, always and without internet.
//! Each passage carries its document and page, taken by the code from the fragment it came from: a citation nobody
//! has to trust. The search by meaning is not here: it asks a model, so it belongs to the AI platform.

use crate::core::archive::storage::Drawer;
use crate::core::error::ServiceError;
use crate::storage::fts;
use crate::storage::{SharedDb, StorageError};
use rusqlite::{params, Connection};
use serde::Serialize;

/// The longest piece of a fragment shown to recognize it.
pub const SNIPPET_CHARS: usize = 240;
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
    words_in(conn, drawer, None, query, limit)
}

/// The same, in one document of the drawer when `document` says which.
pub fn words_in(conn: &Connection, drawer: Drawer, document: Option<&str>, query: &str, limit: usize) -> Result<Vec<i64>, StorageError> {
    let Some(terms) = fts::match_terms(query) else { return Ok(Vec::new()) };
    let mut stmt = conn.prepare(
        "SELECT f.rowid FROM document_chunk_fts f
           JOIN document_chunk c ON c.rowid = f.rowid
           JOIN document d ON d.id = c.document_id
         WHERE document_chunk_fts MATCH ?1 AND d.drawer = ?3 AND d.hidden = 0 AND (?4 IS NULL OR d.id = ?4)
         ORDER BY f.rank LIMIT ?2",
    )?;
    let ids = stmt.query_map(params![terms, limit as i64, drawer.as_str(), document], |r| r.get(0))?.collect::<Result<Vec<i64>, _>>()?;
    Ok(ids)
}


/// The passages of one drawer that best answer a question, by words; in one document of the drawer when `document`
/// says which (the person named its file).
pub fn search_in(conn: &Connection, drawer: Drawer, document: Option<&str>, query: &str, limit: usize, chars: usize) -> Result<Vec<Passage>, StorageError> {
    let words = words_in(conn, drawer, document, query, limit)?;
    passages(conn, &words, query, chars)
}

/// The first fragments of a document of a drawer, as passages: what is read of it when no word of the question is in
/// it («¿qué dice el archivo…?»).
pub fn document_start(conn: &Connection, drawer: Drawer, document: &str, limit: usize, chars: usize) -> Result<Vec<Passage>, StorageError> {
    let chunks: Vec<i64> = conn
        .prepare("SELECT c.rowid FROM document_chunk c JOIN document d ON d.id = c.document_id WHERE d.id = ?1 AND d.drawer = ?2 AND d.hidden = 0 ORDER BY c.ordinal LIMIT ?3")?
        .query_map(params![document, drawer.as_str(), limit as i64], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    passages(conn, &chunks, "", chars)
}

/// Every fragment of a document of a drawer, in its order: its id, page and text.
pub fn document_chunks(conn: &Connection, drawer: Drawer, document: &str) -> Result<Vec<(i64, Option<i64>, String)>, StorageError> {
    let rows = conn
        .prepare("SELECT c.rowid, c.page, c.text FROM document_chunk c JOIN document d ON d.id = c.document_id WHERE d.id = ?1 AND d.drawer = ?2 AND d.hidden = 0 ORDER BY c.ordinal")?
        .query_map(params![document, drawer.as_str()], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// What a search found.
#[derive(Debug, Clone, Serialize)]
pub struct Found {
    pub passages: Vec<Passage>,
}

/// The question as it is searched: trimmed, and neither empty nor longer than `MAX_QUERY_CHARS`.
pub fn checked_query(query: &str) -> Result<&str, ServiceError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(ServiceError::EmptyText);
    }
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(ServiceError::TextTooLarge);
    }
    Ok(query)
}

/// Looks for a question in the documents of one drawer, by words: with no internet and nothing sent anywhere.
pub fn search_documents(db: &SharedDb, drawer: Drawer, query: &str, limit: usize, chars: usize) -> Result<Found, ServiceError> {
    let query = checked_query(query)?;
    let conn = db.lock().map_err(|_| ServiceError::Internal("lock".into()))?;
    Ok(Found { passages: search_in(&conn, drawer, None, query, limit, chars)? })
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
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente")
    }

    /// The pages of a fixture file (`=== PAGINA n ===`).
    pub fn fixture_pages(text: &str) -> Vec<String> {
        text.split("=== PAGINA").skip(1).map(|p| p.split_once("===").map_or("", |x| x.1).trim().to_string()).collect()
    }

    /// The fictitious record of `fixtures/expediente`, saved as documents of the institution, each one the file of its
    /// document of the Expediente as the app leaves it (its type by its words; the contract, as «Otro documento»);
    /// what each fragment is (file, page).
    pub fn load_record(conn: &Connection) -> HashMap<i64, (String, i64)> {
        use crate::core::records::storage::{save, set_file, Input};
        let mut files: Vec<PathBuf> = std::fs::read_dir(fixtures()).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "txt")).collect();
        files.sort();
        for f in &files {
            let name = f.file_name().unwrap().to_string_lossy().to_string();
            let pages = fixture_pages(&std::fs::read_to_string(f).unwrap());
            let doc = add_paged_document(conn, Drawer::Record, &name, "application/pdf", &pages, 0, &PageMarks::default()).unwrap();
            let kind = crate::core::reader::kinds::recognize(&pages);
            let custom = kind.is_none().then_some("Contrato de comodato del inmueble");
            let input = Input { custom_name: custom, status: "have", issued_on: None, valid_until: None, reference: None, notes: None };
            let record = save(conn, None, kind.unwrap_or("other"), &input).unwrap();
            set_file(conn, &record, Some(&doc), &f.to_string_lossy(), &name).unwrap();
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

    pub(crate) fn conn() -> (tempfile::TempDir, Connection) {
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

    #[test]
    fn the_snippet_shows_the_words_around_what_was_asked() {
        let text = format!("{} El régimen es de Personas Morales con Fines no Lucrativos.\n\nOtra cosa. {}", "Relleno largo. ".repeat(20), "Más relleno. ".repeat(30));
        let s = excerpt(&text, "¿Cuál es el regimen fiscal?", SNIPPET_CHARS);
        assert!(s.starts_with('…') && s.ends_with('…') && s.contains("régimen es de Personas Morales"), "{s}");
        assert!(s.chars().count() <= SNIPPET_CHARS + 30 && !s.contains('\n'), "{s}");
        // nothing of the question in it (found by meaning): its beginning
        assert!(excerpt("Uno dos tres.", "zzz", SNIPPET_CHARS).starts_with("Uno dos tres."));
    }

    /// The search answers by words alone, with no internet and nothing sent; it checks the question; and what it
    /// returns says nothing of meaning.
    #[test]
    fn the_search_of_the_record_is_by_words_alone() {
        let dir = tempfile::tempdir().unwrap();
        let db: SharedDb = std::sync::Arc::new(std::sync::Mutex::new(crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap()));
        load_record(&db.lock().unwrap());
        let found = search_documents(&db, Drawer::Record, "¿Cuántos extintores tenemos?", 5, SNIPPET_CHARS).unwrap();
        assert_eq!((found.passages[0].document.as_str(), found.passages[0].page), ("programa-proteccion-civil.txt", Some(1)));
        assert!(found.passages[0].snippet.contains("extintores"), "{:?}", found.passages[0]);
        assert_eq!(serde_json::to_value(&found).unwrap().as_object().unwrap().keys().collect::<Vec<_>>(), vec!["passages"]);
        assert!(matches!(search_documents(&db, Drawer::Record, "  ", 5, SNIPPET_CHARS), Err(ServiceError::EmptyText)));
        assert!(matches!(search_documents(&db, Drawer::Record, &"x".repeat(MAX_QUERY_CHARS + 1), 5, SNIPPET_CHARS), Err(ServiceError::TextTooLarge)));
    }
}
