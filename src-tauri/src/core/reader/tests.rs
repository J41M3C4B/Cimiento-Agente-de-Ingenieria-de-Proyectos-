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
