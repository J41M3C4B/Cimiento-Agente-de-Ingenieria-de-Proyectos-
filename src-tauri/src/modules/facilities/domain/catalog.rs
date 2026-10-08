//! The fixed catalogs of the facilities (ADR-030). Values travel as short codes; the screen and the sheet of the AI
//! turn them into words.

/// The kind of institution: it decides which spaces and equipment are suggested first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    ElderlyHome,
    ChildrenHome,
    Other,
}

impl Flavor {
    /// The flavor of the kind of institution the core says (`core::institution::kind`).
    pub fn from_kind(kind: Option<&str>) -> Self {
        match kind {
            Some("elderly_home") => Flavor::ElderlyHome,
            Some("children_home") => Flavor::ChildrenHome,
            _ => Flavor::Other,
        }
    }
}

pub const SPACE_KINDS: &[&str] = &[
    "bedroom", "bathroom", "kitchen", "dining", "laundry", "infirmary", "therapy", "living", "classroom", "play", "chapel", "office", "storage",
    "yard", "roof", "parking", "other",
];
/// Kinds that hold beds.
pub const WITH_BEDS: &[&str] = &["bedroom", "infirmary"];

pub const EQUIPMENT_KINDS: &[&str] = &[
    "wheelchair", "patient_lift", "pressure_mattress", "oxygen", "washer", "dryer", "fridge", "freezer", "stove", "water_heater", "generator",
    "solar_panels", "water_pump", "computer", "vehicle", "other",
];

/// What is wrong with a space that is not good.
pub const PROBLEMS: &[&str] = &[
    "leaks", "damp", "roof", "electrical", "drainage", "floor", "cracks", "doors_windows", "grab_bars", "ventilation", "pests", "furniture", "paint",
];
/// Problems that may make a building unsafe.
pub const STRUCTURAL: &[&str] = &["cracks", "electrical"];

/// How people go from one floor to another. `none`: only stairs (an empty list is «not said»).
pub const FLOOR_ACCESS: &[&str] = &["ramp", "elevator", "stair_lift", "none"];
/// Own, in «comodato» (lent on paper for some years), rented, lent without papers, other.
pub const TENURES: &[&str] = &["own", "loan", "rent", "borrowed", "other"];
/// Tenures with an end date.
pub const ENDING_TENURES: &[&str] = &["loan", "rent"];

pub const WATER_SOURCES: &[&str] = &["network", "truck", "well", "rain"];
pub const FREQUENCY: &[&str] = &["never", "sometimes", "often"];
pub const GAS: &[&str] = &["lp_tank", "lp_cylinders", "natural", "none"];
pub const DRAINAGE: &[&str] = &["sewer", "septic", "none"];
/// The internal civil-protection program (Programa Interno de Protección Civil).
pub const INTERNAL_PROGRAM: &[&str] = &["yes", "in_progress", "no"];

/// The lowest and highest floor a space may be on: a basement and twenty floors.
pub const LOWEST_FLOOR: i64 = -1;
pub const HIGHEST_FLOOR: i64 = 19;

/// The spaces in the order the screen suggests them, by kind of institution.
pub fn suggested_spaces(flavor: Flavor) -> Vec<&'static str> {
    let first: &[&str] = match flavor {
        Flavor::ElderlyHome => &["bedroom", "bathroom", "kitchen", "dining", "infirmary", "therapy", "laundry", "living", "chapel"],
        Flavor::ChildrenHome => &["bedroom", "bathroom", "kitchen", "dining", "classroom", "play", "laundry", "living", "infirmary"],
        Flavor::Other => &["bedroom", "bathroom", "kitchen", "dining", "laundry", "living", "infirmary"],
    };
    first.iter().copied().chain(SPACE_KINDS.iter().copied().filter(|k| !first.contains(k))).collect()
}

/// The equipment in the order the screen suggests it, by kind of institution.
pub fn suggested_equipment(flavor: Flavor) -> Vec<&'static str> {
    let first: &[&str] = match flavor {
        Flavor::ElderlyHome => &["wheelchair", "patient_lift", "pressure_mattress", "oxygen", "washer", "dryer", "fridge", "stove", "water_heater"],
        Flavor::ChildrenHome => &["washer", "dryer", "fridge", "stove", "water_heater", "computer", "vehicle"],
        Flavor::Other => &["washer", "fridge", "stove", "water_heater"],
    };
    first.iter().copied().chain(EQUIPMENT_KINDS.iter().copied().filter(|k| !first.contains(k))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions_hold_every_kind_once_with_the_usual_ones_first() {
        for flavor in [Flavor::ElderlyHome, Flavor::ChildrenHome, Flavor::Other] {
            let spaces = suggested_spaces(flavor);
            assert_eq!(spaces.len(), SPACE_KINDS.len());
            assert!(SPACE_KINDS.iter().all(|k| spaces.contains(k)));
            let equipment = suggested_equipment(flavor);
            assert_eq!(equipment.len(), EQUIPMENT_KINDS.len());
            assert_eq!(spaces.last(), Some(&"other"));
        }
        assert_eq!(suggested_spaces(Flavor::ChildrenHome)[4], "classroom");
        assert_eq!(suggested_equipment(Flavor::ElderlyHome)[0], "wheelchair");
    }
}
