//! The use cases of the staff module. The covered identifiers (CURP, RFC, NSS, CLABE) leave this file only covered,
//! except through `reveal`, which writes in the audit log who looked and at what (never the value).

use super::domain::catalog::{self, Rules};
use super::domain::ids;
use super::domain::person::{Issue, PersonData, Progress, Secrets};
use super::domain::position::{Position, PositionInput};
use super::storage::{self as store, CustomField, CustomModality, StoredPerson};
use super::HrError;
use crate::audit::{self, AuditKind};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::json;

/// A modality as the form offers it. The built-in ones are named by the screen from their code.
#[derive(Debug, Clone, Serialize)]
pub struct ModalityInfo {
    pub code: String,
    /// Only for the institution's own ones.
    pub title: Option<String>,
    pub builtin: bool,
    pub behaves_as: &'static str,
    pub rules: Rules,
}

/// One row of the list of the staff.
#[derive(Debug, Clone, Serialize)]
pub struct PersonRow {
    pub id: String,
    pub full_name: String,
    pub position_id: Option<String>,
    pub modality: String,
    pub relation: Option<&'static str>,
    pub status: String,
    pub start_date: Option<String>,
    pub phone: Option<String>,
    pub progress: u32,
    /// Heads-ups of the record (it saved, but something is worth a look).
    pub heads_up: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionRow {
    #[serde(flatten)]
    pub position: Position,
    /// People who work in it now.
    pub people: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub people: Vec<PersonRow>,
    pub positions: Vec<PositionRow>,
    pub modalities: Vec<ModalityInfo>,
    pub custom_fields: Vec<CustomField>,
}

/// A record as the screen gets it: the identifiers covered, how far it has come and its heads-ups.
#[derive(Debug, Clone, Serialize)]
pub struct PersonView {
    pub id: String,
    pub data: PersonData,
    /// Which covered identifiers are stored.
    pub secrets: Secrets,
    /// How they look covered: `HEGG••••••••••••04`, `••••••••••••••7771`.
    pub masked: Masked,
    /// The start date is the 1st of January of the year the old roster said.
    pub start_date_approx: bool,
    pub progress: Progress,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Masked {
    pub curp: Option<String>,
    pub rfc: Option<String>,
    pub nss: Option<String>,
    pub clabe: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SaveOutcome {
    Saved { person: PersonView },
    Invalid { issues: Vec<Issue> },
}

/// The modality behind a code: the built-in one whose rules apply, and the rules.
pub fn resolve(conn: &Connection, code: &str) -> Result<Option<(&'static str, Rules)>, HrError> {
    Ok(store::base_of(conn, code)?.and_then(|b| catalog::builtin_rules(b).map(|r| (b, r))))
}

pub fn modalities(conn: &Connection) -> Result<Vec<ModalityInfo>, HrError> {
    let mut out: Vec<ModalityInfo> = catalog::BUILTIN_MODALITIES
        .iter()
        .map(|(c, r)| ModalityInfo { code: (*c).into(), title: None, builtin: true, behaves_as: c, rules: *r })
        .collect();
    for CustomModality { code, title, behaves_as } in store::custom_modalities(conn)? {
        if let Some((base, rules)) = catalog::BUILTIN_MODALITIES.iter().find(|(c, _)| *c == behaves_as) {
            out.push(ModalityInfo { code, title: Some(title), builtin: false, behaves_as: base, rules: *rules });
        }
    }
    Ok(out)
}

fn secrets_of(d: &PersonData) -> Secrets {
    let set = |o: &Option<String>| o.as_deref().is_some_and(|s| !s.is_empty());
    Secrets { curp: set(&d.curp), rfc: set(&d.rfc), nss: set(&d.nss), clabe: set(&d.clabe) }
}

/// The identifiers covered: they come back as `None`, which the form sends back to mean «leave it».
fn covered(mut d: PersonData) -> PersonData {
    d.curp = None;
    d.rfc = None;
    d.nss = None;
    d.clabe = None;
    d
}

fn view(conn: &Connection, p: StoredPerson, today: &str) -> Result<PersonView, HrError> {
    let rules = resolve(conn, &p.data.modality)?.map(|(_, r)| r);
    let secrets = secrets_of(&p.data);
    let issues = p.data.validate(rules, today).into_iter().filter(|i| !i.blocking).collect();
    let m = |field: &str, v: &Option<String>| v.as_deref().filter(|s| !s.is_empty()).map(|s| masked(field, s));
    let masked = Masked { curp: m("curp", &p.data.curp), rfc: m("rfc", &p.data.rfc), nss: m("nss", &p.data.nss), clabe: m("clabe", &p.data.clabe) };
    let data = covered(p.data);
    let progress = data.progress(rules, secrets);
    Ok(PersonView { id: p.id, data, secrets, masked, start_date_approx: p.start_date_approx, progress, issues })
}

/// What a covered identifier looks like on screen.
pub fn masked(field: &str, value: &str) -> String {
    match field {
        "curp" | "rfc" => ids::mask(value, 4, 2),
        _ => ids::mask(value, 0, 4),
    }
}

pub fn overview(conn: &Connection) -> Result<Overview, HrError> {
    let today = store::today(conn)?;
    let mut people = Vec::new();
    for p in store::people(conn)? {
        let resolved = resolve(conn, &p.data.modality)?;
        let rules = resolved.map(|(_, r)| r);
        let secrets = secrets_of(&p.data);
        people.push(PersonRow {
            id: p.id.clone(),
            full_name: p.data.full_name(),
            position_id: p.data.position_id.clone(),
            modality: p.data.modality.clone(),
            relation: rules.map(|r| r.relation.as_str()),
            status: p.data.status.clone(),
            start_date: p.data.start_date.clone(),
            phone: p.data.phone.clone(),
            progress: p.data.progress(rules, secrets).percent,
            heads_up: p.data.validate(rules, &today).iter().filter(|i| !i.blocking).count(),
        });
    }
    let mut positions = Vec::new();
    for position in store::positions(conn)? {
        let people = store::count_in_position(conn, &position.id)?;
        positions.push(PositionRow { position, people });
    }
    Ok(Overview { people, positions, modalities: modalities(conn)?, custom_fields: store::custom_fields(conn)? })
}

pub fn get_person(conn: &Connection, id: &str) -> Result<Option<PersonView>, HrError> {
    let today = store::today(conn)?;
    store::person(conn, id)?.map(|p| view(conn, p, &today)).transpose()
}

/// Saves a record (`id` = `None` adds one). Nothing is saved while there is a blocking problem.
pub fn save_person(conn: &mut Connection, id: Option<&str>, mut data: PersonData) -> Result<SaveOutcome, HrError> {
    let today = store::today(conn)?;
    for s in [&mut data.curp, &mut data.rfc, &mut data.nss, &mut data.clabe].into_iter().flatten() {
        *s = ids::normalize(s);
    }
    if let Some(bank) = data.clabe.as_deref().filter(|c| ids::clabe_is_valid(c)).and_then(ids::bank_of_clabe) {
        data.bank = Some(bank.into());
    }
    if data.status.trim().is_empty() {
        data.status = "active".into();
    }
    let fields: Vec<String> = store::custom_fields(conn)?.into_iter().map(|f| f.key).collect();
    data.extra.retain(|k, v| fields.contains(k) && !v.trim().is_empty());
    // a field hidden while its deletion waits keeps its value: if the deletion is refused, nothing was lost
    if let Some(old) = id.map(|pid| store::person(conn, pid)).transpose()?.flatten() {
        for key in store::hidden_field_keys(conn)? {
            if let Some(v) = old.data.extra.get(&key) {
                data.extra.insert(key, v.clone());
            }
        }
    }

    let rules = resolve(conn, &data.modality)?.map(|(_, r)| r);
    let mut issues = data.validate(rules, &today);
    let positions = store::positions(conn)?;
    if data.position_id.as_deref().is_some_and(|p| !positions.iter().any(|x| x.id == p)) {
        issues.push(Issue { code: "position_missing", field: "position_id".into(), blocking: true });
    }
    let blocking: Vec<Issue> = issues.into_iter().filter(|i| i.blocking).collect();
    if !blocking.is_empty() {
        return Ok(SaveOutcome::Invalid { issues: blocking });
    }

    let tx = conn.transaction()?;
    let approx = match id {
        // the date stays approximate only while the person does not change it
        Some(pid) => {
            let old = store::person(&tx, pid)?.ok_or(HrError::NotFound)?;
            old.start_date_approx && old.data.start_date == data.start_date
        }
        None => false,
    };
    let pid = store::save_person(&tx, id, &data, approx)?.ok_or(HrError::NotFound)?;
    tx.commit()?;
    let stored = store::person(conn, &pid)?.ok_or(HrError::NotFound)?;
    Ok(SaveOutcome::Saved { person: view(conn, stored, &today)? })
}

/// Deletes everything of a person. It is recorded (without who), because it cannot be undone.
pub fn delete_person(conn: &mut Connection, id: &str) -> Result<(), HrError> {
    let tx = conn.transaction()?;
    if !store::delete_person(&tx, id)? {
        return Err(HrError::NotFound);
    }
    audit::record(&tx, AuditKind::HrPersonDeleted, Some("hr_person"), Some(id), json!({}))?;
    tx.commit()?;
    Ok(())
}

/// Shows one covered identifier. The audit log keeps which field of which record was looked at, never the value.
pub fn reveal(conn: &Connection, id: &str, field: &str) -> Result<Option<String>, HrError> {
    let value = store::secret_value(conn, id, field)?;
    if value.is_some() {
        audit::record(conn, AuditKind::HrSensitiveViewed, Some("hr_person"), Some(id), json!({ "field": field, "actor": "manager" }))?;
    }
    Ok(value)
}

/// Adds (`id` = `None`) or edits a position. Blocking problems come back and nothing is saved.
pub fn save_position(conn: &Connection, id: Option<&str>, input: &PositionInput) -> Result<Result<Vec<PositionRow>, Vec<Issue>>, HrError> {
    let known: Vec<String> = modalities(conn)?.into_iter().map(|m| m.code).collect();
    let issues: Vec<Issue> = input.validate(|m| known.iter().any(|k| k == m)).into_iter().filter(|i| i.blocking).collect();
    if !issues.is_empty() {
        return Ok(Err(issues));
    }
    if input.reports_to.is_some() && input.reports_to.as_deref() == id {
        return Ok(Err(vec![Issue { code: "reports_to_itself", field: "reports_to".into(), blocking: true }]));
    }
    match id {
        Some(pid) => {
            if !store::update_position(conn, pid, input)? {
                return Err(HrError::NotFound);
            }
        }
        None => {
            store::insert_position(conn, input)?;
        }
    }
    Ok(Ok(overview(conn)?.positions))
}

/// Archives a position (it stops being offered) or brings it back. One with people in it cannot be archived.
pub fn set_position_active(conn: &Connection, id: &str, active: bool) -> Result<Vec<PositionRow>, HrError> {
    if !active && store::count_in_position(conn, id)? > 0 {
        return Err(HrError::PositionInUse);
    }
    if !store::set_position_active(conn, id, active)? {
        return Err(HrError::NotFound);
    }
    Ok(overview(conn)?.positions)
}

/// An institution's own modality, which follows the rules of a built-in one.
pub fn create_modality(conn: &Connection, title: &str, behaves_as: &str) -> Result<Vec<ModalityInfo>, HrError> {
    if title.trim().is_empty() {
        return Err(HrError::EmptyTitle);
    }
    if catalog::builtin_rules(behaves_as).is_none() {
        return Err(HrError::UnknownModality);
    }
    store::insert_modality(conn, title, behaves_as)?;
    modalities(conn)
}

pub fn save_custom_field(conn: &Connection, key: Option<&str>, title: &str, kind: &str, options: &[String]) -> Result<Vec<CustomField>, HrError> {
    if title.trim().is_empty() {
        return Err(HrError::EmptyTitle);
    }
    if !["text", "select", "number"].contains(&kind) {
        return Err(HrError::UnknownField);
    }
    store::save_custom_field(conn, key, title, kind, options)?;
    Ok(store::custom_fields(conn)?)
}

/// Hides a record or brings it back (a deletion that waits for the administrator, ADR-028). The name comes back to
/// label the request.
pub fn set_person_hidden(conn: &Connection, id: &str, hidden: bool) -> Result<String, HrError> {
    let p = store::person(conn, id)?.ok_or(HrError::NotFound)?;
    store::set_person_hidden(conn, id, hidden)?;
    Ok(p.data.full_name())
}

pub fn set_field_hidden(conn: &Connection, key: &str, hidden: bool) -> Result<String, HrError> {
    let title = store::field_title(conn, key)?.ok_or(HrError::NotFound)?;
    store::set_field_hidden(conn, key, hidden)?;
    Ok(title)
}

pub fn delete_custom_field(conn: &Connection, key: &str) -> Result<Vec<CustomField>, HrError> {
    store::delete_custom_field(conn, key)?;
    Ok(store::custom_fields(conn)?)
}
