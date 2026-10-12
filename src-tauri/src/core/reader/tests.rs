//! The Reader from upload to «Mi institución» (IA6): the kind by rules, the data as suggestions with their citation,
//! the protected data read before the scanner covers them and forgotten once the person decides.

use super::*;
use crate::core::archive::search::tests::fixture_pages;
use crate::core::archive::service::{add_text_document, AddDocumentOutcome};
use crate::core::archive::storage as docs;
use crate::scanner::guard::Decision;
use serde_json::{json, Value};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn conn() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    (dir, c)
}

fn text_of(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente").join(name);
    fixture_pages(&std::fs::read_to_string(path).unwrap()).join("\n\n")
}

fn upload(c: &mut Connection, name: &str, file: &str, category: Option<&str>) -> docs::DocumentSummary {
    let text = text_of(file);
    match add_text_document(c, "internal", name, &text, Some(Decision::Redact), category).unwrap() {
        AddDocumentOutcome::Saved { document } => document,
        other => panic!("{other:?}"),
    }
}

fn suggestions(c: &Connection, form: &str) -> Vec<proposals::Proposal> {
    crate::core::profile::forms::get(c, form).unwrap().proposals
}

/// The constancia del SAT: recognized by its words, its RFC covered in what is kept, and yet suggested in its field
/// with the document and the page, read by a rule and never by the AI.
#[test]
fn the_constancia_suggests_the_rfc_it_says_although_the_saved_text_covers_it() {
    let (_d, mut c) = conn();
    let doc = upload(&mut c, "Constancia SAT", "constancia-situacion-fiscal.txt", None);
    assert_eq!((doc.category.as_deref(), doc.doc_type.as_deref(), doc.doc_type_origin.as_deref()), (Some("legal"), Some("tax_status_certificate"), Some("computed")));
    let kept: String = c.query_row("SELECT extracted_text FROM document WHERE id = ?1", [&doc.id], |r| r.get(0)).unwrap();
    assert!(kept.contains("[RFC OCULTO]") && !kept.contains("CRF150310KX4"), "what is kept stays covered");

    let s = suggestions(&c, "institution.legal");
    let rfc = s.iter().find(|p| p.field_id == "institution.legal_rfc").expect("the RFC is suggested");
    assert_eq!(rfc.value, json!("CRF150310KX4"));
    assert_eq!((rfc.origin, rfc.agent.as_str()), (Origin::Document, AGENT));
    assert_eq!((rfc.document.as_deref(), rfc.page), (Some("Constancia SAT"), Some(1)));
    assert!(rfc.said.is_none(), "a document is not «Usted dijo»");
    assert!(!rfc.source_ref.as_deref().unwrap().contains("CRF150310KX4"), "a protected datum keeps no line with it");
    let regime = s.iter().find(|p| p.field_id == "institution.tax_regime").unwrap();
    // pasted text is one page; a file keeps its pages (`rules::tests`)
    assert_eq!((regime.value.clone(), regime.page), (json!("non_profit"), Some(1)));
    assert!(regime.source_ref.as_deref().unwrap().contains("Fines no Lucrativos"), "a datum that is not protected keeps its line");
    let fields: Vec<&str> = s.iter().map(|p| p.field_id.as_str()).collect();
    for f in ["institution.legal_name", "institution.legal_form", "institution.fiscal_postal_code"] {
        assert!(fields.contains(&f), "{f}: {fields:?}");
    }
}

