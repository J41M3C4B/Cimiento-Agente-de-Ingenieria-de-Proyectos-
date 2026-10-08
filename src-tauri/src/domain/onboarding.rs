//! The onboarding of the institution (ADR-031): the steps of the first start and what each one requires. The data
//! belong to the institution, not to a person: while one required datum is missing, whoever enters with access
//! sees the steps instead of the app (the administrator may leave them to the direction). The rules live here; the
//! screen only shows what is missing.

use super::profile::ProfileInput;
use crate::modules::facilities::domain::site::SiteData;
use serde::Serialize;

/// The steps, in order. The last screen (review and confirm) is not a step: it needs all of them complete.
pub const STEPS: &[&str] = &["institution", "location", "people", "team", "money", "building"];

/// The 32 states: (code, name).
pub const STATES: &[(&str, &str)] = &[
    ("ags", "Aguascalientes"),
    ("bc", "Baja California"),
    ("bcs", "Baja California Sur"),
    ("camp", "Campeche"),
    ("coah", "Coahuila"),
    ("col", "Colima"),
    ("chis", "Chiapas"),
    ("chih", "Chihuahua"),
    ("cdmx", "Ciudad de México"),
    ("dgo", "Durango"),
    ("gto", "Guanajuato"),
    ("gro", "Guerrero"),
    ("hgo", "Hidalgo"),
    ("jal", "Jalisco"),
    ("mex", "Estado de México"),
    ("mich", "Michoacán"),
    ("mor", "Morelos"),
    ("nay", "Nayarit"),
    ("nl", "Nuevo León"),
    ("oax", "Oaxaca"),
    ("pue", "Puebla"),
    ("qro", "Querétaro"),
    ("qroo", "Quintana Roo"),
    ("slp", "San Luis Potosí"),
    ("sin", "Sinaloa"),
    ("son", "Sonora"),
    ("tab", "Tabasco"),
    ("tamps", "Tamaulipas"),
    ("tlax", "Tlaxcala"),
    ("ver", "Veracruz"),
    ("yuc", "Yucatán"),
    ("zac", "Zacatecas"),
];

/// Asociación civil, institución de asistencia privada, institución de beneficencia privada, sociedad civil,
/// asociación de beneficencia privada, asociación religiosa, other.
pub const LEGAL_FORMS: &[&str] = &["ac", "iap", "ibp", "sc", "abp", "religious", "other"];
/// Answers for «donataria autorizada» and «CLUNI».
pub const REGISTRY: &[&str] = &["yes", "in_progress", "no"];
pub const OLDEST_YEAR: i64 = 1800;
/// The most people a quick figure may say.
pub const MAX_ESTIMATE: i64 = 100_000;

pub fn state_name(code: &str) -> Option<&'static str> {
    STATES.iter().find(|(c, _)| *c == code).map(|(_, n)| *n)
}

/// What the steps look at, besides the profile: the records of the modules and the main site.
pub struct Facts<'a> {
    pub input: &'a ProfileInput,
    /// People served with a record in their module.
    pub served_in_module: i64,
    /// People of the staff with a record in their module.
    pub staff_in_module: i64,
    /// People who pay a stay fee, from the records (that is income too).
    pub fee_payers: i64,
    pub site: Option<&'a SiteData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StepStatus {
    pub key: &'static str,
    /// The required data still missing (codes the screen names).
    pub missing: Vec<&'static str>,
    pub complete: bool,
}

fn filled(o: &Option<String>) -> bool {
    o.as_deref().is_some_and(|s| !s.trim().is_empty())
}

/// What each step still needs.
pub fn steps(f: &Facts) -> Vec<StepStatus> {
    let i = f.input;
    let inst = &i.institution;
    STEPS
        .iter()
        .map(|&key| {
            let mut m: Vec<&'static str> = Vec::new();
            match key {
                "institution" => {
                    if inst.name.trim().is_empty() {
                        m.push("name");
                    }
                    if !filled(&inst.mission) {
                        m.push("mission");
                    }
                }
                "location" => {
                    if !filled(&inst.state) {
                        m.push("state");
                    }
                    if !filled(&inst.municipality) {
                        m.push("municipality");
                    }
                    if !filled(&inst.contact_phone) && !filled(&inst.contact_email) {
                        m.push("contact");
                    }
                    if !filled(&inst.legal_form) {
                        m.push("legal_form");
                    }
                    if inst.founded_year.is_none() {
                        m.push("founded_year");
                    }
                    if !filled(&inst.authorized_donee) {
                        m.push("authorized_donee");
                    }
                    if !filled(&inst.cluni) {
                        m.push("cluni");
                    }
                }
                "people" => {
                    if i.capacity_total.is_none() {
                        m.push("capacity_total");
                    }
                    if f.served_in_module == 0 && i.served_estimate.is_none() {
                        m.push("served");
                    }
                }
                "team" => {
                    if f.staff_in_module == 0 && (i.staff_paid_estimate.is_none() || i.staff_volunteer_estimate.is_none()) {
                        m.push("staff");
                    }
                }
                "money" => {
                    if i.annual_budget_mxn.is_none() && !i.expenses.iter().any(|e| e.amount_mxn.is_some()) {
                        m.push("expenses");
                    }
                    if f.fee_payers == 0 && !i.income.iter().any(|x| x.amount_mxn.is_some()) {
                        m.push("income");
                    }
                }
                _ => {
                    let site = f.site;
                    if site.and_then(|s| s.floors).is_none() {
                        m.push("floors");
                    }
                    if !site.is_some_and(|s| filled(&s.tenure)) {
                        m.push("tenure");
                    }
                }
            }
            StepStatus { key, complete: m.is_empty(), missing: m }
        })
        .collect()
}

