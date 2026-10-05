//! Detection and redaction of sensitive personal data. 100 % local, no network.
//!
//! The scanner never stores or returns the value it found: a `Finding` only has
//! the kind, severity, position and confidence.

mod checks;
pub mod guard;
mod names;
mod normalize;

use normalize::Normalized;
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::OnceLock;

/// Above this many blocking findings a file is treated as a list of people.
const ROSTER_BLOCK_LIMIT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Curp,
    RfcPerson,
    RfcCompany,
    VoterKey,
    Phone,
    Email,
    Clabe,
    Card,
    Nss,
    PersonName,
    HealthContext,
}

impl FindingKind {
    /// Text that replaces the value when redacting.
    pub fn label(self) -> &'static str {
        match self {
            FindingKind::Curp => "[CURP OCULTA]",
            FindingKind::RfcPerson | FindingKind::RfcCompany => "[RFC OCULTO]",
            FindingKind::VoterKey => "[CLAVE DE ELECTOR OCULTA]",
            FindingKind::Phone => "[TELÉFONO OCULTO]",
            FindingKind::Email => "[CORREO OCULTO]",
            FindingKind::Clabe => "[CLABE OCULTA]",
            FindingKind::Card => "[TARJETA OCULTA]",
            FindingKind::Nss => "[NSS OCULTO]",
            FindingKind::PersonName => "[NOMBRE OCULTO]",
            FindingKind::HealthContext => "[DATO DE SALUD OCULTO]",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            FindingKind::Curp => "curp",
            FindingKind::RfcPerson => "rfc_person",
            FindingKind::RfcCompany => "rfc_company",
            FindingKind::VoterKey => "voter_key",
            FindingKind::Phone => "phone",
            FindingKind::Email => "email",
            FindingKind::Clabe => "clabe",
            FindingKind::Card => "card",
            FindingKind::Nss => "nss",
            FindingKind::PersonName => "person_name",
            FindingKind::HealthContext => "health_context",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Cannot be ignored: it is redacted or the text is cancelled.
    Block,
    /// The user may confirm it is not personal data.
    Warn,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: Severity,
    /// Byte range in the scanned text.
    pub span: Range<usize>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanReport {
    pub findings: Vec<Finding>,
}

impl ScanReport {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn has_blocking(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Block)
    }

    pub fn count_blocking(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Block).count()
    }

    /// Counts per kind, safe to write in the audit log (never the content).
    pub fn counts(&self) -> BTreeMap<&'static str, usize> {
        let mut m = BTreeMap::new();
        for f in &self.findings {
            *m.entry(f.kind.key()).or_insert(0) += 1;
        }
        m
    }
}

/// Institutional data the scanner must not flag.
#[derive(Debug, Clone, Default)]
pub struct ScannerConfig {
    pub institutional_phones: Vec<String>,
    pub institutional_emails: Vec<String>,
    /// Names of the institution, places, foundations and public representatives.
    pub safe_phrases: Vec<String>,
}

pub trait SensitiveScanner: Send + Sync {
    fn scan(&self, text: &str) -> ScanReport;
    fn redact(&self, text: &str, report: &ScanReport) -> String;
}

struct Patterns {
    curp: Regex,
    rfc_person: Regex,
    rfc_company: Regex,
    voter: Regex,
    phone: Regex,
    email: Regex,
    clabe: Regex,
    card: Regex,
    nss: Regex,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        let r = |s: &str| Regex::new(s).unwrap();
        Patterns {
            curp: r(r"\b[A-Z][AEIOUX][A-Z]{2}\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])[HMX](?:AS|BC|BS|CC|CL|CM|CS|CH|DF|DG|GT|GR|HG|JC|MC|MN|MS|NT|NL|OC|PL|QT|QR|SP|SL|SR|TC|TS|TL|VZ|YN|ZS|NE)[B-DF-HJ-NP-TV-Z]{3}[A-Z\d]\d\b"),
            rfc_person: r(r"\b[A-Z&]{4}\d{6}[A-Z\d]{3}\b"),
            rfc_company: r(r"\b[A-Z&]{3}\d{6}[A-Z\d]{3}\b"),
            voter: r(r"\b[A-Z]{6}\d{8}[HMX]\d{3}\b"),
            phone: r(r"(?:\+?52[ -]?)?(?:\(\d{2,3}\)|\d{2,3})[ -]?\d{3,4}[ -]?\d{4}"),
            email: r(r"[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}"),
            clabe: r(r"\b\d{18}\b"),
            card: r(r"\b\d(?:[ -]?\d){12,18}\b"),
            nss: r(r"\b\d{11}\b"),
        }
    })
}

