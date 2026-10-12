//! The data of the institution that a document of a known kind says, read by rules (IA6, ADR-034): the RFC of the
//! constancia, the number and date of the oficio of donataria, the legal form, year and purpose of the acta, the
//! representative of the poder. No AI: each rule knows where the datum is and how it is written, and every value
//! comes with the page and the line it was read from. What the rule does not find, it does not guess.

use super::dates::dates_in;
use crate::common::forms::is_rfc_moral;
use crate::storage::fts::fold;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::LazyLock;

/// A datum found in a document: the field of the catalog, its value as the form takes it, and where it was read.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    pub field: &'static str,
    pub value: Value,
    /// The page, from 1.
    pub page: usize,
    /// The line it was read from.
    pub line: String,
}

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pat).unwrap());
    };
}

re!(RFC, r"(?i)\bR\.?F\.?C\.?\s*:?\s*([A-ZÑ&]{3}\d{6}[A-Z\d]{3})\b");
re!(LEGAL_NAME, r"(?i)denominaci[oó]n\s*(?:o|/)\s*raz[oó]n\s+social\s*:?\s*(.+)$");
re!(CAPITAL, r"(?i)r[eé]gimen\s+(?:de\s+)?capital\s*:?\s*(.+)$");
re!(POSTAL, r"(?i)c[oó]digo\s+postal\s*:?\s*(\d{5})\b");
re!(OFICIO, r"(?i)oficio\s+(?:n[uú]mero|n[uú]m\.?|no\.?)\s*:?\s*(\d[0-9A-Z.\-/]{4,})");
re!(NAMED, r#"(?i)se\s+denominar[aá]\s*[«"“]([^»"”]{3,120})[»"”]"#);
re!(ARTICLE, r"(?i)\bart[ií]culo\b|\bcap[ií]tulo\b");
re!(IN_FAVOR, r"a\s+favor\s+de\s+(?:(?:la|el)\s+(?:señora|señor|licenciada|licenciado|ciudadana|ciudadano|contadora|contador|C\.)\s+)?([A-ZÁÉÍÓÚÑ]{2,}(?:\s+[A-ZÁÉÍÓÚÑ]{2,}){1,5})");
re!(CLUNI, r"(?i)\bCLUNI\)?\s*:?\s*([A-Z0-9]{10,20})\b");
re!(FOLIO, r"(?i)\bfolio\s*(?:n[uú]mero|no\.?|n[uú]m\.?)?\s*:?\s*([A-Z0-9][A-Z0-9\-/]{2,20})");

/// The longest line kept as the place a datum was read from.
const LINE_CHARS: usize = 200;

fn short(line: &str) -> String {
    let l = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if l.chars().count() <= LINE_CHARS {
        return l;
    }
    format!("{}…", l.chars().take(LINE_CHARS - 1).collect::<String>().trim_end())
}

/// The first line of the document a regex matches: (page, the first group, the line).
fn first(pages: &[String], re: &Regex) -> Option<(usize, String, String)> {
    pages.iter().enumerate().find_map(|(i, p)| p.lines().find_map(|l| re.captures(l).map(|c| (i + 1, c[1].trim().to_string(), short(l)))))
}

/// The first line whose words (without accents) contain all of these: (page, the whole line).
fn whole_line_with(pages: &[String], words: &[&str]) -> Option<(usize, String)> {
    pages.iter().enumerate().find_map(|(i, p)| p.lines().find(|l| words.iter().all(|w| fold(l).contains(w))).map(|l| (i + 1, l.to_string())))
}

/// The same, the line cut to be shown.
fn line_with(pages: &[String], words: &[&str]) -> Option<(usize, String)> {
    whole_line_with(pages, words).map(|(p, l)| (p, short(&l)))
}

fn found(field: &'static str, value: Value, (page, line): (usize, String)) -> Found {
    Found { field, value, page, line }
}

/// The legal form a text names, as the catalog codes it.
fn legal_form_of(text: &str) -> Option<&'static str> {
    let t = fold(text);
    let t = t.replace('.', "");
    [
        ("institucion de asistencia privada", "iap"),
        ("institucion de beneficencia privada", "ibp"),
        ("asociacion de beneficencia privada", "abp"),
        ("asociacion civil", "ac"),
        ("sociedad civil", "sc"),
        (" iap", "iap"),
        (" ibp", "ibp"),
        (" abp", "abp"),
    ]
    .iter()
    .find(|(w, _)| t.contains(w))
    .map(|(_, c)| *c)
}

