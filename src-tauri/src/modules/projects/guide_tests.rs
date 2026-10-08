use super::*;
use crate::modules::projects::diagnosis::advance;
use crate::modules::projects::domain::checklist::Level;
use crate::test_support::*;
use serde_json::json;

fn headings(blocks: &[Block]) -> Vec<String> {
    blocks.iter().filter_map(|b| if let Block::Heading(1, t) = b { Some(t.clone()) } else { None }).collect()
}

#[test]
fn dates_are_written_the_way_a_person_says_them() {
    assert_eq!(spanish_date("2027-05-23"), "23 de mayo de 2027");
    assert_eq!(spanish_date("2026-01-01"), "1 de enero de 2026");
    assert_eq!(spanish_date("mayo"), "mayo");
    assert_eq!(spanish_date("2026-13-40"), "2026-13-40");
}

#[test]
fn a_file_name_is_safe_in_any_folder_and_never_overwrites() {
    assert_eq!(safe_name("Cocina / baños: ¿obra?"), "Cocina baños ¿obra");
    assert_eq!(safe_name(" ... "), "proyecto");
    assert!(safe_name(&"a".repeat(300)).chars().count() <= 80);
    let dir = tempfile::tempdir().unwrap();
    let first = free_path(dir.path(), "guía");
    std::fs::write(&first, b"x").unwrap();
    let second = free_path(dir.path(), "guía");
    assert_ne!(first, second);
    assert!(second.file_name().unwrap().to_string_lossy().ends_with("guía (2).docx"));
}

#[tokio::test]
async fn the_review_names_everything_that_is_missing_and_a_complete_project_is_clean() {
    let (_d, db) = profile_db();
    let pid = project_in_drafting(&db).await;
    let rep = crate::modules::projects::review::review(&db.lock().unwrap(), &pid).unwrap().report;
    let codes: Vec<_> = rep.checks.iter().map(|c| c.code).collect();
    assert!(codes.contains(&"no_budget") && codes.contains(&"no_schedule") && codes.contains(&"section_not_confirmed"), "{codes:?}");
    assert!(!rep.clean());
    // the stage cannot advance while the drafting is incomplete
    assert!(advance(&db, &pid).is_err());

    let (_d2, db2) = profile_db();
    let pid2 = project_fully_drafted(&db2).await;
    let rep = crate::modules::projects::review::review(&db2.lock().unwrap(), &pid2).unwrap().report;
    assert!(rep.clean(), "{:?}", rep.checks.iter().map(|c| (&c.code, &c.text)).collect::<Vec<_>>());
    assert!(rep.checks.iter().any(|c| c.code == "docs_to_gather" || c.level == Level::Info) || rep.checks.is_empty());
}

#[tokio::test]
async fn a_figure_over_what_the_call_allows_blocks_the_review_with_words_that_say_both_numbers() {
    use crate::modules::projects::drafting::{confirm_budget, save_budget_item, BudgetItemInput};
    let (_d, db) = profile_db();
    let pid = project_fully_drafted(&db).await;
    save_budget_item(
        &db,
        &pid,
        BudgetItemInput { id: None, category: "obra".into(), description: "Remodelación completa".into(), quantity: 1.0, unit: None, unit_price_mxn: 300_000.0, vat_included: true, funded_by: crate::modules::projects::domain::budget::Funder::Requested, administrative: false },
        None,
    )
    .unwrap();
    confirm_budget(&db, &pid).unwrap();
    // the text about the cost was marked for review when the budget changed
    let rep = crate::modules::projects::review::review(&db.lock().unwrap(), &pid).unwrap().report;
    let over = rep.checks.iter().find(|c| c.code == "amount_over_max").expect("over the maximum");
    assert!(over.text.contains("$302,320.00") && over.text.contains("$250,000.00"), "{}", over.text);
    assert!(rep.checks.iter().any(|c| c.code == "section_not_confirmed" && c.target.as_deref() == Some("how_much")));
    assert!(!rep.clean());
}

