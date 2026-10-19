//! The move of the staff of the old roster (ADR-020) into the module (ADR-027). Each old record becomes a person
//! with a job: the role becomes a position of the catalog, the contract a modality, the start year a date (the
//! 1st of January, marked as approximate), and the institution's own fields keep their key and title.

use super::domain::catalog::{self, Flavor};
use super::domain::person::PersonData;
use super::domain::position::PositionInput;
use super::storage as store;
use super::HrError;
use rusqlite::{params, Connection};
use std::collections::BTreeMap;

/// A field of their own of the old form.
pub struct LegacyField {
    pub key: String,
    pub title: String,
    /// The old kind: `text`, `select`, `number`, `money`, `year`, `email`, `phone` or `yesno`.
    pub kind: String,
    pub options: Vec<String>,
}

const PARTICLES: &[&str] = &["de", "del", "la", "las", "los", "y", "san", "santa"];

/// Names and the two surnames from a full name, the Mexican way: the last two words are the surnames, with the
/// particles that come before each one («de la Cruz»); there is always at least one name left.
pub fn split_name(full: &str) -> (String, Option<String>, Option<String>) {
    let words: Vec<&str> = full.split_whitespace().collect();
    let take = |end: usize| -> usize {
        // where the surname that ends at `end` starts, with its particles
        let mut start = end - 1;
        while start > 0 && PARTICLES.contains(&words[start - 1].to_lowercase().as_str()) {
            start -= 1;
        }
        start
    };
    if words.len() < 2 {
        return (full.trim().to_string(), None, None);
    }
    let second = take(words.len());
    if second == 0 {
        return (words.join(" "), None, None);
    }
    if second == 1 {
        return (words[0].to_string(), Some(words[1..].join(" ")), None);
    }
    let first = take(second);
    if first == 0 {
        return (words[..second].join(" "), Some(words[second..].join(" ")), None);
    }
    (words[..first].join(" "), Some(words[first..second].join(" ")), Some(words[second..].join(" ")))
}

fn shift_and_schedule(label: &str) -> (Option<&'static str>, Option<&'static str>) {
    let s = label.trim().to_lowercase();
    match s.as_str() {
        "matutino" => (Some("morning"), None),
        "vespertino" => (Some("afternoon"), None),
        "nocturno" => (Some("night"), None),
        "por turnos" => (Some("rotating"), None),
        "24 horas" => (Some("h24"), None),
        "medio tiempo" => (None, Some("part_time")),
        "fines de semana" => (None, Some("weekends")),
        _ => (None, None),
    }
}

fn own_kind(old: &str) -> &'static str {
    match old {
        "select" => "select",
        "number" | "money" | "year" => "number",
        _ => "text",
    }
}

/// Moves the old staff records. Returns how many people came in. Runs inside the caller's transaction.
pub fn import(conn: &Connection, flavor: Flavor, rows: &[BTreeMap<String, String>], own_fields: &[LegacyField]) -> Result<usize, HrError> {
    store::seed_positions(conn, &catalog::default_positions(flavor))?;
    for (i, f) in own_fields.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO hr_custom_field (key,title,kind,options,position) VALUES (?1,?2,?3,?4,?5)",
            params![f.key, f.title, own_kind(&f.kind), serde_json::to_string(&f.options).unwrap_or_else(|_| "[]".into()), i as i64 + 1],
        )?;
    }
    let get = |r: &BTreeMap<String, String>, k: &str| r.get(k).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    for r in rows {
        let (first_names, last_name_1, last_name_2) = split_name(&get(r, "full_name").unwrap_or_else(|| "Sin nombre".into()));
        let role = get(r, "role").unwrap_or_else(|| "Sin puesto".into());
        let position_id = match store::position_by_title(conn, &role)? {
            Some(id) => id,
            None => store::insert_position(conn, &PositionInput { title: role.clone(), ..Default::default() })?,
        };
        let paid = get(r, "paid").as_deref() != Some("no");
        let from_contract = get(r, "contract").as_deref().and_then(catalog::modality_from_label);
        let modality = match (paid, from_contract) {
            (_, Some(m @ ("religious" | "volunteer" | "social_service" | "external"))) => m,
            (false, _) => "volunteer",
            (true, Some(m)) => m,
            (true, None) => "indefinite",
        };
        let (shift, schedule) = get(r, "shift").map_or((None, None), |s| shift_and_schedule(&s));
        let start_year = get(r, "start_year").and_then(|y| y.parse::<i64>().ok()).filter(|y| (1900..=2100).contains(y));
        let mut extra: BTreeMap<String, String> =
            own_fields.iter().filter_map(|f| get(r, &f.key).map(|v| (f.key.clone(), v))).collect();
        if let (None, None, Some(s)) = (shift, schedule, get(r, "shift")) {
            // a schedule the catalog does not know is kept as the institution wrote it
            conn.execute("INSERT OR IGNORE INTO hr_custom_field (key,title,kind,options,position) VALUES ('old_shift','Horario (anterior)','text','[]',0)", [])?;
            extra.insert("old_shift".into(), s);
        }
        let data = PersonData {
            first_names,
            last_name_1,
            last_name_2,
            phone: get(r, "phone"),
            email: get(r, "email"),
            position_id: Some(position_id),
            modality: modality.into(),
            start_date: start_year.map(|y| format!("{y}-01-01")),
            schedule: schedule.map(String::from),
            shift: shift.map(String::from),
            status: "active".into(),
            pay_amount_mxn: if modality == "volunteer" { None } else { get(r, "monthly_salary_mxn").and_then(|v| v.parse().ok()) },
            pay_period: Some("monthly".into()),
            extra,
            ..Default::default()
        };
        store::save_person(conn, None, &data, start_year.is_some())?;
    }
    Ok(rows.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_split_the_mexican_way() {
        let s = |n: &str| split_name(n);
        assert_eq!(s("Carmen Olivia Salazar Rojas"), ("Carmen Olivia".into(), Some("Salazar".into()), Some("Rojas".into())));
        assert_eq!(s("María de la Luz Pérez García"), ("María de la Luz".into(), Some("Pérez".into()), Some("García".into())));
        assert_eq!(s("Juan Pérez de la Cruz"), ("Juan".into(), Some("Pérez".into()), Some("de la Cruz".into())));
        assert_eq!(s("Juan de la Cruz"), ("Juan".into(), Some("de la Cruz".into()), None));
        assert_eq!(s("Ana López"), ("Ana".into(), Some("López".into()), None));
        assert_eq!(s("Rosario"), ("Rosario".into(), None, None));
    }
}
