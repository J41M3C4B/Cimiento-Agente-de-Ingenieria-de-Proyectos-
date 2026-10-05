//! Reading dates, amounts, percentages, durations and whole numbers from the quote that proves them.
//!
//! The model only says where a value is and copies it as written; the code reads it. These readers know
//! how Spanish writes numbers and dates, not how any funder lays out a call. A value that cannot be
//! read comes back `None` and stays unread: it is never guessed.

use regex::Regex;
use serde_json::{json, Value};
use std::sync::OnceLock;

/// What the readers need to know about the package, not about one call's format.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ctx {
    /// The year the package mostly talks about, for a date written without one.
    pub year: Option<u32>,
    /// Some page says amounts are in pesos: a bare «$» is then MXN.
    pub pesos: bool,
}

macro_rules! re {
    ($p:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($p).expect("valid regex"))
    }};
}

const WORDS: [(&str, f64); 34] = [
    ("un", 1.0),
    ("una", 1.0),
    ("uno", 1.0),
    ("dos", 2.0),
    ("tres", 3.0),
    ("cuatro", 4.0),
    ("cinco", 5.0),
    ("seis", 6.0),
    ("siete", 7.0),
    ("ocho", 8.0),
    ("nueve", 9.0),
    ("diez", 10.0),
    ("once", 11.0),
    ("doce", 12.0),
    ("trece", 13.0),
    ("catorce", 14.0),
    ("quince", 15.0),
    ("dieciseis", 16.0),
    ("dieciocho", 18.0),
    ("veinte", 20.0),
    ("veinticuatro", 24.0),
    ("veinticinco", 25.0),
    ("treinta", 30.0),
    ("treinta y seis", 36.0),
    ("cuarenta", 40.0),
    ("cincuenta", 50.0),
    ("sesenta", 60.0),
    ("setenta", 70.0),
    ("ochenta", 80.0),
    ("noventa", 90.0),
    ("cien", 100.0),
    ("ciento", 100.0),
    ("quinientos", 500.0),
    ("mil", 1000.0),
];

fn plain(s: &str) -> String {
    s.to_lowercase().chars().map(|c| match c {
        'á' => 'a',
        'é' => 'e',
        'í' => 'i',
        'ó' => 'o',
        'ú' | 'ü' => 'u',
        'ñ' => 'n',
        other => other,
    }).collect()
}

fn word_number(w: &str) -> Option<f64> {
    let w = plain(w);
    WORDS.iter().find(|(name, _)| *name == w.trim()).map(|(_, v)| *v)
}

/// «250,000», «250 000», «1.250.000», «1,5», «12.5» → a number.
fn parse_number(raw: &str) -> Option<f64> {
    let s = raw.trim().trim_end_matches(['.', ',']);
    if s.is_empty() {
        return None;
    }
    // thousands with commas or spaces, decimals with a dot: «250,000», «250 000», «1,250,000.50»
    if re!(r"^\d{1,3}([, ]\d{3})+(\.\d+)?$").is_match(s) {
        return s.replace([',', ' '], "").parse().ok();
    }
    // thousands with dots, decimals with a comma: «1.250.000», «1.250.000,50»
    if re!(r"^\d{1,3}(\.\d{3}){2,}(,\d+)?$").is_match(s) {
        return s.replace('.', "").replace(',', ".").parse().ok();
    }
    // decimal comma («1,5»)
    if re!(r"^\d+,\d{1,2}$").is_match(s) {
        return s.replace(',', ".").parse().ok();
    }
    s.replace(',', "").replace(' ', "").parse().ok()
}

fn currency_of(text: &str, ctx: &Ctx) -> &'static str {
    let t = plain(text);
    if t.contains("us$") || t.contains("usd") || t.contains("dolar") {
        "USD"
    } else if t.contains("eur") || t.contains('€') || t.contains("euro") {
        "EUR"
    } else if t.contains("mxn") || t.contains("pesos") || t.contains("m.n") {
        "MXN"
    } else if t.contains('$') && ctx.pesos {
        "MXN"
    } else {
        "XXX"
    }
}

