//! What a document is, by its words (IA6, ADR-034): the kinds of the record of the institution (ADR-033 §5,
//! `RECORD_TYPES`) and the class each one belongs to. Rules first: a constancia del SAT or an oficio of donataria
//! say what they are in their first lines. What no rule recognizes stays without a kind until the person says it.
//! Invoices and receipts are not recognized here: they are uploaded only in Finanzas (decision of 2026-10-12).

use crate::storage::fts::fold;

/// The classes the person chooses when uploading a document.
pub const CLASSES: &[&str] = &["legal", "financial", "programs"];

/// A kind of document: its code (that of the record), its class, and the phrases that say it. A phrase is a list of
/// words that must all be there (without accents, lowercase).
pub struct Kind {
    pub code: &'static str,
    pub class: &'static str,
    phrases: &'static [&'static [&'static str]],
}

pub const KINDS: &[Kind] = &[
    Kind { code: "tax_status_certificate", class: "legal", phrases: &[&["constancia de situacion fiscal"], &["cedula de identificacion fiscal"]] },
    Kind { code: "tax_compliance_opinion", class: "legal", phrases: &[&["opinion del cumplimiento"], &["opinion de cumplimiento", "obligaciones fiscales"]] },
    Kind { code: "donee_authorization", class: "legal", phrases: &[&["donativos deducibles", "oficio", "autoriza"], &["donataria autorizada", "oficio"]] },
    Kind { code: "cluni_certificate", class: "legal", phrases: &[&["clave unica de inscripcion"], &["registro federal de las organizaciones de la sociedad civil"]] },
    Kind { code: "power_of_attorney", class: "legal", phrases: &[&["poder general"], &["pleitos y cobranzas", "apoderad"], &["otorga", "poder especial"]] },
    Kind { code: "bylaws_amendments", class: "legal", phrases: &[&["reforma", "estatutos", "protocoliza"], &["modificacion", "estatutos", "acta de asamblea"]] },
    Kind { code: "articles_of_incorporation", class: "legal", phrases: &[&["escritura", "constituir", "estatutos"], &["acta constitutiva"]] },
    Kind { code: "junta_registration", class: "legal", phrases: &[&["junta de asistencia privada", "constancia de registro"], &["junta de asistencia privada", "registro", "folio"]] },
    Kind { code: "social_care_registry", class: "legal", phrases: &[&["registro nacional de centros de asistencia social"]] },
    Kind { code: "sanitary_notice", class: "legal", phrases: &[&["aviso de funcionamiento"]] },
    Kind { code: "land_use_license", class: "legal", phrases: &[&["uso de suelo"]] },
    Kind { code: "civil_protection_program", class: "legal", phrases: &[&["programa interno de proteccion civil"]] },
    Kind { code: "privacy_notice", class: "legal", phrases: &[&["aviso de privacidad"]] },
    Kind { code: "internal_rules", class: "legal", phrases: &[&["reglamento interno"]] },
    Kind { code: "org_chart", class: "legal", phrases: &[&["organigrama"]] },
    Kind { code: "financial_statements", class: "financial", phrases: &[&["estados financieros"], &["estado de actividades"], &["estado de posicion financiera"]] },
    Kind { code: "junta_budget", class: "financial", phrases: &[&["presupuesto", "junta de asistencia privada"]] },
    Kind { code: "annual_report", class: "programs", phrases: &[&["informe anual"], &["informe de actividades"]] },
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
    let all = fold(&pages.join("\n"));
    let body: String = all.chars().take(BODY_CHARS).collect();
    let head: String = all.chars().take(HEAD_CHARS).collect();
    let mut scored: Vec<(u32, &'static str)> = KINDS
        .iter()
        .map(|k| {
            let s: u32 = k.phrases.iter().map(|p| if matches(&head, p) { 3 } else if matches(&body, p) { 1 } else { 0 }).sum();
            (s, k.code)
        })
        .filter(|(s, _)| *s > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    match scored.as_slice() {
        [] => None,
        [(_, only)] => Some(only),
        [(a, best), (b, _), ..] => (a > b).then_some(*best),
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
}
