//! The first start (ADR-031, ADR-042). Three moments: the welcome of each account (once per person), the setup the
//! administrator leaves ready (the AI and the accounts), and the indispensable data of the institution, required
//! before the app is used. Which data they are is decided by the catalog of forms (`essential`) and the steps in
//! `onboarding::domain`; each step is saved through its window of «Mi institución», the same way (checks, scanner,
//! history), so the person can leave and come back where they were. Finishing confirms the profile and sets
//! `institution.onboarded_at` once, for good.

use crate::audit::{self, AuditKind};
use crate::common::forms::{is_filled, FieldSpec, Values};
use crate::core::access::domain::Role;
use crate::core::access::service::CurrentUser;
use crate::core::error::ServiceError;
use crate::core::onboarding::domain::{self as onboarding, STEPS};
use crate::core::profile::domain::ProfileInput;
use crate::core::profile::service;
use crate::core::profile::service::SaveProfileOutcome;
use crate::core::profile::storage as profile_store;
use crate::scanner::guard::{Decision, QuarantineReport};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use serde_json::json;

/// What the modules already have: the quick figures they answer are not asked, and the screen tells it.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Records {
    pub served: i64,
    pub staff: i64,
}

/// What the administrator leaves ready besides the data (only shown to them).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Setup {
    pub ai_ready: bool,
    /// Active accounts of direction and accounting.
    pub managers: i64,
}

/// A step as the screen draws it: its fields come from the catalog, with the same words, options and checks as in
/// their window. It has the shape of a form (`FormRenderer`).
#[derive(Debug, Clone, Serialize)]
pub struct StepForm {
    pub id: String,
    pub sections: [StepSection; 1],
}