/// An amount of money: `{cantidad, moneda}`.
pub fn read_amount(text: &str, ctx: &Ctx) -> Option<Value> {
    let m = re!(r"(?i)(?:US\$|\$|€)?\s*(\d{1,3}(?:[,\s]\d{3})+(?:\.\d+)?|\d+(?:[.,]\d+)?)\s*(millones|millón|millon|mil)?\b").captures_iter(text).find(|c| {
        // a plain number is an amount only if it carries a currency mark, a multiplier or thousands
        let whole = c.get(0).map_or("", |m| m.as_str());
        whole.contains('$') || whole.contains('€') || c.get(2).is_some() || c[1].contains([',', ' ']) || plain(text).contains("peso") || text.len() == c[1].len()
    })?;
    let mut n = parse_number(&m[1])?;
    if let Some(mult) = m.get(2) {
        n *= if plain(mult.as_str()).starts_with("mill") { 1_000_000.0 } else { 1_000.0 };
    }
    Some(json!({ "cantidad": n, "moneda": currency_of(text, ctx) }))
}

/// A percentage: `{valor}`.
pub fn read_percent(text: &str) -> Option<Value> {
    if let Some(c) = re!(r"(\d+(?:[.,]\d+)?)\s*(?:%|por\s*ciento)").captures(text) {
        let v = parse_number(&c[1])?;
        return (v <= 100.0).then(|| json!({ "valor": v }));
    }
    let c = re!(r"(?i)\b([a-záéíóúü]+(?:\s+y\s+[a-záéíóúü]+)?)\s+por\s*ciento").captures(text)?;
    word_number(&c[1]).filter(|v| *v <= 100.0).map(|v| json!({ "valor": v }))
}

/// A length of time: `{valor, unidad}`.
pub fn read_duration(text: &str) -> Option<Value> {
    let c = re!(r"(?i)\b(\d+(?:[.,]\d+)?|[a-záéíóúü]+(?:\s+y\s+[a-záéíóúü]+)?)\s+(años?|anos?|meses|mes|semanas?|días|dias|día|dia)\b").captures_iter(text).find_map(|c| {
        let n = c[1].parse::<f64>().ok().or_else(|| parse_number(&c[1])).or_else(|| word_number(&c[1]))?;
        (n > 0.0).then_some((n, c[2].to_string()))
    })?;
    let unit = match plain(&c.1).as_str() {
        "ano" | "anos" => "anios",
        "mes" | "meses" => "meses",
        "semana" | "semanas" => "semanas",
        _ => "dias",
    };
    Some(json!({ "valor": c.0, "unidad": unit }))
}

