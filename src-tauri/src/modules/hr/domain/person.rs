//! The record of one person of the staff (ADR-027), in four steps: personal data, job and position, emergency
//! contacts, and pay and tax data. Only the name, the position and the modality are needed to save; the rest
//! fills in with time and the record shows how far it has come.

use super::catalog::{self, PayKind, Rules};
use super::payroll;
use crate::common::contact::{email_ok, phone_ok, zip_ok};
use crate::common::ids;
pub use crate::common::dates::{date_is_valid, years_between};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The largest amount a person can write: more is a typing mistake (the same bound as the profile).
pub const MAX_MXN: i64 = 100_000_000_000;
/// How many emergency contacts a record keeps.
pub const MAX_CONTACTS: usize = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub number: Option<String>,
    pub neighborhood: Option<String>,
    pub municipality: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub full_name: String,
    pub relationship: Option<String>,
    pub phone: Option<String>,
    pub phone_alt: Option<String>,
}

/// What the form writes. Codes are those of `catalog`. The four identifiers (`curp`, `rfc`, `nss`, `clabe`) are
/// covered on screen, so when editing `None` means «leave it as it is», `Some("")` clears it and anything else
/// replaces it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PersonData {
    // 1. personal data
    pub first_names: String,
    pub last_name_1: Option<String>,
    pub last_name_2: Option<String>,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub curp: Option<String>,
    pub marital_status: Option<String>,
    pub nationality: Option<String>,
    pub education: Option<String>,
    pub professional_license: Option<String>,
    pub address: Address,
    pub phone: Option<String>,
    pub email: Option<String>,
    // 2. job and position
    pub position_id: Option<String>,
    pub modality: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub schedule: Option<String>,
    pub shift: Option<String>,
    pub work_days: Vec<String>,
    pub weekly_hours: Option<i64>,
    pub status: String,
    pub left_date: Option<String>,
    pub left_reason: Option<String>,
    // 3. emergency contacts
    pub emergency_contacts: Vec<EmergencyContact>,
    // 4. pay and tax data
    pub pay_amount_mxn: Option<i64>,
    pub pay_period: Option<String>,
    pub pay_method: Option<String>,
    pub bank: Option<String>,
    pub clabe: Option<String>,
    pub rfc: Option<String>,
    pub nss: Option<String>,
    pub tax_regime: Option<String>,
    pub tax_zip: Option<String>,
    pub infonavit_credit: Option<bool>,
    /// The institution's own fields, by key.
    pub extra: BTreeMap<String, String>,
}

/// Something the person should look at. `blocking` stops the save; the rest is a heads-up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub code: &'static str,
    pub field: String,
    pub blocking: bool,
}

/// How far each step of the record has come: `total` is 0 when the step does not apply to the modality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StepProgress {
    pub filled: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub personal: StepProgress,
    pub job: StepProgress,
    pub emergency: StepProgress,
    pub pay: StepProgress,
    /// Of everything that applies, in whole percent.
    pub percent: u32,
}

fn filled(o: &Option<String>) -> bool {
    o.as_deref().is_some_and(|s| !s.trim().is_empty())
}

