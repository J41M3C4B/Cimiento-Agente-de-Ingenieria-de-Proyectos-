//! The forms of the institution described as data (ADR-033 §1): what «Mi institución» asks, field by field. The
//! screen draws them; the words live in `src/i18n/es-MX.ts` under `forms.fields` with the same id.

use super::catalog::{CARE_AREAS, MODALITIES, POPULATIONS, SEXES_SERVED};
use crate::common::forms::{Condition, FieldKind, FieldSpec, FormSpec, SectionSpec};

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
                FieldSpec::new("institution.sex_served", FieldKind::Select)
                    .options(SEXES_SERVED)
                    .when(Condition::Filled { field: "institution.populations" })
                    .used_by(&["care", "projects", "ai"]),
                FieldSpec::new("institution.modalities", FieldKind::MultiSelect).options(MODALITIES).required().used_by(&["care", "facilities", "projects", "ai"]),
                FieldSpec::new("institution.care_areas", FieldKind::MultiSelect).options(CARE_AREAS).used_by(&["care", "hr", "projects", "ai"]),
            ],
        },
    ],
};

/// Every form of the core, by id.
pub const FORMS: &[&FormSpec] = &[&IDENTITY];

pub fn form(id: &str) -> Option<&'static FormSpec> {
    FORMS.iter().copied().find(|f| f.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every field has its words in the screen (ADR-033 §1): a label under `forms.fields` with its id, and a label
    /// for each of its codes.
    #[test]
    fn every_field_of_every_form_has_its_words() {
        let es = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/i18n/es-MX.ts")).unwrap();
        let start = es.find("forms: {").expect("es-MX.ts has a `forms` block");
        let block = &es[start..];
        for f in FORMS.iter().flat_map(|f| f.fields()) {
            assert!(block.contains(&format!("\"{}\": {{", f.id)), "{} has no words in es-MX.ts (forms.fields)", f.id);
            for code in f.options {
                assert!(block.contains(&format!("{code}: ")) || block.contains(&format!("\"{code}\": ")), "{}: the code {code} has no label", f.id);
            }
        }
    }

    #[test]
    fn the_forms_have_unique_ids() {
        let ids: Vec<&str> = FORMS.iter().flat_map(|f| f.fields()).map(|f| f.id).collect();
        assert!((1..ids.len()).all(|i| !ids[..i].contains(&ids[i])), "{ids:?}");
        assert!(form("institution.identity").is_some() && form("nope").is_none());
    }
}
