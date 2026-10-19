//! Where a quote of the AI is in a text: the passage it cites, or what the person said. Every agent that cites (the
//! Reader of C2 and C3, the Capturist) checks its quotes here, and keeps the words of the text, never the AI's copy.
//!
//! - **Without capitals, accents nor punctuation,** as `capturist::plain`.
//! - **A slip of copying is forgiven** (C4, measured): the cheapest model writes «Reg1menes» for «Regímenes» or
//!   «r&eacute;gimen» for «régimen», and an exact check dropped a right answer. A quote of 12 letters or more may
//!   differ in one letter every 20, at most 3; a shorter one must be exact.
//! - **Never a figure:** the numbers of the quote are the numbers of the words found, one by one. A slip may change a
//!   letter, never «35» into «53».

use crate::scanner::normalize::Normalized;

/// The shortest quote that may differ in a letter.
const FORGIVEN_FROM: usize = 12;
/// One letter of difference for every this many.
const LETTERS_PER_SLIP: usize = 20;
/// The most letters a quote may differ in.
const MAX_SLIPS: usize = 3;

/// The words of `text` the quote copies, as they are written in `text`; `None` when the quote is not there.
pub fn find(quote: &str, text: &str) -> Option<String> {
    let q = Plain::new(&unescape(quote));
    if q.chars.is_empty() {
        return None;
    }
    let t = Plain::new(text);
    let slips = if q.chars.len() < FORGIVEN_FROM { 0 } else { (q.chars.len() / LETTERS_PER_SLIP).clamp(1, MAX_SLIPS) };
    let (start, end) = closest(&q.chars, &t.chars, slips)?;
    if numbers(&q.chars) != numbers(&t.chars[start..end]) {
        return None;
    }
    Some(t.original(text, start, end).trim().to_string())
}

/// A text without capitals, accents nor punctuation: its letters and digits, words split by one space, and where each
/// one is in the folded text.
struct Plain {
    norm: Normalized,
    chars: Vec<char>,
    /// The byte in `norm.text` of each char (a space points to the letter after it).
    at: Vec<usize>,
}

impl Plain {
    fn new(s: &str) -> Self {
        let norm = Normalized::new(s);
        let (mut chars, mut at) = (Vec::new(), Vec::new());
        let mut gap = false;
        for (i, c) in norm.text.char_indices() {
            if c.is_alphanumeric() {
                if gap && !chars.is_empty() {
                    chars.push(' ');
                    at.push(i);
                }
                gap = false;
                chars.push(c);
                at.push(i);
            } else {
                gap = true;
            }
        }
        Plain { norm, chars, at }
    }

    /// The words of the original text from char `start` to `end` (not included) of the plain one.
    fn original<'a>(&self, text: &'a str, start: usize, end: usize) -> &'a str {
        let last = self.at[end - 1];
        let last_len = self.norm.text[last..].chars().next().map_or(1, char::len_utf8);
        &text[self.norm.orig_span(self.at[start]..last + last_len)]
    }
}

/// Where in `t` the text `q` is with the fewest different letters, if no more than `slips` (an edit distance over a
/// substring, Sellers): the first and last char, not included.
fn closest(q: &[char], t: &[char], slips: usize) -> Option<(usize, usize)> {
    let (m, n) = (q.len(), t.len());
    if n == 0 || m > n + slips {
        return None;
    }
    // cost[i] and start[i]: the fewest edits for q[..i] ending at the current char of t, and where that match starts
    let mut cost: Vec<usize> = (0..=m).collect();
    let mut start: Vec<usize> = vec![0; m + 1];
    let mut best: Option<(usize, usize, usize)> = None; // (edits, start, end)
    for j in 1..=n {
        let (mut diag_cost, mut diag_start) = (cost[0], start[0]);
        cost[0] = 0;
        start[0] = j;
        for i in 1..=m {
            let (up_cost, up_start) = (cost[i], start[i]);
            let same = usize::from(q[i - 1] != t[j - 1]);
            let mut c = (diag_cost + same, diag_start);
            if up_cost + 1 < c.0 {
                c = (up_cost + 1, up_start);
            }
            if cost[i - 1] + 1 < c.0 {
                c = (cost[i - 1] + 1, start[i - 1]);
            }
            diag_cost = up_cost;
            diag_start = up_start;
            cost[i] = c.0;
            start[i] = c.1;
        }
        if cost[m] <= slips && best.is_none_or(|b| cost[m] < b.0) {
            best = Some((cost[m], start[m], j));
            if cost[m] == 0 {
                break;
            }
        }
    }
    // a match does not begin nor end with the space between two words
    best.map(|(_, s, e)| {
        let s = if t.get(s) == Some(&' ') { s + 1 } else { s };
        let e = if e > s && t[e - 1] == ' ' { e - 1 } else { e };
        (s, e)
    })
    .filter(|(s, e)| s < e)
}