impl PersonData {
    /// The name as people say it: names and surnames.
    pub fn full_name(&self) -> String {
        [Some(self.first_names.as_str()), self.last_name_1.as_deref(), self.last_name_2.as_deref()]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Whether the person still works here (on vacation, on leave or sick, they do).
    pub fn is_current(&self) -> bool {
        self.status != "left"
    }

    /// The pay as a month, if there is one.
    pub fn monthly_pay(&self) -> Option<i64> {
        self.pay_amount_mxn.map(|a| payroll::monthly_equivalent(a, self.pay_period.as_deref().unwrap_or("monthly")))
    }

    /// Problems and heads-ups, with the rules of its modality (`None` if the modality is unknown) and `today`
    /// (`YYYY-MM-DD`). The covered identifiers are checked only when they come written (not `None`).
    pub fn validate(&self, rules: Option<Rules>, today: &str) -> Vec<Issue> {
        let mut v = Vec::new();
        let mut add = |code: &'static str, field: &str, blocking: bool| v.push(Issue { code, field: field.to_string(), blocking });

        if self.first_names.trim().is_empty() {
            add("name_missing", "first_names", true);
        }
        if !filled(&self.position_id) {
            add("position_missing", "position_id", true);
        }
        if rules.is_none() {
            add("modality_unknown", "modality", true);
        }
        let codes: [(&str, &Option<String>, &[&str]); 9] = [
            ("sex", &self.sex, catalog::SEXES),
            ("marital_status", &self.marital_status, catalog::MARITAL),
            ("education", &self.education, catalog::EDUCATION),
            ("schedule", &self.schedule, catalog::SCHEDULES),
            ("shift", &self.shift, catalog::SHIFTS),
            ("left_reason", &self.left_reason, catalog::LEFT_REASONS),
            ("pay_period", &self.pay_period, catalog::PAY_PERIODS),
            ("pay_method", &self.pay_method, catalog::PAY_METHODS),
            ("tax_regime", &self.tax_regime, catalog::TAX_REGIMES),
        ];
        for (field, value, allowed) in codes {
            if let Some(x) = value.as_deref().filter(|s| !s.is_empty()) {
                if !allowed.contains(&x) {
                    add("code_unknown", field, true);
                }
            }
        }
        if !catalog::STATUSES.contains(&self.status.as_str()) {
            add("code_unknown", "status", true);
        }
        if self.work_days.iter().any(|d| !catalog::WEEKDAYS.contains(&d.as_str())) {
            add("code_unknown", "work_days", true);
        }

        // dates
        for (field, d) in [("birth_date", &self.birth_date), ("start_date", &self.start_date), ("end_date", &self.end_date), ("left_date", &self.left_date)] {
            if let Some(x) = d.as_deref().filter(|s| !s.is_empty()) {
                if !date_is_valid(x) {
                    add("date_invalid", field, true);
                }
            }
        }
        let date = |d: &Option<String>| d.as_deref().filter(|s| date_is_valid(s)).map(str::to_string);
        if let (Some(s), Some(e)) = (date(&self.start_date), date(&self.end_date)) {
            if e < s {
                add("end_before_start", "end_date", true);
            }
        }
        if let Some(b) = date(&self.birth_date) {
            if years_between(&b, today).is_none_or(|age| !(14..=100).contains(&age)) {
                add("age_out_of_range", "birth_date", false);
            }
        }

        // identifiers, only when they come written
        let given = |o: &Option<String>| o.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        if let Some(c) = given(&self.curp) {
            if !ids::curp_is_valid(&c) {
                add("curp_invalid", "curp", true);
            } else if let (Some((yy, mm, dd)), Some(b)) = (ids::curp_birth(&c), date(&self.birth_date)) {
                let same = b[2..4].parse::<u32>().ok() == Some(yy) && b[5..7].parse::<u32>().ok() == Some(mm) && b[8..10].parse::<u32>().ok() == Some(dd);
                if !same {
                    add("curp_birth_mismatch", "curp", false);
                }
            }
        }
        if let Some(r) = given(&self.rfc) {
            if !ids::rfc_person_is_valid(&r) {
                add("rfc_invalid", "rfc", true);
            } else if let Some(c) = given(&self.curp).filter(|c| ids::curp_is_valid(c)) {
                if ids::normalize(&r)[..10] != ids::normalize(&c)[..10] {
                    add("rfc_curp_mismatch", "rfc", false);
                }
            }
        }
        if let Some(n) = given(&self.nss) {
            if !ids::nss_is_valid(&n) {
                add("nss_invalid", "nss", true);
            }
        }
        if let Some(c) = given(&self.clabe) {
            if !ids::clabe_is_valid(&c) {
                add("clabe_invalid", "clabe", true);
            }
        }
        if let Some(l) = given(&self.professional_license) {
            let n = l.chars().filter(char::is_ascii_digit).count();
            if !(7..=8).contains(&n) || n != l.chars().count() {
                add("license_invalid", "professional_license", true);
            }
        }

        // contact
        if filled(&self.phone) && !phone_ok(self.phone.as_deref().unwrap_or_default()) {
            add("phone_invalid", "phone", true);
        }
        if filled(&self.email) && !email_ok(self.email.as_deref().unwrap_or_default()) {
            add("email_invalid", "email", true);
        }
        if filled(&self.address.zip) && !zip_ok(self.address.zip.as_deref().unwrap_or_default()) {
            add("zip_invalid", "address.zip", true);
        }
        if filled(&self.tax_zip) && !zip_ok(self.tax_zip.as_deref().unwrap_or_default()) {
            add("zip_invalid", "tax_zip", true);
        }
        if self.emergency_contacts.len() > MAX_CONTACTS {
            add("too_many_contacts", "emergency_contacts", true);
        }
        for (i, c) in self.emergency_contacts.iter().enumerate() {
            if c.full_name.trim().is_empty() {
                add("contact_name_missing", &format!("emergency_contacts[{i}].full_name"), true);
            }
            if c.relationship.as_deref().is_some_and(|r| !r.is_empty() && !catalog::RELATIONSHIPS.contains(&r)) {
                add("code_unknown", &format!("emergency_contacts[{i}].relationship"), true);
            }
            for (f, p) in [("phone", &c.phone), ("phone_alt", &c.phone_alt)] {
                if filled(p) && !phone_ok(p.as_deref().unwrap_or_default()) {
                    add("phone_invalid", &format!("emergency_contacts[{i}].{f}"), true);
                }
            }
        }

        // hours and pay
        if let Some(h) = self.weekly_hours {
            if !(1..=168).contains(&h) {
                add("hours_invalid", "weekly_hours", true);
            } else if h > payroll::LEGAL_WEEKLY_HOURS {
                add("hours_over_legal", "weekly_hours", false);
            }
        }
        if let Some(a) = self.pay_amount_mxn {
            if a < 0 {
                add("negative_number", "pay_amount_mxn", true);
            } else if a > MAX_MXN {
                add("amount_too_large", "pay_amount_mxn", true);
            }
        }
        if let Some(r) = rules {
            if r.needs_end_date && !filled(&self.end_date) && self.is_current() {
                add("end_date_missing", "end_date", false);
            }
            if r.pay == PayKind::None && self.pay_amount_mxn.is_some_and(|a| a > 0) {
                add("volunteer_with_pay", "pay_amount_mxn", false);
            }
        }
        if self.status == "left" && !filled(&self.left_date) {
            add("left_date_missing", "left_date", false);
        }
        v
    }

    /// How far each step has come. `secrets` says which covered identifiers are stored (they come as `None`).
    pub fn progress(&self, rules: Option<Rules>, secrets: Secrets) -> Progress {
        let count = |items: &[bool]| StepProgress { filled: items.iter().filter(|x| **x).count() as u32, total: items.len() as u32 };
        let a = &self.address;
        let personal = count(&[
            filled(&self.last_name_1),
            filled(&self.birth_date),
            filled(&self.sex),
            secrets.curp || filled(&self.curp),
            filled(&self.education),
            filled(&a.street) && filled(&a.municipality) && filled(&a.zip),
            filled(&self.phone),
        ]);
        let job = count(&[filled(&self.start_date), filled(&self.schedule), filled(&self.shift), !self.work_days.is_empty(), self.weekly_hours.is_some()]);
        let emergency = count(&[self.emergency_contacts.iter().any(|c| !c.full_name.trim().is_empty() && filled(&c.phone))]);
        let pay = match rules {
            Some(r) if r.pay == PayKind::None => StepProgress { filled: 0, total: 0 },
            Some(r) if r.pay == PayKind::Support || r.pay == PayKind::Invoice => count(&[self.pay_amount_mxn.is_some()]),
            _ => {
                let r = rules.unwrap_or(catalog::builtin_rules("indefinite").expect("built in"));
                let mut items = vec![self.pay_amount_mxn.is_some(), filled(&self.pay_method)];
                if self.pay_method.as_deref() == Some("transfer") {
                    items.push(secrets.clabe || filled(&self.clabe));
                }
                if r.tax_data {
                    items.extend([secrets.rfc || filled(&self.rfc), filled(&self.tax_regime), filled(&self.tax_zip)]);
                }
                if r.imss {
                    items.push(secrets.nss || filled(&self.nss));
                }
                count(&items)
            }
        };
        let (f, t) = [personal, job, emergency, pay].iter().fold((0, 0), |(f, t), s| (f + s.filled, t + s.total));
        Progress { personal, job, emergency, pay, percent: if t == 0 { 100 } else { f * 100 / t } }
    }
}

/// Which covered identifiers a record has stored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Secrets {
    pub curp: bool,
    pub rfc: bool,
    pub nss: bool,
    pub clabe: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(code: &str) -> Option<Rules> {
        catalog::builtin_rules(code)
    }

