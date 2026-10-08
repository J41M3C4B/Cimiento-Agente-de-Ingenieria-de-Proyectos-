//! The first start (ADR-031). Three moments: the welcome of each account (once per person), the setup the
//! administrator leaves ready (the AI and the accounts), and the data of the institution, required before the app is
//! used. What each step needs is decided in `domain::onboarding`; this saves each step as it goes (the profile as a
//! draft and the main site), so the person can leave and come back where they were, and closes it: the profile is
//! confirmed and `institution.onboarded_at` is set once, for good.

use crate::access_service::CurrentUser;
use crate::audit::{self, AuditKind};
use crate::domain::access::Role;
use crate::domain::onboarding::{self, Facts, StepStatus};
use crate::domain::profile::{IncomeSourceInput, InstitutionInput, ProfileInput};
use crate::facilities::domain::site::SiteData;
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::service::{self, SaveProfileOutcome, ServiceError};
use crate::storage::profile as profile_store;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// What the steps fill in, as the screen sends it and gets it back: the profile fields of the steps and the main
/// data of the site.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OnboardingData {
    pub institution: InstitutionInput,
    pub capacity_total: Option<i64>,
    pub served_estimate: Option<i64>,
    pub staff_paid_estimate: Option<i64>,
    pub staff_volunteer_estimate: Option<i64>,
    pub annual_budget_mxn: Option<i64>,
    pub income: Vec<IncomeSourceInput>,
    pub floors: Option<i64>,
    pub built_m2: Option<i64>,
    pub tenure: Option<String>,
    pub tenure_until: Option<i64>,
    pub tenure_documented: Option<bool>,
}

/// What the modules already have: the steps take it into account and the screen tells it.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Records {
    pub served: i64,
    pub staff: i64,
    pub fee_payers: i64,
}