/// A whole number («30 puntos», «tres»).
pub fn read_integer(text: &str) -> Option<Value> {
    if let Some(c) = re!(r"\d{1,3}(?:,\d{3})+|\d+").find(text) {
        let n = parse_number(c.as_str())?;
        return (n.fract() == 0.0).then(|| json!({ "valor": n as i64 }));
    }
    text.split_whitespace().find_map(word_number).map(|n| json!({ "valor": n as i64 }))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Ymd {
    year: Option<u32>,
    month: u32,
    day: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
struct Span {
    from: usize,
    to: usize,
    start: Ymd,
    end: Option<Ymd>,
}

const MONTHS: &str = r"(?:enero|febrero|marzo|abril|mayo|junio|julio|agosto|septiembre|setiembre|octubre|noviembre|diciembre|ene|feb|mar|abr|may|jun|jul|ago|sept|sep|set|oct|nov|dic)";

fn month_number(s: &str) -> Option<u32> {
    let head: String = s.to_lowercase().chars().take(3).collect();
    ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"]
        .iter()
        .position(|m| *m == head || (head == "set" && *m == "sep"))
        .map(|i| i as u32 + 1)
}

fn valid(y: Ymd) -> bool {
    let days = |m: u32| match m {
        2 => 29,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=12).contains(&y.month) && y.day.map_or(true, |d| (1..=days(y.month)).contains(&d))
}

fn num(c: &regex::Captures, name: &str) -> Option<u32> {
    c.name(name).and_then(|m| m.as_str().parse().ok())
}

fn year_of(c: &regex::Captures) -> Option<u32> {
    num(c, "y").or_else(|| num(c, "y2"))
}

fn date_patterns() -> &'static Vec<(Regex, &'static str)> {
    static P: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    P.get_or_init(|| {
        let m = MONTHS;
        let build = |p: String| Regex::new(&p).expect("valid regex");
        vec![
            (build(r"\b(?P<y>\d{4})-(?P<m>\d{2})-(?P<d1>\d{2})\b".into()), "iso"),
            (build(r"\b(?P<d1>\d{1,2})[/.-](?P<m>\d{1,2})[/.-](?P<y>\d{4})\b".into()), "num"),
            (build(format!(r"(?i)\b(?P<d1>\d{{1,2}})\s*(?:al|a|-|–|—)\s*(?P<d2>\d{{1,2}})\s+(?:de\s+)?(?P<m>{m})\b\.?(?:\s+(?:de|del)\s+(?P<y>\d{{4}})|\s+(?P<y2>\d{{4}}))?")), "range_days"),
            (build(format!(r"(?i)\b(?P<m>{m})\b\.?\s+(?P<d1>\d{{1,2}})\s*(?:al|a|-|–|—)\s*(?P<d2>\d{{1,2}})\b(?:,?\s+(?:de\s+)?(?P<y>\d{{4}}))?")), "range_us"),
            (build(format!(r"(?i)\b(?P<d1>\d{{1,2}})\s*(?:de\s+)?(?P<m>{m})\b\.?(?:\s*(?:de|del|,)\s*(?P<y>\d{{4}})|\s+(?P<y2>\d{{4}}))?")), "day_month"),
            (build(format!(r"(?i)\b(?P<m>{m})\b\.?\s+(?P<d1>\d{{1,2}})\b(?:,?\s+(?:de\s+)?(?P<y>\d{{4}}))?")), "month_day"),
            (build(format!(r"(?i)\b(?P<m>{m})\b\.?(?:\s+(?:de|del)\s+(?P<y>\d{{4}})|\s+(?P<y2>\d{{4}}))")), "month_year"),
        ]
    })
}

/// Every date written in the text, in order: «22 de junio de 2026», «22 al 30 de junio», «22 de junio al
/// 15 de julio de 2026», «Mayo 16 al 23», «2026-06-22», «22/06/2026», «junio de 2026». A range is one
/// span. This reads how Spanish writes dates; it knows nothing about any funder's calendar.
fn find_dates(text: &str) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    for (re, kind) in date_patterns() {
        for c in re.captures_iter(text) {
            let whole = c.get(0).unwrap();
            if spans.iter().any(|s| whole.start() < s.to && s.from < whole.end()) {
                continue;
            }
            let month = match *kind {
                "iso" | "num" => num(&c, "m"),
                _ => c.name("m").and_then(|x| month_number(x.as_str())),
            };
            let Some(month) = month else { continue };
            let year = year_of(&c);
            let day = |n: &str| num(&c, n);
            let (start, end) = match *kind {
                "range_days" | "range_us" => (Ymd { year, month, day: day("d1") }, Some(Ymd { year, month, day: day("d2") })),
                "month_year" => (Ymd { year, month, day: None }, None),
                _ => (Ymd { year, month, day: day("d1") }, None),
            };
            if valid(start) && end.map_or(true, valid) {
                spans.push(Span { from: whole.start(), to: whole.end(), start, end });
            }
        }
    }
    spans.sort_by_key(|s| s.from);
    // «22 de junio» «al» «15 de julio de 2026» is one range
    let connector = re!(r"(?i)^\s*(?:al|a|-|–|—|hasta(?:\s+el)?)\s*$");
    let mut merged: Vec<Span> = Vec::new();
    for s in spans {
        if let Some(last) = merged.last_mut() {
            if last.end.is_none() && s.end.is_none() && last.start.day.is_some() && s.start.day.is_some() && connector.is_match(&text[last.to..s.from]) {
                let mut first = last.start;
                if first.year.is_none() {
                    first.year = s.start.year.map(|y| if first.month > s.start.month { y - 1 } else { y });
                }
                *last = Span { from: last.from, to: s.to, start: first, end: Some(s.start) };
                continue;
            }
        }
        merged.push(s);
    }
    merged
}