    fn minimal() -> PersonData {
        PersonData { first_names: "Ana".into(), position_id: Some("pos_1".into()), modality: "indefinite".into(), status: "active".into(), ..Default::default() }
    }

    #[test]
    fn a_name_a_position_and_a_modality_are_enough_to_save() {
        assert!(minimal().validate(rules("indefinite"), "2026-10-07").iter().all(|i| !i.blocking));
        let empty = PersonData { status: "active".into(), ..Default::default() };
        let codes: Vec<_> = empty.validate(None, "2026-10-07").into_iter().filter(|i| i.blocking).map(|i| i.code).collect();
        assert_eq!(codes, vec!["name_missing", "position_missing", "modality_unknown"]);
    }

    #[test]
    fn identifiers_are_checked_and_compared_with_each_other() {
        let mut p = minimal();
        p.curp = Some("HEGG560427MVZRRL04".into());
        p.birth_date = Some("1956-04-27".into());
        p.rfc = Some("HEGG560427AB1".into()); // wrong check digit
        p.nss = Some("12345678904".into());
        p.clabe = Some("002010077777777771".into());
        let issues = p.validate(rules("indefinite"), "2026-10-07");
        let blocking: Vec<_> = issues.iter().filter(|i| i.blocking).map(|i| i.code).collect();
        assert_eq!(blocking, vec!["rfc_invalid", "nss_invalid"]);
        assert!(!issues.iter().any(|i| i.code == "curp_birth_mismatch"));

        p.birth_date = Some("1956-04-28".into());
        p.rfc = Some("GODE561231GR8".into());
        p.nss = None; // stored and covered: not checked again
        let codes: Vec<_> = p.validate(rules("indefinite"), "2026-10-07").into_iter().map(|i| (i.code, i.blocking)).collect();
        assert_eq!(codes, vec![("curp_birth_mismatch", false), ("rfc_curp_mismatch", false)]);
    }

