//! The fixed catalogs of the institution (ADR-031): its state, legal form and federal registries travel as short
//! codes; the screen and the sheet of the AI turn them into words. The profile and the first start both use them.

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

// ------------------------------------------------------------------ the attention profile (ADR-033 §2)

/// Whom the institution serves, by stage of life. Several may be marked.
pub const POPULATIONS: &[&str] = &["early_childhood", "childhood", "adolescence", "youth", "adults", "older_adults"];
/// The stages of life under 18.
pub const MINORS: &[&str] = &["early_childhood", "childhood", "adolescence"];
/// Whether it serves women, men or both.
pub const SEXES_SERVED: &[&str] = &["women", "men", "all"];
/// How it serves them. Several may be marked.
pub const MODALITIES: &[&str] = &["residential", "day_care", "outpatient", "community", "home_care"];
/// What it attends to. Several may be marked.
pub const CARE_AREAS: &[&str] = &[
    "care", "health", "disability", "education", "food", "violence", "addictions", "street", "migration", "mental_health", "other",
];

/// The kind of institution the modules still go by (`elderly_home`, `children_home`, `other`), from whom it serves:
/// only older adults (with or without other adults) is an elderly home; only minors (with or without youth) is a
/// children's home; anything mixed, or nothing marked, is `other`.
pub fn kind_of(populations: &[String]) -> &'static str {
    let has = |list: &[&str]| populations.iter().any(|p| list.contains(&p.as_str()));
    match (has(MINORS), has(&["older_adults"])) {
        (true, false) => "children_home",
        (false, true) => "elderly_home",
        _ => "other",
    }
}

/// What a kind of institution means as an attention profile, for the data written before the profile existed and
/// for the first start, which still asks for the kind: (populations, modalities, areas).
pub fn attention_of_kind(kind: &str) -> (&'static [&'static str], &'static [&'static str], &'static [&'static str]) {
    match kind {
        "elderly_home" => (&["older_adults"], &["residential"], &["care"]),
        "children_home" => (&["early_childhood", "childhood", "adolescence"], &["residential"], &["care"]),
        _ => (&[], &[], &[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_kind_follows_whom_it_serves() {
        assert_eq!(kind_of(&v(&["older_adults"])), "elderly_home");
        assert_eq!(kind_of(&v(&["adults", "older_adults"])), "elderly_home");
        assert_eq!(kind_of(&v(&["childhood", "adolescence", "youth"])), "children_home");
        assert_eq!(kind_of(&v(&["childhood", "older_adults"])), "other", "mixed");
        assert_eq!(kind_of(&v(&["adults"])), "other");
        assert_eq!(kind_of(&[]), "other");
    }

    #[test]
    fn every_kind_turns_into_a_profile_that_gives_the_kind_back() {
        for kind in ["elderly_home", "children_home"] {
            let (populations, modalities, areas) = attention_of_kind(kind);
            assert_eq!(kind_of(&v(populations)), kind);
            assert!(populations.iter().all(|p| POPULATIONS.contains(p)) && modalities.iter().all(|m| MODALITIES.contains(m)) && areas.iter().all(|a| CARE_AREAS.contains(a)));
        }
        assert_eq!(attention_of_kind("other"), (&[][..], &[][..], &[][..]));
    }
}
