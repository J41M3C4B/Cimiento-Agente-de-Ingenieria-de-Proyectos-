//! Persistence of the module of the people served: its own `care_*` tables and nothing else.

use super::domain::person::{BeneficiaryData, ResponsibleContact};
use super::domain::waitlist::WaitlistInput;
use super::CareError;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

pub fn today(conn: &Connection) -> Result<String, CareError> {
    Ok(conn.query_row("SELECT strftime('%Y-%m-%d','now')", [], |r| r.get(0))?)
}

fn text(o: &Option<String>) -> Option<&str> {
    o.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

fn list(v: &[String]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "[]".into())
}

fn bool_of(v: Option<i64>) -> Option<bool> {
    v.map(|x| x != 0)
}

// ------------------------------------------------------------------ people

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPerson {
    pub id: String,
    pub data: BeneficiaryData,
    pub updated_at: String,
}

const COLUMNS: &str = "id, first_names, last_name_1, last_name_2, birth_date, birth_date_approx, sex, curp, origin_municipality,
    origin_state, indigenous_language, education, literate, attends_school, school_grade, school_lag, group_id, entry_date,
    entry_date_approx, stay_mode, referred_by, admission_reasons, status, status_date, discharge_reason, dependency, mobility,
    disabilities, chronic_conditions, continence, orientation, psych_care, vaccines_up_to_date, visits, legal_status,
    monthly_fee_mxn, fee_payer, programs, consent_date, consent_signer, extra, updated_at";

fn person_of(r: &Row) -> rusqlite::Result<StoredPerson> {
    let json_list = |i: usize| -> rusqlite::Result<Vec<String>> { Ok(serde_json::from_str(&r.get::<_, String>(i)?).unwrap_or_default()) };
    let data = BeneficiaryData {
        first_names: r.get(1)?,
        last_name_1: r.get(2)?,
        last_name_2: r.get(3)?,
        birth_date: r.get(4)?,
        birth_date_approx: r.get::<_, i64>(5)? != 0,
        approx_age: None,
        sex: r.get(6)?,
        curp: r.get(7)?,
        origin_municipality: r.get(8)?,
        origin_state: r.get(9)?,
        indigenous_language: r.get(10)?,
        education: r.get(11)?,
        literate: bool_of(r.get(12)?),
        attends_school: bool_of(r.get(13)?),
        school_grade: r.get(14)?,
        school_lag: bool_of(r.get(15)?),
        group_id: r.get(16)?,
        entry_date: r.get(17)?,
        entry_date_approx: r.get::<_, i64>(18)? != 0,
        stay_mode: r.get(19)?,
        referred_by: r.get(20)?,
        admission_reasons: json_list(21)?,
        status: r.get(22)?,
        status_date: r.get(23)?,
        discharge_reason: r.get(24)?,
        dependency: r.get(25)?,
        mobility: r.get(26)?,
        disabilities: json_list(27)?,
        chronic_conditions: json_list(28)?,
        continence: r.get(29)?,
        orientation: r.get(30)?,
        psych_care: bool_of(r.get(31)?),
        vaccines_up_to_date: bool_of(r.get(32)?),
        contacts: Vec::new(),
        visits: r.get(33)?,
        legal_status: r.get(34)?,
        monthly_fee_mxn: r.get(35)?,
        fee_payer: r.get(36)?,
        programs: json_list(37)?,
        consent_date: r.get(38)?,
        consent_signer: r.get(39)?,
        extra: serde_json::from_str(&r.get::<_, String>(40)?).unwrap_or_default(),
    };
    Ok(StoredPerson { id: r.get(0)?, data, updated_at: r.get(41)? })
}

fn contacts(conn: &Connection, person_id: &str) -> Result<Vec<ResponsibleContact>, CareError> {
    Ok(conn
        .prepare("SELECT full_name, relationship, phone, phone_alt, legal_guardian FROM care_contact WHERE person_id=?1 ORDER BY position")?
        .query_map([person_id], |r| Ok(ResponsibleContact { full_name: r.get(0)?, relationship: r.get(1)?, phone: r.get(2)?, phone_alt: r.get(3)?, legal_guardian: r.get::<_, i64>(4)? != 0 }))?
        .collect::<Result<Vec<_>, _>>()?)
}

