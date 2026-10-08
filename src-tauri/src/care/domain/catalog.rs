//! The fixed catalogs of the people served (ADR-029). Values travel as short codes; the screen and the sheet of the
//! AI turn them into words. Health goes only as categories: never a written diagnosis, a medicine or a doctor.

/// The kind of institution: it decides the extra data of the record and the age bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    ElderlyHome,
    ChildrenHome,
    Other,
}

pub const SEXES: &[&str] = &["female", "male", "unsaid"];
pub const STAY_MODES: &[&str] = &["permanent", "temporary", "day_care", "respite"];
pub const REFERRED_BY: &[&str] = &["family", "self", "dif", "prosecutor", "hospital", "other_institution", "other"];
pub const ADMISSION_REASONS: &[&str] = &["no_family_network", "abandonment", "dependency", "violence", "orphanhood", "poverty", "health", "other"];
/// `active` and `hospitalized` are still served; the other two leave the counts but keep the record.
pub const STATUSES: &[&str] = &["active", "hospitalized", "discharged", "deceased"];
pub const DISCHARGE_REASONS: &[&str] = &["family_reintegration", "adoption", "adulthood", "transfer", "voluntary", "other"];
pub const DEPENDENCY: &[&str] = &["low", "medium", "high", "total"];
pub const MOBILITY: &[&str] = &["independent", "cane_walker", "wheelchair", "bedridden"];
pub const DISABILITIES: &[&str] = &["motor", "visual", "hearing", "intellectual", "psychosocial"];
pub const CHRONIC: &[&str] = &["diabetes", "hypertension", "dementia", "copd", "heart_disease", "arthritis", "kidney_disease", "cancer", "other"];
pub const CONTINENCE: &[&str] = &["continent", "partial", "incontinent"];
pub const ORIENTATION: &[&str] = &["oriented", "sometimes", "disoriented"];
/// Highest schooling (elderly home), from least to most.
pub const EDUCATION: &[&str] = &["none", "primary", "secondary", "high_school", "technical", "bachelor", "postgraduate"];
/// Current school grade (children's home).
pub const SCHOOL_GRADES: &[&str] = &[
    "preschool", "primary_1", "primary_2", "primary_3", "primary_4", "primary_5", "primary_6", "secondary_1", "secondary_2", "secondary_3",
    "high_school_1", "high_school_2", "high_school_3", "university",
];
pub const VISITS: &[&str] = &["weekly", "monthly", "rarely", "never"];
pub const RELATIONSHIPS: &[&str] = &["child", "grandchild", "sibling", "spouse", "parent", "other_family", "friend", "dif", "other"];
/// Legal situation of a girl or boy (children's home): a category only, never a file or a case number.
pub const LEGAL_STATUSES: &[&str] = &["family_custody", "dif_custody", "adoption_process", "other_process"];
pub const FEE_PAYERS: &[&str] = &["family", "own_pension", "scholarship", "exempt", "other"];
pub const PROGRAMS: &[&str] = &["imss_issste", "bienestar_elderly", "bienestar_disability", "benito_juarez", "other_program"];
pub const CONSENT_SIGNERS: &[&str] = &["self", "responsible", "dif", "other"];
pub const WAITLIST_STATUSES: &[&str] = &["waiting", "admitted", "declined", "withdrawn"];

/// Whether a status still counts as a person served.
pub fn is_served(status: &str) -> bool {
    matches!(status, "active" | "hospitalized")
}

/// The age bands of the groups, by kind of institution: (code, from, to inclusive).
pub fn age_bands(flavor: Flavor) -> &'static [(&'static str, i64, i64)] {
    match flavor {
        Flavor::ElderlyHome => &[("under_60", 0, 59), ("60_69", 60, 69), ("70_79", 70, 79), ("80_89", 80, 89), ("90_plus", 90, 200)],
        Flavor::ChildrenHome => &[("0_5", 0, 5), ("6_11", 6, 11), ("12_17", 12, 17), ("18_plus", 18, 200)],
        Flavor::Other => &[("0_17", 0, 17), ("18_59", 18, 59), ("60_plus", 60, 200)],
    }
}

pub fn band_of(flavor: Flavor, age: i64) -> &'static str {
    age_bands(flavor).iter().find(|(_, a, b)| (*a..=*b).contains(&age)).map_or("unknown", |(c, _, _)| c)
}

/// The words of a group made by code: «Mujeres de 80 a 89 años», «Niñas de 6 a 11 años».
pub fn group_label(flavor: Flavor, sex: Option<&str>, age: Option<i64>) -> String {
    let who = match (flavor, sex) {
        (Flavor::ChildrenHome, Some("female")) => match age {
            Some(a) if a >= 12 => "Adolescentes mujeres",
            _ => "Niñas",
        },
        (Flavor::ChildrenHome, Some("male")) => match age {
            Some(a) if a >= 12 => "Adolescentes hombres",
            _ => "Niños",
        },
        (Flavor::ElderlyHome, Some("female")) => "Mujeres",
        (Flavor::ElderlyHome, Some("male")) => "Hombres",
        (_, Some("female")) => "Mujeres",
        (_, Some("male")) => "Hombres",
        _ => "Personas",
    };
    let Some(a) = age else { return format!("{who} (edad sin registrar)") };
    let (_, from, to) = age_bands(flavor).iter().find(|(_, x, y)| (*x..=*y).contains(&a)).copied().unwrap_or(("", a, a));
    if to >= 200 {
        format!("{who} de {from} años o más")
    } else if from == 0 {
        format!("{who} de hasta {to} años")
    } else {
        format!("{who} de {from} a {to} años")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_come_from_sex_and_age_by_kind_of_institution() {
        assert_eq!(group_label(Flavor::ElderlyHome, Some("female"), Some(84)), "Mujeres de 80 a 89 años");
        assert_eq!(group_label(Flavor::ElderlyHome, Some("male"), Some(93)), "Hombres de 90 años o más");
        assert_eq!(group_label(Flavor::ChildrenHome, Some("female"), Some(8)), "Niñas de 6 a 11 años");
        assert_eq!(group_label(Flavor::ChildrenHome, Some("female"), Some(15)), "Adolescentes mujeres de 12 a 17 años");
        assert_eq!(group_label(Flavor::ChildrenHome, Some("male"), Some(3)), "Niños de hasta 5 años");
        assert_eq!(group_label(Flavor::Other, None, None), "Personas (edad sin registrar)");
        assert_eq!(band_of(Flavor::ElderlyHome, 72), "70_79");
    }
}
