//! What a document is, by its words (IA6, ADR-034): the kinds of the record of the institution (ADR-033 §5,
//! `RECORD_TYPES`) and the class each one belongs to. Rules first: a constancia del SAT or an oficio of donataria
//! say what they are in their first lines. What no rule recognizes stays without a kind until the person says it.
//! Invoices and receipts are not recognized here: they are uploaded only in Finanzas (decision of 2026-10-12).

use crate::storage::fts::fold;

/// The classes the person chooses when uploading a document.
pub const CLASSES: &[&str] = &["legal", "financial", "programs"];

/// A kind of document: its code (that of the record), its class, the phrases that say it in its text, and the ones
/// that say it in the name of a file. A phrase is a list of words that must all be there (without accents, lowercase).
pub struct Kind {
    pub code: &'static str,
    pub class: &'static str,
    phrases: &'static [&'static [&'static str]],
    names: &'static [&'static [&'static str]],
}

pub const KINDS: &[Kind] = &[
    Kind { code: "tax_status_certificate", class: "legal", phrases: &[&["constancia de situacion fiscal"], &["cedula de identificacion fiscal"]], names: &[&["constancia", "fiscal"], &["csf"]] },
    Kind { code: "tax_compliance_opinion", class: "legal", phrases: &[&["opinion del cumplimiento"], &["opinion de cumplimiento", "obligaciones fiscales"]], names: &[&["opinion"], &["32 d"], &["32d"]] },
    Kind { code: "donee_authorization", class: "legal", phrases: &[&["donativos deducibles", "oficio", "autoriza"], &["donataria autorizada", "oficio"]], names: &[&["donataria"]] },
    Kind { code: "cluni_certificate", class: "legal", phrases: &[&["clave unica de inscripcion"], &["registro federal de las organizaciones de la sociedad civil"]], names: &[&["cluni"]] },
    Kind { code: "power_of_attorney", class: "legal", phrases: &[&["poder general"], &["pleitos y cobranzas", "apoderad"], &["otorga", "poder especial"]], names: &[&["poder"]] },
    Kind { code: "bylaws_amendments", class: "legal", phrases: &[&["reforma", "estatutos", "protocoliza"], &["modificacion", "estatutos", "acta de asamblea"]], names: &[&["reforma"], &["modificacion", "estatutos"]] },
    Kind { code: "articles_of_incorporation", class: "legal", phrases: &[&["escritura", "constituir", "estatutos"], &["acta constitutiva"]], names: &[&["constitutiva"]] },
    // the constancia of registry, or the sheet of the institution in the directory of the Junta («Número de Institución»);
    // «junta» apart from «asistencia privada»: a faded scan reads «Juntal de asistencia privada»
    Kind { code: "junta_registration", class: "legal", phrases: &[&["junta", "asistencia privada", "constancia de registro"], &["junta", "asistencia privada", "registro", "folio"], &["numero de institucion", "objeto"]], names: &[&["registro", "junta"], &["registro", "jap"]] },
    Kind { code: "social_care_registry", class: "legal", phrases: &[&["registro nacional de centros de asistencia social"]], names: &[&["rncas"], &["centros de asistencia"]] },
    Kind { code: "sanitary_notice", class: "legal", phrases: &[&["aviso de funcionamiento"]], names: &[&["aviso", "funcionamiento"]] },
    Kind { code: "land_use_license", class: "legal", phrases: &[&["uso de suelo"], &["uso del suelo"]], names: &[&["uso", "suelo"]] },
    Kind { code: "civil_protection_program", class: "legal", phrases: &[&["programa interno de proteccion civil"]], names: &[&["proteccion civil"]] },
    Kind { code: "privacy_notice", class: "legal", phrases: &[&["aviso de privacidad"]], names: &[&["privacidad"]] },
    Kind { code: "internal_rules", class: "legal", phrases: &[&["reglamento interno"]], names: &[&["reglamento"]] },
    Kind { code: "org_chart", class: "legal", phrases: &[&["organigrama"]], names: &[&["organigrama"]] },
    Kind { code: "financial_statements", class: "financial", phrases: &[&["estados financieros"], &["estado de actividades"], &["estado de posicion financiera"]], names: &[&["estados financieros"], &["estado financiero"]] },
    Kind { code: "junta_budget", class: "financial", phrases: &[&["presupuesto", "junta", "asistencia privada"]], names: &[&["presupuesto"]] },
    Kind { code: "annual_report", class: "programs", phrases: &[&["informe anual"], &["informe de actividades"]], names: &[&["informe"]] },
];

pub fn kind(code: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.code == code)
}

/// How much of the beginning of a document says what it is: its title and first lines weigh more.
const HEAD_CHARS: usize = 400;
/// How much text is read to recognize it.
const BODY_CHARS: usize = 6_000;

fn matches(text: &str, phrase: &[&str]) -> bool {
    phrase.iter().all(|w| text.contains(w))
}

/// The kind of a document by its words, when a rule recognizes it: a phrase in its first lines counts three times
/// one further on, and a tie between two kinds is no kind at all.
pub fn recognize(pages: &[String]) -> Option<&'static str> {
    let mut scored: Vec<(u32, &'static str)> = scores(pages).into_iter().filter(|(s, _)| *s > 0).collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    match scored.as_slice() {
        [] => None,
        [(_, only)] => Some(only),
        [(a, best), (b, _), ..] => (a > b).then_some(*best),
    }
}

