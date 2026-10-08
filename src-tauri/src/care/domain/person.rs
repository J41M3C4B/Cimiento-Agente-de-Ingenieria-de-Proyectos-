//! The record of a person served (ADR-029), in five steps: identification, admission and stay, care and health,
//! family and responsible person, and contribution and support. One record for every kind of institution: the
//! elderly home and the children's home add their own data. The name and the birth date (or an approximate age)
//! are enough to save; the rest fills in with time.

use super::catalog::{self, Flavor};
use crate::common::contact::phone_ok;
use crate::common::dates::{date_is_valid, years_between};
use crate::common::ids;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_MXN: i64 = 100_000_000_000;
pub const MAX_CONTACTS: usize = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResponsibleContact {
    pub full_name: String,
    pub relationship: Option<String>,
    pub phone: Option<String>,
    pub phone_alt: Option<String>,
    /// The legal guardian (tutor), not only a contact.
    pub legal_guardian: bool,
}

/// What the form writes. Codes are those of `catalog`. `curp` is covered on screen: `None` keeps the stored one,
/// `Some("")` clears it, anything else replaces it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BeneficiaryData {
    // 1. identification
    pub first_names: String,
    pub last_name_1: Option<String>,
    pub last_name_2: Option<String>,
    pub birth_date: Option<String>,
    /// Only the age was known: the birth date is the 1st of January of the year it gives.
    pub birth_date_approx: bool,
    /// What the form sends when the birth date is not known; the service turns it into an approximate date.
    pub approx_age: Option<i64>,
    pub sex: Option<String>,
    pub curp: Option<String>,
    pub origin_municipality: Option<String>,
    pub origin_state: Option<String>,
    /// The indigenous language the person speaks, if any.
    pub indigenous_language: Option<String>,
    pub education: Option<String>,
    pub literate: Option<bool>,
    pub attends_school: Option<bool>,
    pub school_grade: Option<String>,
    pub school_lag: Option<bool>,
    /// A group of the institution's own («Pabellón A»); without it, the group comes from sex and age.
    pub group_id: Option<String>,
    // 2. admission and stay
    pub entry_date: Option<String>,
    pub entry_date_approx: bool,
    pub stay_mode: Option<String>,
    pub referred_by: Option<String>,
    pub admission_reasons: Vec<String>,
    pub status: String,
    /// The day of the discharge or the death.
    pub status_date: Option<String>,
    pub discharge_reason: Option<String>,
    // 3. care and health (categories only)
    pub dependency: Option<String>,
    pub mobility: Option<String>,
    pub disabilities: Vec<String>,
    pub chronic_conditions: Vec<String>,
    pub continence: Option<String>,
    pub orientation: Option<String>,
    pub psych_care: Option<bool>,
    pub vaccines_up_to_date: Option<bool>,
    // 4. family and responsible person
    pub contacts: Vec<ResponsibleContact>,
    pub visits: Option<String>,
    pub legal_status: Option<String>,
    // 5. contribution and support
    pub monthly_fee_mxn: Option<i64>,
    pub fee_payer: Option<String>,
    pub programs: Vec<String>,
    /// The privacy notice, signed.
    pub consent_date: Option<String>,
    pub consent_signer: Option<String>,
    /// The institution's own fields, by key.
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub code: &'static str,
    pub field: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StepProgress {
    pub filled: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub identification: StepProgress,
    pub stay: StepProgress,
    pub care: StepProgress,
    pub family: StepProgress,
    pub contribution: StepProgress,
    pub percent: u32,
}

fn filled(o: &Option<String>) -> bool {
    o.as_deref().is_some_and(|s| !s.trim().is_empty())
}

