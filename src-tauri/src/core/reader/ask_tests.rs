use super::*;
use crate::core::agents::capturist::plain;
use crate::ai::mock::MockProvider;
use crate::ai::{AiError, AiResponse, Usage};
use crate::core::archive::search::tests::load_record;
use std::sync::Mutex;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// A base with the fictitious Expediente of `fixtures/expediente`, each file in its document.
fn db(with_record: bool) -> (tempfile::TempDir, SharedDb) {
    let dir = tempfile::tempdir().unwrap();
    let c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    if with_record {
        load_record(&c);
    }
    (dir, Arc::new(Mutex::new(c)))
}

fn reply(answer: Value) -> Result<AiResponse, AiError> {
    Ok(AiResponse { value: json!({ "tool": null, "args": null, "answer": answer }), model: "mock".into(), usage: Usage { input_tokens: 1_500, output_tokens: 120, ..Default::default() } })
}

/// The shown passage whose text says something.
fn with<'a>(shown: &'a [Shown], words: &str) -> &'a Shown {
    shown.iter().find(|s| plain(&s.passage.snippet).contains(&plain(words))).unwrap_or_else(|| panic!("no passage says «{words}»: {shown:#?}"))
}

/// The id in the store of a fixture file.
fn document_id(db: &SharedDb, file: &str) -> String {
    db.lock().unwrap().query_row("SELECT id FROM document WHERE display_name = ?1", [file], |r| r.get(0)).unwrap()
}

/// The AI sees each passage by the type of its document, never by the name of its file (decision of the institution,
/// 2026-10-13); the screen keeps the file name, from the code.
#[tokio::test]
async fn the_passages_go_by_their_type_never_by_the_name_of_the_file() {
    let (_d, db) = db(true);
    let shown = passages_for(&db, None, "¿Cuál es la denominación o razón social?", None).await.unwrap();
    let s = shown.iter().find(|s| s.passage.document == "constancia-situacion-fiscal.txt").expect("the constancia is among the passages");
    assert!(plain(&s.passage.snippet).contains(&plain("CASA DE REPOSO FICTICIA SAN EJEMPLO")));
    assert_eq!((s.name.as_str(), s.passage.document.as_str(), s.passage.page), ("Constancia de situación fiscal", "constancia-situacion-fiscal.txt", Some(1)));
    let told = context(&shown).join("\n");
    assert!(told.contains("Constancia de situación fiscal, página 1") && !told.contains(".txt"), "{told}");
    // a document the institution named itself goes by that name
    let shown = passages_for(&db, None, "¿Quién paga el predial?", None).await.unwrap();
    assert_eq!(with(&shown, "predial").name, "Contrato de comodato del inmueble");
}

/// The code keeps an answer only with a quote that is in a passage shown and figures that are in what it cites.
#[tokio::test]
async fn the_code_keeps_only_what_a_passage_says() {
    let (_d, db) = db(true);
    let q = "¿Cuál es el plazo del comodato?";
    let shown = passages_for(&db, None, q, None).await.unwrap();
    let p = with(&shown, "plazo del comodato").id.clone();
    let good = json!({ "found": true, "text": "El comodato es por veinte años, desde abril de 2016.", "citations": [{ "passage": p, "quote": "El plazo del comodato es de veinte años" }] });
    // «2016» is written in words in the passage («dos mil dieciséis»): it counts
    let ReaderAnswer::Found { citations, .. } = check(&good, &shown, q) else { panic!("{:?}", check(&good, &shown, q)) };
    assert_eq!((citations[0].passage.document.as_str(), citations[0].passage.page, citations[0].name.as_str()), ("contrato-comodato.txt", Some(1), "Contrato de comodato del inmueble"));

    let cases = [
        // a quote that is not in the passage
        json!({ "found": true, "text": "Es por veinte años.", "citations": [{ "passage": p, "quote": "El plazo es de treinta años" }] }),
        // a passage never shown
        json!({ "found": true, "text": "Es por veinte años.", "citations": [{ "passage": "P99", "quote": "El plazo del comodato es de veinte años" }] }),
        // a figure it worked out: twenty years from 2016 is not written anywhere
        json!({ "found": true, "text": "Termina en 2036.", "citations": [{ "passage": p, "quote": "El plazo del comodato es de veinte años" }] }),
        // no quote at all
        json!({ "found": true, "text": "Es por veinte años.", "citations": [] }),
    ];
    for a in cases {
        assert_eq!(check(&a, &shown, q), ReaderAnswer::NotFound { looked: shown.len(), dropped: true }, "{a}");
    }
    let none = json!({ "found": false, "text": "Ningún pasaje lo dice.", "citations": [] });
    assert_eq!(check(&none, &shown, q), ReaderAnswer::NotFound { looked: shown.len(), dropped: false });
}