fn ymd_object(y: Ymd, ctx: &Ctx, time: Option<&str>) -> Value {
    let (year, inferred) = match y.year {
        Some(v) => (Some(v), false),
        None => (ctx.year, true),
    };
    match (year, y.day) {
        (Some(year), Some(day)) => json!({ "fecha": format!("{year:04}-{:02}-{day:02}", y.month), "hora": time, "precision": "dia", "anio_inferido": inferred }),
        (Some(_), None) => json!({ "fecha": null, "hora": null, "precision": "mes", "anio_inferido": inferred }),
        (None, _) => json!({ "fecha": null, "hora": null, "precision": "no_interpretable", "anio_inferido": false }),
    }
}

/// The first time of day written in the text, «11:00» or «9:30 hrs».
fn time_in(text: &str) -> Option<String> {
    let c = re!(r"\b([01]?\d|2[0-3]):([0-5]\d)\b").captures(text)?;
    Some(format!("{:02}:{}", c[1].parse::<u32>().ok()?, &c[2]))
}

/// A date: `{fecha, hora, precision, anio_inferido}`. A range gives its first day.
pub fn read_date(text: &str, _page: usize, ctx: &Ctx) -> Option<Value> {
    let first = find_dates(text).into_iter().next()?;
    let time = time_in(text);
    Some(ymd_object(first.start, ctx, time.as_deref()))
}

/// A calendar milestone: `{inicio, fin}`. The start text may carry the whole range.
pub fn read_milestone(start: &str, end: Option<&str>, quote: &str, _page: usize, ctx: &Ctx) -> Option<Value> {
    let source = if start.trim().is_empty() { quote } else { start };
    let first = find_dates(source).into_iter().next().or_else(|| find_dates(quote).into_iter().next())?;
    let time = time_in(source).or_else(|| time_in(quote));
    let last = match end.filter(|e| !e.trim().is_empty()) {
        Some(e) => find_dates(e).into_iter().next().map(|d| ymd_object(d.start, ctx, None)),
        None => first.end.map(|e| {
            // a range inside one text shares the year the text gives
            let e = Ymd { year: e.year.or(first.start.year), ..e };
            ymd_object(e, ctx, None)
        }),
    };
    Some(json!({ "inicio": ymd_object(first.start, ctx, time.as_deref()), "fin": last }))
}

fn fold(s: &str) -> String {
    s.to_lowercase().chars().map(|c| match c { 'á' => 'a', 'é' => 'e', 'í' => 'i', 'ó' => 'o', 'ú' | 'ü' => 'u', other => other }).collect()
}

