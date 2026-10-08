//! Commands of the people served (ADR-029): thin, they call `care_service`.

use super::guard;
use crate::access_service::{request_deletion, Session};
use crate::care::domain::person::BeneficiaryData;
use crate::care::domain::waitlist::WaitlistInput;
use crate::care::service::PersonView;
use crate::care::storage::{CustomField, Group};
use crate::care_service::{self as svc, CareChange, CareOverview, PersonOutcome, WaitlistOutcome};
use crate::domain::access::DeletionKind;
use crate::error::UiError;
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::service::ServiceError;
use crate::Db;
use serde::Serialize;
use tauri::State;

fn lock<'a>(db: &'a State<'_, Db>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, UiError> {
    db.0.lock().map_err(|_| UiError::internal())
}

#[tauri::command]
pub fn care_overview(session: State<'_, Session>, db: State<'_, Db>) -> Result<CareOverview, UiError> {
    guard(&session, "care_overview")?;
    let conn = lock(&db)?;
    Ok(svc::overview(&conn)?)
}

#[tauri::command]
pub fn care_person_get(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<PersonView, UiError> {
    guard(&session, "care_person_get")?;
    let conn = lock(&db)?;
    Ok(svc::person(&conn, &id)?)
}

#[tauri::command]
pub fn care_person_save(session: State<'_, Session>, db: State<'_, Db>, id: Option<String>, data: BeneficiaryData) -> Result<PersonOutcome, UiError> {
    guard(&session, "care_person_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save_person(&mut conn, id.as_deref(), data)?)
}

/// Deletes a record for good, or (for whoever may not delete) hides it and asks the administrator (ADR-028).
#[tauri::command]
pub fn care_person_delete(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<CareChange, UiError> {
    let gate = guard(&session, "care_person_delete")?;
    let mut conn = lock(&db)?;
    if !gate.may_delete() {
        request_deletion(&mut conn, gate.user()?, DeletionKind::Beneficiary, &id)?;
        return Ok(svc::change(&mut conn)?);
    }
    Ok(svc::delete_person(&mut conn, &id)?)
}

/// Shows the covered CURP; the audit log keeps that it was looked at.
#[tauri::command]
pub fn care_person_reveal(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<String, UiError> {
    guard(&session, "care_person_reveal")?;
    let conn = lock(&db)?;
    Ok(svc::reveal_curp(&conn, &id)?)
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GroupOutcome {
    Saved { groups: Vec<Group> },
    Quarantine { report: QuarantineReport },
}

#[tauri::command]
pub fn care_group_save(session: State<'_, Session>, db: State<'_, Db>, id: Option<String>, title: String, active: bool, decision: Option<Decision>) -> Result<GroupOutcome, UiError> {
    guard(&session, "care_group_save")?;
    let mut conn = lock(&db)?;
    Ok(match svc::save_group(&mut conn, id.as_deref(), &title, active, decision)? {
        Ok(groups) => GroupOutcome::Saved { groups },
        Err(report) => GroupOutcome::Quarantine { report },
    })
}

#[tauri::command]
pub fn care_field_save(session: State<'_, Session>, db: State<'_, Db>, key: Option<String>, title: String, kind: String, options: Vec<String>) -> Result<Vec<CustomField>, UiError> {
    guard(&session, "care_field_save")?;
    let conn = lock(&db)?;
    Ok(svc::save_field(&conn, key.as_deref(), &title, &kind, &options)?)
}

#[tauri::command]
pub fn care_field_delete(session: State<'_, Session>, db: State<'_, Db>, key: String) -> Result<Vec<CustomField>, UiError> {
    let gate = guard(&session, "care_field_delete")?;
    let mut conn = lock(&db)?;
    if !gate.may_delete() {
        request_deletion(&mut conn, gate.user()?, DeletionKind::CareField, &key)?;
        return Ok(crate::care::storage::custom_fields(&conn).map_err(ServiceError::from)?);
    }
    Ok(svc::delete_field(&conn, &key)?)
}

#[tauri::command]
pub fn care_waitlist_save(session: State<'_, Session>, db: State<'_, Db>, id: Option<String>, input: WaitlistInput) -> Result<WaitlistOutcome, UiError> {
    guard(&session, "care_waitlist_save")?;
    let mut conn = lock(&db)?;
    Ok(svc::save_waitlist(&mut conn, id.as_deref(), &input)?)
}

#[tauri::command]
pub fn care_waitlist_admit(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<PersonOutcome, UiError> {
    guard(&session, "care_waitlist_admit")?;
    let mut conn = lock(&db)?;
    Ok(svc::admit_waitlist(&mut conn, &id)?)
}

/// Removes a request of the waiting list for good (direction and accounting mark it as withdrawn instead).
#[tauri::command]
pub fn care_waitlist_delete(session: State<'_, Session>, db: State<'_, Db>, id: String) -> Result<CareChange, UiError> {
    guard(&session, "care_waitlist_delete")?;
    let mut conn = lock(&db)?;
    Ok(svc::delete_waitlist(&mut conn, &id)?)
}