/// Accepting is the usual save: the datum keeps its origin `document`, and the proposal forgets the protected value.
/// A datum already saved with that value is not suggested again.
#[test]
fn accepting_keeps_the_origin_and_forgets_the_protected_value() {
    let (_d, mut c) = conn();
    let name = [("institution.name".to_string(), json!("Casa de Reposo Ficticia San Ejemplo"))].into_iter().collect();
    crate::core::profile::forms::save(&mut c, "institution.identity", name, None).unwrap();
    upload(&mut c, "Constancia SAT", "constancia-situacion-fiscal.txt", Some("legal"));
    let rfc =suggestions(&c, "institution.legal").into_iter().find(|p| p.field_id == "institution.legal_rfc").unwrap();
    let mut values = crate::core::profile::forms::get(&c, "institution.legal").unwrap().values;
    values.insert("institution.legal_rfc".into(), json!("CRF150310KX4"));
    let out = crate::core::profile::forms::save_using(&mut c, "institution.legal", values, None, &[rfc.id.clone()]).unwrap();
    assert!(matches!(out, crate::core::profile::service::SaveProfileOutcome::Saved { .. }), "{out:?}");
    let view = crate::core::profile::forms::get(&c, "institution.legal").unwrap();
    assert_eq!(view.values["institution.legal_rfc"], json!("CRF150310KX4"));
    assert_eq!(view.origins["institution.legal_rfc"].origin, "document");
    let (status, value): (String, String) = c.query_row("SELECT status, value_json FROM ai_proposal WHERE id = ?1", [&rfc.id], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    assert_eq!((status.as_str(), value.as_str()), ("accepted", "null"));

    upload(&mut c, "Constancia otra vez", "constancia-situacion-fiscal.txt", Some("legal"));
    assert!(suggestions(&c, "institution.legal").iter().all(|p| p.field_id != "institution.legal_rfc"), "the same value is not suggested again");
}

/// The oficio of donataria and the acta: their data, each in its field, with no AI; a document no rule knows is kept
/// in its class without a kind and suggests nothing.
#[test]
fn the_oficio_and_the_acta_suggest_their_data_and_an_unknown_document_suggests_nothing() {
    let (_d, mut c) = conn();
    upload(&mut c, "Oficio SAT", "oficio-donataria.txt", Some("legal"));
    upload(&mut c, "Acta", "acta-constitutiva.txt", Some("legal"));
    let s = suggestions(&c, "institution.legal");
    let value = |f: &str| s.iter().find(|p| p.field_id == f).map(|p| p.value.clone()).unwrap_or(Value::Null);
    assert_eq!(value("institution.authorized_donee"), json!("yes"));
    assert_eq!(value("institution.donee_letter_number"), json!("600-04-02-2017-11234"));
    assert_eq!(value("institution.donee_letter_date"), json!("2017-03-14"));
    assert_eq!(value("institution.donee_category"), json!("assistance"));
    assert_eq!(value("institution.founded_year"), json!(2015));
    assert!(value("institution.purpose").as_str().unwrap().contains("adultas mayores"));

    let before = c.query_row("SELECT count(*) FROM ai_proposal", [], |r| r.get::<_, i64>(0)).unwrap();
    let contract = upload(&mut c, "Comodato", "contrato-comodato.txt", Some("legal"));
    assert_eq!((contract.category.as_deref(), contract.doc_type.as_deref()), (Some("legal"), None));
    assert_eq!(c.query_row("SELECT count(*) FROM ai_proposal", [], |r| r.get::<_, i64>(0)).unwrap(), before);
    assert!(!c.query_row("SELECT count(*) > 0 FROM ai_proposal WHERE agent <> 'reader'", [], |r| r.get::<_, bool>(0)).unwrap(), "no AI was asked");
}

/// What the person said of a document is not changed by a rule.
#[test]
fn a_kind_the_person_said_stays() {
    let (_d, mut c) = conn();
    let doc = upload(&mut c, "Reglamento", "reglamento-interno.txt", Some("programs"));
    assert_eq!((doc.category.as_deref(), doc.doc_type.as_deref()), (Some("programs"), Some("internal_rules")), "the class the person chose stays");
    crate::core::archive::service::set_type(&c, &doc.id, Some("org_chart")).unwrap();
    docs::set_class(&c, &doc.id, None, Some("internal_rules"), "computed").unwrap();
    let now = docs::list(&c).unwrap().into_iter().find(|d| d.id == doc.id).unwrap();
    assert_eq!((now.doc_type.as_deref(), now.doc_type_origin.as_deref()), (Some("org_chart"), Some("user")));
    assert!(matches!(crate::core::archive::service::set_type(&c, &doc.id, Some("invoice")), Err(ServiceError::UnknownKind)), "no invoices here");
    crate::core::archive::service::set_type(&c, &doc.id, None).unwrap();
    let none = docs::list(&c).unwrap().into_iter().find(|d| d.id == doc.id).unwrap();
    assert_eq!((none.doc_type, none.category.as_deref()), (None, Some("programs")));
}

/// What the Reader understands of a real file, without saving anything nor printing its text: per page, how many
/// letters were read and whether from a picture; the kind; and per datum found, its page and its length. For a real
/// document of the institution: its text never leaves this computer, and here not even the screen.
///   $env:CIMIENTO_READER_FILE="D:\...\acta.pdf"; cargo test reader_on_a_file -- --ignored --nocapture > salida.txt
///   (optional: $env:CIMIENTO_READER_KIND="articles_of_incorporation", the kind of the document of the record)
#[test]
#[ignore]
fn reader_on_a_file() {
    let path = std::path::PathBuf::from(std::env::var("CIMIENTO_READER_FILE").expect("CIMIENTO_READER_FILE"));
    let known = std::env::var("CIMIENTO_READER_KIND").ok();
    let bytes = std::fs::read(&path).unwrap();
    let started = std::time::Instant::now();
    let read = crate::documents::reading::read_upload(&path.file_name().unwrap().to_string_lossy(), &bytes).unwrap();
    println!("Páginas: {}  leídas de imagen: {}  con dudas: {:?}  ({:.0} s)", read.pages.len(), read.from_picture.len(), read.doubtful, started.elapsed().as_secs_f64());
    for (i, p) in read.pages.iter().enumerate() {
        let letters = p.chars().filter(|c| c.is_alphabetic()).count();
        let folded = crate::storage::fts::fold(p);
        let marks: Vec<&str> = ["por objeto", "objeto", "articulo", "clausula", "denominacion", "escritura", "vigencia"].into_iter().filter(|w| folded.contains(w)).collect();
        println!("  p{:>2}: {:>5} letras  {}  {:?}", i + 1, letters, if read.from_picture.contains(&(i + 1)) { "imagen" } else { "texto " }, marks);
    }
    let pages: Vec<String> = read.pages.iter().map(|p| crate::documents::text::clean_text(p)).collect();
    // only when asked: a few words around the first place each of these appears, to see how the document writes it
    //   $env:CIMIENTO_READER_SHOW="objeto,denominacion"
    if let Ok(show) = std::env::var("CIMIENTO_READER_SHOW") {
        for w in show.split(',').map(str::trim).filter(|w| !w.is_empty()) {
            let hit = pages.iter().enumerate().find_map(|(i, p)| {
                let folded = crate::storage::fts::fold(p);
                folded.find(w).map(|at| (i + 1, folded[..at].chars().count(), p))
            });
            if let Some((page, at, p)) = hit {
                let around: String = p.chars().skip(at.saturating_sub(20)).take(160).collect();
                println!("«{w}» p{page}: …{}…", around.split_whitespace().collect::<Vec<_>>().join(" "));
            }
        }
    }
    // only the labels of a form-like document (the words before each «:»), never what follows them
    //   $env:CIMIENTO_READER_LABELS="1"
    if std::env::var("CIMIENTO_READER_LABELS").is_ok() {
        for (i, p) in pages.iter().enumerate() {
            let labels: Vec<String> = p
                .split(':')
                .take(40)
                .filter_map(|chunk| {
                    let words: Vec<&str> = chunk.split_whitespace().collect();
                    let tail: Vec<&str> = words.iter().rev().take_while(|w| w.chars().next().is_some_and(|c| c.is_alphabetic())).take(4).copied().collect();
                    (!tail.is_empty()).then(|| tail.into_iter().rev().collect::<Vec<_>>().join(" "))
                })
                .collect();
            println!("  rótulos p{}: {:?}", i + 1, labels);
        }
    }
    println!("Tipo por sus palabras: {:?}", kinds::recognize(&pages));
    let reading = super::read(&pages, known.as_deref());
    println!("Tipo usado: {:?}", reading.kind);
    for f in &reading.found {
        let len = f.value.as_str().map_or(f.value.to_string().len(), |s| s.chars().count());
        println!("  {} p{}  {} letras", f.field, f.page, len);
        // where a long text ends, to see that it stops at the next article or label
        if len > 200 {
            let s = f.value.as_str().unwrap_or_default();
            println!("     …{}", s.chars().skip(len - 60).collect::<String>());
        }
    }
}