fn digits(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub struct RegexScanner {
    cfg: ScannerConfig,
    phones: Vec<String>,
    emails: Vec<String>,
}

impl RegexScanner {
    pub fn new(cfg: ScannerConfig) -> Self {
        let phones = cfg
            .institutional_phones
            .iter()
            .map(|p| {
                let d = digits(p);
                d[d.len().saturating_sub(10)..].to_string()
            })
            .collect();
        let emails = cfg
            .institutional_emails
            .iter()
            .map(|e| e.trim().to_uppercase())
            .collect();
        RegexScanner { cfg, phones, emails }
    }

    /// True when the text looks like a list of people (a record file) and must be rejected whole.
    pub fn looks_like_roster(&self, text: &str, report: &ScanReport) -> bool {
        if report.count_blocking() > ROSTER_BLOCK_LIMIT {
            return true;
        }
        let norm = Normalized::new(text);
        norm.text.lines().any(|line| {
            let seps = line.matches(['\t', '|', ';', ',']).count() + line.matches("  ").count();
            seps >= 2
                && line.contains("NOMBRE")
                && ["EDAD", "DIAGNOSTICO", "CURP", "PADECIMIENTO", "FECHA DE NACIMIENTO"]
                    .iter()
                    .any(|k| line.contains(k))
        })
    }
}

impl SensitiveScanner for RegexScanner {
    fn scan(&self, text: &str) -> ScanReport {
        let norm = Normalized::new(text);
        let t = norm.text.as_str();
        let p = patterns();
        let mut found: Vec<Finding> = Vec::new();
        let mut push = |kind, severity, r: Range<usize>, confidence: f32| {
            found.push(Finding { kind, severity, span: norm.orig_span(r), confidence });
        };
        // `\b` is not enough for ids that sit next to hyphens or slashes (folios, dates).
        let clean_edges = |r: &Range<usize>| {
            let before = t[..r.start].chars().next_back();
            let after = t[r.end..].chars().next();
            let bad = |c: Option<char>| c.map_or(false, |c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/');
            !bad(before) && !bad(after)
        };

        for m in p.curp.find_iter(t) {
            if checks::curp_ok(m.as_str()) {
                push(FindingKind::Curp, Severity::Block, m.range(), 1.0);
            } else {
                // Right shape but wrong check digit: probably a typo, still flagged.
                push(FindingKind::Curp, Severity::Warn, m.range(), 0.6);
            }
        }
        for m in p.rfc_person.find_iter(t) {
            if checks::yymmdd_ok(&m.as_str()[4..10]) {
                push(FindingKind::RfcPerson, Severity::Block, m.range(), 0.9);
            }
        }
        for m in p.rfc_company.find_iter(t) {
            if checks::yymmdd_ok(&m.as_str()[3..9]) {
                push(FindingKind::RfcCompany, Severity::Warn, m.range(), 0.8);
            }
        }
        for m in p.voter.find_iter(t) {
            push(FindingKind::VoterKey, Severity::Block, m.range(), 0.9);
        }
        for m in p.clabe.find_iter(t) {
            if checks::clabe_ok(m.as_str()) {
                push(FindingKind::Clabe, Severity::Block, m.range(), 1.0);
            }
        }
        for m in p.card.find_iter(t) {
            let d = digits(m.as_str());
            if (13..=19).contains(&d.len()) && checks::luhn_ok(&d) {
                push(FindingKind::Card, Severity::Block, m.range(), 0.9);
            }
        }
        for m in p.nss.find_iter(t) {
            if checks::luhn_ok(m.as_str()) {
                push(FindingKind::Nss, Severity::Block, m.range(), 0.7);
            }
        }
        for m in p.phone.find_iter(t) {
            let mut d = digits(m.as_str());
            if d.len() == 12 && d.starts_with("52") {
                d.drain(..2);
            }
            if d.len() == 10 && clean_edges(&m.range()) && !self.phones.contains(&d) {
                push(FindingKind::Phone, Severity::Warn, m.range(), 0.8);
            }
        }
        for m in p.email.find_iter(t) {
            if !self.emails.iter().any(|e| e == m.as_str()) {
                push(FindingKind::Email, Severity::Warn, m.range(), 1.0);
            }
        }

        let name_findings = names::find_names(text, &self.cfg);
        let health = names::find_health(&norm, &name_findings);
        found.extend(name_findings);
        found.extend(health);

        ScanReport { findings: resolve_overlaps(found) }
    }

    fn redact(&self, text: &str, report: &ScanReport) -> String {
        let mut spans: Vec<&Finding> = report.findings.iter().collect();
        spans.sort_by_key(|f| f.span.start);
        let mut out = String::with_capacity(text.len());
        let mut cursor = 0;
        for f in spans {
            if f.span.start < cursor || f.span.end > text.len() {
                continue; // overlapping or out of range: already covered
            }
            out.push_str(&text[cursor..f.span.start]);
            out.push_str(f.kind.label());
            cursor = f.span.end;
        }
        out.push_str(&text[cursor..]);
        out
    }
}

/// Keeps the strongest finding where several overlap (Block first, then the longest).
fn resolve_overlaps(mut found: Vec<Finding>) -> Vec<Finding> {
    found.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then((b.span.end - b.span.start).cmp(&(a.span.end - a.span.start)))
            .then(a.kind.cmp(&b.kind))
    });
    let mut kept: Vec<Finding> = Vec::new();
    for f in found {
        if !kept.iter().any(|k| f.span.start < k.span.end && k.span.start < f.span.end) {
            kept.push(f);
        }
    }
    kept.sort_by_key(|f| f.span.start);
    kept
}

#[cfg(test)]
mod tests;