/// How much each kind's phrases are in a document: three for one in its first lines, one for one further on.
fn scores(pages: &[String]) -> Vec<(u32, &'static str)> {
    let all = fold(&pages.join("\n"));
    let body: String = all.chars().take(BODY_CHARS).collect();
    let head: String = all.chars().take(HEAD_CHARS).collect();
    KINDS.iter().map(|k| (k.phrases.iter().map(|p| if matches(&head, p) { 3 } else if matches(&body, p) { 1 } else { 0 }).sum(), k.code)).collect()
}

/// How much the name of a file weighs for the kind it names: as much as a phrase in the text, never more than what
/// the text says of another kind (measured with the full test and the battery, ADR-042 §4).
pub const NAME_WEIGHT: i32 = 1;
/// From this confidence up a file is kept on the document it was put on without asking (measured, ADR-042 §4).
pub const THRESHOLD: i32 = 1;

/// Whether the name of a file says the kind: «04-constancia-situacion-fiscal.pdf», «Registro JAP.png».
fn named(file_name: &str, k: &Kind) -> bool {
    let stem = file_name.rsplit_once('.').map_or(file_name, |(s, _)| s);
    let words = fold(stem).split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" ");
    k.names.iter().any(|p| matches(&words, p))
}

/// How sure the code is that a file is the document it was put on (ADR-042 §4): the phrases of that kind found in
/// its text, less those of the best of the other kinds, plus the name of the file when it says that kind. A file
/// with no text to read (a picture nobody can read) has only its name.
pub fn confidence(pages: Option<&[String]>, file_name: &str, put_on: &str) -> i32 {
    let (words, named) = evidence(pages, file_name, put_on);
    words + if named { NAME_WEIGHT } else { 0 }
}

/// What a file says of the kind it was put on: its phrases less the best of another kind's, and whether its name
/// says it.
fn evidence(pages: Option<&[String]>, file_name: &str, put_on: &str) -> (i32, bool) {
    let Some(k) = kind(put_on) else { return (0, false) };
    let scored = pages.map(scores).unwrap_or_default();
    let own = scored.iter().find(|(_, c)| *c == put_on).map_or(0, |(s, _)| *s as i32);
    let rival = scored.iter().filter(|(_, c)| *c != put_on).map(|(s, _)| *s as i32).max().unwrap_or(0);
    (own - rival, named(file_name, k))
}

/// Whether a file is kept on the document it was put on, by its words (ADR-042 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Its words say it is that document and no other kind beats them, or its name says it and nothing says
    /// otherwise: it is kept, read and filled.
    Sure,
    /// Its words say it is another document: it is not kept there, and the person is asked.
    Other(&'static str),
    /// Nothing says what it is (a picture that could not be read, words no rule knows): the person is asked.
    Unsure,
}

