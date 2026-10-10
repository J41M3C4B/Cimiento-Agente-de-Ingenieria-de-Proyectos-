//! The forms of «Mi institución» as the screen gets and saves them (ADR-033 §1): the description, the values and what
//! is missing come from here, and a save goes the usual way (checks, scanner, confirmation). The screen never maps a
//! field to the profile itself.

use crate::common::forms::{FormSpec, Values};
use crate::core::error::ServiceError;
use crate::core::institution::forms::form;
use crate::core::profile::domain::{Attention, ProfileInput, ProfileIssue};
use crate::core::profile::service::SaveProfileOutcome;
use crate::core::profile::storage as store;
use crate::scanner::guard::Decision;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{json, Value};

/// A form ready to draw: its description, what is saved and what is still missing.
#[derive(Debug, Serialize)]
pub struct FormView {
    pub spec: &'static FormSpec,
    pub values: Values,
    pub missing: Vec<&'static str>,
}

fn text(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn list(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default()
}

/// The values of a form, read from the profile.
fn values_of(id: &str, p: &ProfileInput) -> Values {
    let i = &p.institution;
    let a = i.attention.clone().unwrap_or_default();
    let mut v = Values::new();
    if id == "institution.identity" {
        v.insert("institution.name".into(), json!(i.name));
        v.insert("institution.mission".into(), json!(i.mission));
        v.insert("institution.populations".into(), json!(a.populations));
        v.insert("institution.sex_served".into(), json!(a.sex_served));
        v.insert("institution.modalities".into(), json!(a.modalities));
        v.insert("institution.care_areas".into(), json!(a.care_areas));
    }
    v.retain(|_, x| !x.is_null());
    v
}

/// Writes the values of a form into the profile.
fn apply(id: &str, v: &Values, p: &mut ProfileInput) {
    if id == "institution.identity" {
        let i = &mut p.institution;
        i.name = text(v.get("institution.name")).unwrap_or_default();
        i.mission = text(v.get("institution.mission"));
        i.attention = Some(Attention {
            populations: list(v.get("institution.populations")),
            sex_served: text(v.get("institution.sex_served")),
            modalities: list(v.get("institution.modalities")),
            care_areas: list(v.get("institution.care_areas")),
        });
    }
}

pub fn get(conn: &Connection, id: &str) -> Result<FormView, ServiceError> {
    let spec = form(id).ok_or(ServiceError::NotFound)?;
    let profile = store::load_current(conn)?.map(|p| p.input).unwrap_or_default();
    let values = values_of(id, &profile);
    Ok(FormView { spec, missing: spec.missing(&values), values })
}

/// Saves a form. A value out of its list or range is refused with the field it belongs to; what is missing never
/// stops the save. Then the profile goes the way of «Mi institución» (`save_profile_confirmed`).
pub fn save(conn: &mut Connection, id: &str, mut values: Values, decision: Option<Decision>) -> Result<SaveProfileOutcome, ServiceError> {
    let spec = form(id).ok_or(ServiceError::NotFound)?;
    spec.clear_hidden(&mut values);
    let blocking: Vec<ProfileIssue> = spec
        .validate(&values)
        .into_iter()
        .filter(|i| i.blocking)
        .map(|i| ProfileIssue { code: i.code, field: i.field.to_string(), blocking: true })
        .collect();
    if !blocking.is_empty() {
        return Ok(SaveProfileOutcome::Invalid { issues: blocking });
    }
    let mut profile = store::load_current(conn)?.map(|p| p.input).unwrap_or_default();
    apply(id, &values, &mut profile);
    crate::core::onboarding::service::save_profile_confirmed(conn, profile, decision)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::profile::domain::{InstitutionInput, InstitutionKind};
    use crate::storage::open_encrypted;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn conn() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, c)
    }

    fn values(v: Value) -> Values {
        serde_json::from_value(v).unwrap()
    }

    fn saved(out: SaveProfileOutcome) -> crate::core::profile::service::ProfileView {
        match out {
            SaveProfileOutcome::Saved { profile } => profile,
            other => panic!("not saved: {other:?}"),
        }
    }

    #[test]
    fn an_older_institution_reads_its_kind_as_an_attention_profile() {
        let (_d, mut c) = conn();
        let old = ProfileInput { institution: InstitutionInput { name: "Asilo Ficticio".into(), kind: InstitutionKind::ElderlyHome, ..Default::default() }, ..Default::default() };
        store::save(&mut c, &old).unwrap();
        let view = get(&c, "institution.identity").unwrap();
        assert_eq!(view.values["institution.populations"], json!(["older_adults"]));
        assert_eq!(view.values["institution.modalities"], json!(["residential"]));
        assert_eq!(view.missing, vec!["institution.mission"]);
    }

    #[test]
    fn a_mixed_institution_is_saved_whole_and_its_kind_follows() {
        let (_d, mut c) = conn();
        let v = values(json!({
            "institution.name": "Casa Ficticia", "institution.mission": "Acompañar.",
            "institution.populations": ["childhood", "older_adults"], "institution.sex_served": "all",
            "institution.modalities": ["day_care"], "institution.care_areas": ["care", "food"],
        }));
        let p = saved(save(&mut c, "institution.identity", v, None).unwrap());
        assert_eq!(p.input.institution.kind, InstitutionKind::Other);
        assert_eq!(crate::core::institution::kind(&c).as_deref(), Some("other"));
        let a = p.input.institution.attention.unwrap();
        assert_eq!((a.populations.len(), a.sex_served.as_deref(), a.care_areas.len()), (2, Some("all"), 2));
        assert!(get(&c, "institution.identity").unwrap().missing.is_empty());
        let sheet = crate::core::ai_sheet::profile_context(&c).unwrap();
        assert!(sheet.contains("A quién atiende: niñez, personas mayores (mujeres y hombres)."), "{sheet}");
        assert!(sheet.contains("Cómo atiende: estancia de día."), "{sheet}");
    }

    #[test]
    fn a_code_out_of_its_list_is_refused_and_nothing_is_saved() {
        let (_d, mut c) = conn();
        let v = values(json!({ "institution.name": "Casa", "institution.populations": ["aliens"] }));
        match save(&mut c, "institution.identity", v, None).unwrap() {
            SaveProfileOutcome::Invalid { issues } => assert_eq!((issues[0].code, issues[0].field.as_str()), ("code_unknown", "institution.populations")),
            other => panic!("{other:?}"),
        }
        assert!(store::load_current(&c).unwrap().is_none());
        assert!(matches!(get(&c, "nope"), Err(ServiceError::NotFound)));
    }

    #[test]
    fn the_sex_served_is_not_kept_without_populations_and_the_mission_goes_through_the_scanner() {
        let (_d, mut c) = conn();
        let v = values(json!({ "institution.name": "Casa", "institution.sex_served": "women", "institution.mission": "Llamar a maria.lopez@example.com" }));
        assert!(matches!(save(&mut c, "institution.identity", v.clone(), None).unwrap(), SaveProfileOutcome::Quarantine { .. }));
        let p = saved(save(&mut c, "institution.identity", v, Some(Decision::Redact)).unwrap());
        assert_eq!(p.input.institution.attention.unwrap().sex_served, None, "it does not apply without populations");
        assert!(!p.input.institution.mission.unwrap().contains("maria.lopez"));
    }

    #[test]
    fn saving_another_window_keeps_the_attention_profile() {
        let (_d, mut c) = conn();
        let v = values(json!({ "institution.name": "Casa", "institution.populations": ["adults"], "institution.modalities": ["outpatient"] }));
        saved(save(&mut c, "institution.identity", v, None).unwrap());
        // the contact window sends the whole profile without the attention profile: it stays as it was
        let mut input = store::load_current(&c).unwrap().unwrap().input;
        input.institution.attention = None;
        input.institution.contact_email = Some("contacto@casa.org".into());
        let p = saved(crate::core::onboarding::service::save_profile_confirmed(&mut c, input, None).unwrap());
        assert_eq!(p.input.institution.attention.unwrap().populations, vec!["adults".to_string()]);
    }
}
