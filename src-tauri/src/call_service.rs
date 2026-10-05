//! Use cases of the calls (convocatorias): a project is born from its call, the files are read once in
//! silence, and what was understood is shown (ADR-011, ADR-015, ADR-016).
//!
//! The person drops the PDF of the call (and the annexes, guides and forms that come with it), says what the
//! call is called, who gives it and in which year, and goes on. Creating the project only checks, cleans and
//! saves the files, which is fast; the reading runs afterwards in the background and never blocks anything:
//! with no key, no connection or no allowance the call stays saved and waiting.

use crate::ai::pipeline::{self, AiCall, SqliteLedger};
use crate::ai::settings::{self, ProviderKind};
use crate::ai::{self, AiProvider, AiTask, ModelTier};
use crate::diagnosis_service::{guard_texts, AiStatus, SharedDb};
use crate::domain::figures;
use crate::domain::stage::{self, Stage};
use crate::documents::canonical::assemble::{assemble, Reading};
use crate::documents::canonical::card::{card_of, CallCard};
use crate::documents::canonical::package::{read_pieces, Package};
use crate::documents::canonical::retrieve::{Embedder, GeminiEmbedder, RetrieverKind};
use crate::documents::canonical::run::{read_canonical, Options, Strategy};
use crate::documents::canonical::summary::{summarize, CallSummary};
use crate::documents::text::{clean_text, norm, same_statement};
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::scanner::{RegexScanner, ScanReport, SensitiveScanner, Severity};
use crate::service::ServiceError;
use crate::storage::calls::{self, CallMeta, FileRole, NewFile, ReadingRow, ReadingStatus};
use crate::storage::projects::{self as projects, ProjectRow};
use crate::storage::{self, profile as profile_store};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use ulid::Ulid;

/// The embedding model that chooses, for each block of the call, the pages worth reading (ADR-015).
const EMBEDDING_MODEL: &str = "gemini-embedding-2";
const MAX_FILE_BYTES: usize = 40 * 1024 * 1024;
const MAX_FILES: usize = 12;
/// A page with fewer letters than this has no text layer.
const MIN_LETTERS_PER_PAGE: usize = 20;

fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("database lock poisoned".into()))
}

/// A file as the screen sends it.
pub struct UploadedFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Why a file was not taken. The screen words each one in plain language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unreadable {
    /// Not a PDF, Word or Excel file.
    UnsupportedType,
    TooLarge,
    Damaged,
    /// A PDF that is a picture of the pages: there is no text to read (OCR is out of scope, ADR-011).
    Scanned,
    Empty,
    /// It looks like a list of people, which is never saved (`scanner`).
    Roster,
    NoFiles,
    TooManyFiles,
    /// A package has exactly one file marked as the call itself.
    NoMainFile,
}

/// A file of the package with the character the person gave it.
pub struct PackageFile {
    pub file: UploadedFile,
    pub role: FileRole,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NewProjectOutcome {
    Created { project: ProjectRow, reading: ReadingRow },
    /// The name or the funder look like data of a person: nothing was saved.
    Quarantine { report: QuarantineReport },
    Unreadable { file: String, reason: Unreadable },
}

/// The scanner for a document of the funder, not of the institution. A call is public: the telephone, the
/// e-mail and the names of whoever it tells to write to are part of what it says and the reading must
/// keep them. What is never kept is a person's identity data (CURP, personal RFC, voter key, bank
/// account, card, social security): those findings are covered, here and before the model sees a page.
pub struct PublicDocScanner(RegexScanner);

impl PublicDocScanner {
    pub fn new(inner: RegexScanner) -> Self {
        PublicDocScanner(inner)
    }
}

impl SensitiveScanner for PublicDocScanner {
    fn scan(&self, text: &str) -> ScanReport {
        let mut r = self.0.scan(text);
        r.findings.retain(|f| f.severity == Severity::Block);
        r
    }

    fn redact(&self, text: &str, report: &ScanReport) -> String {
        self.0.redact(text, report)
    }
}

/// A folder for one upload that is gone when it is dropped, whatever happens in between.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> std::io::Result<Scratch> {
        let dir = std::env::temp_dir().join(format!("cimiento-{}", Ulid::generate()));
        std::fs::create_dir_all(&dir)?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn extension(name: &str) -> Option<&'static str> {
    match name.rsplit('.').next().map(str::to_lowercase).as_deref() {
        Some("pdf") => Some("pdf"),
        Some("docx") => Some("docx"),
        Some("xlsx") => Some("xlsx"),
        Some("xlsm") => Some("xlsm"),
        _ => None,
    }
}

fn mime_of(ext: &str) -> &'static str {
    match ext {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        _ => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    }
}

/// The name a person sees: the last piece of the path, with no folder in it.
fn display_name(name: &str) -> String {
    name.rsplit(['/', '\\']).next().unwrap_or(name).trim().to_string()
}

/// Reads one file into clean pages with the identity data covered. `Err` says why it is not taken.
fn prepare(file: &UploadedFile, role: FileRole, scanner: &PublicDocScanner) -> Result<NewFile, Unreadable> {
    let name = display_name(&file.name);
    let ext = extension(&name).ok_or(Unreadable::UnsupportedType)?;
    if file.bytes.len() > MAX_FILE_BYTES {
        return Err(Unreadable::TooLarge);
    }
    let scratch = Scratch::new().map_err(|_| Unreadable::Damaged)?;
    let path = scratch.0.join(format!("upload.{ext}"));
    std::fs::write(&path, &file.bytes).map_err(|_| Unreadable::Damaged)?;
    let raw = read_pieces(&path).map_err(|_| Unreadable::Damaged)?;
    drop(scratch);

    let mut pages: Vec<String> = raw.iter().map(|p| clean_text(p)).collect();
    let letters = |t: &str| t.chars().filter(|c| c.is_alphabetic()).count();
    if pages.iter().all(|p| letters(p) < MIN_LETTERS_PER_PAGE) {
        return Err(if ext == "pdf" && !pages.is_empty() { Unreadable::Scanned } else { Unreadable::Empty });
    }
    // a list of people is rejected whole; here the whole scanner counts, not only the public one
    let joined = pages.join("\n\n");
    if scanner.0.looks_like_roster(&joined, &scanner.0.scan(&joined)) {
        return Err(Unreadable::Roster);
    }
    let mut redactions = 0i64;
    for page in &mut pages {
        let report = scanner.scan(page);
        if !report.is_clean() {
            redactions += report.findings.len() as i64;
            *page = scanner.redact(page, &report);
        }
    }
    Ok(NewFile { name, mime: mime_of(ext), pages, redactions, role })
}

