use super::*;
use crate::core::profile::forms::save as save_form;
use crate::core::profile::service::SaveProfileOutcome;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const TODAY: &str = "2026-10-10";

fn conn() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    (dir, c)
}

fn values(v: Value) -> Values {
    serde_json::from_value(v).unwrap()
}

fn profile(c: &mut Connection, form: &str, v: Value) {
    assert!(matches!(save_form(c, form, values(v), None).unwrap(), SaveProfileOutcome::Saved { .. }));
}

fn kinds(e: &Expediente) -> Vec<&str> {
    e.records.iter().map(|r| r.kind.as_str()).collect()
}

fn saved(o: RecordOutcome) -> String {
    match o {
        RecordOutcome::Saved { id, .. } => id,
        other => panic!("not saved: {other:?}"),
    }
}

fn issues(o: RecordOutcome) -> Vec<(&'static str, String)> {
    match o {
        RecordOutcome::Invalid { issues } => issues.into_iter().map(|i| (i.code, i.field)).collect(),
        other => panic!("not refused: {other:?}"),
    }
}

fn fixture(dir: &Path, name: &str) -> std::path::PathBuf {
    let to = dir.join(name);
    std::fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office").join(name), &to).unwrap();
    to
}

/// ADR-033 §5: which documents apply is decided in Rust from what the institution says of itself.
#[test]
fn the_documents_that_apply_follow_what_the_institution_says() {
    let (_d, mut c) = conn();
    let always: Vec<&str> = RECORD_TYPES.iter().filter(|(_, a)| *a == Applies::Always).map(|(k, _)| *k).collect();
    assert_eq!(kinds(&expediente_on(&c, TODAY).unwrap()), always, "with nothing said, only what every institution needs");

    profile(&mut c, "institution.identity", json!({ "institution.name": "Casa Ficticia", "institution.populations": ["childhood"], "institution.modalities": ["residential"] }));
    profile(&mut c, "institution.legal", json!({ "institution.legal_form": "iap", "institution.authorized_donee": "in_progress", "institution.cluni": "no" }));
    let e = expediente_on(&c, TODAY).unwrap();
    for k in ["donee_authorization", "junta_registration", "junta_budget", "social_care_registry", "sanitary_notice"] {
        assert!(kinds(&e).contains(&k), "{k} applies: {:?}", kinds(&e));
    }
    assert!(!kinds(&e).contains(&"cluni_certificate"), "without the CLUNI");
    assert_eq!((e.summary.expected, e.summary.missing, e.summary.have), (e.records.len(), e.records.len(), 0));
    assert!(e.records.iter().all(|r| r.id.is_none() && r.status == "missing"), "nothing written yet");

    // what it has stays listed though it no longer applies; what it lacks goes
    saved(save(&mut c, "junta_registration", None, values(json!({ "institution.record.status": "have" })), None).unwrap());
    profile(&mut c, "institution.legal", json!({ "institution.legal_form": "ac" }));
    let k = kinds(&expediente_on(&c, TODAY).unwrap()).join(" ");
    assert!(k.contains("junta_registration") && !k.contains("junta_budget"), "{k}");
}

