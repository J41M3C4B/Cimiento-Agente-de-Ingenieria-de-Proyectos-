//! The waiting list (ADR-029): who asked to come in and when. The name is not required: what counts as evidence of
//! demand is how many, of what group and since when. Admitting a request makes a record of it.

use super::catalog;
use super::person::Issue;
use crate::common::contact::phone_ok;
use crate::common::dates::date_is_valid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WaitlistInput {
    pub requested_on: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub sex: Option<String>,
    pub approx_age: Option<i64>,
    pub dependency: Option<String>,
    pub reason: Option<String>,
    pub status: String,
}

impl WaitlistInput {
    pub fn validate(&self) -> Vec<Issue> {
        let mut v = Vec::new();
        let mut add = |code: &'static str, field: &str| v.push(Issue { code, field: field.to_string(), blocking: true });
        if !date_is_valid(&self.requested_on) {
            add("date_invalid", "requested_on");
        }
        for (field, value, allowed) in [
            ("sex", &self.sex, catalog::SEXES),
            ("dependency", &self.dependency, catalog::DEPENDENCY),
            ("reason", &self.reason, catalog::ADMISSION_REASONS),
        ] {
            if value.as_deref().is_some_and(|x| !x.is_empty() && !allowed.contains(&x)) {
                add("code_unknown", field);
            }
        }
        if !catalog::WAITLIST_STATUSES.contains(&self.status.as_str()) {
            add("code_unknown", "status");
        }
        if self.approx_age.is_some_and(|a| !(0..=120).contains(&a)) {
            add("age_invalid", "approx_age");
        }
        if self.phone.as_deref().is_some_and(|p| !p.trim().is_empty() && !phone_ok(p)) {
            add("phone_invalid", "phone");
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_needs_only_its_date() {
        let ok = WaitlistInput { requested_on: "2026-09-01".into(), status: "waiting".into(), ..Default::default() };
        assert!(ok.validate().is_empty());
        let bad = WaitlistInput { requested_on: "ayer".into(), status: "maybe".into(), approx_age: Some(300), ..Default::default() };
        let codes: Vec<_> = bad.validate().into_iter().map(|i| i.field).collect();
        assert_eq!(codes, vec!["requested_on", "status", "approx_age"]);
    }
}
