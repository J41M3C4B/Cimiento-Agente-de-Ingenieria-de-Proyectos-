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
/// The codes of `STATES`, in the same order (the options of a select).
pub const STATE_CODES: &[&str] = &[
    "ags", "bc", "bcs", "camp", "coah", "col", "chis", "chih", "cdmx", "dgo", "gto", "gro", "hgo", "jal", "mex", "mich", "mor",
    "nay", "nl", "oax", "pue", "qro", "qroo", "slp", "sin", "son", "tab", "tamps", "tlax", "ver", "yuc", "zac",
];
/// The legal forms watched over by the Junta de Asistencia (o Beneficencia) Privada of their state.
pub const UNDER_A_JUNTA: &[&str] = &["iap", "ibp", "abp"];
/// The tax regime of the institution (its constancia de situación fiscal): personas morales con fines no
/// lucrativos (603), régimen general de ley (601), or another.
pub const TAX_REGIMES: &[&str] = &["non_profit", "general", "other"];
/// The activity an authorized donee is authorized for (rubro autorizado, Ley del ISR art. 79 and 82).
pub const DONEE_CATEGORIES: &[&str] = &[
    "assistance", "education", "research", "culture", "scholarships", "ecology", "species", "support_donees", "public_works",
    "libraries", "museums", "social_development",
];
/// The most people a quick figure may say.
pub const MAX_ESTIMATE: i64 = 100_000;
/// The roles of the board of trustees or governing body (ADR-033 §3): presidencia, vicepresidencia, secretaría,
/// tesorería, vocal, or another one the institution names.
pub const BOARD_ROLES: &[&str] = &["president", "vice_president", "secretary", "treasurer", "member", "other"];
/// What a bank account of the institution is used for: receiving donations, paying for its operation.
pub const BANK_PURPOSES: &[&str] = &["donations", "operations"];

// ------------------------------------------------------------------ the record of the institution (ADR-033 §5)

/// The state of a document of the record: the institution has it, it is in process, it is missing, or it does not
/// apply. The file is optional: having it without uploading it counts.
pub const RECORD_STATUSES: &[&str] = &["have", "in_progress", "missing", "not_applicable"];

/// When a type of document applies to an institution, from what it says of itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applies {
    Always,
    /// An authorized donee, or one in process (`authorized_donee`).
    Donee,
    /// With the CLUNI, or in process (`cluni`).
    Cluni,
    /// Watched over by a Junta de Asistencia (o Beneficencia) Privada (`UNDER_A_JUNTA`).
    Junta,
    /// It serves girls, boys or teenagers (the Registro Nacional de Centros de Asistencia Social).
    Minors,
    /// People live or spend the day there (the sanitary notice).
    Housing,
}

impl Applies {
    /// The datum of the institution it depends on: that datum is indispensable (ADR-042), or the Expediente would ask
    /// for the wrong documents.
    pub const fn field(self) -> Option<&'static str> {
        match self {
            Applies::Always => None,
            Applies::Donee => Some("institution.authorized_donee"),
            Applies::Cluni => Some("institution.cluni"),
            Applies::Junta => Some("institution.legal_form"),
            Applies::Minors => Some("institution.populations"),
            Applies::Housing => Some("institution.modalities"),
        }
    }
}

/// The documents of the record, in the order the screen lists them, and when each one applies (audit §4.1). A
/// document the institution names itself is an `other`, which is not listed here.
pub const RECORD_TYPES: &[(&str, Applies)] = &[
    ("articles_of_incorporation", Applies::Always),
    ("bylaws_amendments", Applies::Always),
    ("power_of_attorney", Applies::Always),
    ("tax_status_certificate", Applies::Always),
    ("tax_compliance_opinion", Applies::Always),
    ("donee_authorization", Applies::Donee),
    ("cluni_certificate", Applies::Cluni),
    ("junta_registration", Applies::Junta),
    ("social_care_registry", Applies::Minors),
    ("sanitary_notice", Applies::Housing),
    ("land_use_license", Applies::Always),
    ("civil_protection_program", Applies::Always),
    ("privacy_notice", Applies::Always),
    ("internal_rules", Applies::Always),
    ("org_chart", Applies::Always),
    ("financial_statements", Applies::Always),
    ("annual_report", Applies::Always),
    ("junta_budget", Applies::Junta),
];

/// A document the institution names itself.
pub const RECORD_OTHER: &str = "other";

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

    #[test]
    fn the_state_codes_are_the_states() {
        assert_eq!(STATE_CODES.len(), STATES.len());
        assert!(STATES.iter().zip(STATE_CODES).all(|((c, _), code)| c == code));
    }

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
