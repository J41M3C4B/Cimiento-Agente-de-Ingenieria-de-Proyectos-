//! The fixed catalogs of the staff module (ADR-027). Values travel as short codes; the screen and the sheet of the
//! AI turn them into words. The rules of each modality live here and nowhere else.

use serde::Serialize;

/// The kind of relation a modality creates. It decides where the money of a person counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// A labor relation with salary (Ley Federal del Trabajo).
    Employee,
    /// Paid without a labor relation: fees (honorarios) or assimilated to salaries.
    Fees,
    /// A sister or brother of the congregation: no labor relation; an optional contribution.
    Religious,
    Volunteer,
    /// Social service or internship: an optional grant.
    Trainee,
    /// Staff of an outside company (REPSE): the company bills the institution.
    External,
}

impl Relation {
    pub fn as_str(self) -> &'static str {
        match self {
            Relation::Employee => "employee",
            Relation::Fees => "fees",
            Relation::Religious => "religious",
            Relation::Volunteer => "volunteer",
            Relation::Trainee => "trainee",
            Relation::External => "external",
        }
    }
}

/// How a person of a modality is paid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayKind {
    Salary,
    Fees,
    Assimilated,
    /// A contribution to the congregation or a grant: optional, not payroll.
    Support,
    /// The outside company's invoice.
    Invoice,
    None,
}

/// The rules of a modality: what the law and the calculations do with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rules {
    pub relation: Relation,
    /// Aguinaldo and vacation premium of the law.
    pub lft_benefits: bool,
    /// Registered with the IMSS (asks for the NSS).
    pub imss: bool,
    pub pay: PayKind,
    /// A fixed-term job: it should say when it ends.
    pub needs_end_date: bool,
    /// Asks for the tax data (RFC, regime, fiscal zip).
    pub tax_data: bool,
}

/// The modalities the app knows. An institution can add its own, each one «behaving as» one of these.
pub const BUILTIN_MODALITIES: [(&str, Rules); 10] = [
    ("indefinite", Rules { relation: Relation::Employee, lft_benefits: true, imss: true, pay: PayKind::Salary, needs_end_date: false, tax_data: true }),
    ("fixed_term", Rules { relation: Relation::Employee, lft_benefits: true, imss: true, pay: PayKind::Salary, needs_end_date: true, tax_data: true }),
    ("project_based", Rules { relation: Relation::Employee, lft_benefits: true, imss: true, pay: PayKind::Salary, needs_end_date: false, tax_data: true }),
    ("trial", Rules { relation: Relation::Employee, lft_benefits: true, imss: true, pay: PayKind::Salary, needs_end_date: true, tax_data: true }),
    ("fees", Rules { relation: Relation::Fees, lft_benefits: false, imss: false, pay: PayKind::Fees, needs_end_date: false, tax_data: true }),
    ("assimilated", Rules { relation: Relation::Fees, lft_benefits: false, imss: false, pay: PayKind::Assimilated, needs_end_date: false, tax_data: true }),
    ("religious", Rules { relation: Relation::Religious, lft_benefits: false, imss: false, pay: PayKind::Support, needs_end_date: false, tax_data: false }),
    ("volunteer", Rules { relation: Relation::Volunteer, lft_benefits: false, imss: false, pay: PayKind::None, needs_end_date: false, tax_data: false }),
    ("social_service", Rules { relation: Relation::Trainee, lft_benefits: false, imss: false, pay: PayKind::Support, needs_end_date: true, tax_data: false }),
    ("external", Rules { relation: Relation::External, lft_benefits: false, imss: false, pay: PayKind::Invoice, needs_end_date: false, tax_data: false }),
];

pub fn builtin_rules(code: &str) -> Option<Rules> {
    BUILTIN_MODALITIES.iter().find(|(c, _)| *c == code).map(|(_, r)| *r)
}

/// Reads a modality from the words of the old roster selector («De planta», «Honorarios»…). `None` if unknown.
pub fn modality_from_label(label: &str) -> Option<&'static str> {
    let s = label.trim().to_lowercase();
    if let Some((code, _)) = BUILTIN_MODALITIES.iter().find(|(c, _)| *c == s) {
        return Some(code);
    }
    let has = |w: &[&str]| w.iter().any(|w| s.contains(w));
    Some(if has(&["asimilado"]) {
        "assimilated"
    } else if has(&["honorario", "factura"]) {
        "fees"
    } else if has(&["prueba", "capacitación inicial", "capacitacion inicial"]) {
        "trial"
    } else if has(&["por obra"]) {
        "project_based"
    } else if has(&["tiempo definido", "temporal", "eventual", "determinado"]) {
        "fixed_term"
    } else if has(&["planta", "base", "indefinido", "permanente"]) {
        "indefinite"
    } else if has(&["religios", "congregación", "congregacion", "hermana", "madre superiora"]) {
        "religious"
    } else if has(&["voluntari"]) {
        "volunteer"
    } else if has(&["servicio social", "práctica", "practica", "becari"]) {
        "social_service"
    } else if has(&["externa", "outsourcing", "repse", "subcontrat"]) {
        "external"
    } else {
        return None;
    })
}