/// Checks and cleans every file of a package. `Ok(Err(..))` says which file was not taken and why; a call with
/// a document missing gives a wrong picture, so one bad file stops the whole package.
fn prepare_package(db: &SharedDb, files: Vec<PackageFile>) -> Result<Result<Vec<NewFile>, (String, Unreadable)>, ServiceError> {
    if files.is_empty() {
        return Ok(Err((String::new(), Unreadable::NoFiles)));
    }
    if files.len() > MAX_FILES {
        return Ok(Err((String::new(), Unreadable::TooManyFiles)));
    }
    if files.iter().filter(|f| f.role == FileRole::Main).count() != 1 {
        return Ok(Err((String::new(), Unreadable::NoMainFile)));
    }
    let scanner = {
        let conn = lock(db)?;
        PublicDocScanner::new(RegexScanner::new(profile_store::scanner_config(&conn)?))
    };
    let mut prepared = Vec::with_capacity(files.len());
    for f in &files {
        match prepare(&f.file, f.role, &scanner) {
            Ok(p) => prepared.push(p),
            Err(reason) => return Ok(Err((display_name(&f.file.name), reason))),
        }
    }
    // the call goes first: the page numbers of the package follow this order
    prepared.sort_by_key(|f| f.role != FileRole::Main);
    Ok(Ok(prepared))
}

const MIN_YEAR: i64 = 1990;
const MAX_YEAR: i64 = 2100;

/// What the project starts from, in one sentence: the diagnosis conversation reads it as the first thing the
/// institution said it wants. The reading of the call itself reaches the conversation by its own path.
fn request_of(name: &str, funder: Option<&str>, year: Option<i64>) -> String {
    let mut s = format!("La institución quiere postular a la convocatoria «{name}»");
    if let Some(f) = funder {
        s.push_str(&format!(" de {f}"));
    }
    if let Some(y) = year {
        s.push_str(&format!(" ({y})"));
    }
    s.push('.');
    s
}

/// Creates a project from its call: the files are checked and saved, the project and its reading are created
/// together and the project waits in `CALL_SELECTION` until the person confirms the call. The reading itself is `read_call`, which the caller starts
/// in the background. Nothing is saved if anything is wrong with the files, the name or the profile.
pub fn create_project_from_call(
    db: &SharedDb,
    files: Vec<PackageFile>,
    name: &str,
    funder: Option<&str>,
    year: Option<i64>,
    decision: Option<Decision>,
) -> Result<NewProjectOutcome, ServiceError> {
    let name = name.trim();
    let funder = funder.map(str::trim).filter(|f| !f.is_empty());
    if name.is_empty() {
        return Err(ServiceError::EmptyText);
    }
    if year.is_some_and(|y| !(MIN_YEAR..=MAX_YEAR).contains(&y)) {
        return Err(ServiceError::InvalidYear);
    }
    // what the person typed is scanned like any other text, before anything is saved
    let (name, funder) = {
        let conn = lock(db)?;
        let facts = projects::facts(&conn, "")?;
        stage::advance(Stage::Profile, &facts)?; // profile confirmed and recent, else an error before saving anything
        let mut fields = vec![("call_name".to_string(), name.to_string())];
        if let Some(f) = funder {
            fields.push(("call_funder".to_string(), f.to_string()));
        }
        match guard_texts(&conn, "project", &fields, decision)? {
            Ok(mut texts) => {
                let funder = if texts.len() > 1 { Some(texts.remove(1)) } else { None };
                (texts.remove(0), funder)
            }
            Err(report) => return Ok(NewProjectOutcome::Quarantine { report }),
        }
    };
    let prepared = match prepare_package(db, files)? {
        Ok(p) => p,
        Err((file, reason)) => return Ok(NewProjectOutcome::Unreadable { file, reason }),
    };
    let request = request_of(&name, funder.as_deref(), year);
    let meta = CallMeta { name: name.clone(), funder, year };
    let mut conn = lock(db)?;
    let (project_id, reading_id) = calls::create_project_with_call(&mut conn, &name, &request, &meta, &prepared)?;
    projects::set_stage(&mut conn, &project_id, Stage::CallSelection, false)?;
    let project = projects::get_project(&conn, &project_id)?.ok_or(ServiceError::NotFound)?;
    let reading = calls::get(&conn, &reading_id)?.ok_or(ServiceError::NotFound)?;
    Ok(NewProjectOutcome::Created { project, reading })
}

// ------------------------------------------------------------------ the reading

/// What a reading needs from the settings: who answers, and who chooses the pages.
pub struct ReadPlan {
    pub provider: Option<Box<dyn AiProvider>>,
    pub embedder: Option<Arc<dyn Embedder>>,
    /// The provider takes a schema with `$ref` only if it is Gemini (the one measured); any other gets it in line.
    pub inline_schema: bool,
}

/// The provider chosen in the settings if its key is saved. Pages are chosen by embeddings only when the
/// person's provider is Gemini: the text of a call is not sent to a service the person did not choose.
pub fn reading_plan(db: &SharedDb) -> ReadPlan {
    let Ok(conn) = db.lock() else { return ReadPlan { provider: None, embedder: None, inline_schema: false } };
    let Ok(s) = settings::load(&conn) else { return ReadPlan { provider: None, embedder: None, inline_schema: false } };
    drop(conn);
    let key = storage::get_api_key(s.provider.as_str()).ok().flatten();
    let embedder: Option<Arc<dyn Embedder>> = match (&key, s.provider) {
        (Some(k), ProviderKind::Gemini) => Some(Arc::new(GeminiEmbedder::new(k.clone(), EMBEDDING_MODEL))),
        _ => None,
    };
    ReadPlan { provider: key.map(|k| ai::build_provider(&s, k)), embedder, inline_schema: s.provider != ProviderKind::Gemini }
}

/// The code the screen words, for the first call of a reading that failed.
fn note_of(error: &str) -> &'static str {
    match error.split(':').next().unwrap_or("") {
        "no_key" => "not_configured",
        "offline" => "offline",
        "rate_limited" | "timeout" => "busy",
        "budget" => "budget_exhausted",
        "quota_reached" => "quota_reached",
        "auth" => "key_rejected",
        _ => "unavailable",
    }
}

/// The calls of the reading, added up from the usage log: tokens, cost and the model that answered.
fn usage_of(db: &SharedDb, label: &str) -> (u64, u64, f64, Option<String>) {
    let Ok(conn) = db.lock() else { return (0, 0, 0.0, None) };
    let totals = conn
        .query_row(
            "SELECT COALESCE(SUM(input_tokens + cached_tokens),0), COALESCE(SUM(output_tokens),0), COALESCE(SUM(estimated_cost_mxn),0) FROM ai_usage WHERE project_id=?1 AND success=1",
            [label],
            |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, i64>(1)? as u64, r.get::<_, f64>(2)?)),
        )
        .unwrap_or((0, 0, 0.0));
    let model = conn.query_row("SELECT model FROM ai_usage WHERE project_id=?1 AND success=1 ORDER BY at DESC LIMIT 1", [label], |r| r.get::<_, String>(0)).ok();
    (totals.0, totals.1, totals.2, model)
}

