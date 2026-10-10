//! The forms of the institution described as data (ADR-033 §1): what «Mi institución» asks, field by field. The
//! screen draws them; the words live in `src/i18n/es-MX.ts` under `forms.fields` with the same id.

use super::catalog::{
    CARE_AREAS, DONEE_CATEGORIES, LEGAL_FORMS, MAX_ESTIMATE, MODALITIES, OLDEST_YEAR, POPULATIONS, REGISTRY, SEXES_SERVED, STATE_CODES,
    TAX_REGIMES, UNDER_A_JUNTA,
};
use crate::common::forms::{AiUse, Condition, FieldKind, FieldSpec, FormSpec, Rule, SectionSpec, Sensitivity};

/// The latest year a field of a year may hold; the profile checks it again against the clock.
const LATEST_YEAR: i64 = 2100;
/// The oldest a person served can be said to be.
const MAX_AGE: i64 = 120;

/// The institution's own and protected: never to the AI (its contact, address, folios and keys).
const fn private(f: FieldSpec) -> FieldSpec {
    f.sensitivity(Sensitivity::InstitutionalPrivate, AiUse::Never)
}

/// The institution's own, not personal, that may reach the AI as it is (its figures, its regime).
const fn internal(f: FieldSpec) -> FieldSpec {
    f.sensitivity(Sensitivity::Internal, AiUse::AsIs)
}

/// What shows only for a donee (`authorized_donee = yes`).
const DONEE: Condition = Condition::AnyOf { field: "institution.authorized_donee", values: &["yes"] };
/// What shows once the person said whom they serve.
const SERVES: Condition = Condition::Filled { field: "institution.populations" };

/// «Su institución»: who it is and whom and how it serves. The kind the modules go by comes out of the populations.
pub const IDENTITY: FormSpec = FormSpec {
    id: "institution.identity",
    sections: &[
        SectionSpec {
            id: "who",
            columns: 1,
            fields: &[
                FieldSpec::new("institution.name", FieldKind::Text).required().used_by(&["documents", "projects", "ai"]),
                FieldSpec::new("institution.mission", FieldKind::LongText).required().used_by(&["projects", "documents", "ai"]),
            ],
        },
        SectionSpec {
            id: "attention",
            columns: 1,
            fields: &[
                FieldSpec::new("institution.populations", FieldKind::MultiSelect).options(POPULATIONS).required().used_by(&["care", "hr", "facilities", "projects", "ai"]),
                FieldSpec::new("institution.sex_served", FieldKind::Select).options(SEXES_SERVED).when(SERVES).used_by(&["care", "projects", "ai"]),
                FieldSpec::new("institution.modalities", FieldKind::MultiSelect).options(MODALITIES).required().used_by(&["care", "facilities", "projects", "ai"]),
                FieldSpec::new("institution.care_areas", FieldKind::MultiSelect).options(CARE_AREAS).used_by(&["care", "hr", "projects", "ai"]),
                FieldSpec::new("institution.services", FieldKind::LongText).used_by(&["projects", "care", "ai"]),
            ],
        },
        SectionSpec {
            id: "admission",
            columns: 2,
            fields: &[
                FieldSpec::new("institution.age_min", FieldKind::Number).range(0, MAX_AGE).when(SERVES).used_by(&["care", "projects", "ai"]),
                FieldSpec::new("institution.age_max", FieldKind::Number).range(0, MAX_AGE).when(SERVES).used_by(&["care", "projects", "ai"]),
            ],
        },
        SectionSpec {
            id: "admission_rules",
            columns: 1,
            fields: &[FieldSpec::new("institution.admission_criteria", FieldKind::LongText).used_by(&["care", "projects", "ai"])],
        },
    ],
};

