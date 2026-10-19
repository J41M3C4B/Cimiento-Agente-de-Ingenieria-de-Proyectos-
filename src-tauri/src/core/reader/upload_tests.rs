use super::*;
use crate::common::forms::Values;
use crate::core::archive::search::tests::load_record;
use crate::core::profile::forms::save;
use crate::core::profile::service::SaveProfileOutcome;
use crate::core::reader::fill::tests::reference;
use crate::core::agents::proposals;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// An institution with only its name written, and the fictitious Expediente of `fixtures/expediente` in its drawer.
fn db() -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let identity: Values = serde_json::from_value(json!({ "institution.name": "Casa Ficticia" })).unwrap();
    assert!(matches!(save(&mut c, "institution.identity", identity, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    load_record(&c);
    (dir, Arc::new(Mutex::new(c)))
}

/// The documents of the drawer: id and the name of its file.
fn documents(conn: &Connection) -> Vec<(String, String)> {
    let mut stmt = conn.prepare("SELECT id, display_name FROM document WHERE drawer = 'record' ORDER BY display_name").unwrap();
    let v = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    v
}

/// Every field the AI may see, with its window: what is measured, filled or not.
fn every_field() -> Vec<(&'static str, &'static FieldSpec)> {
    FILLABLE
        .iter()
        .flat_map(|(form, _, _)| crate::core::agents::capturist::open_fields(crate::core::institution::forms::form(form).unwrap()).map(move |f| (*form, f)))
        .collect()
}

/// The files `lector.json` says write each field.
fn expected_files(reference: &serde_json::Map<String, Value>, form: &str, field: &str) -> Vec<String> {
    let mut files: Vec<String> =
        reference[form][field]["paginas"].as_array().into_iter().flatten().filter_map(|p| p[0].as_str().map(str::to_string)).collect();
    files.dedup();
    files
}

/// For each document, the `score` of its best fragment for each field.
fn best_words(conn: &Connection) -> Vec<(String, Vec<(&'static str, &'static str, usize)>)> {
    documents(conn)
        .into_iter()
        .map(|(id, name)| {
            let chunks = search::document_chunks(conn, Drawer::Record, &id).unwrap();
            let fields = every_field().into_iter().map(|(form, f)| (form, f.id, chunks.iter().map(|c| score(f, &c.2)).max().unwrap_or(0))).collect();
            (name, fields)
        })
        .collect()
}

/// The measure of ADR-042 §5, in dry: each document of the fictitious Expediente against the fields `lector.json`
/// says it writes, with each threshold. «Hallados»: the fields a document writes that it is asked about; «de más»:
/// the ones asked that it does not write (they only make the call longer).
#[test]
#[ignore]
fn upload_detection_measure() {
    let (_d, db) = db();
    let conn = db.lock().unwrap();
    let reference = reference();
    let best = best_words(&conn);
    let expected: usize = best.iter().map(|(name, fields)| fields.iter().filter(|(form, f, _)| expected_files(&reference, form, f).contains(name)).count()).sum();
    println!("umbral  hallados  de más  campos por archivo");
    for t in 1..=5 {
        let (mut hit, mut extra, mut asked) = (0, 0, 0);
        for (name, fields) in &best {
            for (form, f, n) in fields {
                let writes = expected_files(&reference, form, f).contains(name);
                if *n >= t {
                    asked += 1;
                    if writes { hit += 1 } else { extra += 1 }
                }
            }
        }
        println!("{t:>6}  {hit:>3} de {expected}  {extra:>6}  {:>6.1}", asked as f64 / best.len() as f64);
    }
    println!("\nCon el umbral {SPEAKS_OF}, por archivo (✓ lo escribe y se pregunta; ✗ lo escribe y no se pregunta; + se pregunta de más):");
    for (name, fields) in &best {
        let mut line = Vec::new();
        for (form, f, n) in fields {
            let writes = expected_files(&reference, form, f).contains(name);
            match (writes, *n >= SPEAKS_OF) {
                (true, true) => line.push(format!("✓{}", &f["institution.".len()..])),
                (true, false) => line.push(format!("✗{}({n})", &f["institution.".len()..])),
                (false, true) => line.push(format!("+{}", &f["institution.".len()..])),
                _ => {}
            }
        }
        println!("{name}: {}", line.join(" "));
    }
}

/// ADR-042 §5: every field `lector.json` says a document writes is asked when that file is kept, but the name and
/// what the institution does, which the first start always asks (they are never empty when a file is uploaded).
#[test]
fn each_document_is_asked_about_the_fields_it_writes() {
    let (_d, db) = db();
    let conn = db.lock().unwrap();
    let reference = reference();
    let essential = |f: &str| crate::core::institution::forms::field(f).is_some_and(|s| s.essential);
    for (name, fields) in best_words(&conn) {
        for (form, f, n) in fields {
            if expected_files(&reference, form, f).contains(&name) && !essential(f) {
                assert!(n >= SPEAKS_OF, "{name} writes {f} and is not asked about it ({n})");
            }
        }
    }
}

#[test]
fn a_fragment_speaks_of_a_field_by_its_words_or_by_the_name_of_an_option() {
    let state = crate::core::institution::forms::field("institution.state").unwrap();
    assert!(score(state, "El domicilio de la institución será la Ciudad de México.") >= SPEAKS_OF);
    assert!(score(state, "Ubicado en la Ciudad de México") >= SPEAKS_OF, "the name of an option alone");
    assert!(score(state, "La ciudad tiene muchos parques.") < SPEAKS_OF);
    let donee = crate::core::institution::forms::field("institution.authorized_donee").unwrap();
    assert!(!names_an_option(donee, "No se presentó ningún aviso."), "a short name («No») says nothing");
}

fn reply(answer: Value) -> Result<crate::ai::AiResponse, crate::ai::AiError> {
    Ok(crate::ai::AiResponse {
        value: json!({ "tool": null, "args": null, "answer": answer }),
        model: "mock".into(),
        usage: crate::ai::Usage { input_tokens: 2_000, output_tokens: 200, ..Default::default() },
    })
}

fn document_named(conn: &Connection, name: &str) -> String {
    documents(conn).into_iter().find(|(_, n)| n == name).unwrap().0
}

/// One call for the file just kept, with only the passages of that file and the empty fields it speaks of; what the
/// code accepts waits as a suggestion with its document and page, in the window of its field.
#[tokio::test]
async fn a_file_kept_is_asked_once_about_the_empty_fields_it_speaks_of() {
    let (_d, db) = db();
    let report = document_named(&db.lock().unwrap(), "informe-anual-2025.txt");
    let (fields, passages) = {
        let conn = db.lock().unwrap();
        detect(&conn, &report, &asked(&conn, Some("annual_report")).unwrap()).unwrap()
    };
    assert!(passages.iter().all(|p| p.document == "informe-anual-2025.txt"), "only the file just kept");
    let ids: Vec<&str> = fields.iter().map(|(_, f)| f.id).collect();
    assert!(ids.contains(&"institution.staff_paid_estimate") && ids.contains(&"institution.modalities"), "{ids:?}");
    assert!(!ids.contains(&"institution.name"), "a written field is not asked");
    let staff = passages.iter().find(|p| p.text.contains("31 personas contratadas")).unwrap();
    let answer = json!({
        "proposals": [
            { "field": "institution.staff_paid_estimate", "value": 31, "passage": staff.id, "quote": "31 personas contratadas" },
            { "field": "institution.staff_volunteer_estimate", "value": 18, "passage": staff.id, "quote": "18 personas voluntarias" },
            // the AI counted: 48 + 22 is not in the words
            { "field": "institution.served_estimate", "value": 70, "passage": staff.id, "quote": "Atendimos a 48 personas residentes" }
        ],
        "note": ""
    });
    let provider = crate::ai::mock::MockProvider::new(vec![reply(answer)]);
    let out = after_upload(&db, Some(&provider), &report).await.unwrap();
    let UploadFill::Proposed { fields, dropped } = out else { panic!("{out:?}") };
    assert_eq!(fields, vec![
        ("institution.capacity".to_string(), "institution.staff_paid_estimate".to_string()),
        ("institution.capacity".to_string(), "institution.staff_volunteer_estimate".to_string()),
    ]);
    assert_eq!(dropped, 1);
    assert_eq!(provider.requests().len(), 1, "one call per file");
    let sent: String = provider.requests().iter().map(|r| r.context.join("\n")).collect();
    assert!(!sent.contains("informe-anual-2025.txt"), "the AI sees the type of the document, never the name of its file");
    let conn = db.lock().unwrap();
    let waiting = proposals::pending(&conn, "institution.capacity", None).unwrap();
    let paid = waiting.iter().find(|p| p.field_id == "institution.staff_paid_estimate").unwrap();
    assert_eq!((paid.value.clone(), paid.document.as_deref(), paid.page), (json!(31), Some("informe-anual-2025.txt"), Some(1)));
    // the same file again: those fields wait for a suggestion already, so they are not asked again
    let again = asked(&conn, Some("annual_report")).unwrap();
    assert!(!again.iter().any(|(_, f)| f.id == "institution.staff_paid_estimate"));
}

/// Nothing is called with no AI, with a file that speaks of no empty field, or with a file without text.
#[tokio::test]
async fn nothing_is_called_without_ai_or_without_a_field_to_ask() {
    let (_d, db) = db();
    let (report, opinion) = {
        let conn = db.lock().unwrap();
        (document_named(&conn, "informe-anual-2025.txt"), document_named(&conn, "cluni.txt"))
    };
    let out = after_upload(&db, None, &report).await.unwrap();
    assert!(matches!(out, UploadFill::Unavailable { reason: "not_configured" }), "{out:?}");
    {
        // the CLUNI speaks of the CLUNI and the name: both written, nothing to ask
        let mut conn = db.lock().unwrap();
        let legal: Values = serde_json::from_value(json!({ "institution.cluni": "yes" })).unwrap();
        assert!(matches!(save(&mut conn, "institution.legal", legal, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    }
    let provider = crate::ai::mock::MockProvider::new(vec![]);
    let out = after_upload(&db, Some(&provider), &opinion).await.unwrap();
    assert!(matches!(out, UploadFill::Nothing { reason: "nothing_found" }), "{out:?}");
    let out = after_upload(&db, Some(&provider), "no-such-document").await.unwrap();
    assert!(matches!(out, UploadFill::Nothing { reason: "not_readable" }), "{out:?}");
    assert!(provider.requests().is_empty(), "no call");
}

/// The provider as it is, keeping what the model answered (before the code checks it).
struct Recording<'a> {
    inner: &'a dyn crate::ai::AiProvider,
    log: Mutex<Vec<Value>>,
}

#[async_trait::async_trait]
impl crate::ai::AiProvider for Recording<'_> {
    async fn complete(&self, req: &crate::ai::AiRequest) -> Result<crate::ai::AiResponse, crate::ai::AiError> {
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
    fn model_name(&self, tier: crate::ai::ModelTier) -> String {
        self.inner.model_name(tier)
    }
}

/// The real run of ADR-042 §5, opt-in and capped: the 13 documents of the fictitious Expediente kept one after the
/// other, as a person uploads them, with only the name written: the rules of each file leave their suggestions and
/// then `after_upload` asks about the empty fields that file speaks of (at most 13 calls with the cheapest model, never
/// more than 14 a minute). For every field of `lector.json`: what waits at the end, whether it is right and its page;
/// then calls, tokens (and how many came from the cache), cost and time per file. Only fictitious text goes out. Its
/// whole output goes to a file:
///   cargo test upload_live -- --ignored --nocapture > upload_live.txt
#[tokio::test]
#[ignore]
async fn upload_live() {
    use crate::ai::ModelTier;
    use crate::common::forms::is_filled;
    use crate::core::archive::search::tests::fixture_pages;
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    let identity: Values = serde_json::from_value(json!({ "institution.name": "Casa Ficticia" })).unwrap();
    assert!(matches!(save(&mut c, "institution.identity", identity, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    let db: SharedDb = Arc::new(Mutex::new(c));
    let provider = {
        let c = db.lock().unwrap();
        let cfg = crate::ai::settings::load(&c).unwrap();
        let key = crate::storage::get_api_key(cfg.provider.as_str()).unwrap().expect("save the key in the app first");
        println!("Proveedor: {}  modelo ligero: {}", cfg.provider.as_str(), cfg.model_for(ModelTier::Light));
        crate::ai::paced::provider(crate::ai::build_provider(&cfg, key))
    };
    let recording = Recording { inner: provider.as_ref(), log: Mutex::new(Vec::new()) };
    let usage = |db: &SharedDb| -> (i64, i64, i64, i64, f64) {
        db.lock()
            .unwrap()
            .query_row(
                "SELECT count(*), coalesce(sum(input_tokens),0), coalesce(sum(cached_tokens),0), coalesce(sum(output_tokens),0), coalesce(sum(estimated_cost_mxn),0) FROM ai_usage WHERE task = 'agent.reader'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap()
    };
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente");
    let mut files: Vec<_> = std::fs::read_dir(&folder).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "txt")).collect();
    files.sort();
    let started_all = std::time::Instant::now();
    for f in &files {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        let pages = fixture_pages(&std::fs::read_to_string(f).unwrap());
        let doc = {
            // kept as the app keeps it: in the drawer, its type by its words, and the suggestions of its rules
            let conn = db.lock().unwrap();
            let doc = crate::core::archive::storage::add_paged_document(&conn, Drawer::Record, &name, "application/pdf", &pages, 0, &Default::default()).unwrap();
            let reading = crate::core::reader::read(&pages, None);
            crate::core::archive::storage::set_class(&conn, &doc, reading.kind.and_then(crate::core::reader::kinds::kind).map(|k| k.class), reading.kind, "computed").unwrap();
            crate::core::reader::propose(&conn, &doc, reading.kind, &reading.found).unwrap();
            doc
        };
        let before = usage(&db);
        let started = std::time::Instant::now();
        recording.log.lock().unwrap().clear();
        let out = after_upload(&db, Some(&recording), &doc).await.unwrap();
        let after = usage(&db);
        println!(
            "\n== {name}: {:.1} s  llamadas {}  entrada {} (del caché {})  salida {}  costo ${:.4} MXN\n   {out:?}",
            started.elapsed().as_secs_f64(),
            after.0 - before.0,
            after.1 - before.1,
            after.2 - before.2,
            after.3 - before.3,
            after.4 - before.4
        );
        for answer in recording.log.lock().unwrap().iter() {
            println!("   · el modelo contestó {answer}");
        }
    }
    let reference = reference();
    let (mut right, mut wrong, mut missed, mut invented, mut pages_ok, mut graded) = (0, 0, 0, 0, 0, 0);
    println!("\nLo que espera en «Mi institución» al final:");
    for (form, _, name) in FILLABLE {
        println!("\n-- {name}");
        let conn = db.lock().unwrap();
        let waiting = proposals::pending(&conn, form, None).unwrap();
        let saved = crate::core::profile::forms::saved_values(&conn, form).unwrap();
        for (field, expected) in reference[*form].as_object().unwrap() {
            if is_filled(saved.get(field.as_str())) {
                println!("  {field:<40} ya escrito");
                continue;
            }
            let got = waiting.iter().find(|p| &p.field_id == field);
            let (verdict, page) = crate::core::reader::fill::tests::grade(expected, got);
            let shown = got.map_or("—".to_string(), |p| format!("{}  ({}, p{}, {})", p.value, p.document.as_deref().unwrap_or("?"), p.page.unwrap_or(0), p.agent));
            println!("  {field:<40} {verdict:<12} {}  {shown}", if got.is_some() { if page { "página bien" } else { "PÁGINA MAL" } } else { "" });
            match verdict {
                "bien" => right += 1,
                "MAL" => wrong += 1,
                "FALTÓ" => missed += 1,
                "INVENTÓ" => invented += 1,
                _ => {}
            }
            if got.is_some() && expected["libre"].as_bool() != Some(true) {
                graded += 1;
                pages_ok += usize::from(page);
            }
        }
    }
    let total = usage(&db);
    println!("\nTOTAL: bien {right}  mal {wrong}  faltó {missed}  inventó {invented}  páginas bien {pages_ok} de {graded}");
    println!(
        "Llamadas {}  entrada {} (del caché {})  salida {}  costo ${:.4} MXN  en {:.0} s",
        total.0,
        total.1,
        total.2,
        total.3,
        total.4,
        started_all.elapsed().as_secs_f64()
    );
}

/// A suggestion that waits holds its field only against a file with no more say on it (ADR-038): the services read
/// from the SAT's opinion do not stop the annual report from being asked. What the person wants known is never asked.
#[test]
fn a_file_with_more_say_is_asked_even_if_a_suggestion_waits() {
    let (_d, db) = db();
    let conn = db.lock().unwrap();
    let opinion = document_named(&conn, "opinion-cumplimiento.txt");
    let new = crate::core::agents::proposals::NewProposal {
        form_id: "institution.identity".into(),
        record_id: None,
        field_id: "institution.services".into(),
        value: json!("Casa de reposo"),
        origin: crate::core::agents::proposals::Origin::Document,
        source_ref: Some(json!({ "document_id": opinion, "page": 1, "rank": crate::core::reader::authority("institution.services", Some("tax_compliance_opinion")) }).to_string()),
        agent: crate::core::reader::AGENT,
    };
    crate::core::agents::proposals::propose(&conn, new).unwrap();
    let ids = |kind: &str| asked(&conn, Some(kind)).unwrap().into_iter().map(|(_, f)| f.id).collect::<Vec<_>>();
    assert!(ids("annual_report").contains(&"institution.services"), "the report has more say");
    assert!(!ids("tax_compliance_opinion").contains(&"institution.services"), "the same say: it waits already");
    assert!(!ids("annual_report").contains(&"institution.notes"), "what the person wants known is never asked");
}