impl BeneficiaryData {
    pub fn full_name(&self) -> String {
        [Some(self.first_names.as_str()), self.last_name_1.as_deref(), self.last_name_2.as_deref()]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn is_served(&self) -> bool {
        catalog::is_served(&self.status)
    }

    /// The age on `today`, from the birth date.
    pub fn age(&self, today: &str) -> Option<i64> {
        self.birth_date.as_deref().and_then(|b| years_between(b, today))
    }

    /// Paying a stay fee: an amount, and not marked as exempt.
    pub fn pays_fee(&self) -> bool {
        self.monthly_fee_mxn.is_some_and(|f| f > 0) && self.fee_payer.as_deref() != Some("exempt")
    }

    pub fn validate(&self, flavor: Flavor, today: &str) -> Vec<Issue> {
        let mut v = Vec::new();
        let mut add = |code: &'static str, field: &str, blocking: bool| v.push(Issue { code, field: field.to_string(), blocking });

        if self.first_names.trim().is_empty() {
            add("name_missing", "first_names", true);
        }
        if !filled(&self.birth_date) && self.approx_age.is_none() {
            add("birth_missing", "birth_date", true);
        }
        if self.approx_age.is_some_and(|a| !(0..=120).contains(&a)) {
            add("age_invalid", "approx_age", true);
        }
        let one: [(&str, &Option<String>, &[&str]); 14] = [
            ("sex", &self.sex, catalog::SEXES),
            ("education", &self.education, catalog::EDUCATION),
            ("school_grade", &self.school_grade, catalog::SCHOOL_GRADES),
            ("stay_mode", &self.stay_mode, catalog::STAY_MODES),
            ("referred_by", &self.referred_by, catalog::REFERRED_BY),
            ("discharge_reason", &self.discharge_reason, catalog::DISCHARGE_REASONS),
            ("dependency", &self.dependency, catalog::DEPENDENCY),
            ("mobility", &self.mobility, catalog::MOBILITY),
            ("continence", &self.continence, catalog::CONTINENCE),
            ("orientation", &self.orientation, catalog::ORIENTATION),
            ("visits", &self.visits, catalog::VISITS),
            ("legal_status", &self.legal_status, catalog::LEGAL_STATUSES),
            ("fee_payer", &self.fee_payer, catalog::FEE_PAYERS),
            ("consent_signer", &self.consent_signer, catalog::CONSENT_SIGNERS),
        ];
        for (field, value, allowed) in one {
            if value.as_deref().is_some_and(|x| !x.is_empty() && !allowed.contains(&x)) {
                add("code_unknown", field, true);
            }
        }
        let many: [(&str, &Vec<String>, &[&str]); 4] = [
            ("admission_reasons", &self.admission_reasons, catalog::ADMISSION_REASONS),
            ("disabilities", &self.disabilities, catalog::DISABILITIES),
            ("chronic_conditions", &self.chronic_conditions, catalog::CHRONIC),
            ("programs", &self.programs, catalog::PROGRAMS),
        ];
        for (field, values, allowed) in many {
            if values.iter().any(|x| !allowed.contains(&x.as_str())) {
                add("code_unknown", field, true);
            }
        }
        if !catalog::STATUSES.contains(&self.status.as_str()) {
            add("code_unknown", "status", true);
        }

        for (field, d) in [("birth_date", &self.birth_date), ("entry_date", &self.entry_date), ("status_date", &self.status_date), ("consent_date", &self.consent_date)] {
            if d.as_deref().is_some_and(|x| !x.is_empty() && !date_is_valid(x)) {
                add("date_invalid", field, true);
            }
        }
        let date = |d: &Option<String>| d.as_deref().filter(|s| date_is_valid(s)).map(str::to_string);
        if let (Some(b), Some(e)) = (date(&self.birth_date), date(&self.entry_date)) {
            if e < b {
                add("entry_before_birth", "entry_date", true);
            }
        }
        if date(&self.birth_date).is_some_and(|b| b.as_str() > today) {
            add("birth_in_future", "birth_date", true);
        }
        if let Some(age) = self.age(today).or(self.approx_age) {
            let unusual = match flavor {
                Flavor::ElderlyHome => age < 50,
                Flavor::ChildrenHome => age > 25,
                Flavor::Other => false,
            };
            if unusual {
                add("age_unusual", "birth_date", false);
            }
        }

        if let Some(c) = self.curp.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            if !ids::curp_is_valid(c) {
                add("curp_invalid", "curp", true);
            } else if let (Some((yy, mm, dd)), Some(b)) = (ids::curp_birth(c), date(&self.birth_date)) {
                let same = b[2..4].parse::<u32>().ok() == Some(yy) && b[5..7].parse::<u32>().ok() == Some(mm) && b[8..10].parse::<u32>().ok() == Some(dd);
                if !same && !self.birth_date_approx {
                    add("curp_birth_mismatch", "curp", false);
                }
            }
        }

        if self.contacts.len() > MAX_CONTACTS {
            add("too_many_contacts", "contacts", true);
        }
        for (i, c) in self.contacts.iter().enumerate() {
            if c.full_name.trim().is_empty() {
                add("contact_name_missing", &format!("contacts[{i}].full_name"), true);
            }
            if c.relationship.as_deref().is_some_and(|r| !r.is_empty() && !catalog::RELATIONSHIPS.contains(&r)) {
                add("code_unknown", &format!("contacts[{i}].relationship"), true);
            }
            for (f, p) in [("phone", &c.phone), ("phone_alt", &c.phone_alt)] {
                if filled(p) && !phone_ok(p.as_deref().unwrap_or_default()) {
                    add("phone_invalid", &format!("contacts[{i}].{f}"), true);
                }
            }
        }

        match self.monthly_fee_mxn {
            Some(a) if a < 0 => add("negative_number", "monthly_fee_mxn", true),
            Some(a) if a > MAX_MXN => add("amount_too_large", "monthly_fee_mxn", true),
            _ => {}
        }
        if self.fee_payer.as_deref() == Some("exempt") && self.monthly_fee_mxn.is_some_and(|f| f > 0) {
            add("exempt_with_fee", "monthly_fee_mxn", false);
        }
        if matches!(self.status.as_str(), "discharged" | "deceased") && !filled(&self.status_date) {
            add("status_date_missing", "status_date", false);
        }
        if self.status == "discharged" && !filled(&self.discharge_reason) {
            add("discharge_reason_missing", "discharge_reason", false);
        }
        v
    }