/// The activity a donataria is authorized for, as the catalog codes it.
fn donee_category_of(text: &str) -> Option<&'static str> {
    let t = fold(text);
    [
        ("asistencial", "assistance"),
        ("educativ", "education"),
        ("investigacion cientifica", "research"),
        ("cultural", "culture"),
        ("becante", "scholarships"),
        ("becas", "scholarships"),
        ("ecologic", "ecology"),
        ("especies", "species"),
        ("apoyo economico de donatarias", "support_donees"),
        ("obras o servicios publicos", "public_works"),
        ("bibliotecas", "libraries"),
        ("museos", "museums"),
        ("desarrollo social", "social_development"),
    ]
    .iter()
    .find(|(w, _)| t.contains(w))
    .map(|(_, c)| *c)
}

/// A name written in capitals, with capitals only at the start of each word («MARÍA LÓPEZ» → «María López»).
fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let lower = w.to_lowercase();
            if matches!(lower.as_str(), "de" | "del" | "la" | "las" | "los" | "y") {
                return lower;
            }
            let mut c = lower.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The first date of a page, with its line.
fn first_date(pages: &[String], page: usize) -> Option<(String, (usize, String))> {
    let text = pages.get(page - 1)?;
    let (at, iso) = dates_in(text).into_iter().next()?;
    let line = text[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
    Some((iso, (page, short(&text[line..end]))))
}

/// The text of a document from where it says it has an object (`tendrá por objeto`) to the next article.
fn purpose(pages: &[String]) -> Option<Found> {
    pages.iter().enumerate().find_map(|(i, p)| {
        let folded = fold(p);
        let at = folded.find("por objeto")?;
        // the folded text has the same letters, one for one: the positions match by character
        let start_char = folded[..at].chars().count() + "por objeto".len();
        let rest: String = p.chars().skip(start_char).collect();
        let end = ARTICLE.find(&rest).map_or(rest.len(), |m| m.start());
        let body = rest[..end].trim_start_matches(|c: char| c == ',' || c == ':' || c.is_whitespace());
        let body = body.trim_start_matches("sin propósito de lucro").trim_start_matches(|c: char| c == ',' || c == ':' || c.is_whitespace());
        let body = body.split_whitespace().collect::<Vec<_>>().join(" ");
        (body.chars().count() >= 20).then(|| {
            let body: String = body.chars().take(1_500).collect();
            Found { field: "institution.purpose", value: json!(body.clone()), page: i + 1, line: short(&body) }
        })
    })
}

/// What the rules read from a document of this kind.
pub fn read(kind: &str, pages: &[String]) -> Vec<Found> {
    let mut out = Vec::new();
    match kind {
        "tax_status_certificate" => {
            if let Some((p, rfc, line)) = first(pages, &RFC).filter(|x| is_rfc_moral(&x.1)) {
                out.push(found("institution.legal_rfc", json!(rfc.to_uppercase()), (p, line)));
            }
            if let Some((p, name, line)) = first(pages, &LEGAL_NAME) {
                out.push(found("institution.legal_name", json!(name), (p, line)));
            }
            if let Some((p, form, line)) = first(pages, &CAPITAL).and_then(|(p, v, l)| legal_form_of(&v).map(|f| (p, f, l))) {
                out.push(found("institution.legal_form", json!(form), (p, line)));
            }
            if let Some(at) = line_with(pages, &["personas morales con fines no lucrativos"]) {
                out.push(found("institution.tax_regime", json!("non_profit"), at));
            } else if let Some(at) = line_with(pages, &["regimen general de ley"]) {
                out.push(found("institution.tax_regime", json!("general"), at));
            }
            if let Some((p, cp, line)) = first(pages, &POSTAL) {
                out.push(found("institution.fiscal_postal_code", json!(cp), (p, line)));
            }
        }
        "donee_authorization" => {
            if let Some(at) = line_with(pages, &["autoriz"]).or_else(|| line_with(pages, &["donativos deducibles"])) {
                out.push(found("institution.authorized_donee", json!("yes"), at));
            }
            if let Some((p, n, line)) = first(pages, &OFICIO) {
                out.push(found("institution.donee_letter_number", json!(n.trim_end_matches('.')), (p, line)));
            }
            if let Some((date, at)) = first_date(pages, 1) {
                out.push(found("institution.donee_letter_date", json!(date), at));
            }
            if let Some((p, line)) = whole_line_with(pages, &["rubro"]) {
                // the activity is the words right after «rubro»
                let after: String = fold(&line).split("rubro").nth(1).unwrap_or_default().chars().take(80).collect();
                if let Some(c) = donee_category_of(&after) {
                    out.push(found("institution.donee_category", json!(c), (p, short(&line))));
                }
            }
        }
        "articles_of_incorporation" => {
            if let Some((p, name, line)) = first(pages, &NAMED) {
                out.push(found("institution.legal_name", json!(name), (p, line)));
            }
            if let Some((p, form, line)) = pages.iter().enumerate().find_map(|(i, pg)| pg.lines().find_map(|l| legal_form_of(l).map(|f| (i + 1, f, short(l))))) {
                out.push(found("institution.legal_form", json!(form), (p, line)));
            }
            if let Some((date, at)) = first_date(pages, 1) {
                out.push(found("institution.founded_year", json!(date[..4].parse::<i64>().unwrap_or_default()), at));
            }
            out.extend(purpose(pages));
        }
        "power_of_attorney" => {
            if let Some((p, name, line)) = first(pages, &IN_FAVOR) {
                out.push(found("institution.legal_rep_name", json!(title_case(&name)), (p, line)));
            }
            // the date it is valid until: the first one after «hasta», where the document speaks of its validity
            let until = pages.iter().enumerate().find_map(|(i, p)| {
                let folded = fold(p);
                let from = folded.find("vigencia")?;
                let hasta = folded[from..].find("hasta")? + from;
                let start_char = folded[..hasta].chars().count();
                let rest: String = p.chars().skip(start_char).collect();
                let (at, iso) = dates_in(&rest).into_iter().next()?;
                let end = rest[at..].find('\n').map_or(rest.len(), |e| at + e);
                Some((iso, (i + 1, short(&rest[..end]))))
            });
            if let Some((date, at)) = until {
                out.push(found("institution.legal_rep_valid_until", json!(date), at));
            }
        }
        "cluni_certificate" => {
            if let Some((p, key, line)) = first(pages, &CLUNI) {
                out.push(found("institution.cluni", json!("yes"), (p, line.clone())));
                out.push(found("institution.cluni_key", json!(key), (p, line)));
            }
        }
        "junta_registration" => {
            if let Some((p, folio, line)) = first(pages, &FOLIO).filter(|x| x.1.chars().any(|c| c.is_ascii_digit())) {
                out.push(found("institution.junta_folio", json!(folio), (p, line)));
            }
        }
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::archive::search::tests::fixture_pages;

    fn pages_of(name: &str) -> Vec<String> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/expediente").join(name);
        fixture_pages(&std::fs::read_to_string(path).unwrap())
    }

    fn values(found: &[Found]) -> Vec<(&str, Value, usize)> {
        found.iter().map(|f| (f.field, f.value.clone(), f.page)).collect()
    }

    /// ADR-034, IA6: with fictitious fixtures, the RFC comes out of the constancia by rules, without AI; and so do
    /// the name, the legal form, the regime and the fiscal postal code, each with its page.
    #[test]
    fn the_rfc_comes_out_of_the_constancia_by_rules() {
        let f = read("tax_status_certificate", &pages_of("constancia-situacion-fiscal.txt"));
        assert_eq!(
            values(&f),
            vec![
                ("institution.legal_rfc", json!("CRF150310KX4"), 1),
                ("institution.legal_name", json!("CASA DE REPOSO FICTICIA SAN EJEMPLO"), 1),
                ("institution.legal_form", json!("iap"), 1),
                ("institution.tax_regime", json!("non_profit"), 2),
                ("institution.fiscal_postal_code", json!("03100"), 1),
            ]
        );
        assert_eq!(f[0].line, "RFC: CRF150310KX4");
    }

    #[test]
    fn the_oficio_of_donataria_gives_its_number_date_and_activity() {
        let f = read("donee_authorization", &pages_of("oficio-donataria.txt"));
        assert_eq!(
            values(&f),
            vec![
                ("institution.authorized_donee", json!("yes"), 1),
                ("institution.donee_letter_number", json!("600-04-02-2017-11234"), 1),
                ("institution.donee_letter_date", json!("2017-03-14"), 1),
                ("institution.donee_category", json!("assistance"), 1),
            ]
        );
    }

    #[test]
    fn the_acta_gives_its_name_legal_form_year_and_purpose() {
        let f = read("articles_of_incorporation", &pages_of("acta-constitutiva.txt"));
        let v = values(&f);
        assert_eq!(v[0], ("institution.legal_name", json!("Casa de Reposo Ficticia San Ejemplo"), 1));
        assert_eq!(v[1], ("institution.legal_form", json!("iap"), 1));
        assert_eq!(v[2], ("institution.founded_year", json!(2015), 1));
        let purpose = f[3].value.as_str().unwrap();
        assert_eq!((f[3].field, f[3].page), ("institution.purpose", 2));
        assert!(purpose.starts_with("I. Brindar alojamiento") && purpose.contains("IV. Recibir donativos") && !purpose.contains("ARTÍCULO QUINTO"), "{purpose}");
    }

    /// The fixture names no person (only «la Directora General»): no representative is guessed. A deed that names
    /// one in capitals, as notaries write them, gives it.
    #[test]
    fn the_poder_gives_its_representative_only_when_it_names_one() {
        let f = read("power_of_attorney", &pages_of("poder-notarial.txt"));
        assert_eq!(values(&f), vec![("institution.legal_rep_valid_until", json!("2027-01-18"), 2)]);
        let named = vec!["otorga a favor de la señora ROSA FICTICIA DEL EJEMPLO un PODER GENERAL para actos de administración.".to_string()];
        assert_eq!(values(&read("power_of_attorney", &named)), vec![("institution.legal_rep_name", json!("Rosa Ficticia del Ejemplo"), 1)]);
    }

    #[test]
    fn the_cluni_and_the_folio_of_the_junta() {
        let cluni = vec!["Clave Única de Inscripción (CLUNI): CRF15031009015".to_string()];
        assert_eq!(values(&read("cluni_certificate", &cluni)), vec![("institution.cluni", json!("yes"), 1), ("institution.cluni_key", json!("CRF15031009015"), 1)]);
        let junta = vec!["Constancia de registro ante la Junta de Asistencia Privada\nFolio número: JAP-1234/2015".to_string()];
        assert_eq!(values(&read("junta_registration", &junta)), vec![("institution.junta_folio", json!("JAP-1234/2015"), 1)]);
        assert!(read("internal_rules", &pages_of("reglamento-interno.txt")).is_empty(), "a kind with no data of the institution");
    }
}
