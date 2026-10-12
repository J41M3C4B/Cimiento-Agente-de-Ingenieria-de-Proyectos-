//! What a person writes, turned into terms for FTS5. Nothing they write reaches FTS5 as syntax.

/// Words that say nothing about what is being looked for.
const STOP: &[&str] = &[
    "que", "qué", "como", "cómo", "para", "por", "con", "los", "las", "del", "una", "uno", "unos", "unas", "este", "esta", "esto", "aqui",
    "aquí", "pongo", "poner", "sirve", "donde", "dónde", "hago", "hacer", "cuando", "cuándo", "mis", "sus", "nos", "pero", "más", "mas",
    "cual", "cuál", "cuales", "cuáles", "quien", "quién", "quienes", "quiénes", "hay", "son", "ser", "está", "están", "tiene", "tienen",
    "tenemos", "nuestro", "nuestra", "nuestros", "nuestras", "dice", "puede", "pueden", "podemos", "hasta", "desde", "sobre", "cada",
    "cuanto", "cuánto", "cuanta", "cuánta", "cuantos", "cuántos", "cuantas", "cuántas",
];

/// The words of a question that say something, lowercase and with a plural cut to its singular.
pub fn words(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|w| w.chars().count() >= 3 && !STOP.contains(&w.as_str()))
        .map(|w| if w.chars().count() > 4 && w.ends_with('s') { w[..w.len() - 1].to_string() } else { w })
        .collect()
}

/// The terms of a question for FTS5: each word as a quoted prefix (a plural also finds its singular), any of them.
/// `None` when only empty words are left.
pub fn match_terms(query: &str) -> Option<String> {
    let words = words(query);
    (!words.is_empty()).then(|| words.iter().map(|w| format!("\"{w}\"*")).collect::<Vec<_>>().join(" OR "))
}

/// A word without its accents, as FTS5 compares them (`remove_diacritics`): «régimen» is «regimen».
pub fn fold(word: &str) -> String {
    word.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_become_quoted_prefixes_and_empty_words_go() {
        assert_eq!(match_terms("¿Cuántos extintores tenemos?").as_deref(), Some("\"extintore\"*"));
        assert_eq!(match_terms("¿Qué es esto?"), None);
        assert_eq!(match_terms("\" OR * NEAR( -").as_deref(), Some("\"near\"*"), "syntax never passes: only quoted words");
    }
}
