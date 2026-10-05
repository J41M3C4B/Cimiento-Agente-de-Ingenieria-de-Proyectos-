use super::*;
use std::path::Path;

fn scanner() -> RegexScanner {
    RegexScanner::new(ScannerConfig::default())
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/escaner").join(name))
        .unwrap()
}

fn cases(name: &str) -> Vec<String> {
    fixture(name)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn kinds(report: &ScanReport) -> Vec<&'static str> {
    report.findings.iter().map(|f| f.kind.key()).collect()
}

#[test]
fn every_positive_case_is_detected_with_its_kind() {
    let s = scanner();
    let lines = cases("positivos.txt");
    assert!(lines.len() >= 20);
    let mut missed = Vec::new();
    for line in &lines {
        let (kind, text) = line.split_once(" | ").expect("kind | text");
        let r = s.scan(text);
        if !kinds(&r).contains(&kind) {
            missed.push(format!("{kind}: {text} -> {:?}", kinds(&r)));
        }
    }
    assert!(missed.is_empty(), "not detected:\n{}", missed.join("\n"));
}

#[test]
fn no_negative_case_is_detected() {
    let s = scanner();
    let mut wrong = Vec::new();
    for text in cases("negativos.txt") {
        let r = s.scan(&text);
        if !r.is_clean() {
            wrong.push(format!("{text} -> {:?}", kinds(&r)));
        }
    }
    assert!(wrong.is_empty(), "false positives:\n{}", wrong.join("\n"));
}

#[test]
fn redact_never_leaves_the_original_value() {
    let s = scanner();
    let secrets = [
        "LOPM800101MDFRZN09",
        "012180001234567899",
        "4111 1111 1111 1111",
        "55 1234 5678",
        "maria.lopez@example.com",
        "Rosa Hernández López",
        "LPMRMN80010109H400",
        "42859012348",
    ];
    let text = secrets.iter().map(|x| format!("Dato: {x}. ")).collect::<String>();
    let r = s.scan(&text);
    let out = s.redact(&text, &r);
    for x in secrets {
        assert!(!out.contains(x), "{x} survived in: {out}");
    }
    assert!(out.contains("[CURP OCULTA]"));
    assert!(out.contains("[NOMBRE OCULTO]"));
    // and a second scan of the redacted text is clean
    assert!(s.scan(&out).is_clean(), "{:?}", kinds(&s.scan(&out)));
}

#[test]
fn redact_keeps_the_rest_of_the_text_and_accents() {
    let s = scanner();
    let text = "Atención: la CURP LOPM800101MDFRZN09 es de ella. ¡Gracias!";
    let out = s.redact(text, &s.scan(text));
    assert_eq!(out, "Atención: la CURP [CURP OCULTA] es de ella. ¡Gracias!");
}

#[test]
fn spans_point_to_the_original_text_with_accents() {
    let s = scanner();
    let text = "Ñandú — teléfono: (33) 3456-7890 fin";
    let r = s.scan(text);
    let f = r.findings.iter().find(|f| f.kind == FindingKind::Phone).unwrap();
    assert_eq!(&text[f.span.clone()], "(33) 3456-7890");
}

#[test]
fn institutional_phone_and_email_are_not_flagged() {
    let s = RegexScanner::new(ScannerConfig {
        institutional_phones: vec!["+52 55 1234 5678".into()],
        institutional_emails: vec!["Contacto@CasaHogar.org".into()],
        safe_phrases: vec![],
    });
    let r = s.scan("Llamar al 55 1234 5678 o escribir a contacto@casahogar.org");
    assert!(r.is_clean(), "{:?}", kinds(&r));
    let r = s.scan("Otro teléfono 55 9999 0000 y otro@correo.com");
    assert_eq!(kinds(&r), vec!["phone", "email"]);
}

#[test]
fn safe_phrases_protect_institution_names() {
    let text = "La directora Rosa Hernández López coordina el hogar.";
    assert!(!scanner().scan(text).is_clean());
    let s = RegexScanner::new(ScannerConfig {
        safe_phrases: vec!["Rosa Hernández López".into()],
        ..Default::default()
    });
    assert!(s.scan(text).is_clean());
}

#[test]
fn block_findings_are_block_and_names_are_warn() {
    let r = scanner().scan("CURP LOPM800101MDFRZN09 de Rosa Hernández López");
    let sev = |k| r.findings.iter().find(|f| f.kind == k).unwrap().severity;
    assert_eq!(sev(FindingKind::Curp), Severity::Block);
    assert_eq!(sev(FindingKind::PersonName), Severity::Warn);
    assert!(r.has_blocking());
}

#[test]
fn wrong_check_digit_is_only_a_warning() {
    let r = scanner().scan("LOPM800101MDFRZN08");
    assert_eq!(kinds(&r), vec!["curp"]);
    assert_eq!(r.findings[0].severity, Severity::Warn);
}

#[test]
fn counts_never_contain_the_value() {
    let s = scanner();
    let r = s.scan("CURP LOPM800101MDFRZN09 y tel 55 1234 5678");
    let json = serde_json::to_string(&r).unwrap() + &format!("{:?}", r.counts());
    assert!(!json.contains("LOPM"));
    assert!(!json.contains("1234"));
    assert_eq!(r.counts().get("curp"), Some(&1));
}

#[test]
fn roster_detection() {
    let s = scanner();
    let table = "Nombre\tEdad\tDiagnóstico\nAna\t80\tDiabetes";
    assert!(s.looks_like_roster(table, &s.scan(table)));
    let many: String = (0..6).map(|_| "LOPM800101MDFRZN09\n").collect();
    assert!(s.looks_like_roster(&many, &s.scan(&many)));
    let ok = "Atendemos a 18 personas y 6 usan andadera.";
    assert!(!s.looks_like_roster(ok, &s.scan(ok)));
}

#[test]
fn aggregated_health_phrases_are_valid() {
    let r = scanner().scan("12 de los 18 residentes viven con diabetes. Los niños reciben tratamiento dental.");
    assert!(r.is_clean(), "{:?}", kinds(&r));
}

#[test]
fn hundred_pages_scan_fast() {
    let page = "La casa hogar atiende a 18 adultos mayores. Se requieren 4 regaderas y 6 barras de apoyo. \
                El presupuesto total es de 150000 pesos. Rosa Hernández López llamó al 55 1234 5678.\n"
        .repeat(30);
    let doc = page.repeat(100);
    let s = scanner();
    let t = std::time::Instant::now();
    let r = s.scan(&doc);
    let el = t.elapsed();
    println!("scanned {} KB in {:?}, {} findings", doc.len() / 1024, el, r.findings.len());
    // 1 s in release; debug builds are much slower
    let limit = if cfg!(debug_assertions) { 15 } else { 1 };
    assert!(el.as_secs() < limit, "too slow: {el:?}");
}
