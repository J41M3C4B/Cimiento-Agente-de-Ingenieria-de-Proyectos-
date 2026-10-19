//! A group of spaces or of equipment (ADR-030): one kind, how many, and how many are in each state. «Baños · planta
//! alta · 4 → 3 bien, 1 mal». The states may add up to less than the count: the rest have not been checked, and
//! not knowing is not «good».

use super::catalog;
use super::{text, Issue};
use serde::{Deserialize, Serialize};

pub const MAX_COUNT: i64 = 1_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct States {
    pub good: i64,
    pub fair: i64,
    pub poor: i64,
    pub unusable: i64,
}

impl States {
    pub fn checked(&self) -> i64 {
        self.good + self.fair + self.poor + self.unusable
    }
    /// Those that need a real repair or cannot be used.
    pub fn bad(&self) -> i64 {
        self.poor + self.unusable
    }
    /// Every state but `good`.
    pub fn not_good(&self) -> i64 {
        self.fair + self.poor + self.unusable
    }
    pub fn all(count: i64, state: &str) -> States {
        let mut s = States::default();
        match state {
            "good" => s.good = count,
            "fair" => s.fair = count,
            "poor" => s.poor = count,
            "unusable" => s.unusable = count,
            _ => {}
        }
        s
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpaceData {
    pub kind: String,
    /// A name of their own («Baños de mujeres»); required for `other`.
    pub label: Option<String>,
    /// -1 basement, 0 ground floor, 1 first floor…
    pub floor: i64,
    pub count: i64,
    #[serde(flatten)]
    pub states: States,
    pub problems: Vec<String>,
    /// Whether a person in a wheelchair can use them.
    pub accessible: Option<bool>,
    pub beds: Option<i64>,
    pub hospital_beds: Option<i64>,
    pub grab_bars: Option<bool>,
    pub accessible_shower: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EquipmentData {
    pub kind: String,
    pub label: Option<String>,
    pub count: i64,
    #[serde(flatten)]
    pub states: States,
    pub notes: Option<String>,
}

fn check_group(kind: &str, kinds: &[&str], label: &Option<String>, count: i64, states: &States, v: &mut Vec<Issue>) {
    let mut add = |code: &'static str, field: &str, blocking: bool| v.push(Issue { code, field: field.into(), blocking });
    if !kinds.contains(&kind) {
        add("code_unknown", "kind", true);
    }
    if kind == "other" && text(label).is_none() {
        add("label_missing", "label", true);
    }
    if count < 1 {
        add("count_invalid", "count", true);
    } else if count > MAX_COUNT {
        add("number_too_large", "count", true);
    }
    for (field, n) in [("good", states.good), ("fair", states.fair), ("poor", states.poor), ("unusable", states.unusable)] {
        if n < 0 {
            add("negative_number", field, true);
        }
    }
    if count >= 1 && states.checked() > count {
        add("states_exceed_count", "states", true);
    }
    if count >= 1 && states.checked() >= 0 && states.checked() < count {
        add("states_unchecked", "states", false);
    }
}

impl SpaceData {
    /// Blocking problems and heads-ups. `floors` is how many floors the building has, if known.
    pub fn validate(&self, floors: Option<i64>) -> Vec<Issue> {
        let mut v = Vec::new();
        check_group(&self.kind, catalog::SPACE_KINDS, &self.label, self.count, &self.states, &mut v);
        let mut add = |code: &'static str, field: &str, blocking: bool| v.push(Issue { code, field: field.into(), blocking });
        if !(catalog::LOWEST_FLOOR..=catalog::HIGHEST_FLOOR).contains(&self.floor) {
            add("floor_invalid", "floor", true);
        } else if floors.is_some_and(|f| self.floor >= f) {
            add("floor_above_building", "floor", false);
        }
        if self.problems.iter().any(|p| !catalog::PROBLEMS.contains(&p.as_str())) {
            add("code_unknown", "problems", true);
        }
        for (field, n) in [("beds", self.beds), ("hospital_beds", self.hospital_beds)] {
            if n.is_some_and(|x| x < 0) {
                add("negative_number", field, true);
            } else if n.is_some_and(|x| x > 10 * MAX_COUNT) {
                add("number_too_large", field, true);
            }
        }
        if let (Some(all), Some(hospital)) = (self.beds, self.hospital_beds) {
            if hospital > all {
                add("hospital_beds_exceed", "hospital_beds", true);
            }
        }
        if !self.problems.is_empty() && self.states.checked() > 0 && self.states.not_good() == 0 {
            add("problems_but_good", "problems", false);
        }
        v
    }

    pub fn holds_beds(&self) -> bool {
        catalog::WITH_BEDS.contains(&self.kind.as_str())
    }

    /// Drops the data that do not belong to the kind (beds of a kitchen, grab bars of a bedroom) and trims texts.
    pub fn tidy(&mut self) {
        if !self.holds_beds() {
            self.beds = None;
            self.hospital_beds = None;
        }
        if self.kind != "bathroom" {
            self.grab_bars = None;
            self.accessible_shower = None;
        }
        let mut seen: Vec<String> = Vec::new();
        self.problems.retain(|p| !seen.contains(p) && {
            seen.push(p.clone());
            true
        });
        self.label = text(&self.label).map(String::from);
        self.notes = text(&self.notes).map(String::from);
    }
}

impl EquipmentData {
    pub fn validate(&self) -> Vec<Issue> {
        let mut v = Vec::new();
        check_group(&self.kind, catalog::EQUIPMENT_KINDS, &self.label, self.count, &self.states, &mut v);
        v
    }

    pub fn tidy(&mut self) {
        self.label = text(&self.label).map(String::from);
        self.notes = text(&self.notes).map(String::from);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bathrooms(good: i64, poor: i64) -> SpaceData {
        SpaceData { kind: "bathroom".into(), floor: 1, count: 4, states: States { good, poor, ..Default::default() }, ..Default::default() }
    }

    fn codes(v: Vec<Issue>) -> Vec<(&'static str, bool)> {
        v.into_iter().map(|i| (i.code, i.blocking)).collect()
    }

    #[test]
    fn four_bathrooms_three_good_one_poor_is_valid() {
        let mut b = bathrooms(3, 1);
        b.problems = vec!["leaks".into(), "grab_bars".into()];
        assert!(b.validate(Some(2)).is_empty());
        assert_eq!((b.states.bad(), b.states.not_good()), (1, 1));
    }

    #[test]
    fn states_cannot_add_up_to_more_than_the_count_and_fewer_is_a_heads_up() {
        assert_eq!(codes(bathrooms(4, 1).validate(None)), vec![("states_exceed_count", true)]);
        assert_eq!(codes(bathrooms(1, 1).validate(None)), vec![("states_unchecked", false)]);
    }

    #[test]
    fn kind_label_floor_beds_and_problems_are_checked() {
        let s = SpaceData {
            kind: "other".into(),
            count: 0,
            floor: 25,
            problems: vec!["ghosts".into()],
            beds: Some(2),
            hospital_beds: Some(3),
            ..Default::default()
        };
        assert_eq!(
            codes(s.validate(None)),
            vec![("label_missing", true), ("count_invalid", true), ("floor_invalid", true), ("code_unknown", true), ("hospital_beds_exceed", true)]
        );
        let unknown = SpaceData { kind: "ballroom".into(), count: 1, states: States::all(1, "good"), ..Default::default() };
        assert_eq!(codes(unknown.validate(None)), vec![("code_unknown", true)]);
    }

    #[test]
    fn heads_ups_for_a_floor_above_the_building_and_problems_of_good_spaces() {
        let mut s = bathrooms(4, 0);
        assert_eq!(codes(s.validate(Some(1))), vec![("floor_above_building", false)]);
        s.floor = 0;
        s.problems = vec!["paint".into()];
        assert_eq!(codes(s.validate(Some(1))), vec![("problems_but_good", false)]);
    }

    #[test]
    fn tidy_keeps_only_what_belongs_to_the_kind() {
        let mut kitchen = SpaceData { kind: "kitchen".into(), count: 1, beds: Some(3), grab_bars: Some(true), problems: vec!["leaks".into(), "leaks".into()], label: Some("  ".into()), ..Default::default() };
        kitchen.tidy();
        assert_eq!((kitchen.beds, kitchen.grab_bars, kitchen.label.clone()), (None, None, None));
        assert_eq!(kitchen.problems, vec!["leaks"]);
        let mut bedroom = SpaceData { kind: "bedroom".into(), count: 6, beds: Some(18), hospital_beds: Some(4), grab_bars: Some(false), ..Default::default() };
        bedroom.tidy();
        assert_eq!((bedroom.beds, bedroom.hospital_beds, bedroom.grab_bars), (Some(18), Some(4), None));
    }

    #[test]
    fn equipment_follows_the_same_rules() {
        let washer = EquipmentData { kind: "washer".into(), count: 2, states: States { good: 1, unusable: 1, ..Default::default() }, ..Default::default() };
        assert!(washer.validate().is_empty());
        let other = EquipmentData { kind: "other".into(), count: 1, ..Default::default() };
        assert_eq!(codes(other.validate()), vec![("label_missing", true), ("states_unchecked", false)]);
    }
}