/// The numbers of a plain text: its words made only of digits, in order.
fn numbers(chars: &[char]) -> Vec<String> {
    chars.iter().collect::<String>().split(' ').filter(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_digit())).map(str::to_string).collect()
}

/// The letters a model sometimes writes as HTML (`&eacute;`, `&ntilde;`, `&#237;`), as letters.
pub fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    const NAMED: &[(&str, &str)] = &[
        ("aacute", "á"), ("eacute", "é"), ("iacute", "í"), ("oacute", "ó"), ("uacute", "ú"), ("ntilde", "ñ"), ("uuml", "ü"),
        ("Aacute", "Á"), ("Eacute", "É"), ("Iacute", "Í"), ("Oacute", "Ó"), ("Uacute", "Ú"), ("Ntilde", "Ñ"), ("Uuml", "Ü"),
        ("iquest", "¿"), ("iexcl", "¡"), ("laquo", "«"), ("raquo", "»"), ("quot", "\""), ("apos", "'"), ("nbsp", " "),
        ("lt", "<"), ("gt", ">"), ("amp", "&"),
    ];
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let entity = rest[1..].find(';').filter(|&j| j > 0 && j <= 8).map(|j| &rest[1..=j]);
        let letter = entity.and_then(|e| {
            if let Some(code) = e.strip_prefix('#') {
                code.parse::<u32>().ok().and_then(char::from_u32).map(String::from)
            } else {
                NAMED.iter().find(|(n, _)| *n == e).map(|(_, l)| (*l).to_string())
            }
        });
        match (entity, letter) {
            (Some(e), Some(l)) => {
                out.push_str(&l);
                rest = &rest[e.len() + 2..];
            }
            _ => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// The vowel a digit looks like when a model writes it for a letter.
fn looks_like(d: char) -> Option<(char, char)> {
    match d {
        '3' => Some(('é', 'e')),
        '1' => Some(('í', 'i')),
        '0' => Some(('ó', 'o')),
        '4' => Some(('á', 'a')),
        _ => None,
    }
}

/// Mends the words a model sometimes writes with a digit for an accented letter («dej3», «bot3n», «Reg1menes»; the
/// cheapest model, now and then). A word of lowercase letters (the first may be a capital) with one digit inside is
/// mended only into a word `known` says exists: first with the accented vowel the digit looks like, then with the one
/// accented vowel that makes a known word, then with the plain vowel it looks like. Otherwise it stays as it was.
pub fn mend_digits(text: &str, known: impl Fn(&str) -> bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        out.push_str(&mended(word, &known).unwrap_or_else(|| word.clone()));
        word.clear();
    };
    for c in text.chars() {
        if c.is_alphanumeric() {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

fn mended(word: &str, known: &impl Fn(&str) -> bool) -> Option<String> {
    let chars: Vec<char> = word.chars().collect();
    let digits: Vec<usize> = (0..chars.len()).filter(|&i| chars[i].is_ascii_digit()).collect();
    let [at] = digits[..] else { return None };
    let letters = chars.len() - 1;
    let shaped = chars.iter().enumerate().all(|(i, c)| i == at || c.is_lowercase() || (i == 0 && c.is_uppercase()));
    if at == 0 || letters < 2 || !shaped {
        return None;
    }
    let with = |v: char| {
        let mut w = chars.clone();
        w[at] = v;
        w.into_iter().collect::<String>()
    };
    let is_known = |w: &String| known(&w.to_lowercase());
    let (accented, plain) = looks_like(chars[at]).unzip();
    if let Some(w) = accented.map(with).filter(is_known) {
        return Some(w);
    }
    let options: Vec<String> = ['á', 'é', 'í', 'ó', 'ú'].into_iter().map(with).filter(is_known).collect();
    if let [one] = &options[..] {
        return Some(one.clone());
    }
    plain.map(with).filter(is_known)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_with_a_digit_for_a_letter_is_mended_only_into_a_known_word() {
        let known = |w: &str| ["dejé", "dejó", "deja", "botón", "regímenes", "número"].contains(&w);
        assert_eq!(mend_digits("Con los datos, dej3 sugerencias. Use el bot3n de abajo.", known), "Con los datos, dejé sugerencias. Use el botón de abajo.");
        assert_eq!(mend_digits("Reg1menes fiscales", known), "Regímenes fiscales", "a capital stays");
        // figures, codes and words nobody knows stay as they are
        assert_eq!(mend_digits("Tiene 12 camas, el folio JAP-1234/2015, el formato 32-D y un mp3.", known), "Tiene 12 camas, el folio JAP-1234/2015, el formato 32-D y un mp3.");
        assert_eq!(mend_digits("n1mero 3er piso", known), "número 3er piso", "the one accented vowel that makes a known word; a digit first is a figure");
    }

    const PAGE: &str = "Regímenes\nPersonas Morales con Fines no Lucrativos. Fecha inicio 10/03/2015.\nTenemos 35 camas.";

    #[test]
    fn a_quote_copied_as_it_is_keeps_the_words_of_the_text() {
        assert_eq!(find("personas morales con fines no lucrativos", PAGE).as_deref(), Some("Personas Morales con Fines no Lucrativos"));
        assert_eq!(find("Regimenes. Personas", PAGE).as_deref(), Some("Regímenes\nPersonas"));
        assert_eq!(find("Fines de lucro", PAGE), None);
    }

    #[test]
    fn a_slip_of_copying_is_forgiven_and_the_text_wins() {
        // the cheapest model, measured: a digit for an accented letter, and an HTML letter
        assert_eq!(find("Reg1menes Personas Morales con Fines no Lucrativos", PAGE).as_deref(), Some("Regímenes\nPersonas Morales con Fines no Lucrativos"));
        assert_eq!(find("Reg&iacute;menes Personas Morales", PAGE).as_deref(), Some("Regímenes\nPersonas Morales"));
        // «sua casa» for «su casa»
        assert_eq!(find("les llevamos comida a sua casa", "Les llevamos comida a su casa tres veces.").as_deref(), Some("Les llevamos comida a su casa"));
    }

    #[test]
    fn a_short_quote_must_be_exact_and_a_long_one_may_not_drift() {
        assert_eq!(find("35 camaz", PAGE), None);
        assert_eq!(find("Personas Mayores con Fines de Lucro", PAGE), None);
    }

    #[test]
    fn a_slip_never_changes_a_figure() {
        assert_eq!(find("Tenemos 53 camas", PAGE), None);
        assert_eq!(find("Fecha inicio 10/03/2016", PAGE), None);
        assert_eq!(find("Fecha inicio 10/03/2015", PAGE).as_deref(), Some("Fecha inicio 10/03/2015"));
        // nor drops one
        assert_eq!(find("Fecha inicio 10/03", PAGE).as_deref(), Some("Fecha inicio 10/03"));
        assert_eq!(find("Fecha inicio 10 03 2015 Tenemos camas", PAGE), None);
    }

    #[test]
    fn html_letters_become_letters() {
        assert_eq!(unescape("r&eacute;gimen &amp; &#237;ndice &oops; & ya"), "régimen & índice &oops; & ya");
    }
}