/// «Contacto y ubicación»: how to reach the institution and where it is. Only the municipality and the state reach
/// the AI; the code puts the rest in the documents.
pub const CONTACT: FormSpec = FormSpec {
    id: "institution.contact",
    sections: &[
        SectionSpec {
            id: "contact",
            columns: 2,
            fields: &[
                private(FieldSpec::new("institution.contact_phone", FieldKind::Phone)).used_by(&["documents"]),
                private(FieldSpec::new("institution.contact_email", FieldKind::Email)).used_by(&["documents"]),
            ],
        },
        SectionSpec {
            id: "address",
            columns: 2,
            fields: &[
                private(FieldSpec::new("institution.street", FieldKind::Text)).required().used_by(&["documents", "finance"]),
                private(FieldSpec::new("institution.ext_number", FieldKind::Text)).used_by(&["documents", "finance"]),
                private(FieldSpec::new("institution.int_number", FieldKind::Text)).used_by(&["documents", "finance"]),
                private(FieldSpec::new("institution.neighborhood", FieldKind::Text)).used_by(&["documents", "finance"]),
                private(FieldSpec::new("institution.postal_code", FieldKind::Text)).rule(Rule::PostalCode).required().used_by(&["documents", "finance"]),
                FieldSpec::new("institution.municipality", FieldKind::Text).required().used_by(&["hr", "finance", "projects", "ai"]),
                FieldSpec::new("institution.state", FieldKind::Select).options(STATE_CODES).required().used_by(&["hr", "finance", "projects", "ai"]),
            ],
        },
    ],
};

/// «Datos legales y fiscales»: how the institution stands before the law, the SAT and its Junta.
pub const LEGAL: FormSpec = FormSpec {
    id: "institution.legal",
    sections: &[
        SectionSpec {
            id: "legal_identity",
            columns: 1,
            fields: &[
                FieldSpec::new("institution.legal_name", FieldKind::Text).required().used_by(&["documents", "finance", "projects", "ai"]),
                FieldSpec::new("institution.purpose", FieldKind::LongText).required().used_by(&["projects", "ai"]),
            ],
        },
        SectionSpec {
            id: "constitution",
            columns: 2,
            fields: &[
                FieldSpec::new("institution.legal_form", FieldKind::Select).options(LEGAL_FORMS).required().used_by(&["finance", "projects", "ai"]),
                FieldSpec::new("institution.founded_year", FieldKind::Year).range(OLDEST_YEAR, LATEST_YEAR).required().used_by(&["projects", "ai"]),
                private(FieldSpec::new("institution.junta_folio", FieldKind::Text))
                    .required()
                    .when(Condition::AnyOf { field: "institution.legal_form", values: UNDER_A_JUNTA })
                    .used_by(&["finance", "documents"]),
            ],
        },
        SectionSpec {
            id: "fiscal",
            columns: 2,
            fields: &[
                private(FieldSpec::new("institution.legal_rfc", FieldKind::Text)).rule(Rule::RfcMoral).required().used_by(&["documents", "finance"]),
                internal(FieldSpec::new("institution.tax_regime", FieldKind::Select)).options(TAX_REGIMES).required().used_by(&["finance", "projects", "ai"]),
                private(FieldSpec::new("institution.fiscal_postal_code", FieldKind::Text)).rule(Rule::PostalCode).used_by(&["finance", "documents"]),
            ],
        },
        SectionSpec {
            id: "registries",
            columns: 2,
            fields: &[
                FieldSpec::new("institution.authorized_donee", FieldKind::Select).options(REGISTRY).required().used_by(&["finance", "projects", "ai"]),
                FieldSpec::new("institution.cluni", FieldKind::Select).options(REGISTRY).required().used_by(&["projects", "ai"]),
                FieldSpec::new("institution.donee_category", FieldKind::Select).options(DONEE_CATEGORIES).when(DONEE).used_by(&["finance", "projects", "ai"]),
                private(FieldSpec::new("institution.donee_letter_number", FieldKind::Text)).when(DONEE).used_by(&["finance", "documents"]),
                private(FieldSpec::new("institution.donee_letter_date", FieldKind::Date)).when(DONEE).used_by(&["finance", "documents"]),
                private(FieldSpec::new("institution.cluni_key", FieldKind::Text))
                    .when(Condition::AnyOf { field: "institution.cluni", values: &["yes"] })
                    .used_by(&["projects", "documents"]),
            ],
        },
        SectionSpec {
            id: "legal_rep",
            columns: 2,
            fields: &[
                private(FieldSpec::new("institution.legal_rep_name", FieldKind::Text)).used_by(&["documents", "projects"]),
                private(FieldSpec::new("institution.legal_rep_valid_until", FieldKind::Date)).used_by(&["documents", "projects"]),
            ],
        },
    ],
};