#[tokio::test]
async fn data_of_a_person_that_got_past_the_scanner_blocks_the_review() {
    let (_d, db) = profile_db();
    let pid = project_in_review(&db).await;
    {
        // written straight into the database, as a bug or an old version might have left it
        let conn = db.lock().unwrap();
        crate::modules::projects::storage::drafting::save_section(&conn, &pid, "what", "La señora con CURP LOPM800101MDFRZN09 pidió la obra.", "user", None).unwrap();
        crate::modules::projects::storage::drafting::confirm_section(&conn, &pid, "what").unwrap();
    }
    let rep = crate::modules::projects::review::review(&db.lock().unwrap(), &pid).unwrap().report;
    let c = rep.checks.iter().find(|c| c.code == "scanner_findings").expect("the scanner found it");
    assert!(c.text.contains("1 datos"), "{}", c.text);
    assert!(!rep.clean() && advance(&db, &pid).is_err());
}

#[tokio::test]
async fn the_guide_puts_the_call_first_then_the_project_with_only_what_the_person_confirmed() {
    let (_d, db) = profile_db();
    let pid = project_ready(&db).await;
    let data = gather(&db.lock().unwrap(), &pid).unwrap();
    let blocks = guide_blocks(&data, &["La fecha de cierre ya pasó.".to_string()]);
    assert_eq!(headings(&blocks), vec!["1. Datos de la convocatoria", "2. Qué hay que entregar", "3. Resumen del proyecto", "4. Propuesta del proyecto", "5. Presupuesto", "6. Cronograma", "7. Indicadores y resultados", "8. Datos de la institución", "9. Pendientes por revisar"]);
    let text = blocks_text(&blocks);
    // what is indispensable of the call, with where it says so
    assert!(text.contains("Monto máximo por proyecto") && text.contains("$250,000.00") && text.contains("bases.pdf, página 2"), "{text}");
    assert!(text.contains("Fundación Ficticia") && text.contains("Apoyos 2027"));
    // the project: goal, root cause, who benefits
    assert!(text.contains("Objetivo: Un sistema de mantenimiento preventivo") && text.contains(&format!("Causa de fondo que se quiere atacar: {ROOT}")));
    assert!(text.contains("A quién beneficia: personas que viven en la casa (18 personas)"), "{text}");
    // the proposal: the sections the person confirmed, in the order of the plan
    assert!(text.contains("Texto de la sección «Qué se va a hacer» escrito por la persona."));
    let what = text.find("Qué se va a hacer").unwrap();
    let why = text.find("Por qué hace falta").unwrap();
    assert!(what < why);
    assert!(text.contains("La convocatoria no pide una propuesta aparte"));
    // the budget and the schedule come from the code
    assert!(text.contains("Tubería de cobre") && text.contains("$2,320.00") && text.contains("$3,480.00"), "{text}");
    assert!(text.contains("Lo que aporta la institución") && text.contains("$1,160.00"));
    assert!(text.contains("Revisión mensual") && text.contains("Duración del proyecto: 12 meses."));
    // the institution and what is pending
    assert!(text.contains("Nombre: Asilo Ficticio") && text.contains("Adultos mayores: 18 personas"));
    assert!(text.contains("La fecha de cierre ya pasó.") && text.contains("Cotizaciones"), "{text}");
    // there is no marker of the AI anywhere
    assert!(!text.to_lowercase().contains("ayuda automática"));
}

#[tokio::test]
async fn the_guide_is_written_as_a_word_file_in_the_folder_given_and_nothing_is_overwritten() {
    let (_d, db) = profile_db();
    let pid = project_ready(&db).await;
    let dir = tempfile::tempdir().unwrap();
    let out = export_guide(&db, &pid, dir.path()).unwrap();
    assert!(out.file_name.starts_with("Apoyos 2027 - guía ") && out.file_name.ends_with(".docx"), "{}", out.file_name);
    let again = export_guide(&db, &pid, dir.path()).unwrap();
    assert_ne!(out.path, again.path, "the second one does not replace the first");
    assert!(std::path::Path::new(&again.path).exists() && std::path::Path::new(&out.path).exists());
    // it opens through the same reader the app uses for Word files and says what the guide says
    let text = crate::documents::canonical::package::read_pieces(std::path::Path::new(&out.path)).unwrap().join("\n");
    assert!(text.contains("Guía del proyecto: Apoyos 2027") && text.contains("1. Datos de la convocatoria") && text.contains("$250,000.00") && text.contains("Tubería de cobre"));
    // the log keeps counts only
    let details: String = db.lock().unwrap().query_row("SELECT details_json FROM audit_log WHERE event='export.created' ORDER BY id LIMIT 1", [], |r| r.get(0)).unwrap();
    let v: serde_json::Value = serde_json::from_str(&details).unwrap();
    assert_eq!(v["format"], "docx");
    assert!(v["blocks"].as_u64().unwrap() > 10 && !details.contains("Tubería"));
}

