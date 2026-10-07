//! A position (puesto): the seat, not the person. Its title, area and duties may reach the AI, because they say
//! what the institution needs, not who does it.

use super::catalog;
use super::person::{Issue, MAX_MXN};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PositionInput {
    pub title: String,
    pub area: Option<String>,
    /// What the position does, in the institution's words (it goes through the scanner).
    pub duties: Option<String>,
    pub default_modality: Option<String>,
    pub default_schedule: Option<String>,
    /// A reference monthly pay for the position (the tabulator).
    pub reference_pay_mxn: Option<i64>,
    /// How many seats the institution has for it: the empty ones are a need.
    pub authorized_seats: Option<i64>,
    /// The position it reports to; never a person.
    pub reports_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Position {
    pub id: String,
    #[serde(flatten)]
    pub input: PositionInput,
    pub active: bool,
}

impl PositionInput {
    pub fn validate(&self, modality_known: impl Fn(&str) -> bool) -> Vec<Issue> {
        let mut v = Vec::new();
        let mut add = |code: &'static str, field: &str, blocking: bool| v.push(Issue { code, field: field.to_string(), blocking });
        if self.title.trim().is_empty() {
            add("label_missing", "title", true);
        }
        if self.area.as_deref().is_some_and(|a| !a.is_empty() && !catalog::AREAS.contains(&a)) {
            add("code_unknown", "area", true);
        }
        if self.default_schedule.as_deref().is_some_and(|a| !a.is_empty() && !catalog::SCHEDULES.contains(&a)) {
            add("code_unknown", "default_schedule", true);
        }
        if self.default_modality.as_deref().is_some_and(|m| !m.is_empty() && !modality_known(m)) {
            add("modality_unknown", "default_modality", true);
        }
        match self.reference_pay_mxn {
            Some(a) if a < 0 => add("negative_number", "reference_pay_mxn", true),
            Some(a) if a > MAX_MXN => add("amount_too_large", "reference_pay_mxn", true),
            _ => {}
        }
        if self.authorized_seats.is_some_and(|n| !(0..=10_000).contains(&n)) {
            add("negative_number", "authorized_seats", true);
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_position_needs_a_title_and_known_codes() {
        let ok = PositionInput { title: "Cuidadora de noche".into(), area: Some("care".into()), authorized_seats: Some(3), ..Default::default() };
        assert!(ok.validate(|_| true).is_empty());
        let bad = PositionInput { title: " ".into(), area: Some("garden".into()), default_modality: Some("x".into()), authorized_seats: Some(-1), ..Default::default() };
        let codes: Vec<_> = bad.validate(|m| m == "indefinite").into_iter().map(|i| i.code).collect();
        assert_eq!(codes, vec!["label_missing", "code_unknown", "modality_unknown", "negative_number"]);
    }
}