pub const AREAS: &[&str] =
    &["care", "health", "kitchen", "cleaning", "laundry", "administration", "social_work", "psychology", "rehabilitation", "education", "pastoral", "maintenance", "security", "other"];
pub const SCHEDULES: &[&str] = &["full_time", "part_time", "hourly", "weekends"];
pub const SHIFTS: &[&str] = &["morning", "afternoon", "night", "rotating", "h24"];
pub const WEEKDAYS: &[&str] = &["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
pub const SEXES: &[&str] = &["female", "male", "unsaid"];
pub const MARITAL: &[&str] = &["single", "married", "free_union", "divorced", "widowed", "consecrated"];
/// From least to most; the order is used to group in ranges.
pub const EDUCATION: &[&str] = &["none", "primary", "secondary", "high_school", "technical", "bachelor", "postgraduate"];
pub const STATUSES: &[&str] = &["active", "vacation", "sick_leave", "leave", "left"];
pub const LEFT_REASONS: &[&str] = &["resignation", "dismissal", "end_of_contract", "retirement", "death", "other"];
pub const RELATIONSHIPS: &[&str] = &["mother", "father", "spouse", "child", "sibling", "other_family", "friend", "congregation", "other"];
pub const PAY_PERIODS: &[&str] = &["weekly", "biweekly", "monthly"];
pub const PAY_METHODS: &[&str] = &["transfer", "cash", "check"];
/// SAT regimes a person of the staff usually has.
pub const TAX_REGIMES: &[&str] = &["605", "612", "626", "625", "616"];

/// The kind of institution, to suggest positions.
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

/// The positions a new catalog starts with: title, area.
pub fn default_positions(flavor: Flavor) -> Vec<(&'static str, &'static str)> {
    let mut v = match flavor {
        Flavor::ElderlyHome => vec![
            ("Cuidadora o cuidador", "care"),
            ("Enfermería", "health"),
            ("Medicina", "health"),
            ("Fisioterapia y rehabilitación", "rehabilitation"),
        ],
        Flavor::ChildrenHome => vec![
            ("Cuidadora o cuidador", "care"),
            ("Educadora o educador", "education"),
            ("Apoyo escolar", "education"),
            ("Enfermería", "health"),
        ],
        Flavor::Other => vec![("Cuidadora o cuidador", "care"), ("Enfermería", "health")],
    };
    v.extend([
        ("Psicología", "psychology"),
        ("Trabajo social", "social_work"),
        ("Cocina", "kitchen"),
        ("Limpieza", "cleaning"),
        ("Lavandería", "laundry"),
        ("Administración", "administration"),
        ("Pastoral", "pastoral"),
        ("Mantenimiento", "maintenance"),
        ("Vigilancia", "security"),
    ]);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_modality_has_coherent_rules() {
        for (code, r) in BUILTIN_MODALITIES {
            assert_eq!(r.lft_benefits, r.relation == Relation::Employee, "{code}: only a labor relation carries the benefits of the law");
            assert!(!r.imss || r.relation == Relation::Employee, "{code}");
            assert_eq!(r.pay == PayKind::None, r.relation == Relation::Volunteer, "{code}");
        }
    }

    #[test]
    fn the_old_roster_words_become_modalities() {
        for (label, code) in [
            ("De planta", Some("indefinite")),
            ("Base", Some("indefinite")),
            ("Por tiempo definido", Some("fixed_term")),
            ("Eventual", Some("fixed_term")),
            ("Por obra determinada", Some("project_based")),
            ("Periodo de prueba", Some("trial")),
            ("Honorarios", Some("fees")),
            ("Asimilados a salarios", Some("assimilated")),
            ("Religiosa de la congregación", Some("religious")),
            ("Voluntariado", Some("volunteer")),
            ("Servicio social", Some("social_service")),
            ("Empresa externa", Some("external")),
            ("fees", Some("fees")),
            ("Otro arreglo", None),
        ] {
            assert_eq!(modality_from_label(label), code, "{label}");
        }
    }
}