/// The class of document a text names, from the words every Spanish call uses for itself. Descriptive only:
/// nothing in the reading depends on it. `None` when the text says no known class.
pub fn read_document_kind(text: &str) -> Option<Value> {
    let folded = fold(text);
    let words: Vec<&str> = folded.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let has = |w: &str| words.contains(&w);
    let phrase = |p: &str| words.join(" ").contains(p);
    let class = if phrase("reglas de operacion") || phrase("reglas operativas") {
        "reglas_de_operacion"
    } else if has("lineamientos") || has("reglamento") {
        "lineamientos"
    } else if has("convocatoria") || has("bases") || has("convocatorias") {
        "convocatoria"
    } else if has("aviso") || has("comunicado") || phrase("fe de erratas") {
        "aviso"
    } else if has("guia") || has("manual") || has("instructivo") {
        "guia"
    } else if has("formato") || has("solicitud") || has("formulario") || has("cuestionario") || phrase("carta compromiso") {
        "formato"
    } else if has("anexo") {
        "anexo"
    } else {
        // a title set in wide letters comes out of the PDF as «CONV OC A T ORIA»: read without the spaces, and only
        // for words long enough that they cannot be made by chance out of neighbouring ones
        let compact: String = folded.chars().filter(|c| c.is_alphanumeric()).collect();
        let long = |w: &str| compact.contains(w);
        if long("reglasdeoperacion") {
            "reglas_de_operacion"
        } else if long("lineamientos") || long("reglamento") {
            "lineamientos"
        } else if long("convocatoria") {
            "convocatoria"
        } else if long("comunicado") {
            "aviso"
        } else if long("instructivo") {
            "guia"
        } else if long("formulario") || long("cuestionario") || long("cartacompromiso") {
            "formato"
        } else {
            return None;
        }
    };
    Some(json!({ "clase": class }))
}