fn now_iso(db: &SharedDb) -> String {
    db.lock().ok().and_then(|c| c.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')", [], |r| r.get(0)).ok()).unwrap_or_default()
}

/// Reads a saved call: one call to the model per block of the schema, over the pages each block needs, then a
/// second look at what nobody quoted; the code verifies every quote and assembles the document (ADR-015).
/// It never fails: whatever happens ends in the state of the reading (`ready`, `partial`, `waiting` or `failed`).
pub async fn read_call(db: &SharedDb, plan: &ReadPlan, id: &str) {
    let (name, pages, scanner) = {
        let Ok(conn) = lock(db) else { return };
        let Ok(Some(row)) = calls::get(&conn, id) else { return };
        let Ok(pages) = calls::pages_of(&conn, id) else { return };
        let Ok(cfg) = profile_store::scanner_config(&conn) else { return };
        let _ = calls::set_status(&conn, id, ReadingStatus::Reading, None);
        (row.name, pages, PublicDocScanner::new(RegexScanner::new(cfg)))
    };
    let finish = |status: ReadingStatus, note: Option<&str>, doc: Option<&Value>, report: Value| {
        if let Ok(conn) = lock(db) {
            let _ = calls::finish(&conn, id, status, note, doc, &report);
        }
    };
    let Some(provider) = plan.provider.as_deref() else {
        finish(ReadingStatus::Waiting, Some("not_configured"), None, json!({}));
        return;
    };

    let pkg = Package::from_documents(&name, pages);
    let label = format!("call:{id}");
    let ledger = SqliteLedger(db.clone());
    let opts = Options { strategy: Strategy::BlockRetrieved(RetrieverKind::Embedding), inline_schema: plan.inline_schema, budget: None, residual: true, label: label.clone() };
    let run = read_canonical(provider, &scanner, &ledger, plan.embedder.clone(), &pkg, &opts).await;

    let (input_tokens, output_tokens, cost_mxn, model) = usage_of(db, &label);
    let reading = Reading { model: model.unwrap_or_else(|| provider.model_name(ModelTier::Light)), at: now_iso(db), input_tokens, output_tokens, cost_mxn };
    let (doc, assembly) = assemble(&pkg, &run.answers, &reading);

    let first_error = run.calls.iter().find_map(|c| c.error.as_deref());
    let (status, note) = match (run.calls_ok(), run.calls.len()) {
        (0, _) => {
            // nothing was read: a problem that passes (no connection, no allowance, a busy service) leaves the call
            // waiting for another try; a key that is refused or an answer that cannot be used is a failure
            let note = first_error.map_or("unavailable", note_of);
            (if matches!(note, "key_rejected" | "unavailable") { ReadingStatus::Failed } else { ReadingStatus::Waiting }, Some(note))
        }
        (ok, total) if ok < total => (ReadingStatus::Partial, Some(first_error.map_or("unavailable", note_of))),
        _ => (ReadingStatus::Ready, None),
    };
    let report = json!({
        "assembly": assembly,
        "calls": run.calls,
        "lexical_fallback": run.lexical_fallback,
        "embedding": run.embedding,
        "pages": pkg.pages.len(),
        "chars": pkg.total_chars(),
    });
    finish(status, note, (run.calls_ok() > 0).then_some(&doc), report);
}

/// What a person sees of a reading.
#[derive(Debug, Serialize)]
pub struct ReadingDetail {
    pub reading: ReadingRow,
    /// What the person reads first (ADR-024): short, in plain words.
    pub card: Option<CallCard>,
    /// The call was read and nobody has tried yet to write its «en pocas palabras»: the screen asks for it, once.
    pub brief_pending: bool,
    /// The whole understanding, group by group, for whoever wants to look something up («Consultar el detalle»).
    pub summary: Option<CallSummary>,
    pub quality: Option<Quality>,
    /// What the person said about the call that the documents do not seem to say. Only a heads-up: the
    /// person's word is kept.
    pub differences: Vec<Difference>,
}

/// One thing the person wrote about the call (`field`: `funder` or `year`) next to what the reading found.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Difference {
    pub field: &'static str,
    pub said: String,
    pub read: String,
}

/// Compares what the person wrote with what the reading understood. A funder is the same if one name holds the
/// other («Fundación Nacional Monte de Piedad, I.A.P.» is «Nacional Monte de Piedad»); a year differs only if
/// the documents name a year in their title or edition and it is not the one the person wrote.
fn differences(reading: &ReadingRow, summary: &CallSummary) -> Vec<Difference> {
    let mut out = Vec::new();
    if let (Some(said), Some(read)) = (reading.funder.as_deref(), summary.funder.as_deref()) {
        if !same_statement(&norm(said), &norm(read)) {
            out.push(Difference { field: "funder", said: said.to_string(), read: read.to_string() });
        }
    }
    if let Some(said) = reading.year {
        let years: Vec<i64> = [summary.edition.as_deref(), summary.title.as_deref()]
            .into_iter()
            .flatten()
            .flat_map(|t| t.split(|c: char| !c.is_ascii_digit()).filter(|w| w.len() == 4).filter_map(|w| w.parse::<i64>().ok()).collect::<Vec<_>>())
            .filter(|y| (MIN_YEAR..=MAX_YEAR).contains(y))
            .collect();
        if !years.is_empty() && !years.contains(&said) {
            out.push(Difference { field: "year", said: said.to_string(), read: years[0].to_string() });
        }
    }
    out
}

/// How much of the call the reading took, in numbers a person can weigh.
#[derive(Debug, Serialize)]
pub struct Quality {
    pub pages: usize,
    pub pages_with_quotes: usize,
    /// Share of the quotes the model wrote that were found in the pages, 0 to 100.
    pub quotes_verified_percent: u32,
    pub blocks_read: usize,
    pub blocks_total: usize,
}