#[derive(Debug, Clone, Serialize)]
pub struct StepSection {
    pub id: &'static str,
    pub columns: u8,
    /// Without the quick figures the records already answer.
    pub fields: Vec<&'static FieldSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepView {
    pub key: &'static str,
    /// The indispensable data still missing, by field id.
    pub missing: Vec<&'static str>,
    pub complete: bool,
    pub form: StepForm,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStatus {
    /// The institution finished it once: the app opens. It never goes back.
    pub done: bool,
    pub steps: Vec<StepView>,
    /// Every step is complete: the review can close it.
    pub ready: bool,
    /// What is saved of the fields of the steps, by field id.
    pub values: Values,
    pub records: Records,
    /// This person already saw the welcome.
    pub welcomed: bool,
    /// The administrator may leave the data to the direction and enter the app.
    pub can_postpone: bool,
    pub setup: Option<Setup>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OnboardingOutcome {
    Saved { onboarding: OnboardingStatus },
    Invalid { issues: Vec<OnboardingIssue> },
    Quarantine { report: QuarantineReport },
}

/// A problem of a field of the step.
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingIssue {
    pub code: &'static str,
    pub field: String,
}

fn onboarded(conn: &Connection) -> Result<bool, ServiceError> {
    Ok(conn.query_row("SELECT onboarded_at FROM institution LIMIT 1", [], |r| r.get::<_, Option<String>>(0)).optional()?.flatten().is_some())
}

/// What the modules hold now: the people served and the team with a record.
pub fn records(conn: &Connection) -> Result<Records, ServiceError> {
    let people = crate::modules::care::api::indicators(conn, crate::core::institution::care_flavor(conn))?;
    let staff = crate::modules::hr::api::ai_summary(conn)?;
    Ok(Records { served: people.served, staff: staff.total })
}

fn setup(conn: &Connection) -> Result<Setup, ServiceError> {
    let provider = crate::ai::settings::load(conn)?.provider;
    let ai_ready = crate::storage::get_api_key(provider.as_str()).ok().flatten().is_some();
    let managers = conn.query_row("SELECT count(*) FROM app_user WHERE role = 'manager' AND active = 1", [], |r| r.get(0))?;
    Ok(Setup { ai_ready, managers })
}

/// Every datum of the windows of «Mi institución», by field id.
fn saved_values(conn: &Connection) -> Result<Values, ServiceError> {
    let input: ProfileInput = profile_store::load_current(conn)?.map(|p| p.input).unwrap_or_default();
    Ok(crate::core::profile::forms::catalog_values(&input))
}

/// The quick figures the records of the modules already answer (by field id): they are not asked.
pub fn answered(conn: &Connection) -> Result<Vec<&'static str>, ServiceError> {
    let r = records(conn)?;
    Ok(onboarding::answered_by_records(r.served, r.staff))
}

pub fn status(conn: &Connection, user: &CurrentUser) -> Result<OnboardingStatus, ServiceError> {
    let all = saved_values(conn)?;
    let r = records(conn)?;
    let answered = onboarding::answered_by_records(r.served, r.staff);
    let statuses = onboarding::steps(&all, &answered);
    let steps: Vec<StepView> = STEPS
        .iter()
        .zip(statuses)
        .map(|(s, st)| StepView {
            key: s.key,
            missing: st.missing,
            complete: st.complete,
            form: StepForm {
                id: format!("onboarding.{}", s.key),
                sections: [StepSection { id: s.key, columns: s.columns, fields: s.fields.iter().filter(|f| !answered.contains(f)).map(|f| onboarding::field(f)).collect() }],
            },
        })
        .collect();
    let values = all.into_iter().filter(|(k, _)| STEPS.iter().any(|s| s.fields.contains(&k.as_str()))).collect();
    let welcomed = conn.query_row("SELECT welcomed_at FROM app_user WHERE id = ?1", [&user.id], |r| r.get::<_, Option<String>>(0)).optional()?.flatten().is_some();
    let admin = user.role == Role::Admin;
    Ok(OnboardingStatus {
        done: onboarded(conn)?,
        ready: steps.iter().all(|s| s.complete),
        steps,
        values,
        records: r,
        welcomed,
        can_postpone: admin,
        setup: if admin { Some(setup(conn)?) } else { None },
    })
}

/// Saves one step through its window of «Mi institución» (`profile::forms::save`): the same checks, scanner and
/// history as there. Only the fields of the step change; a field of the step left empty is emptied, and what the
/// records answer is left as it is. It saves whatever is valid even when the step is not complete yet: the status
/// says what is still missing.
pub fn save(conn: &mut Connection, user: &CurrentUser, step: &str, values: Values, decision: Option<Decision>) -> Result<OnboardingOutcome, ServiceError> {
    let s = STEPS.iter().find(|s| s.key == step).ok_or(ServiceError::NotFound)?;
    let answered = answered(conn)?;
    let mut window = crate::core::profile::forms::saved_values(conn, s.form)?;
    for f in s.fields.iter().filter(|f| !answered.contains(f)) {
        match values.get(*f) {
            Some(v) if is_filled(Some(v)) => window.insert(f.to_string(), v.clone()),
            _ => window.remove(*f),
        };
    }
    match crate::core::profile::forms::save(conn, s.form, window, decision)? {
        SaveProfileOutcome::Saved { .. } => Ok(OnboardingOutcome::Saved { onboarding: status(conn, user)? }),
        SaveProfileOutcome::Invalid { issues } => Ok(OnboardingOutcome::Invalid { issues: issues.into_iter().map(|i| OnboardingIssue { code: i.code, field: i.field }).collect() }),
        SaveProfileOutcome::Quarantine { report } => Ok(OnboardingOutcome::Quarantine { report }),
    }
}

/// Sets the date the first start was finished, once, with its line in the log.
fn mark_onboarded(conn: &mut Connection) -> Result<(), ServiceError> {
    let tx = conn.transaction()?;
    tx.execute("UPDATE institution SET onboarded_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE onboarded_at IS NULL", [])?;
    audit::record(&tx, AuditKind::InstitutionOnboarded, Some("institution"), None, json!({}))?;
    tx.commit()?;
    Ok(())
}

/// Whether every indispensable datum is there now.
fn complete_now(conn: &Connection) -> Result<bool, ServiceError> {
    Ok(onboarding::complete(&onboarding::steps(&saved_values(conn)?, &answered(conn)?)))
}

/// Closes it: every step complete, the profile confirmed (its first version) and the date set, once.
pub fn finish(conn: &mut Connection, user: &CurrentUser) -> Result<OnboardingStatus, ServiceError> {
    let s = status(conn, user)?;
    if s.done {
        return Ok(s);
    }
    if !s.ready {
        return Err(ServiceError::OnboardingIncomplete);
    }
    if profile_store::load_current(conn)?.is_some_and(|p| p.confirmed_at.is_none()) {
        service::confirm_profile(conn)?;
    }
    mark_onboarded(conn)?;
    status(conn, user)
}

/// What «Mi institución» saves. Once the institution finished its first start, what the person types there is
/// confirmed at once: the profile never waits for a «review» that nobody could do any more, and the AI sheet and the
/// stage of a project keep reading it as confirmed. Before that (the administrator left the data to the direction) it
/// stays a draft, until the save that leaves no indispensable datum missing: that one finishes the first start, from
/// whatever window it comes (ADR-042: one way to fill the institution).
pub fn save_profile_confirmed(conn: &mut Connection, input: ProfileInput, decision: Option<Decision>) -> Result<SaveProfileOutcome, ServiceError> {
    save_profile_confirmed_from(conn, input, decision, &service::Taken::default())
}

/// As `save_profile_confirmed`, with the suggestions of the AI the person took (IA4).
pub fn save_profile_confirmed_from(conn: &mut Connection, input: ProfileInput, decision: Option<Decision>, taken: &service::Taken<'_>) -> Result<SaveProfileOutcome, ServiceError> {
    match service::save_profile_from(conn, input, decision, taken)? {
        SaveProfileOutcome::Saved { .. } if onboarded(conn)? => Ok(SaveProfileOutcome::Saved { profile: service::confirm_profile(conn)? }),
        SaveProfileOutcome::Saved { .. } if complete_now(conn)? => {
            let profile = service::confirm_profile(conn)?;
            mark_onboarded(conn)?;
            Ok(SaveProfileOutcome::Saved { profile })
        }
        other => Ok(other),
    }
}

/// The person saw the welcome; it does not show again.
pub fn welcome_done(conn: &Connection, user: &CurrentUser) -> Result<(), ServiceError> {
    conn.execute("UPDATE app_user SET welcomed_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1 AND welcomed_at IS NULL", [&user.id])?;
    Ok(())
}

/// Development only: an example loaded from `fixtures/` is a finished institution.
#[cfg(debug_assertions)]
pub fn mark_done(conn: &Connection) -> Result<(), ServiceError> {
    conn.execute("UPDATE institution SET onboarded_at = coalesce(onboarded_at, strftime('%Y-%m-%dT%H:%M:%SZ','now'))", [])?;
    Ok(())
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
