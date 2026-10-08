//! The profile of the institution (ADR-031): validate -> scan -> quarantine -> save -> audit. Commands call these.

use crate::audit::{self, AuditKind};
use crate::core::profile::domain::{ProfileInput, ProfileIssue, ProfileTotals};
use crate::core::error::ServiceError;
use crate::core::screen::scanner_for;
use crate::scanner::guard::{counts_json, guard_fields, Decision, GuardOutcome, QuarantineReport};
use crate::core::profile::storage::{self as store, StoredProfile};
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProfileView {
    pub institution_id: String,
    pub version: i64,
    pub confirmed_at: Option<String>,
    pub is_draft: bool,
    pub input: ProfileInput,
    pub totals: ProfileTotals,
    /// Only heads-ups ("algo no cuadra"); blocking problems stop the save.
    pub issues: Vec<ProfileIssue>,
}

impl From<StoredProfile> for ProfileView {
    fn from(p: StoredProfile) -> Self {
        ProfileView {
            institution_id: p.institution_id,
            version: p.version,
            is_draft: p.confirmed_at.is_none(),
            confirmed_at: p.confirmed_at,
            totals: p.input.totals(p.as_of_year),
            issues: p.input.validate(p.as_of_year),
            input: p.input,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SaveProfileOutcome {
    Saved { profile: ProfileView },
    Quarantine { report: QuarantineReport },
    Invalid { issues: Vec<ProfileIssue> },
}

pub fn get_profile(conn: &Connection) -> Result<Option<ProfileView>, ServiceError> {
    Ok(store::load_current(conn)?.map(Into::into))
}

pub fn save_profile(
    conn: &mut Connection,
    mut input: ProfileInput,
    decision: Option<Decision>,
) -> Result<SaveProfileOutcome, ServiceError> {
    // staff and people served are never written here: the profile keeps what the roster adds up to (ADR-020)
    let (staff, population) = crate::core::profile::sync::derive(conn)?;
    input.staff = staff;
    input.population = population;
    let issues = input.validate(store::current_year(conn)?);
    if issues.iter().any(|i| i.blocking) {
        return Ok(SaveProfileOutcome::Invalid { issues: issues.into_iter().filter(|i| i.blocking).collect() });
    }

    let mut fields: Vec<(String, String)> = Vec::new();
    input.for_each_text_mut(&mut |path, text| fields.push((path.to_string(), text.clone())));
    // Scan with the CURRENT institution config (its own phone/email/name are not flagged).
    let scanner = scanner_for(conn)?;
    let outcome = guard_fields(&scanner, &fields, decision)?;

    let mut audit_event: Option<(AuditKind, serde_json::Value)> = None;
    match outcome {
        GuardOutcome::Clean => {}
        GuardOutcome::Quarantine(report) => return Ok(SaveProfileOutcome::Quarantine { report }),
        GuardOutcome::Redacted { texts, counts } => {
            let mut it = texts.into_iter();
            input.for_each_text_mut(&mut |_, text| *text = it.next().unwrap_or_default());
            audit_event = Some((AuditKind::ScannerQuarantine, counts_json(&counts, "redacted")));
        }
        GuardOutcome::Overridden { counts } => {
            audit_event = Some((AuditKind::ScannerOverride, counts_json(&counts, "not_personal")));
        }
    }

    let saved = store::save(conn, &input)?;
    if let Some((kind, details)) = audit_event {
        audit::record(conn, kind, Some("institution_profile"), Some(&saved.profile_id), details)?;
    }
    Ok(SaveProfileOutcome::Saved { profile: saved.into() })
}

pub fn confirm_profile(conn: &mut Connection) -> Result<ProfileView, ServiceError> {
    if let Some(current) = store::load_current(conn)? {
        let blocking: Vec<_> = current.input.validate(current.as_of_year).into_iter().filter(|i| i.blocking).collect();
        if !blocking.is_empty() {
            return Err(ServiceError::ProfileInvalid(blocking));
        }
    }
    Ok(store::confirm(conn)?.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::profile::domain::*;
    use crate::storage::open_encrypted;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, c)
    }

    fn input() -> ProfileInput {
        ProfileInput {
            institution: InstitutionInput { name: "Casa Hogar Ficticia".into(), ..Default::default() },
            population: vec![PopulationGroupInput { label: "Adultos mayores".into(), count: 18, ..Default::default() }],
            ..Default::default()
        }
    }

    fn audit_events(c: &Connection) -> Vec<(String, String)> {
        let mut s = c.prepare("SELECT event, details_json FROM audit_log ORDER BY id").unwrap();
        let v = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(|r| r.unwrap()).collect();
        v
    }

    #[test]
    fn clean_profile_is_saved() {
        let (_d, mut c) = conn();
        let out = save_profile(&mut c, input(), None).unwrap();
        assert!(matches!(out, SaveProfileOutcome::Saved { .. }));
        assert!(audit_events(&c).is_empty());
    }

    #[test]
    fn invalid_profile_is_not_saved() {
        let (_d, mut c) = conn();
        let mut p = input();
        p.institution.name.clear();
        let SaveProfileOutcome::Invalid { issues } = save_profile(&mut c, p, None).unwrap() else { panic!() };
        assert_eq!(issues[0].code, "name_missing");
        assert!(get_profile(&c).unwrap().is_none());
    }

    #[test]
    fn pasting_a_curp_triggers_quarantine_and_saves_nothing() {
        let (_d, mut c) = conn();
        let mut p = input();
        p.notes = Some("Contacto de la familia: CURP LOPM800101MDFRZN09".into());
        let SaveProfileOutcome::Quarantine { report } = save_profile(&mut c, p, None).unwrap() else { panic!() };
        assert!(report.has_blocking);
        assert_eq!(report.fields[0].path, "notes");
        assert!(report.fields[0].redacted_preview.contains("[CURP OCULTA]"));
        assert!(!serde_json::to_string(&report).unwrap().contains("LOPM8"));
        assert!(get_profile(&c).unwrap().is_none(), "nothing may be saved before the decision");
    }

    #[test]
    fn redacting_saves_only_the_covered_text_and_logs_counts() {
        let (_d, mut c) = conn();
        let mut p = input();
        p.notes = Some("CURP LOPM800101MDFRZN09".into());
        let SaveProfileOutcome::Saved { profile } = save_profile(&mut c, p, Some(Decision::Redact)).unwrap() else { panic!() };
        assert_eq!(profile.input.notes.as_deref(), Some("CURP [CURP OCULTA]"));
        let ev = audit_events(&c);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].0, "scanner.quarantine");
        assert!(ev[0].1.contains("\"curp\":1") && ev[0].1.contains("redacted"));
        assert!(!ev[0].1.contains("LOPM"));
        // the original is nowhere in the database file content
        let n: i64 = c.query_row("SELECT count(*) FROM institution_profile WHERE notes LIKE '%LOPM8%'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn blocking_cannot_be_overridden_but_warnings_can() {
        let (_d, mut c) = conn();
        let mut p = input();
        p.notes = Some("CURP LOPM800101MDFRZN09".into());
        assert!(matches!(
            save_profile(&mut c, p, Some(Decision::NotPersonal)),
            Err(ServiceError::Guard(crate::scanner::guard::GuardError::BlockCannotBeIgnored))
        ));
        let mut p = input();
        p.notes = Some("Llamar al 55 1234 5678 para visitas".into());
        let out = save_profile(&mut c, p, Some(Decision::NotPersonal)).unwrap();
        assert!(matches!(out, SaveProfileOutcome::Saved { .. }));
        assert_eq!(audit_events(&c)[0].0, "scanner.override");
    }

    #[test]
    fn institutional_contact_is_not_scanned_and_not_flagged_elsewhere() {
        let (_d, mut c) = conn();
        let mut p = input();
        p.institution.contact_phone = Some("55 1234 5678".into());
        p.institution.legal_rep_name = Some("Rosa Hernández López".into());
        assert!(matches!(save_profile(&mut c, p.clone(), None).unwrap(), SaveProfileOutcome::Saved { .. }));
        // now the institution's own phone in a note is fine
        p.notes = Some("Informes al 55 1234 5678".into());
        assert!(matches!(save_profile(&mut c, p, None).unwrap(), SaveProfileOutcome::Saved { .. }));
    }

    #[test]
    fn confirm_creates_version_and_audit() {
        let (_d, mut c) = conn();
        save_profile(&mut c, input(), None).unwrap();
        let v = confirm_profile(&mut c).unwrap();
        assert!(!v.is_draft);
        assert_eq!(audit_events(&c)[0].0, "profile.confirmed");
    }

}

#[cfg(test)]
mod fixture_tests {
    use super::*;
    use crate::core::profile::domain::ProfileInput;
    use crate::storage::open_encrypted;

