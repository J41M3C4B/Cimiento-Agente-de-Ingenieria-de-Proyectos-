//! Quarantine: nothing is saved until the person decides what to do with findings.
//!
//! Flow (see `docs/04-escaner-datos-sensibles.md`): scan -> if findings, return a
//! report with the REDACTED preview (never the original) -> the person chooses
//! to redact or to say "this is not personal data" (only for warnings).
//! Cancelling is done by the UI simply by not calling again.

use super::{ScanReport, SensitiveScanner};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Cover the data and continue.
    Redact,
    /// "This is not personal data". Refused if any finding is blocking.
    NotPersonal,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldReport {
    pub path: String,
    pub counts: BTreeMap<&'static str, usize>,
    pub blocking: bool,
    /// The text with findings covered. The original never leaves the backend.
    pub redacted_preview: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuarantineReport {
    pub fields: Vec<FieldReport>,
    pub counts: BTreeMap<&'static str, usize>,
    pub has_blocking: bool,
}

#[derive(Debug)]
pub enum GuardOutcome {
    /// Nothing found: save as is.
    Clean,
    /// Findings and no decision yet: show the report, save nothing.
    Quarantine(QuarantineReport),
    /// Decision `Redact`: save these texts (same order and paths as given).
    Redacted { texts: Vec<String>, counts: BTreeMap<&'static str, usize> },
    /// Decision `NotPersonal` accepted (warnings only): save as is, log an override.
    Overridden { counts: BTreeMap<&'static str, usize> },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GuardError {
    #[error("blocking findings cannot be ignored")]
    BlockCannotBeIgnored,
}

fn merge(into: &mut BTreeMap<&'static str, usize>, from: &BTreeMap<&'static str, usize>) {
    for (k, v) in from {
        *into.entry(k).or_insert(0) += v;
    }
}

/// Scans every `(path, text)` field and applies the decision.
pub fn guard_fields(
    scanner: &dyn SensitiveScanner,
    fields: &[(String, String)],
    decision: Option<Decision>,
) -> Result<GuardOutcome, GuardError> {
    let mut reports = Vec::new();
    let mut scans: Vec<ScanReport> = Vec::new();
    for (path, text) in fields {
        let r = scanner.scan(text);
        if !r.is_clean() {
            reports.push(FieldReport {
                path: path.clone(),
                counts: r.counts(),
                blocking: r.has_blocking(),
                redacted_preview: scanner.redact(text, &r),
            });
        }
        scans.push(r);
    }
    if reports.is_empty() {
        return Ok(GuardOutcome::Clean);
    }
    let mut counts = BTreeMap::new();
    for r in &reports {
        merge(&mut counts, &r.counts);
    }
    let has_blocking = reports.iter().any(|r| r.blocking);

    match decision {
        None => Ok(GuardOutcome::Quarantine(QuarantineReport { fields: reports, counts, has_blocking })),
        Some(Decision::NotPersonal) if has_blocking => Err(GuardError::BlockCannotBeIgnored),
        Some(Decision::NotPersonal) => Ok(GuardOutcome::Overridden { counts }),
        Some(Decision::Redact) => {
            let texts = fields
                .iter()
                .zip(&scans)
                .map(|((_, text), r)| if r.is_clean() { text.clone() } else { scanner.redact(text, r) })
                .collect();
            Ok(GuardOutcome::Redacted { texts, counts })
        }
    }
}

/// What `screen_texts` can fail with: a decision that is not allowed, or an audit log that could not be written.
#[derive(Debug, thiserror::Error)]
pub enum ScreenError {
    #[error(transparent)]
    Guard(#[from] GuardError),
    #[error(transparent)]
    Audit(#[from] crate::audit::AuditError),
}

/// The counts of a decision as the audit log keeps them (never the text).
pub fn counts_json(counts: &BTreeMap<&'static str, usize>, decision: &str) -> serde_json::Value {
    serde_json::json!({ "findings": counts, "decision": decision })
}

/// Scans free texts and writes the decision to the audit log. `Ok(Err(report))` means "quarantine: show it, save
/// nothing"; `Ok(Ok(texts))` are the texts to save (covered if the person chose so).
pub fn screen_texts(
    conn: &rusqlite::Connection,
    scanner: &dyn SensitiveScanner,
    entity: &str,
    fields: &[(String, String)],
    decision: Option<Decision>,
) -> Result<Result<Vec<String>, QuarantineReport>, ScreenError> {
    use crate::audit::{self, AuditKind};
    match guard_fields(scanner, fields, decision)? {
        GuardOutcome::Clean => Ok(Ok(fields.iter().map(|(_, t)| t.clone()).collect())),
        GuardOutcome::Quarantine(r) => Ok(Err(r)),
        GuardOutcome::Redacted { texts, counts } => {
            audit::record(conn, AuditKind::ScannerQuarantine, Some(entity), None, counts_json(&counts, "redacted"))?;
            Ok(Ok(texts))
        }
        GuardOutcome::Overridden { counts } => {
            audit::record(conn, AuditKind::ScannerOverride, Some(entity), None, counts_json(&counts, "not_personal"))?;
            Ok(Ok(fields.iter().map(|(_, t)| t.clone()).collect()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{RegexScanner, ScannerConfig};

    fn s() -> RegexScanner {
        RegexScanner::new(ScannerConfig::default())
    }

    fn fields() -> Vec<(String, String)> {
        vec![
            ("notes".into(), "Todo en orden, 18 personas.".into()),
            ("mission".into(), "Contacto: 55 1234 5678".into()),
            ("other".into(), "La CURP es LOPM800101MDFRZN09".into()),
        ]
    }

    #[test]
    fn clean_text_passes() {
        let f = vec![("notes".to_string(), "18 adultos mayores".to_string())];
        assert!(matches!(guard_fields(&s(), &f, None).unwrap(), GuardOutcome::Clean));
    }

    #[test]
    fn findings_without_decision_go_to_quarantine_with_redacted_preview_only() {
        let GuardOutcome::Quarantine(q) = guard_fields(&s(), &fields(), None).unwrap() else { panic!() };
        assert_eq!(q.fields.len(), 2);
        assert!(q.has_blocking);
        assert_eq!(q.counts.get("curp"), Some(&1));
        assert_eq!(q.counts.get("phone"), Some(&1));
        let json = serde_json::to_string(&q).unwrap();
        assert!(!json.contains("LOPM8"));
        assert!(!json.contains("1234 5678"));
        assert!(json.contains("[CURP OCULTA]"));
    }

    #[test]
    fn redact_returns_texts_in_order_and_keeps_clean_ones() {
        let GuardOutcome::Redacted { texts, .. } =
            guard_fields(&s(), &fields(), Some(Decision::Redact)).unwrap() else { panic!() };
        assert_eq!(texts[0], "Todo en orden, 18 personas.");
        assert_eq!(texts[1], "Contacto: [TELÉFONO OCULTO]");
        assert_eq!(texts[2], "La CURP es [CURP OCULTA]");
    }

    #[test]
    fn blocking_findings_cannot_be_overridden() {
        assert_eq!(
            guard_fields(&s(), &fields(), Some(Decision::NotPersonal)).unwrap_err(),
            GuardError::BlockCannotBeIgnored
        );
    }

    #[test]
    fn warnings_can_be_overridden() {
        let f = vec![("mission".to_string(), "Contacto: 55 1234 5678".to_string())];
        assert!(matches!(
            guard_fields(&s(), &f, Some(Decision::NotPersonal)).unwrap(),
            GuardOutcome::Overridden { .. }
        ));
    }
}
