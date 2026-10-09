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