    /// Both fictitious profiles must load clean (no quarantine, no blocking issues).
    #[test]
    fn dev_fixtures_load_without_quarantine() {
        for (raw, padron) in [
            (include_str!("../../../../fixtures/institucion-asilo.json"), include_str!("../../../../fixtures/padron-asilo.json")),
            (include_str!("../../../../fixtures/institucion-casa-hogar.json"), include_str!("../../../../fixtures/padron-casa-hogar.json")),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let mut c = open_encrypted(&dir.path().join("t.db"), "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff").unwrap();
            let input: ProfileInput = serde_json::from_str(raw).unwrap();
            // as the example loader does: the institution, then its people, and the profile adds them up
            save_profile(&mut c, input.clone(), None).unwrap();
            crate::core::profile::sync::seed_examples(&mut c, padron).unwrap();
            let out = save_profile(&mut c, input, None).unwrap();
            let SaveProfileOutcome::Saved { profile } = out else { panic!("{out:?}") };
            assert!(profile.issues.is_empty(), "{:?}", profile.issues);
            assert!(profile.totals.population > 0 && profile.totals.staff_paid > 0);
            assert!(profile.totals.payroll_monthly_mxn > 0 && profile.totals.fees_monthly_mxn > 0, "the examples carry salaries and fees");
            let everything = serde_json::to_string(&profile.input).unwrap();
            assert!(!everything.contains("Esperanza Robles") && !everything.contains("Vázquez"), "no name reaches the profile");
        }
    }
}