pub fn detail(db: &SharedDb, id: &str) -> Result<ReadingDetail, ServiceError> {
    let conn = lock(db)?;
    let reading = calls::get(&conn, id)?.ok_or(ServiceError::NotFound)?;
    let result = calls::result(&conn, id)?;
    let summary = result.as_ref().map(|(doc, _)| summarize(doc));
    let quality = result.as_ref().map(|(_, report)| {
        let a = &report["assembly"];
        let emitted = a["citations_emitted"].as_u64().unwrap_or(0);
        let verified = a["citations_verified"].as_u64().unwrap_or(0);
        let calls = report["calls"].as_array().cloned().unwrap_or_default();
        Quality {
            pages: report["pages"].as_u64().unwrap_or(0) as usize,
            pages_with_quotes: a["pages_cited"].as_array().map_or(0, Vec::len),
            quotes_verified_percent: if emitted == 0 { 100 } else { (verified * 100 / emitted) as u32 },
            blocks_read: calls.iter().filter(|c| c["ok"] == true && !c["block"].as_str().unwrap_or("").ends_with("+segunda")).count(),
            blocks_total: crate::documents::canonical::contract::SECTIONS.len(),
        }
    });
    let differences = summary.as_ref().map(|s| differences(&reading, s)).unwrap_or_default();
    let brief = calls::brief_raw(&conn, id)?;
    let readable = matches!(reading.status, ReadingStatus::Ready | ReadingStatus::Partial);
    let card = match (&result, &summary) {
        (Some((doc, _)), Some(s)) => Some(card_of(doc, s, brief.clone())),
        _ => None,
    };
    Ok(ReadingDetail { brief_pending: readable && summary.is_some() && brief.is_none(), card, reading, summary, quality, differences })
}

/// How much of the reading goes to the AI to write the brief: the same short text the conversation reads.
const BRIEF_CONTEXT_CHARS: usize = 4000;

/// What the AI reads to write the brief: the short text the conversation reads about the call plus the closing date
/// and the longest duration, which that text leaves out and the brief should say. Nothing else: the figure check
/// compares the brief against exactly this.
fn brief_context(doc: &Value) -> String {
    let summary = summarize(doc);
    let mut text = summary.context_text(BRIEF_CONTEXT_CHARS);
    if text.is_empty() {
        return text;
    }
    for fact in card_of(doc, &summary, None).facts {
        match fact.kind {
            "closing" => text.push_str(&format!("\nFecha de cierre: {}", fact.value)),
            "registration" => text.push_str(&format!("\nFecha de registro o postulación: {}", fact.value)),
            "duration" => text.push_str(&format!("\nDuración máxima: {}", fact.value)),
            _ => {}
        }
    }
    text
}

/// Writes the «en pocas palabras» of a read call, once (ADR-024): the AI is given what the reading understood, and
/// its text is kept only if it brings no figure that text does not have. It is asked again once if it did. If it
/// still does, an empty brief is kept so the same call is not paid for again; if the AI is not available, nothing is
/// kept and a later visit tries again. The card is complete without it: a failure never blocks the person.
pub async fn make_brief(db: &SharedDb, provider: Option<&dyn AiProvider>, id: &str) -> Result<(ReadingDetail, AiStatus), ServiceError> {
    let (context, scanner) = {
        let conn = lock(db)?;
        let row = calls::get(&conn, id)?.ok_or(ServiceError::NotFound)?;
        if !matches!(row.status, ReadingStatus::Ready | ReadingStatus::Partial) {
            return Err(ServiceError::WrongStage);
        }
        let (doc, _) = calls::result(&conn, id)?.ok_or(ServiceError::NotFound)?;
        let cfg = profile_store::scanner_config(&conn)?;
        // already tried (written, or nothing usable came): nothing is asked again
        let context = if calls::brief_raw(&conn, id)?.is_some() { String::new() } else { brief_context(&doc) };
        (context, PublicDocScanner::new(RegexScanner::new(cfg)))
    };
    if context.trim().is_empty() {
        return Ok((detail(db, id)?, AiStatus::Skipped));
    }
    let Some(provider) = provider else { return Ok((detail(db, id)?, AiStatus::NotConfigured)) };

    let ledger = SqliteLedger(db.clone());
    let blocks = vec![format!("Lo que se entendió de la convocatoria:\n{context}")];
    let user = "Escriba el resumen de la convocatoria.".to_string();
    let ask = |user: String| pipeline::run(provider, &scanner, &ledger, AiCall { task: AiTask::CallBrief, context: blocks.clone(), user, project_id: Some(format!("callbrief:{id}")) });
    let bad_figures = |v: &Value| figures::unsupported_figures(&[v["brief"].as_str().unwrap_or("")], &[context.as_str()]);

    let mut result = ask(user.clone()).await;
    if let Ok(v) = &result {
        let bad = bad_figures(v);
        if !bad.is_empty() {
            let again = format!("{user}\n\nEstas cifras no están en el texto: {}. No las use; use solo las cifras que están en el texto.", bad.join(", "));
            if let Ok(v2) = ask(again).await {
                result = Ok(v2);
            }
        }
    }
    let ai = match result {
        Ok(v) => {
            let brief = v["brief"].as_str().unwrap_or("").trim().to_string();
            let usable = !brief.is_empty() && bad_figures(&v).is_empty();
            let conn = lock(db)?;
            calls::set_brief(&conn, id, if usable { &brief } else { "" })?;
            if usable { AiStatus::Used } else { AiStatus::Unavailable }
        }
        Err(e) => AiStatus::from(&e),
    };
    Ok((detail(db, id)?, ai))
}

/// Puts a call that is not read (waiting, failed or partial) back to be read again. `false` if it is being read
/// or is already complete: reading it again would only spend the allowance.
/// The person says this is the right call. `false` if it has not been read yet: there is nothing to confirm.
pub fn confirm(db: &SharedDb, id: &str) -> Result<bool, ServiceError> {
    let conn = lock(db)?;
    calls::get(&conn, id)?.ok_or(ServiceError::NotFound)?;
    Ok(calls::confirm(&conn, id)?)
}

