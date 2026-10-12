//! Dates as documents write them: «14 de marzo de 2017», «10/03/2015», or in words as a notary does: «a los diez
//! días del mes de marzo de dos mil quince», «el dieciocho de enero de dos mil veintisiete». They become ISO dates
//! (`2015-03-10`) that the code checks; the AI never reads a date.

use crate::common::forms::is_date;
use crate::storage::fts::fold;
use regex::Regex;
use std::sync::LazyLock;

const MONTHS: &[&str] = &["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

fn month(word: &str) -> Option<u32> {
    let w = fold(word);
    let w = if w == "setiembre" { "septiembre".to_string() } else { w };
    MONTHS.iter().position(|m| *m == w).map(|i| i as u32 + 1)
}

fn unit(word: &str) -> Option<u32> {
    Some(match word {
        "un" | "uno" | "una" | "primero" | "primer" => 1,
        "dos" => 2,
        "tres" => 3,
        "cuatro" => 4,
        "cinco" => 5,
        "seis" => 6,
        "siete" => 7,
        "ocho" => 8,
        "nueve" => 9,
        "diez" => 10,
        "once" => 11,
        "doce" => 12,
        "trece" => 13,
        "catorce" => 14,
        "quince" => 15,
        "dieciseis" => 16,
        "diecisiete" => 17,
        "dieciocho" => 18,
        "diecinueve" => 19,
        "veinte" => 20,
        "veintiun" | "veintiuno" => 21,
        "veintidos" => 22,
        "veintitres" => 23,
        "veinticuatro" => 24,
        "veinticinco" => 25,
        "veintiseis" => 26,
        "veintisiete" => 27,
        "veintiocho" => 28,
        "veintinueve" => 29,
        "treinta" => 30,
        "cuarenta" => 40,
        "cincuenta" => 50,
        "sesenta" => 60,
        "setenta" => 70,
        "ochenta" => 80,
        "noventa" => 90,
        "cien" | "ciento" => 100,
        "doscientos" => 200,
        "trescientos" => 300,
        "cuatrocientos" => 400,
        "quinientos" => 500,
        "seiscientos" => 600,
        "setecientos" => 700,
        "ochocientos" => 800,
        "novecientos" => 900,
        _ => return None,
    })
}

/// A number up to the thousands written in words («dos mil veintisiete», «treinta y uno»); `None` if a word is not
/// a number.
pub fn number_in_words(text: &str) -> Option<u32> {
    let mut total = 0u32;
    let mut current = 0u32;
    let mut any = false;
    for w in fold(text).split_whitespace() {
        match w {
            "y" => continue,
            "mil" => {
                total += current.max(1) * 1000;
                current = 0;
            }
            _ => current += unit(w)?,
        }
        any = true;
    }
    any.then_some(total + current)
}

fn iso(y: u32, m: u32, d: u32) -> Option<String> {
    let s = format!("{y:04}-{m:02}-{d:02}");
    is_date(&s).then_some(s)
}

static NUMERIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(\d{1,2})\s+de\s+([a-zá-ú]+)\s+(?:de|del)\s+(\d{4})\b").unwrap());
static SLASHED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b(\d{1,2})/(\d{1,2})/(\d{4})\b").unwrap());
// «diez días del mes de marzo de dos mil quince», «dieciocho de enero de dos mil veintisiete»; «del año» also as a scan
// reads it («afio», «aho», «ario»)
static WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b((?:[a-zá-ú]+\s+){0,2}?[a-zá-ú]+)\s+(?:d[ií]as?\s+del\s+mes\s+)?de\s+([a-zá-ú]+)\s+(?:del\s+a(?:ñ|n|fi|h|ri|rì)o\s+|de\s+)((?:dos\s+mil|mil)(?:\s+[a-zá-ú]+){0,4})").unwrap()
});

/// The dates written in words, as a notary writes them, in the order they appear: (where it starts, ISO date).
pub fn dates_in_words(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    words_dates(text, &mut out);
    out
}

/// The dates of a text, in the order they appear: (where it starts, ISO date).
pub fn dates_in(text: &str) -> Vec<(usize, String)> {
    let mut out: Vec<(usize, String)> = Vec::new();
    for c in NUMERIC.captures_iter(text) {
        if let (Ok(d), Some(m), Ok(y)) = (c[1].parse(), month(&c[2]), c[3].parse()) {
            out.extend(iso(y, m, d).map(|s| (c.get(0).unwrap().start(), s)));
        }
    }
    for c in SLASHED.captures_iter(text) {
        if let (Ok(d), Ok(m), Ok(y)) = (c[1].parse(), c[2].parse(), c[3].parse()) {
            out.extend(iso(y, m, d).map(|s| (c.get(0).unwrap().start(), s)));
        }
    }
    words_dates(text, &mut out);
    out.sort();
    out.dedup_by(|a, b| a.1 == b.1);
    out
}

fn words_dates(text: &str, out: &mut Vec<(usize, String)>) {
    for c in WORDS.captures_iter(text) {
        let Some(m) = month(&c[2]) else { continue };
        // the day is the last words before «de <mes>» that make a number («a los diez», «el treinta y uno»)
        let words: Vec<&str> = c[1].split_whitespace().collect();
        let day = (0..words.len()).find_map(|i| number_in_words(&words[i..].join(" ")).filter(|d| (1..=31).contains(d)));
        // the year: as many words as still make a number
        let ys: Vec<&str> = c[3].split_whitespace().collect();
        let year = (1..=ys.len()).rev().find_map(|n| number_in_words(&ys[..n].join(" ")).filter(|y| (1800..=2100).contains(y)));
        if let (Some(d), Some(y)) = (day, year) {
            out.extend(iso(y, m, d).map(|s| (c.get(0).unwrap().start(), s)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_in_words() {
        assert_eq!(number_in_words("dos mil quince"), Some(2015));
        assert_eq!(number_in_words("dos mil veintisiete"), Some(2027));
        assert_eq!(number_in_words("mil novecientos noventa y ocho"), Some(1998));
        assert_eq!(number_in_words("treinta y uno"), Some(31));
        assert_eq!(number_in_words("diez"), Some(10));
        assert_eq!(number_in_words("patronato"), None);
    }

    #[test]
    fn dates_as_documents_write_them() {
        let one = |t: &str| dates_in(t).into_iter().map(|d| d.1).collect::<Vec<_>>();
        assert_eq!(one("Ciudad de México, a 14 de marzo de 2017"), vec!["2017-03-14"]);
        assert_eq!(one("Fecha inicio de operaciones: 10/03/2015."), vec!["2015-03-10"]);
        assert_eq!(one("En la Ciudad de México, a los diez días del mes de marzo de dos mil quince, ante mí"), vec!["2015-03-10"]);
        assert_eq!(one("es decir, hasta el dieciocho de enero de dos mil veintisiete, salvo que"), vec!["2027-01-18"]);
        assert_eq!(one("el primero de abril de dos mil dieciséis"), vec!["2016-04-01"]);
        // as a scan reads «año»
        assert_eq!(one("siendo las veinte horas del dia veinticuatro de enero del afio dos mil dos, reunidos"), vec!["2002-01-24"]);
        assert_eq!(one("a 31 de febrero de 2017"), Vec::<String>::new(), "a date that does not exist is none");
        assert_eq!(one("de martes a domingo, de las diez a las trece horas"), Vec::<String>::new());
    }
}