#[tokio::test]
async fn the_guide_only_exists_in_the_last_stage_and_when_the_review_is_clean() {
    let (_d, db) = profile_db();
    let pid = project_in_review(&db).await;
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(export_guide(&db, &pid, dir.path()), Err(ProjectsError::WrongStage)));
    advance(&db, &pid).unwrap();
    // something changes after the review: the guide is refused until it is fixed
    crate::modules::projects::storage::drafting::unconfirm_section(&db.lock().unwrap(), &pid, "what").unwrap();
    assert!(matches!(export_guide(&db, &pid, dir.path()), Err(ProjectsError::Stage(_))));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0, "nothing was written");
    let _ = json!(null);
}

#[tokio::test]
async fn when_the_call_asks_for_a_proposal_the_guide_follows_its_structure() {
    use crate::modules::projects::drafting::{confirm_text, save_text};
    let (_d, db) = profile_db();
    let pid = project_fully_drafted(&db).await;
    // the call lists what the proposal must include; the person confirms that it asks for one
    {
        let conn = db.lock().unwrap();
        let reading = crate::modules::projects::storage::projects::get_project(&conn, &pid).unwrap().unwrap().call_reading_id.unwrap();
        let mut doc = canonical_doc();
        doc["proyecto"] = json!({ "requisitos_del_proyecto": { "estado": "encontrado", "elementos": [
            { "titulo": "Justificación del problema", "evidencias": [{ "cita": "Justificación del problema", "pagina": 7, "documento": "bases.pdf" }], "aplica_a": null, "obligatoriedad": "obligatorio" }
        ] } });
        crate::modules::projects::storage::calls::finish(&conn, &reading, crate::modules::projects::storage::calls::ReadingStatus::Ready, None, Some(&doc), &json!({})).unwrap();
    }
    crate::modules::projects::drafting::set_asks_for_proposal(&db, &pid, true).unwrap();
    // the new requirement has no text yet, so the project is not ready to move on
    assert!(advance(&db, &pid).is_err());
    let key = crate::modules::projects::drafting::drafting_view(&db.lock().unwrap(), &pid).unwrap().sections.iter().find(|s| s.spec.title == "Justificación del problema").unwrap().spec.key.clone();
    save_text(&db, &pid, &key, "La casa necesita un sistema de mantenimiento.", None).unwrap();
    confirm_text(&db, &pid, &key).unwrap();
    // the budget and the schedule keep their confirmation; the call-driven sections are confirmed: the stage moves
    // (the base sections the call does not ask for are optional now)
    assert!(advance(&db, &pid).is_ok());
    let data = gather(&db.lock().unwrap(), &pid).unwrap();
    let text = blocks_text(&guide_blocks(&data, &[]));
    assert!(text.contains("La convocatoria pide una propuesta del proyecto en un documento aparte"));
    assert!(text.contains("Justificación del problema") && text.contains("La casa necesita un sistema de mantenimiento."));
}

/// Writes a sample guide to the folder named in CIMIENTO_SAMPLE_DIR so it can be opened in Word by hand.
///   $env:CIMIENTO_SAMPLE_DIR="D:\...\muestra"; cargo test write_a_sample_guide -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn write_a_sample_guide() {
    let dir = std::env::var("CIMIENTO_SAMPLE_DIR").expect("CIMIENTO_SAMPLE_DIR");
    let (_d, db) = profile_db();
    let pid = project_ready(&db).await;
    let out = export_guide(&db, &pid, std::path::Path::new(&dir)).unwrap();
    println!("{}", out.path);
}
