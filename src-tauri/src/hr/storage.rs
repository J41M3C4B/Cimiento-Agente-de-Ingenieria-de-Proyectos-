//! Persistence of the staff module: its own `hr_*` tables and nothing else.

use super::domain::catalog;
use super::domain::person::{Address, EmergencyContact, PersonData};
use super::domain::position::{Position, PositionInput};
use super::HrError;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Ulid::generate())
}

/// Today, from the clock of the database (`YYYY-MM-DD`).
pub fn today(conn: &Connection) -> Result<String, HrError> {
    Ok(conn.query_row("SELECT strftime('%Y-%m-%d','now')", [], |r| r.get(0))?)
}

fn text(o: &Option<String>) -> Option<&str> {
    o.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

// ------------------------------------------------------------------ positions

fn position_of(r: &Row) -> rusqlite::Result<Position> {
    Ok(Position {
        id: r.get(0)?,
        input: PositionInput {
            title: r.get(1)?,
            area: r.get(2)?,
            duties: r.get(3)?,
            default_modality: r.get(4)?,
            default_schedule: r.get(5)?,
            reference_pay_mxn: r.get(6)?,
            authorized_seats: r.get(7)?,
            reports_to: r.get(8)?,
        },
        active: r.get::<_, i64>(9)? != 0,
    })
}

pub fn positions(conn: &Connection) -> Result<Vec<Position>, HrError> {
    Ok(conn
        .prepare("SELECT id,title,area,duties,default_modality,default_schedule,reference_pay_mxn,authorized_seats,reports_to,active FROM hr_position ORDER BY rowid")?
        .query_map([], position_of)?
        .collect::<Result<Vec<_>, _>>()?)
}

/// The id of the position with this title (case does not matter), if there is one.
pub fn position_by_title(conn: &Connection, title: &str) -> Result<Option<String>, HrError> {
    Ok(conn.query_row("SELECT id FROM hr_position WHERE lower(title) = lower(?1)", [title.trim()], |r| r.get(0)).optional()?)
}

/// Writes the suggested positions the first time the catalog is used.
pub fn seed_positions(conn: &Connection, defaults: &[(&str, &str)]) -> Result<(), HrError> {
    let n: i64 = conn.query_row("SELECT count(*) FROM hr_position", [], |r| r.get(0))?;
    if n > 0 {
        return Ok(());
    }
    for (title, area) in defaults {
        insert_position(conn, &PositionInput { title: (*title).into(), area: Some((*area).into()), ..Default::default() })?;
    }
    Ok(())
}

pub fn insert_position(conn: &Connection, p: &PositionInput) -> Result<String, HrError> {
    let id = new_id("pos");
    conn.execute(
        &format!(
            "INSERT INTO hr_position (id,title,area,duties,default_modality,default_schedule,reference_pay_mxn,authorized_seats,reports_to,active,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,1,{NOW},{NOW})"
        ),
        params![id, p.title.trim(), text(&p.area), text(&p.duties), text(&p.default_modality), text(&p.default_schedule), p.reference_pay_mxn, p.authorized_seats, text(&p.reports_to)],
    )
    .map_err(|e| duplicate(e))?;
    Ok(id)
}

pub fn update_position(conn: &Connection, id: &str, p: &PositionInput) -> Result<bool, HrError> {
    let n = conn
        .execute(
            &format!(
                "UPDATE hr_position SET title=?2, area=?3, duties=?4, default_modality=?5, default_schedule=?6, reference_pay_mxn=?7,
                        authorized_seats=?8, reports_to=?9, updated_at={NOW} WHERE id=?1"
            ),
            params![id, p.title.trim(), text(&p.area), text(&p.duties), text(&p.default_modality), text(&p.default_schedule), p.reference_pay_mxn, p.authorized_seats, text(&p.reports_to)],
        )
        .map_err(|e| duplicate(e))?;
    Ok(n > 0)
}

/// A position in use is never deleted: it is archived, and it can come back.
pub fn set_position_active(conn: &Connection, id: &str, active: bool) -> Result<bool, HrError> {
    Ok(conn.execute(&format!("UPDATE hr_position SET active=?2, updated_at={NOW} WHERE id=?1"), params![id, active as i64])? > 0)
}

fn duplicate(e: rusqlite::Error) -> HrError {
    match &e {
        rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::ConstraintViolation => HrError::DuplicateTitle,
        _ => HrError::Db(e),
    }
}

// ------------------------------------------------------------------ modalities

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CustomModality {
    pub code: String,
    pub title: String,
    pub behaves_as: String,
}

pub fn custom_modalities(conn: &Connection) -> Result<Vec<CustomModality>, HrError> {
    Ok(conn
        .prepare("SELECT code, title, behaves_as FROM hr_modality ORDER BY rowid")?
        .query_map([], |r| Ok(CustomModality { code: r.get(0)?, title: r.get(1)?, behaves_as: r.get(2)? }))?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn insert_modality(conn: &Connection, title: &str, behaves_as: &str) -> Result<String, HrError> {
    let code = format!("custom_{}", Ulid::generate().to_string().to_lowercase());
    conn.execute(&format!("INSERT INTO hr_modality (code,title,behaves_as,created_at) VALUES (?1,?2,?3,{NOW})"), params![code, title.trim(), behaves_as])?;
    Ok(code)
}

/// The built-in modality whose rules a code follows (itself, or what an institution's own one behaves as).
pub fn base_of(conn: &Connection, code: &str) -> Result<Option<&'static str>, HrError> {
    if let Some((c, _)) = catalog::BUILTIN_MODALITIES.iter().find(|(c, _)| *c == code) {
        return Ok(Some(c));
    }
    let behaves: Option<String> = conn.query_row("SELECT behaves_as FROM hr_modality WHERE code=?1", [code], |r| r.get(0)).optional()?;
    Ok(behaves.and_then(|b| catalog::BUILTIN_MODALITIES.iter().find(|(c, _)| *c == b).map(|(c, _)| *c)))
}

// ------------------------------------------------------------------ people

/// A stored record: the data with the identifiers in clear (they never leave the service like this).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPerson {
    pub id: String,
    pub data: PersonData,
    pub start_date_approx: bool,
    pub updated_at: String,
}

const PERSON_COLUMNS: &str = "p.id, p.first_names, p.last_name_1, p.last_name_2, p.birth_date, p.sex, p.curp, p.marital_status, p.nationality,
    p.education, p.professional_license, p.address_street, p.address_number, p.address_neighborhood, p.address_municipality,
    p.address_state, p.address_zip, p.phone, p.email, p.bank, p.clabe, p.rfc, p.nss, p.tax_regime, p.tax_zip, p.infonavit_credit,
    p.extra, p.updated_at, j.position_id, j.modality, j.start_date, j.start_date_approx, j.end_date, j.schedule, j.shift,
    j.work_days, j.weekly_hours, j.status, j.left_date, j.left_reason, j.pay_amount_mxn, j.pay_period, j.pay_method";

fn person_of(r: &Row) -> rusqlite::Result<StoredPerson> {
    let data = PersonData {
        first_names: r.get(1)?,
        last_name_1: r.get(2)?,
        last_name_2: r.get(3)?,
        birth_date: r.get(4)?,
        sex: r.get(5)?,
        curp: r.get(6)?,
        marital_status: r.get(7)?,
        nationality: r.get(8)?,
        education: r.get(9)?,
        professional_license: r.get(10)?,
        address: Address { street: r.get(11)?, number: r.get(12)?, neighborhood: r.get(13)?, municipality: r.get(14)?, state: r.get(15)?, zip: r.get(16)? },
        phone: r.get(17)?,
        email: r.get(18)?,
        bank: r.get(19)?,
        clabe: r.get(20)?,
        rfc: r.get(21)?,
        nss: r.get(22)?,
        tax_regime: r.get(23)?,
        tax_zip: r.get(24)?,
        infonavit_credit: r.get::<_, Option<i64>>(25)?.map(|v| v != 0),
        extra: serde_json::from_str(&r.get::<_, String>(26)?).unwrap_or_default(),
        position_id: r.get(28)?,
        modality: r.get(29)?,
        start_date: r.get(30)?,
        end_date: r.get(32)?,
        schedule: r.get(33)?,
        shift: r.get(34)?,
        work_days: serde_json::from_str(&r.get::<_, String>(35)?).unwrap_or_default(),
        weekly_hours: r.get(36)?,
        status: r.get(37)?,
        left_date: r.get(38)?,
        left_reason: r.get(39)?,
        pay_amount_mxn: r.get(40)?,
        pay_period: r.get(41)?,
        pay_method: r.get(42)?,
        emergency_contacts: Vec::new(),
    };
    Ok(StoredPerson { id: r.get(0)?, data, start_date_approx: r.get::<_, i64>(31)? != 0, updated_at: r.get(27)? })
}

fn contacts(conn: &Connection, person_id: &str) -> Result<Vec<EmergencyContact>, HrError> {
    Ok(conn
        .prepare("SELECT full_name, relationship, phone, phone_alt FROM hr_emergency_contact WHERE person_id=?1 ORDER BY position")?
        .query_map([person_id], |r| Ok(EmergencyContact { full_name: r.get(0)?, relationship: r.get(1)?, phone: r.get(2)?, phone_alt: r.get(3)? }))?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn people(conn: &Connection) -> Result<Vec<StoredPerson>, HrError> {
    let mut out = conn
        .prepare(&format!("SELECT {PERSON_COLUMNS} FROM hr_person p JOIN hr_job j ON j.person_id = p.id AND j.current = 1 ORDER BY p.rowid"))?
        .query_map([], person_of)?
        .collect::<Result<Vec<_>, _>>()?;
    for p in &mut out {
        p.data.emergency_contacts = contacts(conn, &p.id)?;
    }
    Ok(out)
}

pub fn person(conn: &Connection, id: &str) -> Result<Option<StoredPerson>, HrError> {
    let p = conn
        .query_row(&format!("SELECT {PERSON_COLUMNS} FROM hr_person p JOIN hr_job j ON j.person_id = p.id AND j.current = 1 WHERE p.id = ?1"), [id], person_of)
        .optional()?;
    Ok(match p {
        Some(mut p) => {
            p.data.emergency_contacts = contacts(conn, &p.id)?;
            Some(p)
        }
        None => None,
    })
}

/// A covered identifier as the database takes it: `None` keeps what is stored, `""` clears it.
fn secret(o: &Option<String>) -> (bool, Option<&str>) {
    match o {
        None => (true, None),
        Some(s) => (false, Some(s.trim()).filter(|s| !s.is_empty())),
    }
}

/// Inserts (`id` = `None`) or updates a record with its current job and contacts. Returns its id.
pub fn save_person(conn: &Connection, id: Option<&str>, d: &PersonData, start_date_approx: bool) -> Result<Option<String>, HrError> {
    let a = &d.address;
    let extra = serde_json::to_string(&d.extra).unwrap_or_else(|_| "{}".into());
    let (keep_curp, curp) = secret(&d.curp);
    let (keep_rfc, rfc) = secret(&d.rfc);
    let (keep_nss, nss) = secret(&d.nss);
    let (keep_clabe, clabe) = secret(&d.clabe);
    let pid = match id {
        Some(pid) => {
            let n = conn.execute(
                &format!(
                    "UPDATE hr_person SET first_names=?2, last_name_1=?3, last_name_2=?4, birth_date=?5, sex=?6,
                        curp = CASE WHEN ?7 THEN curp ELSE ?8 END, marital_status=?9, nationality=?10, education=?11,
                        professional_license=?12, address_street=?13, address_number=?14, address_neighborhood=?15,
                        address_municipality=?16, address_state=?17, address_zip=?18, phone=?19, email=?20, bank=?21,
                        clabe = CASE WHEN ?22 THEN clabe ELSE ?23 END, rfc = CASE WHEN ?24 THEN rfc ELSE ?25 END,
                        nss = CASE WHEN ?26 THEN nss ELSE ?27 END, tax_regime=?28, tax_zip=?29, infonavit_credit=?30,
                        extra=?31, updated_at={NOW}
                     WHERE id=?1"
                ),
                params![
                    pid, d.first_names.trim(), text(&d.last_name_1), text(&d.last_name_2), text(&d.birth_date), text(&d.sex),
                    keep_curp, curp, text(&d.marital_status), text(&d.nationality), text(&d.education),
                    text(&d.professional_license), text(&a.street), text(&a.number), text(&a.neighborhood),
                    text(&a.municipality), text(&a.state), text(&a.zip), text(&d.phone), text(&d.email), text(&d.bank),
                    keep_clabe, clabe, keep_rfc, rfc, keep_nss, nss, text(&d.tax_regime), text(&d.tax_zip),
                    d.infonavit_credit.map(|b| b as i64), extra
                ],
            )?;
            if n == 0 {
                return Ok(None);
            }
            pid.to_string()
        }
        None => {
            let pid = new_id("per");
            conn.execute(
                &format!(
                    "INSERT INTO hr_person (id,first_names,last_name_1,last_name_2,birth_date,sex,curp,marital_status,nationality,
                        education,professional_license,address_street,address_number,address_neighborhood,address_municipality,
                        address_state,address_zip,phone,email,bank,clabe,rfc,nss,tax_regime,tax_zip,infonavit_credit,extra,
                        created_at,updated_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,{NOW},{NOW})"
                ),
                params![
                    pid, d.first_names.trim(), text(&d.last_name_1), text(&d.last_name_2), text(&d.birth_date), text(&d.sex),
                    curp, text(&d.marital_status), text(&d.nationality), text(&d.education), text(&d.professional_license),
                    text(&a.street), text(&a.number), text(&a.neighborhood), text(&a.municipality), text(&a.state), text(&a.zip),
                    text(&d.phone), text(&d.email), text(&d.bank), clabe, rfc, nss, text(&d.tax_regime), text(&d.tax_zip),
                    d.infonavit_credit.map(|b| b as i64), extra
                ],
            )?;
            pid
        }
    };
    let work_days = serde_json::to_string(&d.work_days).unwrap_or_else(|_| "[]".into());
    let job = params![
        pid, text(&d.position_id), d.modality.trim(), text(&d.start_date), start_date_approx as i64, text(&d.end_date),
        text(&d.schedule), text(&d.shift), work_days, d.weekly_hours, d.status.as_str(), text(&d.left_date), text(&d.left_reason),
        d.pay_amount_mxn, text(&d.pay_period), text(&d.pay_method)
    ];
    let updated = conn.execute(
        &format!(
            "UPDATE hr_job SET position_id=?2, modality=?3, start_date=?4, start_date_approx=?5, end_date=?6, schedule=?7, shift=?8,
                work_days=?9, weekly_hours=?10, status=?11, left_date=?12, left_reason=?13, pay_amount_mxn=?14, pay_period=?15,
                pay_method=?16, updated_at={NOW}
             WHERE person_id=?1 AND current=1"
        ),
        job,
    )?;
    if updated == 0 {
        conn.execute(
            &format!(
                "INSERT INTO hr_job (id,person_id,position_id,modality,start_date,start_date_approx,end_date,schedule,shift,work_days,
                    weekly_hours,status,left_date,left_reason,pay_amount_mxn,pay_period,pay_method,current,created_at,updated_at)
                 VALUES ('{}',?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,1,{NOW},{NOW})",
                new_id("job")
            ),
            job,
        )?;
    }
    conn.execute("DELETE FROM hr_emergency_contact WHERE person_id=?1", [&pid])?;
    for (i, c) in d.emergency_contacts.iter().enumerate() {
        conn.execute(
            "INSERT INTO hr_emergency_contact (id,person_id,position,full_name,relationship,phone,phone_alt) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![new_id("ice"), pid, i as i64, c.full_name.trim(), text(&c.relationship), text(&c.phone), text(&c.phone_alt)],
        )?;
    }
    Ok(Some(pid))
}

/// Deletes everything of a person: record, jobs and contacts.
pub fn delete_person(conn: &Connection, id: &str) -> Result<bool, HrError> {
    Ok(conn.execute("DELETE FROM hr_person WHERE id=?1", [id])? > 0)
}

pub fn count_in_position(conn: &Connection, position_id: &str) -> Result<i64, HrError> {
    Ok(conn.query_row("SELECT count(*) FROM hr_job WHERE position_id=?1 AND current=1 AND status <> 'left'", [position_id], |r| r.get(0))?)
}

// ------------------------------------------------------------------ the institution's own fields

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    pub key: String,
    pub title: String,
    /// `text`, `select` or `number`.
    pub kind: String,
    pub options: Vec<String>,
    pub position: i64,
}