    /// How far each step has come, with what applies to the kind of institution. `curp_stored` says whether a covered
    /// CURP is stored (it comes as `None`).
    pub fn progress(&self, flavor: Flavor, curp_stored: bool) -> Progress {
        let count = |items: &[bool]| StepProgress { filled: items.iter().filter(|x| **x).count() as u32, total: items.len() as u32 };
        let mut ident = vec![filled(&self.last_name_1), filled(&self.sex), curp_stored || filled(&self.curp), filled(&self.origin_state)];
        match flavor {
            Flavor::ElderlyHome => ident.extend([filled(&self.education), self.literate.is_some()]),
            Flavor::ChildrenHome => ident.extend([self.attends_school.is_some(), filled(&self.school_grade)]),
            Flavor::Other => {}
        }
        let stay = count(&[filled(&self.entry_date), filled(&self.stay_mode), filled(&self.referred_by), !self.admission_reasons.is_empty()]);
        let mut care = vec![filled(&self.dependency), filled(&self.mobility)];
        match flavor {
            Flavor::ElderlyHome => care.extend([filled(&self.continence), filled(&self.orientation)]),
            Flavor::ChildrenHome => care.extend([self.vaccines_up_to_date.is_some(), self.psych_care.is_some()]),
            Flavor::Other => {}
        }
        let mut family = vec![self.contacts.iter().any(|c| !c.full_name.trim().is_empty() && filled(&c.phone)), filled(&self.visits)];
        if flavor == Flavor::ChildrenHome {
            family.push(filled(&self.legal_status));
        }
        let contribution = count(&[filled(&self.fee_payer), self.monthly_fee_mxn.is_some() || self.fee_payer.as_deref() == Some("exempt"), filled(&self.consent_date)]);
        let (identification, care, family) = (count(&ident), count(&care), count(&family));
        let (f, t) = [identification, stay, care, family, contribution].iter().fold((0, 0), |(f, t), s| (f + s.filled, t + s.total));
        Progress { identification, stay, care, family, contribution, percent: if t == 0 { 100 } else { f * 100 / t } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> BeneficiaryData {
        BeneficiaryData { first_names: "Luz".into(), approx_age: Some(84), status: "active".into(), ..Default::default() }
    }

    #[test]
    fn a_name_and_a_birth_date_or_an_age_are_enough() {
        assert!(minimal().validate(Flavor::ElderlyHome, "2026-10-07").iter().all(|i| !i.blocking));
        let bare = BeneficiaryData { status: "active".into(), ..Default::default() };
        let codes: Vec<_> = bare.validate(Flavor::ElderlyHome, "2026-10-07").into_iter().filter(|i| i.blocking).map(|i| i.code).collect();
        assert_eq!(codes, vec!["name_missing", "birth_missing"]);
    }

    #[test]
    fn codes_dates_and_contacts_are_checked() {
        let mut p = minimal();
        p.mobility = Some("flying".into());
        p.chronic_conditions = vec!["diabetes".into(), "gripa".into()];
        p.birth_date = Some("1940-02-30".into());
        p.contacts = vec![ResponsibleContact { full_name: " ".into(), phone: Some("12".into()), ..Default::default() }];
        let codes: Vec<_> = p.validate(Flavor::ElderlyHome, "2026-10-07").into_iter().map(|i| (i.code, i.field)).collect();
        for want in [("code_unknown", "mobility"), ("code_unknown", "chronic_conditions"), ("date_invalid", "birth_date"), ("contact_name_missing", "contacts[0].full_name"), ("phone_invalid", "contacts[0].phone")] {
            assert!(codes.contains(&(want.0, want.1.to_string())), "{want:?} in {codes:?}");
        }
    }

    #[test]
    fn heads_ups_for_what_does_not_add_up() {
        let mut p = minimal();
        p.approx_age = None;
        p.birth_date = Some("2015-05-01".into());
        p.fee_payer = Some("exempt".into());
        p.monthly_fee_mxn = Some(1_500);
        p.status = "discharged".into();
        let codes: Vec<_> = p.validate(Flavor::ElderlyHome, "2026-10-07").into_iter().map(|i| (i.code, i.blocking)).collect();
        assert_eq!(codes, vec![("age_unusual", false), ("exempt_with_fee", false), ("status_date_missing", false), ("discharge_reason_missing", false)]);
        assert!(!p.pays_fee());
        p.birth_date = Some("1999-01-01".into());
        assert!(p.validate(Flavor::ChildrenHome, "2026-10-07").iter().any(|i| i.code == "age_unusual"), "27 years in a children's home");
    }

    #[test]
    fn progress_counts_what_applies_to_each_kind_of_institution() {
        let p = minimal();
        let elderly = p.progress(Flavor::ElderlyHome, false);
        let children = p.progress(Flavor::ChildrenHome, false);
        assert_eq!((elderly.identification.total, elderly.care.total, elderly.family.total), (6, 4, 2));
        assert_eq!((children.identification.total, children.care.total, children.family.total), (6, 4, 3), "the legal situation counts in a children's home");
        let mut q = minimal();
        q.dependency = Some("high".into());
        q.mobility = Some("wheelchair".into());
        q.fee_payer = Some("exempt".into());
        let r = q.progress(Flavor::ElderlyHome, true);
        assert_eq!((r.care.filled, r.contribution.filled, r.identification.filled), (2, 2, 1));
    }
}
