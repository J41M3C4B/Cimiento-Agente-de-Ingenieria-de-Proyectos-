//! The full test of the Expediente (C4): the files of `fixtures/expediente/prueba-completa/archivos`, made from the
//! fictitious institution, put one by one on their documents through the same road as the screen (read, kind, rules,
//! scanner, own data, suggestions). What must happen with each one is in `esperado.json`, and in plain words in
//! `docs/16-institucion-de-prueba.md`. No AI and nothing real.

use super::*;
use crate::core::profile::forms::save as save_form;
use crate::core::profile::service::SaveProfileOutcome;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const FORMS: [&str; 4] = ["institution.identity", "institution.contact", "institution.legal", "institution.capacity"];

fn folder() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente/prueba-completa")
}

fn reason(r: Unreadable) -> &'static str {
    match r {
        Unreadable::UnsupportedType => "unsupported_type",
        Unreadable::TooLarge => "too_large",
        Unreadable::NotFound => "not_found",
        Unreadable::Damaged => "damaged",
        Unreadable::TooManyPages => "too_many_pages",
        Unreadable::Protected => "protected",
    }
}

/// Every file of the full test, in the order of `esperado.json`, with what the program did and what was expected.
/// Scanned pages and photos are read with OCR: about a minute in all.
///   cargo test full_expediente -- --ignored --nocapture
#[test]
#[ignore]
fn full_expediente() {
    let expected: Value = serde_json::from_str(&std::fs::read_to_string(folder().join("esperado.json")).unwrap()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
    // what the person says first, so that the Expediente shows the documents of an I.A.P. that is a donee
    for (form, values) in expected["antes"].as_object().unwrap() {
        let v: Values = serde_json::from_value(values.clone()).unwrap();
        let out = save_form(&mut c, form, v, None).unwrap();
        assert!(matches!(out, SaveProfileOutcome::Saved { .. }), "{form}: {out:?}");
    }
    let mut wrong: Vec<String> = vec![];
    let mut check = |file: &str, what: &str, got: Value, want: Option<&Value>| {
        if let Some(want) = want {
            if &got != want {
                wrong.push(format!("{file}: {what} fue {got}, se esperaba {want}"));
            }
        }
    };
    for step in expected["pasos"].as_array().unwrap() {
        let file = step["archivo"].as_str().unwrap();
        let kind = step["documento"].as_str().unwrap();
        let want = &step["espera"];
        let started = std::time::Instant::now();
        // an «Otro documento» is named first, as the screen asks
        let id = step["nombre"].as_str().map(|name| {
            let v: Values = serde_json::from_value(json!({ "institution.record.custom_name": name, "institution.record.status": "have" })).unwrap();
            match save(&mut c, kind, None, v, None).unwrap() {
                RecordOutcome::Saved { id, .. } => id,
                other => panic!("{other:?}"),
            }
        });
        let mut out = attach(&mut c, kind, id.as_deref(), &folder().join("archivos").join(file)).unwrap();
        // a file the code is not sure of waits for the person (ADR-042 §4): the test answers as `responde` says
        let asked = match &out {
            AttachOutcome::Ask { looks_like, .. } => json!({ "parece": looks_like }),
            _ => Value::Null,
        };
        check(file, "pregunta", asked.clone(), Some(want.get("pregunta").unwrap_or(&Value::Null)));
        if !asked.is_null() {
            let answer = match step["responde"].as_str() {
                Some("aqui") => placing::Answer::Here,
                Some("alla") => placing::Answer::There,
                _ => placing::Answer::Cancel,
            };
            println!("{file:<48} {kind:<26} PREGUNTA {asked}  responde {answer:?}");
            let waiting = placing::take(&background::key(kind, id.as_deref())).expect("it waits");
            match placing::keep_as(&mut c, waiting, answer).unwrap() {
                Some(kept) => out = kept,
                None => continue,
            }
        }
        let secs = started.elapsed().as_secs_f64();
        match out {
            AttachOutcome::Ask { .. } => unreachable!("answered above"),
            AttachOutcome::Unreadable { reason: r } => {
                println!("{file:<48} {kind:<26} NO SE PUDO: {}  ({secs:.1} s)", reason(r));
                check(file, "no se pudo", json!(reason(r)), Some(want.get("no_se_pudo").unwrap_or(&Value::Null)));
            }
            AttachOutcome::Attached { readable, from_picture, doubtful, covered, filled, suggested, .. } => {
                println!(
                    "{file:<48} {kind:<26} legible {readable}  de imagen {from_picture}  dudas {doubtful:?}  tapados {covered}  llenos {filled:?}  sugerencias {suggested}  ({secs:.1} s)"
                );
                check(file, "no se pudo", Value::Null, Some(want.get("no_se_pudo").unwrap_or(&Value::Null)));
                check(file, "legible", json!(readable), want.get("legible"));
                check(file, "de imagen", json!(from_picture), want.get("de_imagen"));
                check(file, "con dudas", json!(!doubtful.is_empty()), want.get("con_dudas"));
                check(file, "tapados", json!(covered), want.get("tapados"));
                let mut filled = filled;
                filled.sort();
                check(file, "llenos", json!(filled), want.get("llenos"));
                check(file, "sugerencias", json!(suggested), want.get("sugerencias"));
            }
        }
    }

    // the documents' own data, as the person sees them in each one's window
    let today = expected["hoy"].as_str().unwrap();
    let e = expediente_on(&c, today).unwrap();
    println!("\nExpediente al {today}: tiene {} de {}", e.summary.have, e.summary.expected);
    let mut own = serde_json::Map::new();
    for r in &e.records {
        let row = r.id.as_deref().and_then(|id| store::by_id(&c, id).unwrap());
        let reference = row.as_ref().and_then(|r| r.reference.clone());
        println!(
            "  {:<28} {:<10} emisión {:<12} vigencia {:<12} número {:<24} {:?}  archivo {:?}",
            r.kind,
            r.status,
            r.issued_on.as_deref().unwrap_or("—"),
            r.valid_until.as_deref().unwrap_or("—"),
            reference.as_deref().unwrap_or("—"),
            r.validity,
            r.file.as_ref().map(|f| f.name.as_str())
        );
        let key = r.custom_name.clone().unwrap_or_else(|| r.kind.clone());
        own.insert(key, json!({ "emision": r.issued_on, "vigencia": r.valid_until, "numero": reference, "estado": r.status, "validez": r.validity.map(|v| format!("{v:?}")) }));
    }
    for (kind, want) in expected["expediente"].as_object().unwrap() {
        let got = own.get(kind).cloned().unwrap_or(Value::Null);
        for (field, value) in want.as_object().unwrap() {
            check("expediente", &format!("{kind}.{field}"), got.get(field).cloned().unwrap_or(Value::Null), Some(value));
        }
    }

    // what waits in «Mi institución» to be used, with its document and page
    println!("\nSugerencias que esperan en «Mi institución»:");
    let mut waiting = serde_json::Map::new();
    for form in FORMS {
        for p in crate::core::proposals::pending(&c, form, None).unwrap() {
            println!("  {:<36} {:<40} {:?} p{}", p.field_id, p.value.to_string(), p.document.as_deref().unwrap_or("?"), p.page.unwrap_or(0));
            waiting.insert(p.field_id.clone(), p.value.clone());
        }
    }
    let want_waiting = expected["sugerencias"].as_object().unwrap();
    for (field, value) in want_waiting {
        check("sugerencias", field, waiting.get(field).cloned().unwrap_or(Value::Null), Some(value));
    }
    for field in waiting.keys().filter(|f| !want_waiting.contains_key(*f)) {
        wrong.push(format!("sugerencias: {field} no se esperaba ({})", waiting[field]));
    }

    // what the program does not do right yet is reported apart; once it does, it leaves the list
    let known: Vec<&str> = expected["conocidos"].as_array().map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    let (still, new): (Vec<String>, Vec<String>) = wrong.into_iter().partition(|w| known.iter().any(|k| w.starts_with(k)));
    let fixed: Vec<&&str> = known.iter().filter(|k| !still.iter().any(|w| w.starts_with(**k))).collect();
    println!("\nPendientes conocidos: {}", still.len());
    for w in &still {
        println!("  {w}");
    }
    println!("Distinto de lo esperado: {}", new.len());
    for w in &new {
        println!("  {w}");
    }
    for k in &fixed {
        println!("Ya no pasa, quítelo de «conocidos»: {k}");
    }
    assert!(new.is_empty() && fixed.is_empty(), "{} distintos de lo esperado, {} conocidos que ya no pasan", new.len(), fixed.len());
}