/// The records that are not hidden (a deletion that waits hides one).
pub fn people(conn: &Connection) -> Result<Vec<StoredPerson>, CareError> {
    let mut out = conn.prepare(&format!("SELECT {COLUMNS} FROM care_person WHERE hidden = 0 ORDER BY rowid"))?.query_map([], person_of)?.collect::<Result<Vec<_>, _>>()?;
    for p in &mut out {
        p.data.contacts = contacts(conn, &p.id)?;
    }
    Ok(out)
}

/// One record, hidden or not.
pub fn person(conn: &Connection, id: &str) -> Result<Option<StoredPerson>, CareError> {
    let p = conn.query_row(&format!("SELECT {COLUMNS} FROM care_person WHERE id = ?1"), [id], person_of).optional()?;
    Ok(match p {
        Some(mut p) => {
            p.data.contacts = contacts(conn, &p.id)?;
            Some(p)
        }
        None => None,
    })
}

/// Inserts (`id` = `None`, or a given id for the move from the roster) or updates a record. `insert_id` forces the
/// id of a new one.
pub fn save_person(conn: &Connection, id: Option<&str>, insert_id: Option<&str>, d: &BeneficiaryData) -> Result<Option<String>, CareError> {
    let extra = serde_json::to_string(&d.extra).unwrap_or_else(|_| "{}".into());
    let (keep_curp, curp) = match &d.curp {
        None => (true, None),
        Some(s) => (false, Some(s.trim()).filter(|s| !s.is_empty())),
    };
    let b = |v: Option<bool>| v.map(|x| x as i64);
    let pid = match id {
        Some(pid) => {
            let n = conn.execute(
                &format!(
                    "UPDATE care_person SET first_names=?2, last_name_1=?3, last_name_2=?4, birth_date=?5, birth_date_approx=?6, sex=?7,
                        curp = CASE WHEN ?8 THEN curp ELSE ?9 END, origin_municipality=?10, origin_state=?11, indigenous_language=?12,
                        education=?13, literate=?14, attends_school=?15, school_grade=?16, school_lag=?17, group_id=?18, entry_date=?19,
                        entry_date_approx=?20, stay_mode=?21, referred_by=?22, admission_reasons=?23, status=?24, status_date=?25,
                        discharge_reason=?26, dependency=?27, mobility=?28, disabilities=?29, chronic_conditions=?30, continence=?31,
                        orientation=?32, psych_care=?33, vaccines_up_to_date=?34, visits=?35, legal_status=?36, monthly_fee_mxn=?37,
                        fee_payer=?38, programs=?39, consent_date=?40, consent_signer=?41, extra=?42, updated_at={NOW}
                     WHERE id=?1"
                ),
                params![
                    pid, d.first_names.trim(), text(&d.last_name_1), text(&d.last_name_2), text(&d.birth_date), d.birth_date_approx as i64,
                    text(&d.sex), keep_curp, curp, text(&d.origin_municipality), text(&d.origin_state), text(&d.indigenous_language),
                    text(&d.education), b(d.literate), b(d.attends_school), text(&d.school_grade), b(d.school_lag), text(&d.group_id),
                    text(&d.entry_date), d.entry_date_approx as i64, text(&d.stay_mode), text(&d.referred_by), list(&d.admission_reasons),
                    d.status.as_str(), text(&d.status_date), text(&d.discharge_reason), text(&d.dependency), text(&d.mobility),
                    list(&d.disabilities), list(&d.chronic_conditions), text(&d.continence), text(&d.orientation), b(d.psych_care),
                    b(d.vaccines_up_to_date), text(&d.visits), text(&d.legal_status), d.monthly_fee_mxn, text(&d.fee_payer),
                    list(&d.programs), text(&d.consent_date), text(&d.consent_signer), extra
                ],
            )?;
            if n == 0 {
                return Ok(None);
            }
            pid.to_string()
        }
        None => {
            let pid = insert_id.map(String::from).unwrap_or_else(|| new_id("ben"));
            conn.execute(
                &format!(
                    "INSERT INTO care_person (id, first_names, last_name_1, last_name_2, birth_date, birth_date_approx, sex, curp,
                        origin_municipality, origin_state, indigenous_language, education, literate, attends_school, school_grade,
                        school_lag, group_id, entry_date, entry_date_approx, stay_mode, referred_by, admission_reasons, status,
                        status_date, discharge_reason, dependency, mobility, disabilities, chronic_conditions, continence, orientation,
                        psych_care, vaccines_up_to_date, visits, legal_status, monthly_fee_mxn, fee_payer, programs, consent_date,
                        consent_signer, extra, created_at, updated_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,
                             ?30,?31,?32,?33,?34,?35,?36,?37,?38,?39,?40,?41,{NOW},{NOW})"
                ),
                params![
                    pid, d.first_names.trim(), text(&d.last_name_1), text(&d.last_name_2), text(&d.birth_date), d.birth_date_approx as i64,
                    text(&d.sex), curp, text(&d.origin_municipality), text(&d.origin_state), text(&d.indigenous_language),
                    text(&d.education), b(d.literate), b(d.attends_school), text(&d.school_grade), b(d.school_lag), text(&d.group_id),
                    text(&d.entry_date), d.entry_date_approx as i64, text(&d.stay_mode), text(&d.referred_by), list(&d.admission_reasons),
                    d.status.as_str(), text(&d.status_date), text(&d.discharge_reason), text(&d.dependency), text(&d.mobility),
                    list(&d.disabilities), list(&d.chronic_conditions), text(&d.continence), text(&d.orientation), b(d.psych_care),
                    b(d.vaccines_up_to_date), text(&d.visits), text(&d.legal_status), d.monthly_fee_mxn, text(&d.fee_payer),
                    list(&d.programs), text(&d.consent_date), text(&d.consent_signer), extra
                ],
            )?;
            pid
        }
    };
    conn.execute("DELETE FROM care_contact WHERE person_id=?1", [&pid])?;
    for (i, c) in d.contacts.iter().enumerate() {
        conn.execute(
            "INSERT INTO care_contact (id,person_id,position,full_name,relationship,phone,phone_alt,legal_guardian) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![new_id("bct"), pid, i as i64, c.full_name.trim(), text(&c.relationship), text(&c.phone), text(&c.phone_alt), c.legal_guardian as i64],
        )?;
    }
    Ok(Some(pid))
}

