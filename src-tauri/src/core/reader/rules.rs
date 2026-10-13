//! The data of the institution that a document of a known kind says, read by rules (IA6, ADR-034): the RFC of the
//! constancia, the number and date of the oficio of donataria, the legal form, year and purpose of the acta, the
//! representative of the poder. No AI: each rule knows where the datum is and how it is written, and every value
//! comes with the page and the line it was read from. What the rule does not find, it does not guess.

use super::dates::{dates_in, dates_in_words, leading_number_in_words};
use crate::common::dates::add_days;
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
    /// Worked out by a rule from what the document says (origin `computed`), not written in it as such.
    pub computed: bool,
}

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pat).unwrap());
    };
}

// the six digits of the date may come from a picture with a letter in place of a digit («O» for «0»): read, then fixed
re!(RFC, r"(?i)\bR\.?F\.?C\.?\s*:?\s*([A-ZÑ&]{3}[0-9OoIlSB]{6}[A-Z\d]{3})\b");
re!(LEGAL_NAME, r"(?i)denominaci[oó]n\s*(?:o|/)\s*raz[oó]n\s+social\s*:?\s*(.+)$");
re!(CAPITAL, r"(?i)r[eé]gimen\s+(?:de\s+)?capital\s*:?\s*(.+)$");
re!(POSTAL, r"(?i)c[oó]digo\s+postal\s*:?\s*(\d{5})\b");
re!(OFICIO, r"(?i)oficio\s+(?:n[uú]mero|n[uú]m\.?|no\.?)\s*:?\s*(\d[0-9A-Z.\-/]{4,})");
// how a deed or a registry names the institution: «se denominará “X”», «denominada “X”», «bajo la denominación X, AC.-»,
// «Denominación: X, I.A.P. Número de Institución…»
re!(NAMED, r#"(?i)se\s+denominar[aá]\s*[«"“]([^»"”]{3,120})[»"”]"#);
re!(NAMED_QUOTED, r#"(?i)denominad[ao]s?\s*[«"“]([^»"”]{3,120})[»"”]"#);
re!(UNDER_NAME, r#"(?i)bajo\s+la\s+denominaci[oó]n\s+(?:de\s+)?[«"“]?([^«»"“”:]{3,120}?)[»"”]?\s*(?:,\s*(?:A\.?\s*C|[IL]\.?\s*A\.?\s*P|I\.?\s*B\.?\s*P|A\.?\s*B\.?\s*P|S\.?\s*C)\b|\.\s*-|\.-)"#);
re!(LABEL_NAME, r"(?i)denominaci.n\s*:\s*([^:]{3,120}?)\s+(?:n[uú]mero|objeto|domicilio)\b");
// the abbreviation of the legal form at the end of a name; a scan may read «I.A.P.» as «LAP»
re!(LEGAL_SUFFIX, r"(?i),?\s*\b([IL1]\.?\s*A\.?\s*P|A\.?\s*C|I\.?\s*B\.?\s*P|A\.?\s*B\.?\s*P|S\.?\s*C)\.?\s*$");
// where the purpose starts: «tendrá por objeto», «tendrá como objetivo general», «el objeto social será», «Objeto:»;
// never «con objeto de acordar», which is the purpose of a meeting
re!(PURPOSE_START, r"(?i)\b(?:tendr[aá]|tiene|tenga|ser[aá])\s+(?:por|como)\s+objet(?:o|ivo)(?:\s+general|\s+social)?|\bobjeto\s+social\s+(?:ser[aá]|es|consiste\s+en)|\bcuyo\s+objeto\s+(?:ser[aá]|es)|\bobjeto\s*:");
// where it ends: the next article, clause or chapter («Artículo VI.-», «ARTÍCULO QUINTO.»), or the next label of a
// registry («Patronato Presidente:», «Domicilio legal:»)
re!(SECTION_END, r"(?i)\b(?:art[ií]culo|cl[aá]usula|cap[ií]tulo)\s+(?:[IVXLC]+|\d+|[a-záéíóú]+)\s*(?:\.\s*-|\.-|-|\.|:)|\b(?:patronato|presidente|presidenta|patron[oa]|servicios|domicilio|establecimiento|alcald[ií]a|p[aá]gina\s+web)\b[^:.\n]{0,25}:");
// what a scan leaves of the lines a notary fills with dashes
// in the sheet of the Junta the board follows the purpose, its roles with or without «:» («Patrono protesorero 7.»):
// the purpose ends at the first label, so no name of a person ever goes in it
re!(REGISTRY_END, r"\b(?:Patronat[a-z]*|PATRONAT[A-Z]*|Presidente|Presidenta|PRESIDENTE|Patrono|Patrona|Vicepresidente|Secretari[oa]|Tesorer[oa]|Servicios|SERVICIOS|Domicilio|DOMICILIO|Establecimiento|Alcald[ií]a|P[aá]gina\s+web)\b");
re!(NOISE, r"[-—–=_|]{2,}|\s\|\s");
re!(INSTITUTION_NUMBER, r"(?i)n[uú]mero\s+de\s+(?:instituci[oó]n|registro)\s*:?\s*(\d[0-9A-Z\-/]{0,19})\b");
re!(IN_FAVOR, r"a\s+favor\s+de\s+(?:(?:la|el)\s+(?:señora|señor|licenciada|licenciado|ciudadana|ciudadano|contadora|contador|C\.)\s+)?([A-ZÁÉÍÓÚÑ]{2,}(?:\s+[A-ZÁÉÍÓÚÑ]{2,}){1,5})");
re!(CLUNI, r"(?i)\bCLUNI\)?\s*:?\s*([A-Z0-9]{10,20})\b");
re!(FOLIO, r"(?i)\bfolio\s*(?:n[uú]mero|no\.?|n[uú]m\.?)?\s*:?\s*([A-Z0-9][A-Z0-9\-/]{2,20})");
// the data of a document's own window (ADR-038): when it was issued, until when it is valid, its number
re!(ISSUED, r"(?i)fecha\s+de\s+(?:emisi[oó]n|expedici[oó]n|presentaci[oó]n|inscripci[oó]n|registro)");
re!(VALID_TO, r"(?i)\bvigen(?:te|cia)\b[^.]{0,160}?\bhasta\b");
re!(DEED, r"(?i)\bescritura\s+(?:p[uú]blica\s+)?(?:n[uú]mero|n[uú]m\.?|no\.?)\s*:?\s*(.+)$");
re!(ID_CIF, r"(?i)\bidCIF\s*:?\s*(\d{8,12})\b");

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

/// A page on one line, as a scan breaks a sentence across lines.
fn flat(page: &str) -> String {
    page.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The first place of the document (each page on one line) a regex matches: (page, the first group, the words around).
fn first_flat(pages: &[String], re: &Regex) -> Option<(usize, String, String)> {
    pages.iter().enumerate().find_map(|(i, p)| {
        let text = flat(p);
        let c = re.captures(&text)?;
        let m = c.get(0)?;
        let from = text[..m.start()].char_indices().rev().nth(40).map_or(0, |x| x.0);
        Some((i + 1, c[1].trim().to_string(), short(&text[from..m.end()])))
    })
}

/// The name of the institution as a deed or a registry writes it, without the abbreviation of its legal form, and
/// that form when the abbreviation says it: (page, name, form, the words around).
fn legal_name_in(pages: &[String]) -> Option<(usize, String, Option<&'static str>, String)> {
    // the first page that names it, in any of its ways
    let (page, raw, around) = (0..pages.len()).find_map(|i| {
        [&*NAMED, &*NAMED_QUOTED, &*UNDER_NAME, &*LABEL_NAME].into_iter().find_map(|re| first_flat(&pages[i..=i], re)).map(|(_, raw, around)| (i + 1, raw, around))
    })?;
    let raw = raw.trim_matches(|c: char| c == ',' || c == '.' || c == '“' || c == '”' || c == '"' || c.is_whitespace());
    let form = LEGAL_SUFFIX.captures(raw).and_then(|c| suffix_form(&c[1]));
    let name = LEGAL_SUFFIX.replace(raw, "").trim_matches(|c: char| c == ',' || c.is_whitespace()).to_string();
    (name.chars().count() >= 3).then_some((page, name, form, around))
}

/// The legal form an abbreviation names («A. C.», «I.A.P.», and «LAP» as a scan reads «I.A.P.»).
fn suffix_form(abbr: &str) -> Option<&'static str> {
    // a picture reads the «I» of «I.A.P.» as «1» too
    let a: String = abbr.replace('1', "I").chars().filter(|c| c.is_alphabetic()).collect::<String>().to_lowercase();
    Some(match a.as_str() {
        "iap" | "lap" => "iap",
        "ac" => "ac",
        "ibp" => "ibp",
        "abp" => "abp",
        "sc" => "sc",
        _ => return None,
    })
}

fn found(field: &'static str, value: Value, (page, line): (usize, String)) -> Found {
    Found { field, value, page, line, computed: false }
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

/// The first date written in words in the first `pages` pages, with the words around it.
fn first_date_in_words(pages: &[String], within: usize) -> Option<(String, (usize, String))> {
    pages.iter().take(within).enumerate().find_map(|(i, p)| {
        let text = flat(p);
        let (at, iso) = dates_in_words(&text).into_iter().next()?;
        let end = text[at..].char_indices().nth(90).map_or(text.len(), |x| at + x.0);
        Some((iso, (i + 1, short(&text[at..end]))))
    })
}

/// The first date of a page, with its line.
fn first_date(pages: &[String], page: usize) -> Option<(String, (usize, String))> {
    let text = pages.get(page - 1)?;
    let (at, iso) = dates_in(text).into_iter().next()?;
    let line = text[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
    Some((iso, (page, short(&text[line..end]))))
}

/// The longest purpose kept.
const PURPOSE_CHARS: usize = 1_500;

/// The purpose of the institution: from where the document says it (`tendrá por objeto`, `tendrá como objetivo`,
/// `Objeto:`) to the next article or label, across pages if it goes on, without what a scan leaves of the dashes.
fn purpose(pages: &[String], registry: bool) -> Option<Found> {
    pages.iter().enumerate().find_map(|(i, p)| {
        let text = flat(p);
        let start = PURPOSE_START.find(&text)?;
        // the rest of this page and the next ones, enough to find where it ends
        let mut rest = text[start.end()..].to_string();
        for next in &pages[i + 1..] {
            if rest.chars().count() > PURPOSE_CHARS * 2 {
                break;
            }
            rest.push(' ');
            rest.push_str(&flat(next));
        }
        let mut end = [SECTION_END.find(&rest), registry.then(|| REGISTRY_END.find(&rest)).flatten()]
            .into_iter()
            .flatten()
            .map(|m| m.start())
            .min()
            .unwrap_or(rest.len());
        // it ends at the end of a sentence: what a scan leaves after the last period is not the purpose
        if let Some(dot) = rest[..end].rfind(['.', ';']) {
            if rest[..end].chars().count() - rest[..dot].chars().count() < 160 {
                end = dot + 1;
            }
        }
        let body = NOISE.replace_all(&rest[..end], " ");
        let body = body.trim_start_matches(|c: char| matches!(c, ',' | ':' | '.' | '-') || c.is_whitespace());
        let body = body.trim_start_matches("sin propósito de lucro").trim_start_matches(|c: char| matches!(c, ',' | ':' | '.' | '-') || c.is_whitespace());
        let body = flat(body);
        (body.chars().count() >= 20).then(|| {
            let body: String = body.chars().take(PURPOSE_CHARS).collect();
            Found { field: "institution.purpose", value: json!(body.trim_end()), page: i + 1, line: short(&body), computed: false }
        })
    })
}

/// The name and, when its abbreviation says it, the legal form.
fn name_and_form(pages: &[String], out: &mut Vec<Found>) -> bool {
    let Some((p, name, form, around)) = legal_name_in(pages) else { return false };
    out.push(found("institution.legal_name", json!(name), (p, around.clone())));
    if let Some(f) = form {
        out.push(found("institution.legal_form", json!(f), (p, around)));
    }
    form.is_some()
}

/// What the rules read from a document of this kind.
pub fn read(kind: &str, pages: &[String]) -> Vec<Found> {
    let mut out = Vec::new();
    match kind {
        "tax_status_certificate" => {
            if let Some((p, rfc, line)) = first(pages, &RFC).map(|(p, r, l)| (p, rfc_digits(&r), l)).filter(|x| is_rfc_moral(&x.1)) {
                out.push(found("institution.legal_rfc", json!(rfc), (p, line)));
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
            let has_form = name_and_form(pages, &mut out);
            if !has_form {
                if let Some((p, form, line)) = pages.iter().enumerate().find_map(|(i, pg)| pg.lines().find_map(|l| legal_form_of(l).map(|f| (i + 1, f, short(l))))) {
                    out.push(found("institution.legal_form", json!(form), (p, line)));
                }
            }
            // the date of the deed, as a notary writes it in words in its first pages; else the first date of the first
            if let Some((date, at)) = first_date_in_words(pages, 3).or_else(|| first_date(pages, 1)) {
                out.push(found("institution.founded_year", json!(date[..4].parse::<i64>().unwrap_or_default()), at));
            }
            out.extend(purpose(pages, false));
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
            // the folio of a constancia, or the number of institution of the registry sheet of the Junta
            let folio = first(pages, &FOLIO).filter(|x| x.1.chars().any(|c| c.is_ascii_digit())).or_else(|| first_flat(pages, &INSTITUTION_NUMBER));
            if let Some((p, folio, line)) = folio {
                out.push(found("institution.junta_folio", json!(folio), (p, line)));
            }
            name_and_form(pages, &mut out);
            out.extend(purpose(pages, true));
        }
        _ => {}
    }
    own(kind, pages, &mut out);
    out
}

/// An RFC as a picture may read it, in capitals, with the letters a scan puts in place of the six digits of its date
/// fixed (O→0, I and l→1, S→5, B→8).
fn rfc_digits(rfc: &str) -> String {
    rfc.to_uppercase()
        .chars()
        .enumerate()
        .map(|(i, c)| match (i, c) {
            (3..=8, 'O') => '0',
            (3..=8, 'I' | 'L') => '1',
            (3..=8, 'S') => '5',
            (3..=8, 'B') => '8',
            _ => c,
        })
        .collect()
}

/// The first date after a label («Fecha de emisión: …»): on its line, else on the next one.
fn date_after(pages: &[String], label: &Regex) -> Option<(String, (usize, String))> {
    pages.iter().enumerate().find_map(|(i, p)| {
        let lines: Vec<&str> = p.lines().collect();
        lines.iter().enumerate().find_map(|(j, l)| {
            let m = label.find(l)?;
            let on_it = dates_in(&l[m.end()..]).into_iter().next().map(|(_, d)| (d, (i + 1, short(l))));
            on_it.or_else(|| {
                let next = lines.get(j + 1)?;
                dates_in(next).into_iter().next().map(|(_, d)| (d, (i + 1, short(&format!("{l} {next}")))))
            })
        })
    })
}

/// The date a document is valid until: the first date right after «vigente hasta» or «vigencia … hasta».
fn valid_until(pages: &[String]) -> Option<(String, (usize, String))> {
    pages.iter().enumerate().find_map(|(i, p)| {
        let text = flat(p);
        VALID_TO.find_iter(&text).find_map(|m| {
            let rest: String = text[m.end()..].chars().take(120).collect();
            let (at, iso) = dates_in(&rest).into_iter().next()?;
            let gap = rest[..at].chars().count();
            let around: String = text[m.start()..].chars().take(m.as_str().chars().count() + gap + 50).collect();
            (gap <= 20).then(|| (iso, (i + 1, short(&around))))
        })
    })
}

/// The number of a notarial deed, in figures or in words («ESCRITURA PÚBLICA NÚMERO CUARENTA Y DOS MIL CIENTO DIEZ»).
fn deed_number(pages: &[String]) -> Option<(String, (usize, String))> {
    let (p, raw, line) = first(pages, &DEED)?;
    let figures: String = raw.chars().take_while(|c| c.is_ascii_digit() || matches!(c, ',' | '.')).filter(char::is_ascii_digit).collect();
    let number = if figures.is_empty() { leading_number_in_words(&raw)?.to_string() } else { figures };
    Some((number, (p, line)))
}

/// What a document says of itself, for its own window (ADR-038): when it was issued, until when it is valid and its
/// number. A deed is dated in words in its first pages; an oficio, by the first date of its first page; the rest, by
/// the date after «fecha de emisión» (or of expedición, presentación, inscripción, registro). A 32-D is valid 30
/// calendar days from its issue unless it says otherwise: that date is worked out (origin `computed`).
fn own(kind: &str, pages: &[String], out: &mut Vec<Found>) {
    let issued = match kind {
        "articles_of_incorporation" | "bylaws_amendments" | "power_of_attorney" => first_date_in_words(pages, 2),
        "donee_authorization" => first_date(pages, 1),
        _ => None,
    }
    .or_else(|| date_after(pages, &ISSUED));
    if let Some((date, at)) = &issued {
        out.push(found("institution.record.issued_on", json!(date), at.clone()));
    }
    if let Some((date, at)) = valid_until(pages) {
        out.push(found("institution.record.valid_until", json!(date), at));
    } else if kind == "tax_compliance_opinion" {
        if let Some((until, page)) = issued.as_ref().and_then(|(d, (p, _))| add_days(d, 30).map(|u| (u, *p))) {
            out.push(Found { field: "institution.record.valid_until", value: json!(until), page, line: "30 días naturales desde su emisión".into(), computed: true });
        }
    }
    let reference = match kind {
        "articles_of_incorporation" | "bylaws_amendments" | "power_of_attorney" => deed_number(pages),
        "donee_authorization" => first(pages, &OFICIO).map(|(p, n, l)| (n.trim_end_matches('.').to_string(), (p, l))),
        "cluni_certificate" => first(pages, &CLUNI).map(|(p, n, l)| (n, (p, l))),
        "junta_registration" | "tax_compliance_opinion" => first(pages, &FOLIO).filter(|x| x.1.chars().any(|c| c.is_ascii_digit())).map(|(p, n, l)| (n, (p, l))),
        "tax_status_certificate" => first(pages, &ID_CIF).map(|(p, n, l)| (n, (p, l))),
        _ => None,
    };
    if let Some((number, at)) = reference {
        out.push(found("institution.record.reference", json!(number), at));
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

    /// What goes to «Mi institución»; the document's own window is checked by `what_each_document_says_of_itself`.
    fn values(found: &[Found]) -> Vec<(&str, Value, usize)> {
        found.iter().filter(|f| !f.field.starts_with("institution.record.")).map(|f| (f.field, f.value.clone(), f.page)).collect()
    }

    /// The document's own window: (field without its prefix, value).
    fn own_values(found: &[Found]) -> Vec<(&str, Value)> {
        found.iter().filter_map(|f| f.field.strip_prefix("institution.record.").map(|k| (k, f.value.clone()))).collect()
    }

    /// The battery of the document's own window (ADR-038), one line per fictitious document of the record: when it
    /// was issued, until when it is valid and its number, read by rules from where each kind writes them. This table
    /// is the pattern every Reader of a module follows (`docs/agents/metodologia-lectores.md`).
    #[test]
    fn what_each_document_says_of_itself() {
        let cases: &[(&str, &str, &[(&str, Value)])] = &[
            ("constancia-situacion-fiscal.txt", "tax_status_certificate", &[("issued_on", json!("2026-02-06"))]),
            ("oficio-donataria.txt", "donee_authorization", &[("issued_on", json!("2017-03-14")), ("reference", json!("600-04-02-2017-11234"))]),
            ("acta-constitutiva.txt", "articles_of_incorporation", &[("issued_on", json!("2015-03-10")), ("reference", json!("42110"))]),
            ("poder-notarial.txt", "power_of_attorney", &[("issued_on", json!("2024-01-18")), ("valid_until", json!("2027-01-18")), ("reference", json!("51300"))]),
            ("aviso-funcionamiento.txt", "sanitary_notice", &[("issued_on", json!("2025-09-03"))]),
            ("opinion-cumplimiento.txt", "tax_compliance_opinion", &[("issued_on", json!("2026-09-20")), ("valid_until", json!("2026-10-20")), ("reference", json!("26NB1234567"))]),
            ("cluni.txt", "cluni_certificate", &[("issued_on", json!("2016-05-11")), ("reference", json!("CRF15031009015"))]),
            ("registro-junta.txt", "junta_registration", &[("issued_on", json!("2015-06-02")), ("reference", json!("JAP-1234/2015"))]),
            ("reglamento-interno.txt", "internal_rules", &[]),
        ];
        for (file, kind, expected) in cases {
            let found = read(kind, &pages_of(file));
            assert_eq!(own_values(&found), expected.to_vec(), "{file}");
        }
        // the validity of a 32-D is worked out from its issue, and says so
        let f = read("tax_compliance_opinion", &pages_of("opinion-cumplimiento.txt"));
        let until = f.iter().find(|x| x.field == "institution.record.valid_until").unwrap();
        assert!(until.computed && until.line.contains("30 días"), "{until:?}");
    }

    /// What a picture reads wrong in a formatted datum is fixed by rule: an «O» in the date of an RFC, «1.A.P.».
    #[test]
    fn what_a_picture_misreads_in_an_rfc_or_a_legal_form_is_fixed() {
        let pages = vec!["CONSTANCIA DE SITUACIÓN FISCAL\nRFC: CRF15O31OKX4\nDenominación/Razón Social: CASA FICTICIA\nRégimen capital: INSTITUCION DE ASISTENCIA PRIVADA".to_string()];
        assert_eq!(values(&read("tax_status_certificate", &pages))[0], ("institution.legal_rfc", json!("CRF150310KX4"), 1));
        let acta = vec!["Se constituye una institución que se denominará «Casa Ficticia del Ejemplo, 1.A.P.» con domicilio".to_string()];
        let read_acta = read("articles_of_incorporation", &acta);
        let f = values(&read_acta);
        assert_eq!((f[0].1.clone(), f[1].1.clone()), (json!("Casa Ficticia del Ejemplo"), json!("iap")));
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

    /// An acta as a real scan reads it (fictitious names): the name «bajo la denominación … AC.-» and «denominada “…”»,
    /// the date with «año» read as «afio», the purpose under «OBJETO. Artículo V.- … tendrá como objetivo general»
    /// after a «con objeto de acordar» that is not it, and the dashes a notary fills lines with.
    #[test]
    fn an_acta_as_a_scan_reads_it() {
        let pages = vec![
            "SECRETARÍA DE RELACIONES EXTERIORES\nPERMISO 0901234\nEsta Secretaría concede el permiso para constituir una AC bajo la denominación CONSEJO FICTICIO\nDE EJEMPLO, AC.- Este permiso quedará condicionado a que en los estatutos se inserte la cláusula.".to_string(),
            "En la Ciudad de México, siendo las veinte horas del dia veinticuatro de enero del afio dos mil dos, reunidos los\nseñores asociados, celebran la presente Asamblea con objeto de acordar la Constitución de una Asociación Civil\ndenominada “CONSEJO FICTICIO DE EJEMPLO”, A. C., y los Estatutos Sociales que rijan a la misma.".to_string(),
            "——e——eeer fs ------ OBJETO. ———— Artículo V.- La asociación tendrá como objetivo general ACREDITAR los programas\nficticios de ejemplo en todas sus modalidades,-----------------\nasí como promover su calidad. Artículo VI.- El patrimonio de la asociación se formará con las cuotas.".to_string(),
        ];
        let f = read("articles_of_incorporation", &pages);
        let v = values(&f);
        assert_eq!(v[0], ("institution.legal_name", json!("CONSEJO FICTICIO DE EJEMPLO"), 1));
        assert_eq!(v[1], ("institution.legal_form", json!("ac"), 2), "«Asociación Civil denominada…»");
        assert_eq!(v[2], ("institution.founded_year", json!(2002), 2));
        assert_eq!((f[3].field, f[3].page), ("institution.purpose", 3));
        assert_eq!(f[3].value, json!("ACREDITAR los programas ficticios de ejemplo en todas sus modalidades, así como promover su calidad."));
    }

    /// The sheet of the institution in the directory of the Junta (fictitious): recognized, with its number, name,
    /// legal form (a scan reads «I.A.P.» as «LAP») and purpose up to the next label; the board is never read.
    #[test]
    fn the_sheet_of_the_junta_gives_its_number_name_and_purpose() {
        // the board as a scan reads it: «Patronat» without its «o», roles without «:»
        let pages = vec!["Denominación: Patronato Casa Ficticia del Ejemplo, LAP Número de Institución: 77 Objeto: La Institución tendrá por objeto\nla atención a los requerimientos básicos de subsistencia en materia de alimentación, vestido y vivienda.\nPatronat 1. ROSA FICTICIA Presidenta 2. OTRA PERSONA Patrono protesorero 7. TERCERA PERSONA Domicilio Legal: Calle Ficticia 1".to_string()];
        assert_eq!(super::super::kinds::recognize(&pages), Some("junta_registration"));
        let f = read("junta_registration", &pages);
        let v = values(&f);
        assert_eq!(v[0], ("institution.junta_folio", json!("77"), 1));
        assert_eq!(v[1], ("institution.legal_name", json!("Patronato Casa Ficticia del Ejemplo"), 1));
        assert_eq!(v[2], ("institution.legal_form", json!("iap"), 1));
        let purpose = f[3].value.as_str().unwrap();
        assert!(purpose.starts_with("La Institución tendrá por objeto la atención") && purpose.ends_with("vestido y vivienda."), "{purpose}");
        assert!(f.iter().all(|x| !x.value.to_string().contains("ROSA")), "the board is never read");
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