/// What the administrator leaves ready besides the data (only shown to them).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Setup {
    pub ai_ready: bool,
    /// Active accounts of direction and accounting.
    pub managers: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStatus {
    /// The institution finished it once: the app opens. It never goes back.
    pub done: bool,
    pub steps: Vec<StepStatus>,
    /// Every step is complete: the review can close it.
    pub ready: bool,
    pub data: OnboardingData,
    pub records: Records,
    /// This person already saw the welcome.
    pub welcomed: bool,
    /// The administrator may leave the data to the direction and enter the app.
    pub can_postpone: bool,
    pub setup: Option<Setup>,
    pub states: Vec<(&'static str, &'static str)>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OnboardingOutcome {
    Saved { onboarding: OnboardingStatus },
    Invalid { issues: Vec<OnboardingIssue> },
    Quarantine { report: QuarantineReport },
}

/// A problem of a field, from the profile or from the site.
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingIssue {
    pub code: &'static str,
    pub field: String,
}

fn onboarded(conn: &Connection) -> Result<bool, ServiceError> {
    Ok(conn.query_row("SELECT onboarded_at FROM institution LIMIT 1", [], |r| r.get::<_, Option<String>>(0)).optional()?.flatten().is_some())
}

fn records(conn: &Connection) -> Result<Records, ServiceError> {
    let people = crate::care::api::indicators(conn, crate::core::institution::care_flavor(conn))?;
    let staff = crate::hr::api::ai_summary(conn)?;
    let totals = crate::profile_sync::totals(conn)?;
    Ok(Records { served: people.served, staff: staff.total, fee_payers: totals.fee_payers })
}

fn setup(conn: &Connection) -> Result<Setup, ServiceError> {
    let provider = crate::ai::settings::load(conn)?.provider;
    let ai_ready = crate::storage::get_api_key(provider.as_str()).ok().flatten().is_some();
    let managers = conn.query_row("SELECT count(*) FROM app_user WHERE role = 'manager' AND active = 1", [], |r| r.get(0))?;
    Ok(Setup { ai_ready, managers })
}

fn main_site(conn: &Connection) -> Result<Option<SiteData>, ServiceError> {
    Ok(crate::facilities::api::summaries(conn)?.into_iter().next().map(|s| s.site))
}

pub fn status(conn: &Connection, user: &CurrentUser) -> Result<OnboardingStatus, ServiceError> {
    let input = profile_store::load_current(conn)?.map(|p| p.input).unwrap_or_default();
    let site = main_site(conn)?;
    let r = records(conn)?;
    let steps = onboarding::steps(&Facts { input: &input, served_in_module: r.served, staff_in_module: r.staff, fee_payers: r.fee_payers, site: site.as_ref() });
    let welcomed = conn.query_row("SELECT welcomed_at FROM app_user WHERE id = ?1", [&user.id], |r| r.get::<_, Option<String>>(0)).optional()?.flatten().is_some();
    let admin = user.role == Role::Admin;
    let s = site.unwrap_or_default();
    Ok(OnboardingStatus {
        done: onboarded(conn)?,
        ready: onboarding::complete(&steps),
        steps,
        data: OnboardingData {
            institution: input.institution,
            capacity_total: input.capacity_total,
            served_estimate: input.served_estimate,
            staff_paid_estimate: input.staff_paid_estimate,
            staff_volunteer_estimate: input.staff_volunteer_estimate,
            annual_budget_mxn: input.annual_budget_mxn,
            income: input.income,
            floors: s.floors,
            built_m2: s.built_m2,
            tenure: s.tenure,
            tenure_until: s.tenure_until,
            tenure_documented: s.tenure_documented,
        },
        records: r,
        welcomed,
        can_postpone: admin,
        setup: if admin { Some(setup(conn)?) } else { None },
        states: onboarding::STATES.to_vec(),
    })
}

/// Saves what the steps carry: the profile (through the scanner, as a draft) and the main data of the site. It saves
/// whatever is valid even when a step is not complete yet: the status says what is still missing.
pub fn save(conn: &mut Connection, user: &CurrentUser, data: OnboardingData, decision: Option<Decision>) -> Result<OnboardingOutcome, ServiceError> {
    let mut input: ProfileInput = profile_store::load_current(conn)?.map(|p| p.input).unwrap_or_default();
    input.institution = data.institution;
    input.capacity_total = data.capacity_total;
    input.served_estimate = data.served_estimate;
    input.staff_paid_estimate = data.staff_paid_estimate;
    input.staff_volunteer_estimate = data.staff_volunteer_estimate;
    input.annual_budget_mxn = data.annual_budget_mxn;
    input.income = data.income;
    match service::save_profile(conn, input, decision)? {
        SaveProfileOutcome::Saved { .. } => {}
        SaveProfileOutcome::Invalid { issues } => {
            return Ok(OnboardingOutcome::Invalid { issues: issues.into_iter().map(|i| OnboardingIssue { code: i.code, field: i.field }).collect() })
        }
        SaveProfileOutcome::Quarantine { report } => return Ok(OnboardingOutcome::Quarantine { report }),
    }

    // the site: only its main data change; the rest of what the facilities module has stays
    let mut site = main_site(conn)?.unwrap_or_default();
    let before = site.clone();
    site.floors = data.floors;
    site.built_m2 = data.built_m2;
    site.tenure = data.tenure;
    site.tenure_until = data.tenure_until;
    site.tenure_documented = data.tenure_documented;
    let touched = site != before;
    if touched {
        if let crate::facilities::service::SaveOutcome::Invalid { issues } = crate::facilities::service::save_site(conn, site)? {
            return Ok(OnboardingOutcome::Invalid { issues: issues.into_iter().filter(|i| i.blocking).map(|i| OnboardingIssue { code: i.code, field: format!("site.{}", i.field) }).collect() });
        }
    }
    Ok(OnboardingOutcome::Saved { onboarding: status(conn, user)? })
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
    let tx = conn.transaction()?;
    tx.execute("UPDATE institution SET onboarded_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE onboarded_at IS NULL", [])?;
    audit::record(&tx, AuditKind::InstitutionOnboarded, Some("institution"), None, json!({}))?;
    tx.commit()?;
    status(conn, user)
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
#[path = "onboarding_service_tests.rs"]
mod tests;