pub fn delete_person(conn: &Connection, id: &str) -> Result<bool, CareError> {
    Ok(conn.execute("DELETE FROM care_person WHERE id=?1", [id])? > 0)
}

pub fn set_person_hidden(conn: &Connection, id: &str, hidden: bool) -> Result<bool, CareError> {
    Ok(conn.execute("UPDATE care_person SET hidden=?2 WHERE id=?1", params![id, hidden as i64])? > 0)
}

pub fn curp_of(conn: &Connection, id: &str) -> Result<Option<String>, CareError> {
    Ok(conn.query_row("SELECT curp FROM care_person WHERE id=?1", [id], |r| r.get::<_, Option<String>>(0)).optional()?.flatten())
}

// ------------------------------------------------------------------ groups of their own

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Group {
    pub id: String,
    pub title: String,
    pub active: bool,
}

pub fn groups(conn: &Connection) -> Result<Vec<Group>, CareError> {
    Ok(conn
        .prepare("SELECT id, title, active FROM care_group ORDER BY rowid")?
        .query_map([], |r| Ok(Group { id: r.get(0)?, title: r.get(1)?, active: r.get::<_, i64>(2)? != 0 }))?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn save_group(conn: &Connection, id: Option<&str>, title: &str, active: bool) -> Result<String, CareError> {
    let dup = |e: rusqlite::Error| match &e {
        rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::ConstraintViolation => CareError::DuplicateTitle,
        _ => CareError::Db(e),
    };
    match id {
        Some(gid) => {
            conn.execute("UPDATE care_group SET title=?2, active=?3 WHERE id=?1", params![gid, title.trim(), active as i64]).map_err(dup)?;
            Ok(gid.to_string())
        }
        None => {
            let gid = new_id("grp");
            conn.execute(&format!("INSERT INTO care_group (id,title,active,created_at) VALUES (?1,?2,1,{NOW})"), params![gid, title.trim()]).map_err(dup)?;
            Ok(gid)
        }
    }
}

// ------------------------------------------------------------------ the institution's own fields

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    pub key: String,
    pub title: String,
    pub kind: String,
    pub options: Vec<String>,
    pub position: i64,
}

pub fn custom_fields(conn: &Connection) -> Result<Vec<CustomField>, CareError> {
    Ok(conn
        .prepare("SELECT key, title, kind, options, position FROM care_custom_field WHERE hidden = 0 ORDER BY position")?
        .query_map([], |r| Ok(CustomField { key: r.get(0)?, title: r.get(1)?, kind: r.get(2)?, options: serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default(), position: r.get(4)? }))?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn hidden_field_keys(conn: &Connection) -> Result<Vec<String>, CareError> {
    Ok(conn.prepare("SELECT key FROM care_custom_field WHERE hidden = 1")?.query_map([], |r| r.get(0))?.collect::<Result<Vec<_>, _>>()?)
}

pub fn save_custom_field(conn: &Connection, key: Option<&str>, title: &str, kind: &str, options: &[String]) -> Result<String, CareError> {
    let options = serde_json::to_string(&options.iter().map(|o| o.trim()).filter(|o| !o.is_empty()).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".into());
    match key {
        Some(k) => {
            conn.execute("UPDATE care_custom_field SET title=?2, kind=?3, options=?4 WHERE key=?1", params![k, title.trim(), kind, options])?;
            Ok(k.to_string())
        }
        None => {
            let k = format!("own_{}", Ulid::generate().to_string().to_lowercase());
            let next: i64 = conn.query_row("SELECT coalesce(max(position),0)+1 FROM care_custom_field", [], |r| r.get(0))?;
            conn.execute("INSERT INTO care_custom_field (key,title,kind,options,position) VALUES (?1,?2,?3,?4,?5)", params![k, title.trim(), kind, options, next])?;
            Ok(k)
        }
    }
}

pub fn delete_custom_field(conn: &Connection, key: &str) -> Result<(), CareError> {
    conn.execute("DELETE FROM care_custom_field WHERE key=?1", [key])?;
    conn.execute("UPDATE care_person SET extra = json_remove(extra, '$.' || json_quote(?1)) WHERE json_extract(extra, '$.' || json_quote(?1)) IS NOT NULL", [key])?;
    Ok(())
}

pub fn set_field_hidden(conn: &Connection, key: &str, hidden: bool) -> Result<bool, CareError> {
    Ok(conn.execute("UPDATE care_custom_field SET hidden=?2 WHERE key=?1", params![key, hidden as i64])? > 0)
}

pub fn field_title(conn: &Connection, key: &str) -> Result<Option<String>, CareError> {
    Ok(conn.query_row("SELECT title FROM care_custom_field WHERE key=?1", [key], |r| r.get(0)).optional()?)
}

// ------------------------------------------------------------------ the waiting list

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WaitlistRow {
    pub id: String,
    #[serde(flatten)]
    pub input: WaitlistInput,
    pub person_id: Option<String>,
}

pub fn waitlist(conn: &Connection) -> Result<Vec<WaitlistRow>, CareError> {
    Ok(conn
        .prepare("SELECT id, requested_on, name, phone, sex, approx_age, dependency, reason, status, person_id FROM care_waitlist ORDER BY requested_on, rowid")?
        .query_map([], |r| {
            Ok(WaitlistRow {
                id: r.get(0)?,
                input: WaitlistInput { requested_on: r.get(1)?, name: r.get(2)?, phone: r.get(3)?, sex: r.get(4)?, approx_age: r.get(5)?, dependency: r.get(6)?, reason: r.get(7)?, status: r.get(8)? },
                person_id: r.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn save_waitlist(conn: &Connection, id: Option<&str>, w: &WaitlistInput) -> Result<bool, CareError> {
    let p = params![id.map(String::from).unwrap_or_else(|| new_id("wait")), w.requested_on, text(&w.name), text(&w.phone), text(&w.sex), w.approx_age, text(&w.dependency), text(&w.reason), w.status];
    let n = match id {
        Some(_) => conn.execute(&format!("UPDATE care_waitlist SET requested_on=?2, name=?3, phone=?4, sex=?5, approx_age=?6, dependency=?7, reason=?8, status=?9, updated_at={NOW} WHERE id=?1"), p)?,
        None => conn.execute(&format!("INSERT INTO care_waitlist (id,requested_on,name,phone,sex,approx_age,dependency,reason,status,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,{NOW},{NOW})"), p)?,
    };
    Ok(n > 0)
}

pub fn admit_waitlist(conn: &Connection, id: &str, person_id: &str) -> Result<bool, CareError> {
    Ok(conn.execute(&format!("UPDATE care_waitlist SET status='admitted', person_id=?2, updated_at={NOW} WHERE id=?1 AND status='waiting'"), params![id, person_id])? > 0)
}

pub fn delete_waitlist(conn: &Connection, id: &str) -> Result<bool, CareError> {
    Ok(conn.execute("DELETE FROM care_waitlist WHERE id=?1", [id])? > 0)
}
