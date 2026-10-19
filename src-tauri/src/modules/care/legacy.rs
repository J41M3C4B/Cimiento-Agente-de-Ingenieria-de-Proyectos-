//! The move of the people served of the old roster (ADR-020) into this module (ADR-029). Each old record keeps its
//! id (so a deletion that was waiting still finds it): the group becomes the sex, the age an approximate birth date,
//! the entry year an approximate entry date, the phone and mail of contact a responsible person, and the
//! institution's own fields keep their key and title.

use super::domain::person::{BeneficiaryData, ResponsibleContact};
use super::storage as store;
use super::CareError;
use rusqlite::{params, Connection};
use std::collections::BTreeMap;

pub struct LegacyField {
    pub key: String,
    pub title: String,
    /// The old kind: `text`, `select`, `number`, `money`, `year`, `email`, `phone` or `yesno`.
    pub kind: String,
    pub options: Vec<String>,
}

pub struct LegacyRow {
    pub id: String,
    pub data: BTreeMap<String, String>,
    pub hidden: bool,
}

/// The sex an old group name says: «Niñas», «Mujeres adultas mayores» -> female.
fn sex_of(group: &str) -> Option<&'static str> {
    let g = group.to_lowercase();
    if ["niña", "mujer", "femen", "señora", "adolescentes mujeres"].iter().any(|w| g.contains(w)) {
        Some("female")
    } else if ["niño", "hombre", "mascul", "señor", "adolescentes hombres"].iter().any(|w| g.contains(w)) {
        Some("male")
    } else {
        None
    }
}

/// Moves the old records, inside the caller's transaction. `year` is the current year (for the ages).
pub fn import(conn: &Connection, year: i64, rows: &[LegacyRow], own_fields: &[LegacyField]) -> Result<usize, CareError> {
    for (i, f) in own_fields.iter().enumerate() {
        let kind = match f.kind.as_str() {
            "select" => "select",
            "number" | "money" | "year" => "number",
            _ => "text",
        };
        conn.execute(
            "INSERT OR IGNORE INTO care_custom_field (key,title,kind,options,position) VALUES (?1,?2,?3,?4,?5)",
            params![f.key, f.title, kind, serde_json::to_string(&f.options).unwrap_or_else(|_| "[]".into()), i as i64 + 1],
        )?;
    }
    let get = |r: &BTreeMap<String, String>, k: &str| r.get(k).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    for row in rows {
        let r = &row.data;
        let full = get(r, "full_name").unwrap_or_else(|| "Sin nombre".into());
        let mut words = full.split_whitespace();
        let first = words.next().unwrap_or("Sin").to_string();
        let rest: Vec<&str> = words.collect();
        let age = get(r, "age").and_then(|a| a.parse::<i64>().ok()).filter(|a| (0..=120).contains(a));
        let entry = get(r, "entry_year").and_then(|y| y.parse::<i64>().ok()).filter(|y| (1900..=2100).contains(y));
        let group = get(r, "category").unwrap_or_default();
        let contact = (get(r, "phone").is_some() || get(r, "email").is_some()).then(|| ResponsibleContact {
            full_name: "Contacto (del registro anterior)".into(),
            phone: get(r, "phone"),
            ..Default::default()
        });
        let mut extra: BTreeMap<String, String> = own_fields.iter().filter_map(|f| get(r, &f.key).map(|v| (f.key.clone(), v))).collect();
        if let Some(mail) = get(r, "email") {
            conn.execute("INSERT OR IGNORE INTO care_custom_field (key,title,kind,options,position) VALUES ('old_email','Correo de contacto (anterior)','text','[]',0)", [])?;
            extra.insert("old_email".into(), mail);
        }
        let fee = get(r, "monthly_fee_mxn").and_then(|v| v.parse::<i64>().ok());
        let data = BeneficiaryData {
            // a name of one word stays whole; the rest are the surnames
            first_names: if rest.len() >= 2 { [first.as_str()].iter().chain(rest[..rest.len() - 2].iter()).copied().collect::<Vec<_>>().join(" ") } else { first.clone() },
            last_name_1: match rest.len() {
                0 => None,
                1 => Some(rest[0].to_string()),
                n => Some(rest[n - 2].to_string()),
            },
            last_name_2: (rest.len() >= 2).then(|| rest[rest.len() - 1].to_string()),
            birth_date: age.map(|a| format!("{}-01-01", year - a)),
            birth_date_approx: age.is_some(),
            sex: sex_of(&group).map(String::from),
            entry_date: entry.map(|y| format!("{y}-01-01")),
            entry_date_approx: entry.is_some(),
            status: "active".into(),
            dependency: get(r, "dependency"),
            monthly_fee_mxn: fee,
            fee_payer: fee.map(|_| "family".into()),
            contacts: contact.into_iter().collect(),
            extra,
            ..Default::default()
        };
        store::save_person(conn, None, Some(&row.id), &data)?;
        if row.hidden {
            store::set_person_hidden(conn, &row.id, true)?;
        }
    }
    Ok(rows.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_group_names_say_the_sex() {
        assert_eq!(sex_of("Mujeres adultas mayores"), Some("female"));
        assert_eq!(sex_of("Niños"), Some("male"));
        assert_eq!(sex_of("Adolescentes mujeres"), Some("female"));
        assert_eq!(sex_of("Personas atendidas"), None);
    }
}