/// «Capacidad y cifras rápidas»: how many fit, and how many are served and work there while their records are not
/// there yet. The records of the modules win over the quick figures.
pub const CAPACITY: FormSpec = FormSpec {
    id: "institution.capacity",
    sections: &[
        SectionSpec {
            id: "capacity_people",
            columns: 2,
            fields: &[
                internal(FieldSpec::new("institution.capacity_total", FieldKind::Number)).range(0, MAX_ESTIMATE).used_by(&["care", "facilities", "projects", "ai"]),
                internal(FieldSpec::new("institution.served_estimate", FieldKind::Number)).range(0, MAX_ESTIMATE).used_by(&["care", "projects", "ai"]),
            ],
        },
        SectionSpec {
            id: "capacity_staff",
            columns: 2,
            fields: &[
                internal(FieldSpec::new("institution.staff_paid_estimate", FieldKind::Number)).range(0, MAX_ESTIMATE).used_by(&["hr", "projects", "ai"]),
                internal(FieldSpec::new("institution.staff_volunteer_estimate", FieldKind::Number)).range(0, MAX_ESTIMATE).used_by(&["hr", "projects", "ai"]),
            ],
        },
        SectionSpec {
            id: "capacity_notes",
            columns: 1,
            fields: &[internal(FieldSpec::new("institution.notes", FieldKind::LongText)).used_by(&["projects", "ai"])],
        },
    ],
};

/// Every form of the core, by id.
pub const FORMS: &[&FormSpec] = &[&IDENTITY, &CONTACT, &LEGAL, &CAPACITY];

pub fn form(id: &str) -> Option<&'static FormSpec> {
    FORMS.iter().copied().find(|f| f.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every field has its words in the screen (ADR-033 §1): a label under `forms.fields` with its id, and a label
    /// for each of its codes, in its own entry or in the shared list it names (`options: STATES`).
    #[test]
    fn every_field_of_every_form_has_its_words() {
        let es = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/i18n/es-MX.ts")).unwrap();
        let start = es.find("forms: {").expect("es-MX.ts has a `forms` block");
        let block = &es[start..];
        for f in FORMS.iter().flat_map(|f| f.fields()) {
            let head = format!("\"{}\": {{", f.id);
            let at = block.find(&head).unwrap_or_else(|| panic!("{} has no words in es-MX.ts (forms.fields)", f.id));
            let rest = &block[at + head.len()..];
            let entry = &rest[..rest.find("\n      \"").or_else(|| rest.find("\n    }")).unwrap_or(rest.len())];
            if f.options.is_empty() {
                continue;
            }
            // the codes are in the entry, or in a list of their own the entry names
            let shared = entry.split("options: ").nth(1).map(|x| x.trim_start()).filter(|x| !x.starts_with('{')).map(|x| {
                let name: String = x.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                let decl = format!("const {name}");
                let from = es.find(&decl).unwrap_or_else(|| panic!("{}: the list {name} is not declared in es-MX.ts", f.id));
                &es[from..from + es[from..].find("};").unwrap_or(es.len() - from)]
            });
            let words = shared.unwrap_or(entry);
            for code in f.options {
                assert!(words.contains(&format!("{code}: ")) || words.contains(&format!("\"{code}\": ")), "{}: the code {code} has no label", f.id);
            }
        }
    }

    #[test]
    fn the_forms_have_unique_ids() {
        let ids: Vec<&str> = FORMS.iter().flat_map(|f| f.fields()).map(|f| f.id).collect();
        assert!((1..ids.len()).all(|i| !ids[..i].contains(&ids[i])), "{ids:?}");
        assert!(form("institution.identity").is_some() && form("nope").is_none());
    }

    /// The contact, the address, the RFC, the folios and the keys of the institution never reach the AI (ADR-033 §3).
    #[test]
    fn what_is_the_institutions_own_never_reaches_the_ai() {
        for id in [
            "institution.contact_phone", "institution.contact_email", "institution.street", "institution.postal_code", "institution.legal_rfc",
            "institution.junta_folio", "institution.donee_letter_number", "institution.cluni_key", "institution.legal_rep_name",
        ] {
            let f = FORMS.iter().find_map(|f| f.field(id)).unwrap();
            assert_eq!((f.sensitivity, f.ai), (Sensitivity::InstitutionalPrivate, AiUse::Never), "{id}");
        }
    }

    /// A field kept as a detail has its column, and every column belongs to a field (`profile::storage::DETAILS`).
    #[test]
    fn every_detail_is_a_field_of_a_form() {
        for (id, _) in crate::core::profile::storage::DETAILS {
            assert!(FORMS.iter().any(|f| f.field(id).is_some()), "{id} is not in any form");
        }
    }
}