pub fn prepare_retry(db: &SharedDb, id: &str) -> Result<bool, ServiceError> {
    let conn = lock(db)?;
    let row = calls::get(&conn, id)?.ok_or(ServiceError::NotFound)?;
    if matches!(row.status, ReadingStatus::Reading | ReadingStatus::Ready) {
        return Ok(false);
    }
    calls::set_status(&conn, id, ReadingStatus::Waiting, None)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::mock::MockProvider;
    use crate::ai::{AiError, AiResponse, Usage};
    use crate::documents::canonical::contract::{blank_section, canonical_schema, EXTRAS, SECTIONS};
    use crate::storage::open_encrypted;
    use serde_json::Map;
    use std::sync::Mutex;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn setup() -> (tempfile::TempDir, SharedDb) {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, Arc::new(Mutex::new(conn)))
    }

    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office").join(name)).unwrap()
    }

    fn upload(name: &str, bytes: Vec<u8>) -> UploadedFile {
        UploadedFile { name: name.into(), bytes }
    }

    enum IngestOutcome {
        Saved { reading: ReadingRow },
        Unreadable { file: String, reason: Unreadable },
    }

    /// Saves the files as a reading with no project (the first file is the call, the rest are annexes), named
    /// after the first file unless a name is given.
    fn ingest(db: &SharedDb, files: Vec<UploadedFile>, name: Option<String>) -> Result<IngestOutcome, ServiceError> {
        let package = files.into_iter().enumerate().map(|(i, file)| PackageFile { file, role: if i == 0 { FileRole::Main } else { FileRole::Annex } }).collect::<Vec<_>>();
        let first = package.first().map(|f| display_name(&f.file.name)).unwrap_or_default();
        if package.is_empty() {
            return Ok(IngestOutcome::Unreadable { file: String::new(), reason: Unreadable::NoFiles });
        }
        match prepare_package(db, package)? {
            Err((file, reason)) => Ok(IngestOutcome::Unreadable { file, reason }),
            Ok(prepared) => {
                let title = name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| first.rsplit_once('.').map_or(first.clone(), |(stem, _)| stem.to_string()));
                let mut conn = lock(db)?;
                let id = calls::create(&mut conn, &CallMeta { name: title, funder: None, year: None }, &prepared)?;
                Ok(IngestOutcome::Saved { reading: calls::get(&conn, &id)?.ok_or(ServiceError::NotFound)? })
            }
        }
    }

    fn saved(o: IngestOutcome) -> ReadingRow {
        match o {
            IngestOutcome::Saved { reading } => reading,
            IngestOutcome::Unreadable { file, reason } => panic!("not saved: {file} {reason:?}"),
        }
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

    /// A model that answers «no_aparece» to everything, one block per call.
    fn blank(section: &str) -> Value {
        let mut b = blank_section(&canonical_schema(), section);
        strip(&mut b);
        let mut m = Map::new();
        m.insert(section.to_string(), b);
        for e in EXTRAS {
            m.insert(e.to_string(), json!([]));
        }
        Value::Object(m)
    }

    fn reply(v: Value) -> Result<AiResponse, AiError> {
        Ok(AiResponse { value: v, model: "gemini-3.5-flash-lite".into(), usage: Usage { input_tokens: 4000, output_tokens: 600, ..Default::default() } })
    }

    fn plan(replies: Vec<Result<AiResponse, AiError>>) -> ReadPlan {
        ReadPlan { provider: Some(Box::new(MockProvider::new(replies))), embedder: None, inline_schema: false }
    }

    #[test]
    fn a_pdf_with_tables_is_saved_page_by_page_and_named_after_its_file() {
        let (_d, db) = setup();
        let r = saved(ingest(&db, vec![upload("C:\\Mis archivos\\convocatoria-con-tablas.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        assert_eq!((r.name.as_str(), r.status), ("convocatoria-con-tablas", ReadingStatus::Waiting));
        assert_eq!((r.files.len(), r.files[0].name.as_str(), r.files[0].pages), (1, "convocatoria-con-tablas.pdf", 2));
        let pages = calls::pages_of(&lock(&db).unwrap(), &r.id).unwrap();
        assert!(pages[0].1[0].contains("Convocatoria ficticia 2026"));
    }

    #[test]
    fn the_folder_of_an_upload_is_gone_when_it_is_dropped() {
        let scratch = Scratch::new().unwrap();
        let path = scratch.0.clone();
        std::fs::write(path.join("upload.pdf"), b"x").unwrap();
        assert!(path.exists());
        drop(scratch);
        assert!(!path.exists());
    }

    #[test]
    fn word_and_excel_files_come_with_the_call_as_read_only_pages() {
        let (_d, db) = setup();
        let files = vec![
            upload("bases.pdf", fixture("convocatoria-con-tablas.pdf")),
            upload("formato.docx", fixture("plantilla-prueba.docx")),
            upload("cuestionario.xlsx", fixture("cuestionario-prueba.xlsx")),
        ];
        let r = saved(ingest(&db, files, Some("  Apoyos 2027 ".into())).unwrap());
        assert_eq!(r.name, "Apoyos 2027");
        assert_eq!(r.files.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), vec!["bases.pdf", "formato.docx", "cuestionario.xlsx"]);
        assert!(r.files.iter().all(|f| f.pages >= 1));
    }

    #[test]
    fn what_cannot_be_read_says_why_and_saves_nothing() {
        let (_d, db) = setup();
        let why = |files: Vec<UploadedFile>| match ingest(&db, files, None).unwrap() {
            IngestOutcome::Unreadable { reason, .. } => reason,
            IngestOutcome::Saved { .. } => panic!("it was saved"),
        };
        assert_eq!(why(vec![upload("foto.png", vec![1, 2, 3])]), Unreadable::UnsupportedType);
        assert_eq!(why(vec![upload("roto.pdf", b"esto no es un pdf".to_vec())]), Unreadable::Damaged);
        assert_eq!(why(vec![upload("escaneada.pdf", fixture("convocatoria-escaneada.pdf"))]), Unreadable::Scanned);
        assert_eq!(why(vec![]), Unreadable::NoFiles);
        // one bad file among good ones stops the whole call: a call with a document missing gives a wrong picture
        assert_eq!(why(vec![upload("bases.pdf", fixture("convocatoria-con-tablas.pdf")), upload("roto.docx", b"x".to_vec())]), Unreadable::Damaged);
        assert!(calls::list(&lock(&db).unwrap()).unwrap().is_empty());
        assert_eq!(lock(&db).unwrap().query_row("SELECT count(*) FROM document", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    }

    #[test]
    fn identity_data_is_covered_but_the_contact_of_the_funder_is_kept() {
        let scanner = PublicDocScanner::new(RegexScanner::new(Default::default()));
        let text = "Dudas: fundacion@ejemplo.org o 55 1234 5678. Persona atendida: LOPM800101MDFRZN09.";
        let report = scanner.scan(text);
        let out = scanner.redact(text, &report);
        assert!(out.contains("fundacion@ejemplo.org") && out.contains("55 1234 5678"), "{out}");
        assert!(!out.contains("GOMC800101HDFRRL09") && out.contains("[CURP OCULTA]"), "{out}");
    }

    #[tokio::test]
    async fn without_a_key_the_call_stays_saved_and_waiting_for_one() {
        let (_d, db) = setup();
        let r = saved(ingest(&db, vec![upload("bases.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        read_call(&db, &ReadPlan { provider: None, embedder: None, inline_schema: false }, &r.id).await;
        let d = detail(&db, &r.id).unwrap();
        assert_eq!((d.reading.status, d.reading.note.as_deref()), (ReadingStatus::Waiting, Some("not_configured")));
        assert!(d.summary.is_none() && d.quality.is_none());
        assert!(prepare_retry(&db, &r.id).unwrap());
    }

    #[tokio::test]
    async fn a_call_is_read_block_by_block_and_the_person_gets_a_summary_with_its_quality() {
        let (_d, db) = setup();
        let r = saved(ingest(&db, vec![upload("bases.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        // the model finds the name of the call on page 1, quoting it as written
        let page_one = calls::pages_of(&lock(&db).unwrap(), &r.id).unwrap()[0].1[0].clone();
        let quote: String = page_one.lines().find(|l| l.contains("Convocatoria ficticia 2026")).unwrap().trim().to_string();
        let mut replies: Vec<_> = SECTIONS
            .iter()
            .map(|s| {
                let mut a = blank(s);
                if *s == "identidad" {
                    a["identidad"]["nombre"] = json!({ "estado": "encontrado", "valor_texto": "Convocatoria ficticia 2026", "evidencias": [{ "cita": quote, "pagina": 1 }] });
                }
                reply(a)
            })
            .collect();
        // the second look at the pages nobody quoted: one more call per block, finding nothing new
        replies.extend(SECTIONS.iter().map(|s| reply(blank(s))));
        read_call(&db, &plan(replies), &r.id).await;

        let d = detail(&db, &r.id).unwrap();
        assert_eq!((d.reading.status, d.reading.note.as_deref()), (ReadingStatus::Ready, None), "{:?}", d.reading);
        assert_eq!(d.summary.as_ref().unwrap().title.as_deref(), Some("Convocatoria ficticia 2026"));
        let q = d.quality.unwrap();
        assert_eq!((q.blocks_read, q.blocks_total, q.quotes_verified_percent), (SECTIONS.len(), SECTIONS.len(), 100));
        assert!(q.pages_with_quotes >= 1 && q.pages == 2);
        // what the reading cost is in the usage log, labelled with the call
        let spent: i64 = lock(&db).unwrap().query_row("SELECT count(*) FROM ai_usage WHERE project_id=?1", [format!("call:{}", r.id)], |r| r.get(0)).unwrap();
        assert!(spent >= SECTIONS.len() as i64);
        // the audit log says it happened and nothing about what was read
        let logged: String = lock(&db).unwrap().query_row("SELECT details_json FROM audit_log WHERE event='call.read'", [], |r| r.get(0)).unwrap();
        assert!(!logged.contains("ficticia"));
        // a call that is complete is not read again
        assert!(!prepare_retry(&db, &r.id).unwrap());
    }

    #[tokio::test]
    async fn when_some_blocks_fail_the_call_is_partial_and_when_the_allowance_is_gone_it_waits() {
        let (_d, db) = setup();
        let r = saved(ingest(&db, vec![upload("bases.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        let mut replies: Vec<Result<AiResponse, AiError>> = SECTIONS.iter().map(|s| reply(blank(s))).collect();
        replies[3] = Err(AiError::Refused);
        replies.extend(SECTIONS.iter().map(|s| reply(blank(s))));
        read_call(&db, &plan(replies), &r.id).await;
        let d = detail(&db, &r.id).unwrap();
        assert_eq!((d.reading.status, d.reading.note.as_deref()), (ReadingStatus::Partial, Some("unavailable")));
        assert_eq!(d.quality.unwrap().blocks_read, SECTIONS.len() - 1);
        assert!(prepare_retry(&db, &r.id).unwrap(), "a partial reading can be tried again");

        let r2 = saved(ingest(&db, vec![upload("otra.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        read_call(&db, &plan(vec![Err(AiError::QuotaReached)]), &r2.id).await;
        let d2 = detail(&db, &r2.id).unwrap();
        assert_eq!((d2.reading.status, d2.reading.note.as_deref()), (ReadingStatus::Waiting, Some("quota_reached")));
        assert!(d2.summary.is_none());

        let r3 = saved(ingest(&db, vec![upload("tercera.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        read_call(&db, &plan(vec![Err(AiError::Auth)]), &r3.id).await;
        assert_eq!(detail(&db, &r3.id).unwrap().reading.status, ReadingStatus::Failed);
    }

    /// A read call: its reading is `ready` and holds a canonical document with a funder, a maximum amount and a closing date.
    fn read_call_with_document() -> (tempfile::TempDir, SharedDb, String) {
        let (d, db) = setup();
        let r = saved(ingest(&db, vec![upload("bases.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        let mut doc = crate::test_support::canonical_doc();
        doc["temporalidad"] = json!({ "hitos": { "estado": "encontrado", "hitos": [
            { "tipo": "cierre", "etiqueta": "Cierre", "inicio_texto": "23 de mayo de 2027", "evidencias": [{ "cita": "Cierre: 23 de mayo de 2027", "pagina": 3, "documento": "bases.pdf" }], "normalizado": { "inicio": { "fecha": "2027-05-23", "precision": "dia" } } }
        ] } });
        calls::finish(&lock(&db).unwrap(), &r.id, ReadingStatus::Ready, None, Some(&doc), &json!({})).unwrap();
        (d, db, r.id)
    }

    fn brief_reply(text: &str) -> Result<AiResponse, AiError> {
        reply(json!({ "brief": text }))
    }

    #[test]
    fn the_person_gets_a_card_first_and_the_detail_is_kept_for_looking_things_up() {
        let (_d, db, id) = read_call_with_document();
        let d = detail(&db, &id).unwrap();
        let card = d.card.expect("a read call has a card");
        assert_eq!(card.funder.as_deref(), Some("Fundación Ficticia"));
        assert_eq!(card.facts.iter().map(|f| (f.kind, f.value.as_str())).collect::<Vec<_>>(), [("max_amount", "$250,000"), ("closing", "23 de mayo de 2027")]);
        assert!(card.brief.is_none() && d.brief_pending, "nobody has written the «en pocas palabras» yet");
        assert!(d.summary.is_some(), "the structured detail is still there, for consulting");
        // a call that has not been read has neither
        let waiting = saved(ingest(&db, vec![upload("otra.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        let w = detail(&db, &waiting.id).unwrap();
        assert!(w.card.is_none() && !w.brief_pending);
    }

    #[tokio::test]
    async fn the_brief_is_written_once_from_what_the_reading_understood_and_kept() {
        let (_d, db, id) = read_call_with_document();
        let p = MockProvider::new(vec![brief_reply("Es una convocatoria de la Fundación Ficticia. Da hasta $250,000 por proyecto y se puede participar hasta el 23 de mayo de 2027.")]);
        let (d, ai) = make_brief(&db, Some(&p), &id).await.unwrap();
        assert_eq!(ai, AiStatus::Used);
        assert!(d.card.unwrap().brief.unwrap().contains("$250,000") && !d.brief_pending);
        let seen = p.requests();
        assert_eq!(seen.len(), 1);
        let context = seen[0].context.join("\n");
        assert!(context.contains("Monto máximo por proyecto: $250,000") && context.contains("Fecha de cierre: 23 de mayo de 2027"), "{context}");
        assert!(!seen[0].system.contains("única fuente de hechos sobre la institución"), "the call is not about the institution");
        // it is not asked again, and nothing is spent
        let again = MockProvider::new(vec![brief_reply("Otra cosa.")]);
        let (d2, ai2) = make_brief(&db, Some(&again), &id).await.unwrap();
        assert_eq!(ai2, AiStatus::Skipped);
        assert!(again.requests().is_empty() && d2.card.unwrap().brief.unwrap().starts_with("Es una convocatoria"));
        // its cost goes under its own label, apart from the reading's
        let spent: i64 = lock(&db).unwrap().query_row("SELECT count(*) FROM ai_usage WHERE project_id=?1 AND task='call.brief'", [format!("callbrief:{id}")], |r| r.get(0)).unwrap();
        assert_eq!(spent, 1);
    }

    #[tokio::test]
    async fn a_brief_with_an_invented_figure_is_asked_again_once_and_if_it_persists_nothing_is_kept_and_not_paid_for_again() {
        let (_d, db, id) = read_call_with_document();
        let p = MockProvider::new(vec![brief_reply("Da hasta $900,000."), brief_reply("Da hasta $250,000.")]);
        let (d, ai) = make_brief(&db, Some(&p), &id).await.unwrap();
        assert_eq!(ai, AiStatus::Used);
        assert_eq!(p.requests().len(), 2);
        assert!(p.requests()[1].user.contains("Estas cifras no están en el texto: 900000"));
        assert_eq!(d.card.unwrap().brief.as_deref(), Some("Da hasta $250,000."));

        let (_d2, db2, id2) = read_call_with_document();
        let p2 = MockProvider::new(vec![brief_reply("Da hasta $900,000."), brief_reply("Da hasta $800,000.")]);
        let (d2, ai2) = make_brief(&db2, Some(&p2), &id2).await.unwrap();
        assert_eq!(ai2, AiStatus::Unavailable);
        assert!(d2.card.as_ref().unwrap().brief.is_none(), "an invented figure never reaches the person");
        assert!(!d2.brief_pending, "it was tried: the same call is not paid for again");
        let third = MockProvider::new(vec![brief_reply("Da hasta $250,000.")]);
        make_brief(&db2, Some(&third), &id2).await.unwrap();
        assert!(third.requests().is_empty());
    }

    #[tokio::test]
    async fn without_the_ai_the_card_is_still_there_and_a_later_visit_tries_again() {
        let (_d, db, id) = read_call_with_document();
        assert_eq!(make_brief(&db, None, &id).await.unwrap().1, AiStatus::NotConfigured);
        let down = MockProvider::new(vec![Err(AiError::Auth)]);
        let (d, ai) = make_brief(&db, Some(&down), &id).await.unwrap();
        assert_eq!(ai, AiStatus::KeyRejected);
        assert!(d.card.is_some() && d.brief_pending, "nothing was kept: it can be tried again");
        // a call that has not been read cannot have one
        let waiting = saved(ingest(&db, vec![upload("otra.pdf", fixture("convocatoria-con-tablas.pdf"))], None).unwrap());
        assert!(matches!(make_brief(&db, None, &waiting.id).await, Err(ServiceError::WrongStage)));
    }

    #[tokio::test]
    async fn reading_the_call_again_throws_the_brief_away_because_it_summarized_another_document() {
        let (_d, db, id) = read_call_with_document();
        let p = MockProvider::new(vec![brief_reply("Da hasta $250,000.")]);
        make_brief(&db, Some(&p), &id).await.unwrap();
        // a reading that produced nothing keeps the brief
        calls::finish(&lock(&db).unwrap(), &id, ReadingStatus::Waiting, Some("offline"), None, &json!({})).unwrap();
        assert!(calls::brief_raw(&lock(&db).unwrap(), &id).unwrap().is_some());
        // one that produced a document makes it stale
        calls::finish(&lock(&db).unwrap(), &id, ReadingStatus::Ready, None, Some(&crate::test_support::canonical_doc()), &json!({})).unwrap();
        assert!(calls::brief_raw(&lock(&db).unwrap(), &id).unwrap().is_none());
        assert!(detail(&db, &id).unwrap().brief_pending);
    }

    fn setup_with_profile() -> (tempfile::TempDir, SharedDb) {
        use crate::domain::profile::*;
        let (d, db) = setup();
        {
            let mut c = db.lock().unwrap();
            let input = ProfileInput {
                institution: InstitutionInput { name: "Asilo Ficticio".into(), ..Default::default() },
                population: vec![PopulationGroupInput { label: "Adultos".into(), count: 18, ..Default::default() }],
                ..Default::default()
            };
            profile_store::save(&mut c, &input).unwrap();
            profile_store::confirm(&mut c).unwrap();
        }
        (d, db)
    }

    fn package(files: Vec<(&str, Vec<u8>, FileRole)>) -> Vec<PackageFile> {
        files.into_iter().map(|(n, b, role)| PackageFile { file: upload(n, b), role }).collect()
    }

    fn count(db: &SharedDb, table: &str) -> i64 {
        db.lock().unwrap().query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn a_project_is_born_from_its_call_with_the_call_first_and_what_the_person_said() {
        let (_d, db) = setup_with_profile();
        let files = package(vec![
            ("guia.docx", fixture("plantilla-prueba.docx"), FileRole::Guide),
            ("bases.pdf", fixture("convocatoria-con-tablas.pdf"), FileRole::Main),
        ]);
        let NewProjectOutcome::Created { project, reading } = create_project_from_call(&db, files, "  Apoyos 2027 ", Some(" Fundación Ficticia "), Some(2027), None).unwrap() else {
            panic!("not created")
        };
        assert_eq!((project.kind.as_str(), project.stage, project.title.as_str()), ("call", Stage::CallSelection, "Apoyos 2027"));
        assert_eq!(project.call_reading_id.as_deref(), Some(reading.id.as_str()));
        assert_eq!(project.initial_request.as_deref(), Some("La institución quiere postular a la convocatoria «Apoyos 2027» de Fundación Ficticia (2027)."));
        assert_eq!((reading.name.as_str(), reading.funder.as_deref(), reading.year), ("Apoyos 2027", Some("Fundación Ficticia"), Some(2027)));
        // the call goes first whatever the order of the upload, and each file keeps the character the person gave it
        assert_eq!(reading.files.iter().map(|f| (f.name.as_str(), f.role)).collect::<Vec<_>>(), vec![("bases.pdf", FileRole::Main), ("guia.docx", FileRole::Guide)]);
        assert_eq!(reading.status, ReadingStatus::Waiting);
    }

    #[test]
    fn nothing_is_saved_when_the_profile_the_files_or_the_text_are_not_right() {
        let (_d, db) = setup();
        let good = || package(vec![("bases.pdf", fixture("convocatoria-con-tablas.pdf"), FileRole::Main)]);
        // no confirmed profile
        assert!(matches!(create_project_from_call(&db, good(), "X", None, None, None), Err(ServiceError::Stage(_))));
        let (_d2, db) = setup_with_profile();
        let unreadable = |files: Vec<PackageFile>| match create_project_from_call(&db, files, "X", None, None, None).unwrap() {
            NewProjectOutcome::Unreadable { reason, .. } => reason,
            _ => panic!("it was created"),
        };
        assert_eq!(unreadable(package(vec![("roto.pdf", b"no es pdf".to_vec(), FileRole::Main)])), Unreadable::Damaged);
        assert_eq!(unreadable(vec![]), Unreadable::NoFiles);
        // exactly one file is the call itself
        assert_eq!(unreadable(package(vec![("a.pdf", fixture("convocatoria-con-tablas.pdf"), FileRole::Annex)])), Unreadable::NoMainFile);
        assert_eq!(unreadable(package(vec![("a.pdf", fixture("convocatoria-con-tablas.pdf"), FileRole::Main), ("b.pdf", fixture("convocatoria-con-tablas.pdf"), FileRole::Main)])), Unreadable::NoMainFile);
        // a bad file among good ones stops the whole package
        assert_eq!(unreadable(package(vec![("a.pdf", fixture("convocatoria-con-tablas.pdf"), FileRole::Main), ("b.docx", b"x".to_vec(), FileRole::Annex)])), Unreadable::Damaged);
        assert!(matches!(create_project_from_call(&db, good(), "   ", None, None, None), Err(ServiceError::EmptyText)));
        assert!(matches!(create_project_from_call(&db, good(), "X", None, Some(26), None), Err(ServiceError::InvalidYear)));
        // a name that looks like data of a person is held for the person to decide
        let held = create_project_from_call(&db, good(), "Para LOPM800101MDFRZN09", None, None, None).unwrap();
        assert!(matches!(held, NewProjectOutcome::Quarantine { .. }));
        for table in ["project", "call_reading", "call_reading_file", "document"] {
            assert_eq!(count(&db, table), 0, "{table}");
        }
    }

    fn summary_of(funder: Option<&str>, edition: Option<&str>, title: Option<&str>) -> CallSummary {
        CallSummary {
            document_kind: None,
            title: title.map(String::from),
            funder: funder.map(String::from),
            edition: edition.map(String::from),
            objective: None,
            dates: vec![],
            amounts: vec![],
            groups: vec![],
            conflicts: vec![],
            doubts: vec![],
            missing: vec![],
        }
    }

    fn said(funder: Option<&str>, year: Option<i64>) -> ReadingRow {
        ReadingRow { id: "r".into(), name: "N".into(), funder: funder.map(String::from), year, status: ReadingStatus::Ready, note: None, created_at: String::new(), finished_at: None, files: vec![], confirmed_at: None }
    }

    #[test]
    fn what_the_person_wrote_is_compared_with_what_was_read_and_only_a_real_clash_is_flagged() {
        let read = summary_of(Some("Fundación Nacional Monte de Piedad, I.A.P."), Some("Convocatoria 2026"), None);
        // the same funder written shorter, and the year that the documents give: nothing to say
        assert!(differences(&said(Some("Nacional Monte de Piedad"), Some(2026)), &read).is_empty());
        // another funder and another year: both are flagged, with what each side says
        let d = differences(&said(Some("Fundación Alsea"), Some(2025)), &read);
        assert_eq!(d.iter().map(|d| (d.field, d.said.as_str(), d.read.as_str())).collect::<Vec<_>>(), vec![("funder", "Fundación Alsea", "Fundación Nacional Monte de Piedad, I.A.P."), ("year", "2025", "2026")]);
        // when the documents do not say a year or a funder there is nothing to compare
        assert!(differences(&said(Some("Fundación Alsea"), Some(2025)), &summary_of(None, None, Some("Apoyos a casas hogar"))).is_empty());
        // and when the person wrote nothing, neither
        assert!(differences(&said(None, None), &read).is_empty());
    }

    #[test]
    fn deleting_the_project_leaves_nothing_of_its_call_and_other_projects_are_not_touched() {
        let (_d, db) = setup_with_profile();
        let make = |name: &str| match create_project_from_call(&db, package(vec![("bases.pdf", fixture("convocatoria-con-tablas.pdf"), FileRole::Main)]), name, None, None, None).unwrap() {
            NewProjectOutcome::Created { project, .. } => project,
            _ => panic!("not created"),
        };
        let (a, b) = (make("Uno"), make("Dos"));
        assert_eq!((count(&db, "project"), count(&db, "call_reading"), count(&db, "document")), (2, 2, 2));
        assert!(projects::delete_project(&mut db.lock().unwrap(), &a.id).unwrap());
        assert_eq!((count(&db, "project"), count(&db, "call_reading"), count(&db, "document")), (1, 1, 1));
        assert_eq!(count(&db, "document_chunk") > 0, true);
        let left: String = db.lock().unwrap().query_row("SELECT id FROM project", [], |r| r.get(0)).unwrap();
        assert_eq!(left, b.id);
        assert!(!projects::delete_project(&mut db.lock().unwrap(), &a.id).unwrap());
    }

    /// Uploads the packages of a measurement manifest as a person would, without calling any service:
    /// how long it takes, how many pages and how much was covered. `CIMIENTO_CANON_MANIFEST` names the manifest.
    ///   cargo test call_ingest_real -- --ignored --nocapture
    #[test]
    #[ignore]
    fn call_ingest_real() {
        let path = std::env::var("CIMIENTO_CANON_MANIFEST").expect("CIMIENTO_CANON_MANIFEST");
        let m: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let (_d, db) = setup();
        for p in m["packages"].as_array().unwrap() {
            let files: Vec<UploadedFile> = p["docs"].as_array().unwrap().iter().map(|d| {
                let d = d.as_str().unwrap();
                UploadedFile { name: d.to_string(), bytes: std::fs::read(d).unwrap() }
            }).collect();
            let started = std::time::Instant::now();
            match ingest(&db, files, Some(p["name"].as_str().unwrap().to_string())).unwrap() {
                IngestOutcome::Saved { reading } => {
                    let conn = db.lock().unwrap();
                    let covered: i64 = conn.query_row("SELECT COALESCE(SUM(redactions_count),0) FROM document d JOIN call_reading_file f ON f.document_id=d.id WHERE f.reading_id=?1", [&reading.id], |r| r.get(0)).unwrap();
                    println!("{}: {} archivos, {} páginas, {} datos tapados, {:.1} s", reading.name, reading.files.len(), reading.files.iter().map(|f| f.pages).sum::<i64>(), covered, started.elapsed().as_secs_f64());
                }
                IngestOutcome::Unreadable { file, reason } => println!("{}: NO se tomó {file}: {reason:?}", p["name"]),
            }
        }
    }
}