/// «Según mi registro, ¿qué nombre legal pongo?»: one call with the cheapest model and the passages, and the answer
/// with where it came from. No file name reaches the AI.
#[tokio::test]
async fn the_reader_answers_with_the_documents_and_where() {
    let (_d, db) = db(true);
    let q = "Según mi registro, ¿qué nombre legal pongo? Denominación o razón social";
    let shown = passages_for(&db, None, q, None).await.unwrap();
    let p = shown.iter().find(|s| s.passage.document == "constancia-situacion-fiscal.txt").unwrap().id.clone();
    let m = MockProvider::new(vec![reply(json!({
        "found": true,
        "text": "Su nombre legal es Casa de Reposo Ficticia San Ejemplo.",
        "citations": [{ "passage": p, "quote": "Denominación o razón social: CASA DE REPOSO FICTICIA SAN EJEMPLO" }]
    }))]);
    let out = answer(&db, Some(&m), None, q, None).await.unwrap();
    let ReaderAnswer::Found { text, citations } = out else { panic!("{out:?}") };
    assert!(text.contains("Ficticia San Ejemplo"));
    assert_eq!((citations[0].passage.document.as_str(), citations[0].passage.page), ("constancia-situacion-fiscal.txt", Some(1)));
    let reqs = m.requests();
    assert_eq!(reqs.len(), 1, "one call");
    assert_eq!(reqs[0].task.as_str(), "agent.reader_answer");
    assert!(reqs[0].system.contains("Usted es el Lector del Expediente") && reqs[0].user.contains(q));
    let all = format!("{} {}", reqs[0].context.join(" "), reqs[0].user);
    assert!(!all.contains(".txt"), "no file name reaches the AI");
    assert!(!all.contains("CRF150310KX4"), "the RFC of the text was covered when it was saved");
    assert_eq!(m.personal_data_sent(&[]), Vec::<String>::new(), "no value of a personal datum reached the model");
}

/// When the person names a file, the Reader reads only that document: what speaks of the question in it or, with no
/// word of it there, its first pages.
#[tokio::test]
async fn in_one_document_it_reads_only_that_one() {
    let (_d, db) = db(true);
    let poder = document_id(&db, "poder-notarial.txt");
    let shown = passages_for(&db, None, "¿Hasta cuándo vale?", Some(&poder)).await.unwrap();
    assert!(!shown.is_empty() && shown.iter().all(|s| s.passage.document_id == poder && s.name == "Poder del representante legal"), "{shown:#?}");
    let shown = passages_for(&db, None, "zzzz", Some(&poder)).await.unwrap();
    assert_eq!(shown.iter().map(|s| s.passage.page).collect::<Vec<_>>().first(), Some(&Some(1)), "its first pages");
    assert!(shown.iter().all(|s| s.passage.document_id == poder));
}

/// No documents, no AI, or the AI failing: nothing is invented and the reason is said.
#[tokio::test]
async fn without_documents_or_without_the_ai_it_says_so() {
    let (_d, empty) = db(false);
    let m = MockProvider::new(vec![]);
    assert_eq!(answer(&empty, Some(&m), None, "¿Cuál es nuestro RFC?", None).await.unwrap(), ReaderAnswer::NoDocuments);
    assert!(m.requests().is_empty());

    let (_d, db) = db(true);
    assert_eq!(answer(&db, None, None, "¿Cuál es el plazo del comodato?", None).await.unwrap(), ReaderAnswer::Unavailable { reason: "not_configured" });
    let m = MockProvider::new(vec![Err(AiError::Offline), Err(AiError::Offline)]);
    assert_eq!(answer(&db, Some(&m), None, "¿Cuál es el plazo del comodato?", None).await.unwrap(), ReaderAnswer::Unavailable { reason: "offline" });
    // nothing in the drawer speaks of it: no call
    let m = MockProvider::new(vec![]);
    assert_eq!(answer(&db, Some(&m), None, "xyzzy plugh", None).await.unwrap(), ReaderAnswer::NotFound { looked: 0, dropped: false });
    assert!(m.requests().is_empty());
}