/// A document counts without its file; its validity is reckoned from the date written in it, 30 days ahead.
#[test]
fn a_document_counts_without_a_file_and_its_validity_is_reckoned() {
    let (_d, mut c) = conn();
    let have = |until: Option<&str>| values(json!({ "institution.record.status": "have", "institution.record.issued_on": "2026-01-15", "institution.record.valid_until": until }));
    saved(save(&mut c, "articles_of_incorporation", None, have(None), None).unwrap());
    saved(save(&mut c, "tax_compliance_opinion", None, have(Some("2026-10-20")), None).unwrap());
    saved(save(&mut c, "power_of_attorney", None, have(Some("2026-10-01")), None).unwrap());
    saved(save(&mut c, "civil_protection_program", None, have(Some("2027-06-30")), None).unwrap());
    saved(save(&mut c, "land_use_license", None, values(json!({ "institution.record.status": "in_progress", "institution.record.reference": "TRAM-123" })), None).unwrap());
    saved(save(&mut c, "org_chart", None, values(json!({ "institution.record.status": "not_applicable" })), None).unwrap());

    let e = expediente_on(&c, TODAY).unwrap();
    let of = |k: &str| e.records.iter().find(|r| r.kind == k).unwrap();
    assert_eq!((of("articles_of_incorporation").validity, of("articles_of_incorporation").file.clone()), (None, None), "no date, no file: had all the same");
    assert_eq!((of("tax_compliance_opinion").validity, of("tax_compliance_opinion").days_left), (Some(Validity::Expiring), Some(10)));
    assert_eq!((of("power_of_attorney").validity, of("power_of_attorney").days_left), (Some(Validity::Expired), Some(-9)));
    assert_eq!(of("civil_protection_program").validity, Some(Validity::Valid));
    let s = &e.summary;
    assert_eq!((s.expected, s.have, s.in_progress, s.expiring, s.expired, s.current()), (11, 4, 1, 1, 1, 3), "{s:?}");
    assert_eq!(s.missing, 11 - 4 - 1, "the 12 every institution needs, but «No aplica»");
    // the folio is kept but never shown in the list
    assert!(!serde_json::to_string(&e).unwrap().contains("TRAM-123"));
    assert_eq!(form(&c, "land_use_license", None).unwrap().values["institution.record.reference"], json!("TRAM-123"));
    // the history says the record changed, never a value
    let (n, protected): (i64, i64) = c.query_row("SELECT count(*), min(protected) FROM core_change WHERE field = 'institution.record' AND value_json IS NULL", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    assert_eq!((n, protected), (6, 1));
}

/// What a document cannot hold is refused with its field; what does not apply to its state is not kept.
#[test]
fn what_cannot_be_kept_is_refused_with_its_field() {
    let (_d, mut c) = conn();
    let v = |x: Value| values(x);
    assert_eq!(issues(save(&mut c, "articles_of_incorporation", None, v(json!({})), None).unwrap()), vec![("required", "institution.record.status".into())]);
    assert_eq!(issues(save(&mut c, "articles_of_incorporation", None, v(json!({ "institution.record.status": "lost" })), None).unwrap())[0].0, "code_unknown");
    let backwards = v(json!({ "institution.record.status": "have", "institution.record.issued_on": "2026-05-01", "institution.record.valid_until": "2026-04-30" }));
    assert_eq!(issues(save(&mut c, "power_of_attorney", None, backwards, None).unwrap()), vec![("valid_before_issued", "institution.record.valid_until".into())]);
    assert_eq!(issues(save(&mut c, "power_of_attorney", None, v(json!({ "institution.record.status": "have", "institution.record.valid_until": "2026-02-30" })), None).unwrap())[0].0, "date_invalid");
    assert_eq!(issues(save(&mut c, "other", None, v(json!({ "institution.record.status": "have" })), None).unwrap()), vec![("required", "institution.record.custom_name".into())]);
    assert!(matches!(save(&mut c, "invented", None, v(json!({ "institution.record.status": "have" })), None), Err(ServiceError::NotFound)));

    // missing: its dates and folio do not apply and are not kept
    let id = saved(save(&mut c, "privacy_notice", None, v(json!({ "institution.record.status": "missing", "institution.record.valid_until": "2027-01-01", "institution.record.reference": "X1" })), None).unwrap());
    let row = store::by_id(&c, &id).unwrap().unwrap();
    assert_eq!((row.valid_until, row.reference), (None, None));
    // a type of the catalog keeps one row: saving again writes the same one
    let again = saved(save(&mut c, "privacy_notice", None, v(json!({ "institution.record.status": "have" })), None).unwrap());
    assert_eq!(again, id);
}

/// The institution names its own documents; their name and notes go through the scanner.
#[test]
fn the_institution_names_its_own_documents_and_their_texts_are_scanned() {
    let (_d, mut c) = conn();
    let v = values(json!({ "institution.record.custom_name": "Convenio con el DIF", "institution.record.status": "have", "institution.record.notes": "Lo tiene maria.lopez@example.com" }));
    assert!(matches!(save(&mut c, "other", None, v.clone(), None).unwrap(), RecordOutcome::Quarantine { .. }));
    assert!(store::rows(&c).unwrap().is_empty(), "nothing saved until the person decides");
    let id = saved(save(&mut c, "other", None, v, Some(Decision::Redact)).unwrap());
    let row = store::by_id(&c, &id).unwrap().unwrap();
    assert!(!row.notes.unwrap().contains("maria.lopez"));
    let second = saved(save(&mut c, "other", None, values(json!({ "institution.record.custom_name": "Permiso de fumigación", "institution.record.status": "missing" })), None).unwrap());
    assert_ne!(id, second, "as many as the institution names");
    let e = expediente_on(&c, TODAY).unwrap();
    assert_eq!(e.records.iter().filter(|r| r.kind == "other").filter_map(|r| r.custom_name.as_deref()).collect::<Vec<_>>(), ["Convenio con el DIF", "Permiso de fumigación"]);
    // an id of another type is not this document
    assert!(matches!(form(&c, "privacy_notice", Some(&id)), Err(ServiceError::NotFound)));
}

/// A PDF is read page by page and only its clean text is kept; of the original, where it is. A scan is read as
/// pictures (OCR, ADR-035), and a picture with no text to read counts as had all the same.
#[test]
fn a_file_is_read_and_only_its_clean_text_and_where_it_is_are_kept() {
    let (dir, mut c) = conn();
    let pdf = fixture(dir.path(), "convocatoria-con-tablas.pdf");
    let AttachOutcome::Attached { id, readable, from_picture, expediente, .. } = attach(&mut c, "articles_of_incorporation", None, &pdf).unwrap() else { panic!() };
    assert!(readable && !from_picture);
    let r = expediente.records.iter().find(|r| r.kind == "articles_of_incorporation").unwrap();
    let file = RecordFile { name: "convocatoria-con-tablas.pdf".into(), readable: true, from_picture: false, doubtful: vec![] };
    assert_eq!((r.status.as_str(), r.file.clone()), ("have", Some(file)));
    let docs = crate::core::archive::storage::list_drawer(&c, crate::core::archive::storage::Drawer::Record).unwrap();
    assert_eq!((docs.len(), docs[0].kind.as_str(), docs[0].display_name.as_str()), (1, "internal", "convocatoria-con-tablas.pdf"));
    assert!(docs[0].chunks > 1, "one fragment per page");
    let blob: Option<Vec<u8>> = c.query_row("SELECT blob FROM document", [], |r| r.get(0)).unwrap();
    assert!(blob.is_none(), "the original is never copied in (principle 6)");
    assert_eq!(original(&c, &id).unwrap().as_deref(), Some(pdf.as_path()));

    // a scanned deed is read as pictures: its email is covered without asking, and its worn page is read with doubts
    let scan = fixture(dir.path(), "acta-escaneada.pdf");
    let AttachOutcome::Attached { readable, from_picture, doubtful, covered, expediente, .. } = attach(&mut c, "bylaws_amendments", None, &scan).unwrap() else { panic!() };
    assert_eq!((readable, from_picture, doubtful, covered), (true, true, vec![2], 1));
    let r = expediente.records.iter().find(|r| r.kind == "bylaws_amendments").unwrap();
    assert_eq!(r.file.as_ref().map(|f| (f.from_picture, f.doubtful.clone())), Some((true, vec![2])));
    assert_eq!(crate::core::archive::storage::list_drawer(&c, crate::core::archive::storage::Drawer::Record).unwrap().len(), 2);

    // a picture with no text to read: of it, only where it is
    let blank = dir.path().join("en-blanco.png");
    std::fs::write(&blank, [1, 2, 3]).unwrap();
    let AttachOutcome::Attached { readable, .. } = attach(&mut c, "tax_status_certificate", None, &blank).unwrap() else { panic!() };
    assert!(!readable, "a picture with no text has nothing to read");
    assert_eq!(crate::core::archive::storage::list_drawer(&c, crate::core::archive::storage::Drawer::Record).unwrap().len(), 2, "nothing to keep of it but where it is");

    // taking the file off: its clean text leaves the drawer too (ADR-036); once the original is gone it is not opened
    let e = detach(&mut c, &id).unwrap();
    assert!(e.records.iter().find(|r| r.kind == "articles_of_incorporation").unwrap().file.is_none());
    assert_eq!(crate::core::archive::storage::list_drawer(&c, crate::core::archive::storage::Drawer::Record).unwrap().len(), 1);
    let scan_id = store::by_type(&c, "tax_status_certificate").unwrap().unwrap().id;
    std::fs::remove_file(&blank).unwrap();
    assert_eq!(original(&c, &scan_id).unwrap(), None);
}

/// What cannot be read says why and saves nothing; a Word file with data of a person is kept with them covered.
#[test]
fn what_cannot_be_taken_says_why_and_a_word_file_goes_through_the_scanner() {
    let (dir, mut c) = conn();
    let why = |c: &mut Connection, p: &Path| match attach(c, "privacy_notice", None, p).unwrap() {
        AttachOutcome::Unreadable { reason } => reason,
        other => panic!("{other:?}"),
    };
    let txt = dir.path().join("notas.txt");
    std::fs::write(&txt, "texto").unwrap();
    assert_eq!(why(&mut c, &txt), Unreadable::UnsupportedType);
    assert_eq!(why(&mut c, &dir.path().join("no-esta.pdf")), Unreadable::NotFound);
    let broken = dir.path().join("roto.pdf");
    std::fs::write(&broken, b"esto no es un pdf").unwrap();
    assert_eq!(why(&mut c, &broken), Unreadable::Damaged);
    // a file with a password says so: the person learns why, nothing ends in silence
    assert_eq!(why(&mut c, &fixture(dir.path(), "protegido.pdf")), Unreadable::Protected);
    assert!(store::rows(&c).unwrap().is_empty());

    let blocks = [crate::documents::docx::Block::Paragraph("Aviso de privacidad de la institución. Dudas con maria.lopez@example.com, responsable de los datos personales que se recaban.".into())];
    let word = dir.path().join("aviso.docx");
    std::fs::write(&word, crate::documents::docx::build(&blocks, "Aviso", "Casa Ficticia", "2026-10-10T00:00:00Z").unwrap()).unwrap();
    let AttachOutcome::Attached { readable, covered, .. } = attach(&mut c, "privacy_notice", None, &word).unwrap() else { panic!() };
    assert_eq!((readable, covered), (true, 1));
    let text: String = c.query_row("SELECT extracted_text FROM document", [], |r| r.get(0)).unwrap();
    assert!(text.contains("Aviso de privacidad") && !text.contains("maria.lopez"), "{text}");
    // an `other` gets its file once it exists
    assert!(matches!(attach(&mut c, "other", None, &word), Err(ServiceError::NotFound)));
}

/// The file of a document of the record goes to the Reader with its kind already said (IA6): the poder gives the date
/// it is valid until as a suggestion of «Datos legales y fiscales», citing its page; the document is that kind, said by
/// the person, in the class of its kind.
/// A paper of the institution full of people (an act with its partners, their CURP and RFC, even a table with
/// «NOMBRE, EDAD, CURP») is kept with all of them covered: never refused as a list, never held for the person to
/// decide (decision of the institution, 2026-10-13). The covering goes to the log.
#[test]
fn an_act_full_of_partners_is_kept_with_their_data_covered() {
    let (_d, mut c) = conn();
    let partners = ["LOPM800101MDFRZN09", "RAMC921215MNLNRL00", "GAHJ150930HJCRRSA7"];
    let mut page = String::from("ACTA CONSTITUTIVA de la sociedad. Comparecen los asociados fundadores:\n");
    for (i, curp) in partners.iter().chain(partners.iter()).enumerate() {
        page.push_str(&format!("Asociado {}: CURP {curp}, RFC {}AB3, con domicilio en la propiedad de la actividad.\n", i + 1, &curp[..10]));
    }
    page.push_str("NOMBRE, EDAD, CURP\n");
    let file = FileText::of_pages("acta.pdf", vec![page]);
    let AttachOutcome::Attached { readable, covered, .. } = attach_read(&mut c, "articles_of_incorporation", None, Path::new("C:/acta.pdf"), file).unwrap() else { panic!() };
    assert!(readable && covered >= 12, "{covered}");
    let text: String = c.query_row("SELECT extracted_text FROM document", [], |r| r.get(0)).unwrap();
    assert!(text.contains("ACTA CONSTITUTIVA") && partners.iter().all(|p| !text.contains(p) && !text.contains(&p[..10])), "{text}");
    let logged: i64 = c.query_row("SELECT COUNT(*) FROM audit_log WHERE entity = 'core_record' AND details_json LIKE '%redacted%'", [], |r| r.get(0)).unwrap();
    assert_eq!(logged, 1);
}

#[test]
fn a_file_of_the_record_suggests_the_data_it_says() {
    let (_d, mut c) = conn();
    let raw = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente/poder-notarial.txt")).unwrap();
    let pages = crate::core::archive::search::tests::fixture_pages(&raw);
    let file = FileText { name: "poder.pdf".into(), mime: "application/pdf", pages: Some(pages), marks: PageMarks::default() };
    let AttachOutcome::Attached { readable, .. } = attach_read(&mut c, "power_of_attorney", None, Path::new("C:/poder.pdf"), file).unwrap() else { panic!() };
    assert!(readable);
    let doc = crate::core::archive::storage::list(&c).unwrap().remove(0);
    assert_eq!((doc.category.as_deref(), doc.doc_type.as_deref(), doc.doc_type_origin.as_deref()), (Some("legal"), Some("power_of_attorney"), Some("user")));
    let s = crate::core::profile::forms::get(&c, "institution.legal").unwrap().proposals;
    let until = s.iter().find(|p| p.field_id == "institution.legal_rep_valid_until").expect("the validity is suggested");
    assert_eq!((until.value.clone(), until.document.as_deref(), until.page), (serde_json::json!("2027-01-18"), Some("poder.pdf"), Some(2)));
}