/// Whether every step is complete.
pub fn complete(steps: &[StepStatus]) -> bool {
    steps.iter().all(|s| s.complete)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{IncomeKind, IncomeSourceInput, InstitutionInput, Period};

    fn full() -> ProfileInput {
        ProfileInput {
            institution: InstitutionInput {
                name: "Asilo Ficticio".into(),
                mission: Some("Un hogar digno.".into()),
                state: Some("jal".into()),
                municipality: Some("Zapopan".into()),
                contact_email: Some("contacto@asilo-ficticio.org".into()),
                legal_form: Some("ac".into()),
                founded_year: Some(1987),
                authorized_donee: Some("yes".into()),
                cluni: Some("in_progress".into()),
                ..Default::default()
            },
            capacity_total: Some(25),
            served_estimate: Some(22),
            staff_paid_estimate: Some(6),
            staff_volunteer_estimate: Some(0),
            annual_budget_mxn: Some(1_800_000),
            income: vec![IncomeSourceInput { label: "Donativos".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(500_000), period: Period::Annual }],
            ..Default::default()
        }
    }

    fn site() -> SiteData {
        SiteData { name: "Casa".into(), floors: Some(2), tenure: Some("own".into()), ..Default::default() }
    }

    fn missing(f: &Facts) -> Vec<(&'static str, Vec<&'static str>)> {
        steps(f).into_iter().filter(|s| !s.complete).map(|s| (s.key, s.missing)).collect()
    }

    #[test]
    fn a_full_institution_is_complete() {
        let (p, s) = (full(), site());
        let f = Facts { input: &p, served_in_module: 0, staff_in_module: 0, fee_payers: 0, site: Some(&s) };
        assert!(complete(&steps(&f)), "{:?}", missing(&f));
    }

    #[test]
    fn an_empty_start_names_everything_it_needs() {
        let p = ProfileInput::default();
        let f = Facts { input: &p, served_in_module: 0, staff_in_module: 0, fee_payers: 0, site: None };
        assert_eq!(missing(&f), vec![
            ("institution", vec!["name", "mission"]),
            ("location", vec!["state", "municipality", "contact", "legal_form", "founded_year", "authorized_donee", "cluni"]),
            ("people", vec!["capacity_total", "served"]),
            ("team", vec!["staff"]),
            ("money", vec!["expenses", "income"]),
            ("building", vec!["floors", "tenure"]),
        ]);
    }

    #[test]
    fn the_records_of_the_modules_count_instead_of_the_quick_figures() {
        let mut p = full();
        p.served_estimate = None;
        p.staff_paid_estimate = None;
        p.income.clear();
        let s = site();
        let f = Facts { input: &p, served_in_module: 18, staff_in_module: 7, fee_payers: 5, site: Some(&s) };
        assert!(complete(&steps(&f)), "{:?}", missing(&f));
        let f = Facts { input: &p, served_in_module: 0, staff_in_module: 0, fee_payers: 0, site: Some(&s) };
        assert_eq!(missing(&f), vec![("people", vec!["served"]), ("team", vec!["staff"]), ("money", vec!["income"])]);
    }

    #[test]
    fn zero_volunteers_is_an_answer_but_blank_text_is_not() {
        let mut p = full();
        p.institution.municipality = Some("   ".into());
        let s = site();
        let f = Facts { input: &p, served_in_module: 0, staff_in_module: 0, fee_payers: 0, site: Some(&s) };
        assert_eq!(missing(&f), vec![("location", vec!["municipality"])]);
        assert_eq!(state_name("qroo"), Some("Quintana Roo"));
        assert_eq!(STATES.len(), 32);
    }
}