pub fn custom_fields(conn: &Connection) -> Result<Vec<CustomField>, HrError> {
    Ok(conn
        .prepare("SELECT key, title, kind, options, position FROM hr_custom_field ORDER BY position")?
        .query_map([], |r| {
            Ok(CustomField { key: r.get(0)?, title: r.get(1)?, kind: r.get(2)?, options: serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default(), position: r.get(4)? })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

/// Adds (`key` = `None`) or edits a field of the institution's own. Returns the key.
pub fn save_custom_field(conn: &Connection, key: Option<&str>, title: &str, kind: &str, options: &[String]) -> Result<String, HrError> {
    let options = serde_json::to_string(&options.iter().map(|o| o.trim()).filter(|o| !o.is_empty()).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".into());
    match key {
        Some(k) => {
            conn.execute("UPDATE hr_custom_field SET title=?2, kind=?3, options=?4 WHERE key=?1", params![k, title.trim(), kind, options])?;
            Ok(k.to_string())
        }
        None => {
            let k = format!("own_{}", Ulid::generate().to_string().to_lowercase());
            let next: i64 = conn.query_row("SELECT coalesce(max(position),0)+1 FROM hr_custom_field", [], |r| r.get(0))?;
            conn.execute("INSERT INTO hr_custom_field (key,title,kind,options,position) VALUES (?1,?2,?3,?4,?5)", params![k, title.trim(), kind, options, next])?;
            Ok(k)
        }
    }
}

/// Removes a field of their own and its value from every record.
pub fn delete_custom_field(conn: &Connection, key: &str) -> Result<(), HrError> {
    conn.execute("DELETE FROM hr_custom_field WHERE key=?1", [key])?;
    conn.execute("UPDATE hr_person SET extra = json_remove(extra, '$.' || json_quote(?1)) WHERE json_extract(extra, '$.' || json_quote(?1)) IS NOT NULL", [key])?;
    Ok(())
}

/// The covered identifiers a record has stored, by name.
pub fn secret_value(conn: &Connection, id: &str, field: &str) -> Result<Option<String>, HrError> {
    let column = match field {
        "curp" | "rfc" | "nss" | "clabe" => field,
        _ => return Err(HrError::UnknownField),
    };
    Ok(conn.query_row(&format!("SELECT {column} FROM hr_person WHERE id=?1"), [id], |r| r.get::<_, Option<String>>(0)).optional()?.flatten())
}
