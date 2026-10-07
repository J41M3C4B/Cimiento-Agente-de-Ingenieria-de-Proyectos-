//! The roster (ADR-020): one record per staff member and per person served, with a form the person can extend.
//! It lives apart from the profile and never goes to the AI: the profile only receives the aggregates
//! computed here (`derive_staff`, `derive_population`).

use super::profile::{ContractKind, DependencyLevel, InstitutionKind, PopulationGroupInput, StaffGroupInput};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entity {
    Staff,
    Beneficiary,
}

impl Entity {
    pub fn as_db(self) -> &'static str {
        match self {
            Entity::Staff => "staff",
            Entity::Beneficiary => "beneficiary",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    Text,
    Select,
    Number,
    Money,
    Year,
    Email,
    Phone,
    /// `snake_case` would write `yes_no`; the app and the database say `yesno`.
    #[serde(rename = "yesno")]
    YesNo,
}

impl FieldKind {
    pub fn as_db(self) -> &'static str {
        match self {
            FieldKind::Text => "text",
            FieldKind::Select => "select",
            FieldKind::Number => "number",
            FieldKind::Money => "money",
            FieldKind::Year => "year",
            FieldKind::Email => "email",
            FieldKind::Phone => "phone",
            FieldKind::YesNo => "yesno",
        }
    }
    pub fn from_db(s: &str) -> Option<Self> {
        Some(match s {
            "text" => FieldKind::Text,
            "select" => FieldKind::Select,
            "number" => FieldKind::Number,
            "money" => FieldKind::Money,
            "year" => FieldKind::Year,
            "email" => FieldKind::Email,
            "phone" => FieldKind::Phone,
            "yesno" => FieldKind::YesNo,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldOption {
    pub value: String,
    pub label: String,
}

/// A field of the form. The keys of the built-in fields are the ones the app computes with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterField {
    pub key: String,
    pub title: String,
    pub kind: FieldKind,
    pub options: Vec<FieldOption>,
    pub builtin: bool,
    pub locked_options: bool,
    pub required: bool,
    pub position: i64,
}

/// The values of a record by field key. Everything is text; numbers are read when they are needed.
pub type Data = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterEntry {
    pub id: String,
    pub data: Data,
}

/// Keys of the built-in fields the app computes with.
pub mod key {
    pub const FULL_NAME: &str = "full_name";
    pub const ROLE: &str = "role";
    pub const PAID: &str = "paid";
    pub const SALARY: &str = "monthly_salary_mxn";
    pub const CATEGORY: &str = "category";
    pub const AGE: &str = "age";
    pub const DEPENDENCY: &str = "dependency";
    pub const FEE: &str = "monthly_fee_mxn";
    pub const CONTRACT: &str = "contract";
    pub const START_YEAR: &str = "start_year";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterError {
    Required,
    NotANumber,
    YearInvalid,
    EmailInvalid,
}

impl RosterError {
    pub fn code(self) -> &'static str {
        match self {
            RosterError::Required => "required",
            RosterError::NotANumber => "not_a_number",
            RosterError::YearInvalid => "year_invalid",
            RosterError::EmailInvalid => "email_invalid",
        }
    }
}

fn labels(items: &[&str]) -> Vec<FieldOption> {
    items.iter().map(|s| FieldOption { value: (*s).into(), label: (*s).into() }).collect()
}

fn dependency_options() -> Vec<FieldOption> {
    [("low", "Poco apoyo"), ("medium", "Apoyo regular"), ("high", "Mucho apoyo"), ("total", "Apoyo en todo")]
        .iter()
        .map(|(v, l)| FieldOption { value: (*v).into(), label: (*l).into() })
        .collect()
}

/// The fields a form starts with. The categories of the people served depend on the kind of institution.
pub fn default_fields(entity: Entity, institution: InstitutionKind) -> Vec<RosterField> {
    let mut n = 0;
    let mut f = |key: &str, title: &str, kind: FieldKind, options: Vec<FieldOption>, required: bool, locked: bool| {
        n += 1;
        RosterField {
            key: key.into(),
            title: title.into(),
            kind,
            options,
            builtin: true,
            locked_options: locked,
            required,
            position: n,
        }
    };
    use FieldKind::*;
    match entity {
        Entity::Staff => vec![
            f(key::FULL_NAME, "Nombre completo", Text, vec![], true, false),
            f(
                key::ROLE,
                "Cargo",
                Select,
                labels(&[
                    "Cuidadora o cuidador",
                    "Enfermería",
                    "Cocina",
                    "Limpieza",
                    "Administración",
                    "Psicología",
                    "Trabajo social",
                    "Docencia",
                    "Medicina",
                    "Religiosa o religioso",
                    "Mantenimiento",
                    "Vigilancia",
                    "Voluntariado",
                ]),
                true,
                false,
            ),
            f(key::CONTRACT, "Tipo de contrato", Select, labels(&["De planta", "Por tiempo definido", "Honorarios"]), false, false),
            f(
                "shift",
                "Horario",
                Select,
                labels(&["Matutino", "Vespertino", "Nocturno", "Por turnos", "24 horas", "Medio tiempo", "Fines de semana"]),
                false,
                false,
            ),
            f(key::SALARY, "Sueldo mensual", Money, vec![], false, false),
            f(key::PAID, "¿Recibe sueldo?", YesNo, vec![], false, false),
            f(key::START_YEAR, "Año en que entró", Year, vec![], false, false),
            f("phone", "Teléfono", Phone, vec![], false, false),
            f("email", "Correo electrónico", Email, vec![], false, false),
        ],
        Entity::Beneficiary => {
            let categories = match institution {
                InstitutionKind::ChildrenHome => labels(&["Niñas", "Niños", "Adolescentes mujeres", "Adolescentes hombres"]),
                InstitutionKind::ElderlyHome => labels(&["Mujeres adultas mayores", "Hombres adultos mayores"]),
                InstitutionKind::Other => labels(&["Personas atendidas"]),
            };
            vec![
                f(key::FULL_NAME, "Nombre completo", Text, vec![], true, false),
                f(key::CATEGORY, "Grupo", Select, categories, true, false),
                f(key::AGE, "Edad", Number, vec![], false, false),
                f(key::DEPENDENCY, "Nivel de apoyo que necesita", Select, dependency_options(), false, true),
                f(key::FEE, "Cuota mensual de estancia", Money, vec![], false, false),
                f("entry_year", "Año en que ingresó", Year, vec![], false, false),
                f("phone", "Teléfono de contacto", Phone, vec![], false, false),
                f("email", "Correo de contacto", Email, vec![], false, false),
            ]
        }
    }
}

fn whole(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// Keeps only the known fields, trims the values and checks them. The result is what is stored.
pub fn clean_entry(fields: &[RosterField], data: &Data) -> Result<Data, RosterError> {
    let mut out = Data::new();
    for f in fields {
        let v = data.get(&f.key).map(|v| v.trim().to_string()).unwrap_or_default();
        if v.is_empty() {
            if f.required {
                return Err(RosterError::Required);
            }
            continue;
        }
        match f.kind {
            FieldKind::Number => {
                whole(&v).ok_or(RosterError::NotANumber)?;
            }
            FieldKind::Money => {
                // «9,500» and «$9 500» are what people write: kept as plain digits
                let n = super::finances::parse_pesos(&v).ok_or(RosterError::NotANumber)?;
                out.insert(f.key.clone(), n.to_string());
                continue;
            }
            FieldKind::Year => {
                let y = whole(&v).ok_or(RosterError::NotANumber)?;
                if !(1900..=2100).contains(&y) {
                    return Err(RosterError::YearInvalid);
                }
            }
            FieldKind::Email => {
                if !v.contains('@') || v.contains(char::is_whitespace) {
                    return Err(RosterError::EmailInvalid);
                }
            }
            FieldKind::YesNo => {
                if v != "yes" && v != "no" {
                    return Err(RosterError::Required);
                }
            }
            FieldKind::Text | FieldKind::Select | FieldKind::Phone => {}
        }
        out.insert(f.key.clone(), v);
    }
    Ok(out)
}

fn num(d: &Data, k: &str) -> Option<i64> {
    d.get(k).and_then(|v| whole(v))
}

/// One line per distinct position (role, with or without pay, same salary, same contract and start year: the last
/// two set the benefits of the law): the people of the roster, counted.
pub fn derive_staff(entries: &[RosterEntry]) -> Vec<StaffGroupInput> {
    let mut out: Vec<StaffGroupInput> = Vec::new();
    for e in entries {
        let role = e.data.get(key::ROLE).map(|r| r.trim()).filter(|r| !r.is_empty()).unwrap_or("Sin cargo").to_string();
        let paid = e.data.get(key::PAID).map_or(true, |v| v != "no");
        let salary = if paid { num(&e.data, key::SALARY) } else { None };
        let contract = if paid { e.data.get(key::CONTRACT).and_then(|c| ContractKind::from_label(c)) } else { None };
        let start_year = if paid { num(&e.data, key::START_YEAR) } else { None };
        match out.iter_mut().find(|g| {
            g.role == role && g.paid == paid && g.monthly_salary_mxn == salary && g.contract == contract && g.start_year == start_year
        }) {
            Some(g) => g.count += 1,
            None => out.push(StaffGroupInput { role, count: 1, paid, monthly_salary_mxn: salary, contract, start_year, ..Default::default() }),
        }
    }
    out
}

/// One line per distinct group (category, level of support, fee): the people served, counted and never named.
pub fn derive_population(entries: &[RosterEntry]) -> Vec<PopulationGroupInput> {
    let mut out: Vec<PopulationGroupInput> = Vec::new();
    for e in entries {
        let label = e.data.get(key::CATEGORY).map(|r| r.trim()).filter(|r| !r.is_empty()).unwrap_or("Sin grupo").to_string();
        let dependency = e.data.get(key::DEPENDENCY).and_then(|v| DependencyLevel::from_db(v));
        let fee = num(&e.data, key::FEE).filter(|f| *f > 0);
        let age = num(&e.data, key::AGE);
        match out.iter_mut().find(|g| g.label == label && g.dependency_level == dependency && g.monthly_fee_mxn == fee) {
            Some(g) => {
                g.count += 1;
                g.paying_count = fee.map(|_| g.count);
                if let Some(a) = age {
                    g.age_min = Some(g.age_min.map_or(a, |m| m.min(a)));
                    g.age_max = Some(g.age_max.map_or(a, |m| m.max(a)));
                }
            }
            None => out.push(PopulationGroupInput {
                label,
                age_min: age,
                age_max: age,
                count: 1,
                dependency_level: dependency,
                notes: None,
                paying_count: fee.map(|_| 1),
                monthly_fee_mxn: fee,
            }),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::ProfileInput;

    fn entry(pairs: &[(&str, &str)]) -> RosterEntry {
        RosterEntry { id: "e".into(), data: pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() }
    }

    #[test]
    fn every_kind_is_written_the_same_way_as_the_database_and_the_screen_say_it() {
        use FieldKind::*;
        for k in [Text, Select, Number, Money, Year, Email, Phone, YesNo] {
            assert_eq!(serde_json::to_string(&k).unwrap(), format!("\"{}\"", k.as_db()));
            assert_eq!(FieldKind::from_db(k.as_db()), Some(k));
        }
    }

    #[test]
    fn the_categories_depend_on_the_kind_of_institution() {
        let cat = |k| default_fields(Entity::Beneficiary, k).into_iter().find(|f| f.key == key::CATEGORY).unwrap().options;
        assert!(cat(InstitutionKind::ChildrenHome).iter().any(|o| o.label == "Niñas"));
        assert!(cat(InstitutionKind::ElderlyHome).iter().any(|o| o.label == "Hombres adultos mayores"));
        assert_ne!(cat(InstitutionKind::ChildrenHome), cat(InstitutionKind::ElderlyHome));
    }

    #[test]
    fn a_record_keeps_only_known_fields_and_checks_them() {
        let fields = default_fields(Entity::Staff, InstitutionKind::Other);
        let ok = clean_entry(&fields, &entry(&[("full_name", " Ana Ejemplo "), ("role", "Cocina"), ("secret", "x"), ("start_year", "2019")]).data).unwrap();
        assert_eq!(ok.get("full_name").map(String::as_str), Some("Ana Ejemplo"));
        assert!(!ok.contains_key("secret"));

        let bad = |pairs: &[(&str, &str)]| clean_entry(&fields, &entry(pairs).data).unwrap_err();
        assert_eq!(bad(&[("role", "Cocina")]), RosterError::Required);
        assert_eq!(bad(&[("full_name", "A"), ("role", "Cocina"), ("monthly_salary_mxn", "9 mil")]), RosterError::NotANumber);
        assert_eq!(bad(&[("full_name", "A"), ("role", "Cocina"), ("start_year", "19")]), RosterError::YearInvalid);
        assert_eq!(bad(&[("full_name", "A"), ("role", "Cocina"), ("email", "sin arroba")]), RosterError::EmailInvalid);
    }

    #[test]
    fn staff_is_counted_by_position_and_the_payroll_adds_up() {
        let staff = derive_staff(&[
            entry(&[("role", "Cocina"), ("monthly_salary_mxn", "7000")]),
            entry(&[("role", "Cocina"), ("monthly_salary_mxn", "7000")]),
            entry(&[("role", "Enfermería"), ("monthly_salary_mxn", "9500")]),
            entry(&[("role", "Voluntariado"), ("paid", "no"), ("monthly_salary_mxn", "1")]),
        ]);
        assert_eq!(staff.len(), 3);
        assert_eq!((staff[0].role.as_str(), staff[0].count), ("Cocina", 2));
        let t = ProfileInput { staff, ..Default::default() }.totals(2026);
        assert_eq!((t.staff_paid, t.staff_volunteer, t.payroll_monthly_mxn), (3, 1, 23_500));
    }

    #[test]
    fn contract_and_start_year_reach_the_profile_for_the_benefits_and_volunteers_carry_none() {
        let staff = derive_staff(&[
            entry(&[("role", "Cocina"), ("monthly_salary_mxn", "9000"), ("contract", "De planta"), ("start_year", "2018")]),
            entry(&[("role", "Cocina"), ("monthly_salary_mxn", "9000"), ("contract", "De planta"), ("start_year", "2018")]),
            entry(&[("role", "Cocina"), ("monthly_salary_mxn", "9000"), ("contract", "Honorarios"), ("start_year", "2018")]),
            entry(&[("role", "Voluntariado"), ("paid", "no"), ("contract", "De planta"), ("start_year", "2020")]),
        ]);
        let lines: Vec<_> = staff.iter().map(|g| (g.role.as_str(), g.count, g.contract, g.start_year)).collect();
        assert_eq!(lines, vec![
            ("Cocina", 2, Some(ContractKind::Permanent), Some(2018)),
            ("Cocina", 1, Some(ContractKind::Fees), Some(2018)),
            ("Voluntariado", 1, None, None),
        ]);
        let t = ProfileInput { staff, ..Default::default() }.totals(2026);
        // two cooks with benefits (8 years: 6,150 each); fees carry none
        assert_eq!((t.payroll_benefits_annual_mxn, t.payroll_cost_annual_mxn), (12_300, 27_000 * 12 + 12_300));
    }

    #[test]
    fn money_is_kept_as_plain_digits_however_it_was_written() {
        let fields = default_fields(Entity::Staff, InstitutionKind::Other);
        let ok = clean_entry(&fields, &entry(&[("full_name", "A"), ("role", "Cocina"), ("monthly_salary_mxn", "$9,500")]).data).unwrap();
        assert_eq!(ok.get("monthly_salary_mxn").map(String::as_str), Some("9500"));
    }

    #[test]
    fn people_served_are_counted_by_group_with_their_fees() {
        let people = derive_population(&[
            entry(&[("category", "Niñas"), ("age", "7"), ("monthly_fee_mxn", "1500")]),
            entry(&[("category", "Niñas"), ("age", "10"), ("monthly_fee_mxn", "1500")]),
            entry(&[("category", "Niñas"), ("age", "9")]),
            entry(&[("category", "Niños"), ("dependency", "high")]),
        ]);
        let t = ProfileInput { population: people.clone(), ..Default::default() }.totals(2026);
        assert_eq!((t.population, t.fee_payers, t.fees_monthly_mxn, t.fees_annual_mxn), (4, 2, 3_000, 36_000));
        let paying = people.iter().find(|g| g.monthly_fee_mxn == Some(1500)).unwrap();
        assert_eq!((paying.age_min, paying.age_max), (Some(7), Some(10)));
        assert!(people.iter().all(|g| g.notes.is_none()));
    }
}