/// Where a file goes. A document the institution named itself («Otro documento») is what the person says.
pub fn placement(pages: Option<&[String]>, file_name: &str, put_on: &str) -> Placement {
    if kind(put_on).is_none() || confidence(pages, file_name, put_on) >= THRESHOLD {
        return Placement::Sure;
    }
    match pages.and_then(recognize).filter(|k| *k != put_on) {
        Some(other) => Placement::Other(other),
        None => Placement::Unsure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::archive::search::tests::fixture_pages;

    fn pages_of(name: &str) -> Vec<String> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente").join(name);
        fixture_pages(&std::fs::read_to_string(path).unwrap())
    }

    /// Each fictitious document of the record is recognized by its words, with no AI; a contract that is no kind of
    /// the record stays without one.
    #[test]
    fn each_document_of_the_record_is_recognized_by_its_words() {
        for (file, expected) in [
            ("acta-constitutiva.txt", Some("articles_of_incorporation")),
            ("poder-notarial.txt", Some("power_of_attorney")),
            ("constancia-situacion-fiscal.txt", Some("tax_status_certificate")),
            ("oficio-donataria.txt", Some("donee_authorization")),
            ("reglamento-interno.txt", Some("internal_rules")),
            ("aviso-funcionamiento.txt", Some("sanitary_notice")),
            ("estados-financieros-2025.txt", Some("financial_statements")),
            ("informe-anual-2025.txt", Some("annual_report")),
            ("programa-proteccion-civil.txt", Some("civil_protection_program")),
            ("opinion-cumplimiento.txt", Some("tax_compliance_opinion")),
            ("cluni.txt", Some("cluni_certificate")),
            ("registro-junta.txt", Some("junta_registration")),
            ("contrato-comodato.txt", None),
        ] {
            assert_eq!(recognize(&pages_of(file)), expected, "{file}");
        }
    }

    #[test]
    fn every_kind_is_a_document_of_the_record_with_a_class() {
        for k in KINDS {
            assert!(crate::core::institution::catalog::RECORD_TYPES.iter().any(|(c, _)| *c == k.code), "{}", k.code);
            assert!(CLASSES.contains(&k.class), "{}", k.code);
            assert!(k.phrases.iter().flat_map(|p| p.iter()).all(|w| *w == fold(w)), "{}: phrases without accents, lowercase", k.code);
        }
        assert_eq!(recognize(&["Factura. Total con IVA: $1,160.00".into()]), None, "an invoice is not recognized here");
    }

    fn txt_fixtures() -> Vec<(String, Vec<String>)> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente");
        let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect();
        files.sort();
        files.iter().map(|p| { let n = p.file_name().unwrap().to_string_lossy().to_string(); (n.clone(), pages_of(&n)) }).collect()
    }

    /// ADR-042 §4, the battery: each fictitious document is kept without asking on its own document, by its words
    /// alone (a file named «archivo.pdf»); on any other document it is never kept, and the person is told what it
    /// looks like. The contract is no kind: on a document of the catalog it is asked about.
    #[test]
    fn a_file_is_kept_only_on_its_own_document() {
        for (file, pages) in txt_fixtures() {
            let own = recognize(&pages);
            for k in KINDS {
                let got = placement(Some(&pages), "archivo.pdf", k.code);
                let want = match own {
                    Some(o) if o == k.code => Placement::Sure,
                    Some(o) => Placement::Other(o),
                    None => Placement::Unsure,
                };
                assert_eq!(got, want, "{file} on {}", k.code);
            }
            assert_eq!(placement(Some(&pages), "archivo.pdf", "other"), Placement::Sure, "an «Otro documento» is what the person says");
        }
    }

    /// The name of a file raises the confidence only when the words do not say another document: a constancia named
    /// «acta-constitutiva-version-final.pdf» is still a constancia; a photo nobody can read, named «licencia uso de
    /// suelo.jpg», is kept on its license; named «foto.jpg», the person is asked.
    #[test]
    fn the_name_of_a_file_never_beats_its_words() {
        let constancia = pages_of("constancia-situacion-fiscal.txt");
        assert_eq!(placement(Some(&constancia), "19-acta-constitutiva-version-final.pdf", "articles_of_incorporation"), Placement::Other("tax_status_certificate"));
        assert_eq!(placement(None, "licencia uso de suelo.jpg", "land_use_license"), Placement::Sure);
        assert_eq!(placement(None, "IMG_2031.jpg", "land_use_license"), Placement::Unsure);
        assert_eq!(placement(None, "Registro_JAP.png", "junta_registration"), Placement::Sure);
        assert_eq!(placement(None, "Registro_JAP.png", "social_care_registry"), Placement::Unsure);
        assert!(named("04-constancia-situacion-fiscal.pdf", kind("tax_status_certificate").unwrap()));
        assert!(named("Opinión 32-D.jpg", kind("tax_compliance_opinion").unwrap()));
        assert!(!named("constancia-cluni.pdf", kind("tax_status_certificate").unwrap()));
        assert_eq!(placement(Some(&["Certificado único de zonificación de uso del suelo".into()]), "a.jpg", "land_use_license"), Placement::Sure);
    }

    /// The measurement of ADR-042 §4: every file of the full test (read as the screen reads it, scans and photos with
    /// OCR, about a minute) and of the battery, put on its own document and on every other one, with each name weight
    /// and threshold; how many end on a wrong document and how many are asked on their own. Prints the table.
    ///   cargo test kind_confidence_measure -- --ignored --nocapture   (with $env:CIMIENTO_KIND_SHOW="08" it shows what file 08 says)
    #[test]
    #[ignore]
    fn kind_confidence_measure() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente/prueba-completa");
        let expected: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(folder.join("esperado.json")).unwrap()).unwrap();
        // (file, its true kind or none, its pages or none)
        let mut files: Vec<(String, Option<&'static str>, Option<Vec<String>>)> = vec![];
        for step in expected["pasos"].as_array().unwrap() {
            let name = step["archivo"].as_str().unwrap();
            let Ok(read) = crate::core::records::read_original(&folder.join("archivos").join(name)) else { continue };
            let truth = step["espera"]["pregunta"]["parece"].as_str().or(step["documento"].as_str()).and_then(|c| kind(c)).map(|k| k.code);
            files.push((name.to_string(), truth, read.pages_for_test()));
        }
        for (name, pages) in txt_fixtures() {
            let truth = recognize(&pages);
            files.push((name, truth, Some(pages)));
        }
        println!("{:<48} {:<26} {:>6} {:>6}  puntos", "archivo", "tipo", "suyo", "nombre");
        for (name, truth, pages) in &files {
            let mut s: Vec<_> = pages.as_deref().map(scores).unwrap_or_default().into_iter().filter(|(s, _)| *s > 0).collect();
            s.sort_by(|a, b| b.0.cmp(&a.0));
            let (words, by_name) = truth.map_or((0, false), |t| evidence(pages.as_deref(), name, t));
            println!("{name:<48} {:<26} {words:>6} {by_name:>6}  {s:?}", truth.unwrap_or("—"));
            // what a file of the test says, to see why a rule does not recognize it (fictitious files only)
            if std::env::var("CIMIENTO_KIND_SHOW").is_ok_and(|v| name.starts_with(&v)) {
                println!("{}", fold(&pages.as_deref().unwrap_or_default().join("
")).chars().take(HEAD_CHARS * 2).collect::<String>());
            }
        }
        println!("
peso del nombre / umbral: en un lugar equivocado · preguntas en su lugar (de {} en su lugar)", files.iter().filter(|f| f.1.is_some()).count());
        for w in 0..=3 {
            for t in 1..=3 {
                let (mut wrong, mut asked) = (0, 0);
                for (name, truth, pages) in &files {
                    for k in KINDS {
                        let (words, by_name) = evidence(pages.as_deref(), name, k.code);
                        let kept = words + if by_name { w } else { 0 } >= t;
                        match truth {
                            Some(tk) if *tk == k.code => asked += usize::from(!kept),
                            _ => wrong += usize::from(kept),
                        }
                    }
                }
                let chosen = if (w, t) == (NAME_WEIGHT, THRESHOLD) { "  ← el elegido" } else { "" };
                println!("  {w} / {t}: {wrong} · {asked}{chosen}");
            }
        }
    }
}
