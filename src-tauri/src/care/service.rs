//! The use cases of the module of the people served. The CURP leaves this file only covered, except through
//! `reveal_curp`, which writes in the audit log that it was looked at (never the value). The kind of institution
//! comes from the app (`flavor`): the module does not read the profile.

use super::domain::catalog::{self, Flavor};
use super::domain::person::{BeneficiaryData, Issue, Progress};
use super::domain::waitlist::WaitlistInput;
use super::storage::{self as store, CustomField, Group, StoredPerson, WaitlistRow};
use super::CareError;
use crate::audit::{self, AuditKind};
use crate::common::ids;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct PersonRow {
    pub id: String,
    pub full_name: String,
    pub group: String,
    pub age: Option<i64>,
    pub sex: Option<String>,
    pub status: String,
    pub dependency: Option<String>,
    pub entry_date: Option<String>,
    pub progress: u32,
    pub heads_up: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub people: Vec<PersonRow>,
    pub groups: Vec<Group>,
    pub custom_fields: Vec<CustomField>,
    pub waitlist: Vec<WaitlistRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersonView {
    pub id: String,
    pub data: BeneficiaryData,
    pub curp_stored: bool,
    /// How the CURP looks covered: `HEGG••••••••••••04`.
    pub curp_masked: Option<String>,
    pub group: String,
    pub age: Option<i64>,
    pub progress: Progress,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SaveOutcome {
    Saved { person: PersonView },
    Invalid { issues: Vec<Issue> },
}

fn group_of(d: &BeneficiaryData, groups: &[Group], flavor: Flavor, today: &str) -> String {
    d.group_id
        .as_deref()
        .and_then(|g| groups.iter().find(|x| x.id == g))
        .map(|g| g.title.clone())
        .unwrap_or_else(|| catalog::group_label(flavor, d.sex.as_deref(), d.age(today)))
}

fn view(p: StoredPerson, groups: &[Group], flavor: Flavor, today: &str) -> PersonView {
    let curp_stored = p.data.curp.as_deref().is_some_and(|c| !c.is_empty());
    let curp_masked = p.data.curp.as_deref().filter(|c| !c.is_empty()).map(|c| ids::mask(c, 4, 2));
    let issues = p.data.validate(flavor, today).into_iter().filter(|i| !i.blocking).collect();
    let mut data = p.data;
    data.curp = None;
    PersonView {
        id: p.id,
        group: group_of(&data, groups, flavor, today),
        age: data.age(today),
        progress: data.progress(flavor, curp_stored),
        curp_stored,
        curp_masked,
        issues,
        data,
    }
}

pub fn overview(conn: &Connection, flavor: Flavor) -> Result<Overview, CareError> {
    let today = store::today(conn)?;
    let groups = store::groups(conn)?;
    let people = store::people(conn)?
        .into_iter()
        .map(|p| {
            let stored = p.data.curp.as_deref().is_some_and(|c| !c.is_empty());
            PersonRow {
                full_name: p.data.full_name(),
                group: group_of(&p.data, &groups, flavor, &today),
                age: p.data.age(&today),
                sex: p.data.sex.clone(),
                status: p.data.status.clone(),
                dependency: p.data.dependency.clone(),
                entry_date: p.data.entry_date.clone(),
                progress: p.data.progress(flavor, stored).percent,
                heads_up: p.data.validate(flavor, &today).iter().filter(|i| !i.blocking).count(),
                id: p.id,
            }
        })
        .collect();
    Ok(Overview { people, groups, custom_fields: store::custom_fields(conn)?, waitlist: store::waitlist(conn)? })
}

pub fn get_person(conn: &Connection, flavor: Flavor, id: &str) -> Result<Option<PersonView>, CareError> {
    let today = store::today(conn)?;
    let groups = store::groups(conn)?;
    Ok(store::person(conn, id)?.map(|p| view(p, &groups, flavor, &today)))
}

/// Saves a record (`id` = `None` adds one). Nothing is saved while there is a blocking problem.
pub fn save_person(conn: &mut Connection, flavor: Flavor, id: Option<&str>, mut data: BeneficiaryData) -> Result<SaveOutcome, CareError> {
    let today = store::today(conn)?;
    if data.status.trim().is_empty() {
        data.status = "active".into();
    }
    if let Some(c) = data.curp.as_mut() {
        *c = ids::normalize(c);
    }
    let old = id.map(|pid| store::person(conn, pid)).transpose()?.flatten();
    if id.is_some() && old.is_none() {
        return Err(CareError::NotFound);
    }
    let issues = data.validate(flavor, &today);
    let mut blocking: Vec<Issue> = issues.into_iter().filter(|i| i.blocking).collect();
    let groups = store::groups(conn)?;
    if data.group_id.as_deref().is_some_and(|g| !groups.iter().any(|x| x.id == g)) {
        blocking.push(Issue { code: "code_unknown", field: "group_id".into(), blocking: true });
    }
    if !blocking.is_empty() {
        return Ok(SaveOutcome::Invalid { issues: blocking });
    }

    // only an age was known: the 1st of January of the year it gives, marked as approximate
    match (data.birth_date.as_deref().filter(|b| !b.is_empty()), data.approx_age) {
        (None, Some(age)) => {
            let year: i64 = today[..4].parse().unwrap_or(2026);
            data.birth_date = Some(format!("{}-01-01", year - age));
            data.birth_date_approx = true;
        }
        _ => data.birth_date_approx = old.as_ref().is_some_and(|o| o.data.birth_date_approx && o.data.birth_date == data.birth_date),
    }
    data.approx_age = None;
    data.entry_date_approx = old.as_ref().is_some_and(|o| o.data.entry_date_approx && o.data.entry_date == data.entry_date);
    let fields: Vec<String> = store::custom_fields(conn)?.into_iter().map(|f| f.key).collect();
    data.extra.retain(|k, v| fields.contains(k) && !v.trim().is_empty());
    if let Some(o) = &old {
        // a field hidden while its deletion waits keeps its value
        for key in store::hidden_field_keys(conn)? {
            if let Some(v) = o.data.extra.get(&key) {
                data.extra.insert(key, v.clone());
            }
        }
    }

    let tx = conn.transaction()?;
    let pid = store::save_person(&tx, id, None, &data)?.ok_or(CareError::NotFound)?;
    tx.commit()?;
    let stored = store::person(conn, &pid)?.ok_or(CareError::NotFound)?;
    Ok(SaveOutcome::Saved { person: view(stored, &groups, flavor, &today) })
}

pub fn delete_person(conn: &mut Connection, id: &str) -> Result<(), CareError> {
    let tx = conn.transaction()?;
    if !store::delete_person(&tx, id)? {
        return Err(CareError::NotFound);
    }
    audit::record(&tx, AuditKind::CarePersonDeleted, Some("care_person"), Some(id), json!({}))?;
    tx.commit()?;
    Ok(())
}

/// Hides a record or brings it back (a deletion that waits for the administrator, ADR-028). Returns the name.
pub fn set_person_hidden(conn: &Connection, id: &str, hidden: bool) -> Result<String, CareError> {
    let p = store::person(conn, id)?.ok_or(CareError::NotFound)?;
    store::set_person_hidden(conn, id, hidden)?;
    Ok(p.data.full_name())
}

pub fn set_field_hidden(conn: &Connection, key: &str, hidden: bool) -> Result<String, CareError> {
    let title = store::field_title(conn, key)?.ok_or(CareError::NotFound)?;
    store::set_field_hidden(conn, key, hidden)?;
    Ok(title)
}

pub fn reveal_curp(conn: &Connection, id: &str) -> Result<Option<String>, CareError> {
    let value = store::curp_of(conn, id)?;
    if value.is_some() {
        audit::record(conn, AuditKind::CareSensitiveViewed, Some("care_person"), Some(id), json!({ "field": "curp" }))?;
    }
    Ok(value)
}

pub fn save_group(conn: &Connection, id: Option<&str>, title: &str, active: bool) -> Result<Vec<Group>, CareError> {
    if title.trim().is_empty() {
        return Err(CareError::EmptyTitle);
    }
    store::save_group(conn, id, title, active)?;
    store::groups(conn)
}

pub fn save_custom_field(conn: &Connection, key: Option<&str>, title: &str, kind: &str, options: &[String]) -> Result<Vec<CustomField>, CareError> {
    if title.trim().is_empty() {
        return Err(CareError::EmptyTitle);
    }
    if !["text", "select", "number"].contains(&kind) {
        return Err(CareError::UnknownField);
    }
    store::save_custom_field(conn, key, title, kind, options)?;
    store::custom_fields(conn)
}

pub fn delete_custom_field(conn: &Connection, key: &str) -> Result<Vec<CustomField>, CareError> {
    store::delete_custom_field(conn, key)?;
    store::custom_fields(conn)
}

pub fn save_waitlist(conn: &Connection, id: Option<&str>, input: &WaitlistInput) -> Result<Result<Vec<WaitlistRow>, Vec<Issue>>, CareError> {
    let mut input = input.clone();
    if input.status.trim().is_empty() {
        input.status = "waiting".into();
    }
    let issues = input.validate();
    if !issues.is_empty() {
        return Ok(Err(issues));
    }
    if !store::save_waitlist(conn, id, &input)? {
        return Err(CareError::NotFound);
    }
    Ok(Ok(store::waitlist(conn)?))
}

/// A request of the waiting list comes in: it becomes a record with what the request said, to complete it.
pub fn admit_waitlist(conn: &mut Connection, flavor: Flavor, id: &str) -> Result<PersonView, CareError> {
    let row = store::waitlist(conn)?.into_iter().find(|w| w.id == id && w.input.status == "waiting").ok_or(CareError::NotFound)?;
    let today = store::today(conn)?;
    let w = row.input;
    let mut words = w.name.as_deref().unwrap_or_default().split_whitespace();
    let first = words.next().map(String::from).unwrap_or_else(|| "Por registrar".into());
    let rest: Vec<&str> = words.collect();
    let data = BeneficiaryData {
        first_names: first,
        last_name_1: rest.first().map(|s| s.to_string()),
        last_name_2: (rest.len() > 1).then(|| rest[1..].join(" ")),
        sex: w.sex,
        approx_age: Some(w.approx_age.ok_or(CareError::AgeNeeded)?),
        birth_date: None,
        dependency: w.dependency,
        admission_reasons: w.reason.into_iter().collect(),
        entry_date: Some(today),
        status: "active".into(),
        contacts: Vec::new(),
        ..Default::default()
    };
    match save_person(conn, flavor, None, data)? {
        SaveOutcome::Saved { person } => {
            store::admit_waitlist(conn, id, &person.id)?;
            Ok(person)
        }
        SaveOutcome::Invalid { .. } => Err(CareError::NotFound),
    }
}

pub fn delete_waitlist(conn: &Connection, id: &str) -> Result<Vec<WaitlistRow>, CareError> {
    if !store::delete_waitlist(conn, id)? {
        return Err(CareError::NotFound);
    }
    store::waitlist(conn)
}