    #[test]
    fn dates_hours_and_contacts() {
        let mut p = minimal();
        p.start_date = Some("2024-02-30".into());
        p.end_date = Some("2023-01-01".into());
        p.weekly_hours = Some(56);
        p.emergency_contacts = vec![EmergencyContact { full_name: " ".into(), phone: Some("123".into()), ..Default::default() }];
        let codes: Vec<_> = p.validate(rules("fixed_term"), "2026-10-07").into_iter().map(|i| (i.code, i.field, i.blocking)).collect();
        assert!(codes.contains(&("date_invalid", "start_date".into(), true)));
        assert!(codes.contains(&("hours_over_legal", "weekly_hours".into(), false)));
        assert!(codes.contains(&("contact_name_missing", "emergency_contacts[0].full_name".into(), true)));
        assert!(codes.contains(&("phone_invalid", "emergency_contacts[0].phone".into(), true)));

        p.start_date = Some("2024-02-29".into());
        let codes: Vec<_> = p.validate(rules("fixed_term"), "2026-10-07").into_iter().map(|i| i.code).collect();
        assert!(codes.contains(&"end_before_start"), "{codes:?}");
        assert!(!codes.contains(&"date_invalid"), "2024 is a leap year");
    }

    #[test]
    fn a_fixed_term_job_should_say_when_it_ends() {
        let mut p = minimal();
        p.modality = "fixed_term".into();
        assert!(p.validate(rules("fixed_term"), "2026-10-07").iter().any(|i| i.code == "end_date_missing" && !i.blocking));
        p.status = "left".into();
        let codes: Vec<_> = p.validate(rules("fixed_term"), "2026-10-07").into_iter().map(|i| i.code).collect();
        assert_eq!(codes, vec!["left_date_missing"], "someone who left needs no end date, but the day they left");
    }

    #[test]
    fn years_and_ages() {
        assert_eq!(years_between("1956-04-27", "2026-04-26"), Some(69));
        assert_eq!(years_between("1956-04-27", "2026-04-27"), Some(70));
        let mut p = minimal();
        p.birth_date = Some("2015-01-01".into());
        assert!(p.validate(rules("indefinite"), "2026-10-07").iter().any(|i| i.code == "age_out_of_range"));
    }

    #[test]
    fn progress_counts_only_what_applies_to_the_modality() {
        let mut p = minimal();
        p.modality = "volunteer".into();
        let v = p.progress(rules("volunteer"), Secrets::default());
        assert_eq!(v.pay.total, 0, "a volunteer has no pay step");

        let e = minimal().progress(rules("indefinite"), Secrets::default());
        assert_eq!(e.pay.total, 6, "amount, method, RFC, regime, fiscal zip, NSS");
        let f = minimal().progress(rules("fees"), Secrets::default());
        assert_eq!(f.pay.total, 5, "fees: no NSS");

        let mut q = minimal();
        q.pay_amount_mxn = Some(9_000);
        q.pay_method = Some("transfer".into());
        let with = q.progress(rules("indefinite"), Secrets { rfc: true, nss: true, ..Default::default() });
        assert_eq!((with.pay.filled, with.pay.total), (4, 7), "a transfer asks for the CLABE");
        assert_eq!(q.monthly_pay(), Some(9_000));
        q.pay_period = Some("weekly".into());
        q.pay_amount_mxn = Some(2_100);
        assert_eq!(q.monthly_pay(), Some(9_000));
    }
}
