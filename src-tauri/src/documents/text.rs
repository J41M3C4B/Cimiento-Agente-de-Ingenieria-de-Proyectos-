//! Text helpers shared by the reading of calls: cleaning what a PDF extractor writes, and comparing
//! two statements by the words they use. They know nothing about any funder or document layout.

use regex::Regex;
use std::collections::BTreeSet;
use std::sync::OnceLock;

macro_rules! re {
    ($p:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($p).expect("valid regex"))
    }};
}

/// Repairs the letters some PDF fonts decode wrongly. In Spanish text `æ`, `Æ`, `Ø`, `œ` and
/// `˝` never appear, so when they sit inside words they are the accented letters of a font with
/// a custom encoding: `aæo` is `año`, `rØgimen` is `régimen`.
fn repair_mojibake(s: &str) -> Option<String> {
    if !re!(r"\p{L}[æÆØœ]\p{L}|˝\p{L}").is_match(s) {
        return None;
    }
    Some(
        s.chars()
            .map(|c| match c {
                'æ' => 'ñ',
                'Æ' => 'á',
                'Ø' => 'é',
                'œ' => 'ú',
                '˝' => 'Í',
                other => other,
            })
            .collect(),
    )
}

/// Normalizes text for matching: ligatures, odd spaces, broken URLs, wrongly decoded letters.
/// Returns the text and whether letters were repaired.
pub fn clean_text_report(raw: &str) -> (String, bool) {
    let raw = raw.replace("\r\n", "\n");
    let mut s = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '\u{FB00}' => s.push_str("ff"),
            '\u{FB01}' => s.push_str("fi"),
            '\u{FB02}' => s.push_str("fl"),
            '\u{FB03}' => s.push_str("ffi"),
            '\u{FB04}' => s.push_str("ffl"),
            '\u{FB05}' | '\u{FB06}' => s.push_str("st"),
            '\u{00A0}' | '\u{2007}' | '\u{202F}' => s.push(' '),
            '\u{00AD}' | '\u{200B}' | '\u{FEFF}' => {}
            c => s.push(c),
        }
    }
    // some extractors leave the ligature as a literal "/f_i"
    let s = s.replace("/f_i", "fi").replace("/f_l", "fl").replace("/f_f", "ff");
    let (s, repaired) = match repair_mojibake(&s) {
        Some(fixed) => (fixed, true),
        None => (s, false),
    };
    let s = s.replace("https:/ /", "https://").replace("http:/ /", "http://");
    let s = re!(r"[ \t]+").replace_all(&s, " ").to_string();
    let s = re!(r" *\n *").replace_all(&s, "\n").to_string();
    let s = re!(r"\n{3,}").replace_all(&s, "\n\n").to_string();
    (s.trim().to_string(), repaired)
}

pub fn clean_text(raw: &str) -> String {
    clean_text_report(raw).0
}

/// Lowercase letters and digits separated by single spaces.
pub fn norm(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = true;
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim().to_string()
}

/// Two normalized texts say the same thing: one contains the other, or most of their words are
/// shared. Different figures always make them different ("12 meses" is not "10 meses").
pub fn same_statement(a: &str, b: &str) -> bool {
    let words = |s: &str| s.split_whitespace().map(str::to_string).collect::<BTreeSet<String>>();
    let digits = |w: &BTreeSet<String>| w.iter().filter(|t| t.chars().any(|c| c.is_ascii_digit())).cloned().collect::<Vec<_>>();
    let (wa, wb) = (words(a), words(b));
    if wa.is_empty() || wb.is_empty() || digits(&wa) != digits(&wb) {
        return false;
    }
    if a.contains(b) || b.contains(a) {
        return true;
    }
    let shared = wa.intersection(&wb).count();
    let all = wa.union(&wb).count();
    shared * 10 >= all * 7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleaning_repairs_ligatures_odd_spaces_and_wrongly_decoded_letters() {
        assert_eq!(clean_text("o\u{FB01}cina\u{00A0}del   año"), "oficina del año");
        let (t, repaired) = clean_text_report("el aæo y el rØgimen");
        assert_eq!(t, "el año y el régimen");
        assert!(repaired);
        assert_eq!(clean_text("a \n\n\n\n b"), "a\n\nb");
    }

    #[test]
    fn statements_are_compared_by_words_and_figures() {
        assert_eq!(norm("  Monto: $250,000 pesos. "), "monto 250 000 pesos");
        assert!(same_statement(&norm("hasta 12 meses"), &norm("Hasta 12 meses de ejecución")));
        assert!(!same_statement(&norm("hasta 12 meses"), &norm("hasta 10 meses")));
    }
}