/// Reads `value` as `kind` (`fecha`, `monto`, `porcentaje`, `duracion`, `entero`, `tipo_documento`, `texto`), and when the
/// value alone says nothing, the quote around it. `texto` has nothing to read.
pub fn normalize(kind: &str, value: &str, quote: &str, page: usize, ctx: &Ctx) -> Option<Value> {
    let read = |text: &str| -> Option<Value> {
        match kind {
            "fecha" => read_date(text, page, ctx),
            "monto" => read_amount(text, ctx),
            "porcentaje" => read_percent(text),
            "duracion" => read_duration(text),
            "entero" => read_integer(text),
            _ => None,
        }
    };
    if kind == "texto" {
        return None;
    }
    if kind == "tipo_documento" {
        // the document's own words first, then the sentence around them; a document that names itself with
        // words no class knows is «otro», not unread: the field only describes it
        return read_document_kind(value).or_else(|| read_document_kind(quote)).or_else(|| Some(json!({ "clase": "otro" })));
    }
    read(value).or_else(|| read(quote))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Ctx {
        Ctx { year: Some(2027), pesos: true }
    }

    #[test]
    fn amounts_in_the_ways_spanish_writes_them() {
        let a = |t: &str| read_amount(t, &ctx()).map(|v| (v["cantidad"].as_f64().unwrap(), v["moneda"].as_str().unwrap().to_string()));
        assert_eq!(a("$250,000 pesos"), Some((250000.0, "MXN".into())));
        assert_eq!(a("hasta $300,000.00 (Trescientos mil pesos 00/100 M.N.)"), Some((300000.0, "MXN".into())));
        assert_eq!(a("300 mil pesos"), Some((300000.0, "MXN".into())));
        assert_eq!(a("1.5 millones de pesos"), Some((1500000.0, "MXN".into())));
        assert_eq!(a("USD 50,000"), Some((50000.0, "USD".into())));
        assert_eq!(a("sin cifra alguna"), None);
        // a bare « $ » is pesos only if the package says so
        let no_pesos = Ctx { year: None, pesos: false };
        assert_eq!(read_amount("$40,000", &no_pesos).unwrap()["moneda"], "XXX");
    }

    #[test]
    fn percentages_durations_and_integers() {
        assert_eq!(read_percent("contrapartida mínima del 30%"), Some(json!({"valor": 30.0})));
        assert_eq!(read_percent("el 12.5 por ciento"), Some(json!({"valor": 12.5})));
        assert_eq!(read_percent("treinta por ciento"), Some(json!({"valor": 30.0})));
        assert_eq!(read_percent("el 130%"), None);
        assert_eq!(read_percent("sin porcentaje"), None);
        assert_eq!(read_duration("hasta 12 meses"), Some(json!({"valor": 12.0, "unidad": "meses"})));
        assert_eq!(read_duration("por un año"), Some(json!({"valor": 1.0, "unidad": "anios"})));
        assert_eq!(read_duration("dos años"), Some(json!({"valor": 2.0, "unidad": "anios"})));
        assert_eq!(read_duration("quince días hábiles"), Some(json!({"valor": 15.0, "unidad": "dias"})));
        assert_eq!(read_duration("nada"), None);
        assert_eq!(read_integer("30 puntos"), Some(json!({"valor": 30})));
        assert_eq!(read_integer("1,000 puntos"), Some(json!({"valor": 1000})));
        assert_eq!(read_integer("tres"), Some(json!({"valor": 3})));
        assert_eq!(read_integer("ninguno"), None);
    }

    #[test]
    fn the_class_of_a_document_is_read_from_the_words_it_uses_for_itself() {
        let k = |t: &str| read_document_kind(t).map(|v| v["clase"].as_str().unwrap().to_string());
        assert_eq!(k("CONVOCATORIA de Inversión Social 2026").as_deref(), Some("convocatoria"));
        assert_eq!(k("Reglas de Operación del Programa para el Bienestar").as_deref(), Some("reglas_de_operacion"));
        assert_eq!(k("REGLAS DE OPERACIÓN y convocatoria").as_deref(), Some("reglas_de_operacion"));
        assert_eq!(k("Lineamientos para la presentación de proyectos").as_deref(), Some("lineamientos"));
        assert_eq!(k("Aviso electoral").as_deref(), Some("aviso"));
        assert_eq!(k("Guía de información general y financiera").as_deref(), Some("guia"));
        assert_eq!(k("Formato Único de Registro").as_deref(), Some("formato"));
        assert_eq!(k("Anexo de conceptos y rubros").as_deref(), Some("anexo"));
        assert_eq!(k("Programa de apoyo a la comunidad"), None);
        // «bases» is a word of its own, not the end of «bases de datos» or the start of «basesita»
        assert_eq!(k("Bases de la convocatoria").as_deref(), Some("convocatoria"));
        assert_eq!(k("basesita"), None);
        // a title in wide letters, as a PDF writes it
        assert_eq!(k("CONV OC A T ORIA").as_deref(), Some("convocatoria"));
        assert_eq!(k("R E G L A S  D E  O P E R A C I Ó N").as_deref(), Some("reglas_de_operacion"));
        let ctx = Ctx { year: None, pesos: true };
        assert_eq!(normalize("tipo_documento", "Reglas de Operación", "REGLAS DE OPERACIÓN 2026", 1, &ctx), Some(json!({ "clase": "reglas_de_operacion" })));
        assert_eq!(normalize("tipo_documento", "Invitación", "Invitación a la convocatoria 2026", 1, &ctx), Some(json!({ "clase": "convocatoria" })));
        assert_eq!(normalize("tipo_documento", "Programa", "Programa de apoyo", 1, &ctx), Some(json!({ "clase": "otro" })));
    }

    #[test]
    fn numbers_with_separators() {
        for (raw, want) in [("250,000", 250000.0), ("250 000", 250000.0), ("1,250,000.50", 1250000.5), ("1,5", 1.5), ("12.5", 12.5), ("300", 300.0)] {
            assert_eq!(parse_number(raw), Some(want), "{raw}");
        }
    }

    fn spans(t: &str) -> Vec<(String, Option<String>)> {
        let c = Ctx { year: Some(2027), pesos: true };
        find_dates(t)
            .into_iter()
            .map(|s| {
                let f = |y: Ymd| ymd_object(y, &c, None)["fecha"].as_str().unwrap_or("-").to_string();
                (f(s.start), s.end.map(|e| f(Ymd { year: e.year.or(s.start.year), ..e })))
            })
            .collect()
    }

    #[test]
    fn dates_the_way_spanish_writes_them() {
        let one = |d: &str| vec![(d.to_string(), None)];
        let range = |a: &str, b: &str| vec![(a.to_string(), Some(b.to_string()))];
        assert_eq!(spans("15 de julio de 2026"), one("2026-07-15"));
        assert_eq!(spans("22 de junio al 15 de julio de 2026"), range("2026-06-22", "2026-07-15"));
        assert_eq!(spans("del 22 de junio al 15 de julio de 2026, de 09:00 a 18:00 horas"), range("2026-06-22", "2026-07-15"));
        assert_eq!(spans("20 de diciembre al 10 de enero de 2027"), range("2026-12-20", "2027-01-10"));
        assert_eq!(spans("16 al 23 de mayo"), range("2027-05-16", "2027-05-23"));
        assert_eq!(spans("Mayo 16 al 23"), range("2027-05-16", "2027-05-23"));
        assert_eq!(spans("Tienes hasta el 23 de mayo"), one("2027-05-23"));
        assert_eq!(spans("2026-06-22"), one("2026-06-22"));
        assert_eq!(spans("22/06/2026"), one("2026-06-22"));
        assert_eq!(spans("17 mar 2026"), one("2026-03-17"));
        assert_eq!(spans("sesión el 10 de abril a las 11:00 hrs"), one("2027-04-10"));
        assert_eq!(spans("durante mayo de 2026"), one("-"));
        // two separate dates are two spans, not a range
        assert_eq!(spans("el 5 de mayo y el 9 de junio de 2026").len(), 2);
        // impossible days are not dates; words that begin like a month are not months
        assert!(spans("el 30 de febrero de 2026").iter().all(|s| s.0 == "-"), "no day-precision date");
        assert!(spans("mayor de edad, los marcos y las ofertas").is_empty());
        assert!(spans("sin fecha alguna").is_empty());
    }

    #[test]
    fn dates_and_milestones_use_the_package_year_only_when_the_text_has_none() {
        let d = read_date("hasta el 23 de mayo de 2027", 1, &ctx()).unwrap();
        assert_eq!((d["fecha"].as_str(), d["precision"].as_str(), d["anio_inferido"].as_bool()), (Some("2027-05-23"), Some("dia"), Some(false)));
        let t = read_date("el 10 de abril a las 9:30 hrs", 1, &ctx()).unwrap();
        assert_eq!((t["fecha"].as_str(), t["hora"].as_str(), t["anio_inferido"].as_bool()), (Some("2027-04-10"), Some("09:30"), Some(true)));
        let m = read_milestone("22 de junio al 15 de julio de 2026", None, "del 22 de junio al 15 de julio de 2026", 1, &ctx()).unwrap();
        assert_eq!((m["inicio"]["fecha"].as_str(), m["fin"]["fecha"].as_str()), (Some("2026-06-22"), Some("2026-07-15")));
        assert_eq!(m["inicio"]["anio_inferido"], false);
        let split = read_milestone("16 de mayo", Some("23 de mayo"), "Postulación del 16 al 23 de mayo", 1, &ctx()).unwrap();
        assert_eq!((split["inicio"]["fecha"].as_str(), split["fin"]["fecha"].as_str()), (Some("2027-05-16"), Some("2027-05-23")));
        let w = read_milestone("23 de mayo", None, "Tienes hasta el 23 de mayo", 1, &ctx()).unwrap();
        assert_eq!((w["inicio"]["fecha"].as_str(), w["inicio"]["anio_inferido"].as_bool(), w["fin"].is_null()), (Some("2027-05-23"), Some(true), true));
        assert!(read_date("sin fecha alguna", 1, &ctx()).is_none());
        let none = Ctx { year: None, pesos: false };
        assert_eq!(read_date("el 23 de mayo", 1, &none).unwrap()["precision"], "no_interpretable");
    }

    #[test]
    fn the_quote_is_read_when_the_value_alone_says_nothing() {
        let c = ctx();
        assert_eq!(normalize("porcentaje", "la mitad", "contrapartida del 30% del proyecto", 1, &c), Some(json!({"valor": 30.0})));
        assert_eq!(normalize("texto", "Fundación", "Fundación Ficticia", 1, &c), None);
        assert_eq!(normalize("fecha", "sin fecha", "sin fecha", 1, &c), None);
    }
}
