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

/// A form ready to draw: its description, what is saved, where each saved datum comes from and what is still missing.
#[derive(Debug, Serialize)]
pub struct FormView {
    pub spec: &'static FormSpec,
    pub values: Values,
    pub origins: std::collections::BTreeMap<String, crate::core::history::FieldOrigin>,
    pub missing: Vec<&'static str>,
}

/// Every datum of the catalog the profile holds, by field id: what the history compares (ADR-033 §4).
pub fn catalog_values(p: &ProfileInput) -> Values {
    crate::core::institution::forms::FORMS.iter().flat_map(|f| values_of(f.id, p)).collect()
}

fn text(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn list(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default()
}

fn number(v: Option<&Value>) -> Option<i64> {
    v.and_then(Value::as_i64)
}

/// The values of a form, read from the profile. The data that live as details go by their id
/// (`storage::DETAILS`); the older columns are named here one by one.
fn values_of(id: &str, p: &ProfileInput) -> Values {
    let i = &p.institution;
    let a = i.attention.clone().unwrap_or_default();
    let mut v = Values::new();
    let mut put = |field: &str, value: Value| {
        v.insert(field.to_string(), value);
    };
    match id {
        "institution.identity" => {
            put("institution.name", json!(i.name));
            put("institution.mission", json!(i.mission));
            put("institution.populations", json!(a.populations));
            put("institution.sex_served", json!(a.sex_served));
            put("institution.modalities", json!(a.modalities));
            put("institution.care_areas", json!(a.care_areas));
        }
        "institution.contact" => {
            put("institution.contact_phone", json!(i.contact_phone));
            put("institution.contact_email", json!(i.contact_email));
            put("institution.state", json!(i.state));
            put("institution.municipality", json!(i.municipality));
        }
        "institution.legal" => {
            put("institution.legal_rfc", json!(i.legal_rfc));
            put("institution.legal_rep_name", json!(i.legal_rep_name));
            put("institution.legal_form", json!(i.legal_form));
            put("institution.founded_year", json!(i.founded_year));
            put("institution.authorized_donee", json!(i.authorized_donee));
            put("institution.cluni", json!(i.cluni));
        }
        "institution.capacity" => {
            put("institution.capacity_total", json!(p.capacity_total));
            put("institution.served_estimate", json!(p.served_estimate));
            put("institution.staff_paid_estimate", json!(p.staff_paid_estimate));
            put("institution.staff_volunteer_estimate", json!(p.staff_volunteer_estimate));
            put("institution.notes", json!(p.notes));
        }
        _ => {}
    }
    if let (Some(spec), Some(details)) = (form(id), &i.details) {
        for f in spec.fields().filter(|f| store::is_detail(f.id)) {
            if let Some(x) = details.get(f.id) {
                v.insert(f.id.to_string(), x.clone());
            }
        }
    }
    v.retain(|_, x| crate::common::forms::is_filled(Some(x)));
    v
}

/// Writes the values of a form into the profile. A field of the form left empty is emptied; the fields of the other
/// forms stay as they are.
fn apply(id: &str, v: &Values, p: &mut ProfileInput) {
    let i = &mut p.institution;
    match id {
        "institution.identity" => {
            i.name = text(v.get("institution.name")).unwrap_or_default();
            i.mission = text(v.get("institution.mission"));
            i.attention = Some(Attention {
                populations: list(v.get("institution.populations")),
                sex_served: text(v.get("institution.sex_served")),
                modalities: list(v.get("institution.modalities")),
                care_areas: list(v.get("institution.care_areas")),
            });
        }
        "institution.contact" => {
            i.contact_phone = text(v.get("institution.contact_phone"));
            i.contact_email = text(v.get("institution.contact_email"));
            i.state = text(v.get("institution.state"));
            i.municipality = text(v.get("institution.municipality"));
        }
        "institution.legal" => {
            // the RFC goes in capitals, as the SAT writes it
            i.legal_rfc = text(v.get("institution.legal_rfc")).map(|r| r.to_uppercase());
            i.legal_rep_name = text(v.get("institution.legal_rep_name"));
            i.legal_form = text(v.get("institution.legal_form"));
            i.founded_year = number(v.get("institution.founded_year"));
            i.authorized_donee = text(v.get("institution.authorized_donee"));
            i.cluni = text(v.get("institution.cluni"));
        }
        "institution.capacity" => {
            p.capacity_total = number(v.get("institution.capacity_total"));
            p.served_estimate = number(v.get("institution.served_estimate"));
            p.staff_paid_estimate = number(v.get("institution.staff_paid_estimate"));
            p.staff_volunteer_estimate = number(v.get("institution.staff_volunteer_estimate"));
            p.notes = text(v.get("institution.notes"));
        }
        _ => {}
    }
    if let Some(spec) = form(id) {
        let details = p.institution.details.get_or_insert_with(Values::new);
        for f in spec.fields().filter(|f| store::is_detail(f.id)) {
            match v.get(f.id) {
                Some(x) if crate::common::forms::is_filled(Some(x)) => details.insert(f.id.to_string(), x.clone()),
                _ => details.remove(f.id),
            };
        }
    }
}

pub fn get(conn: &Connection, id: &str) -> Result<FormView, ServiceError> {
    let spec = form(id).ok_or(ServiceError::NotFound)?;
    let profile = store::load_current(conn)?.map(|p| p.input).unwrap_or_default();
    let values = values_of(id, &profile);
    let origins = crate::core::history::origins(conn, &values)?;
    Ok(FormView { spec, missing: spec.missing(&values), origins, values })
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

    fn invalid(out: SaveProfileOutcome) -> Vec<(&'static str, String)> {
        match out {
            SaveProfileOutcome::Invalid { issues } => issues.into_iter().map(|i| (i.code, i.field)).collect(),
            other => panic!("not refused: {other:?}"),
        }
    }

    /// Each window of «Mi institución» saves its own fields, the new ones included, and leaves the others as they were.
    #[test]
    fn every_window_saves_its_fields_and_keeps_the_rest() {
        let (_d, mut c) = conn();
        let identity = values(json!({
            "institution.name": "Asilo Ficticio", "institution.mission": "Cuidar.", "institution.populations": ["older_adults"],
            "institution.modalities": ["residential"], "institution.age_min": 60, "institution.services": "Residencia y comedor.",
        }));
        saved(save(&mut c, "institution.identity", identity, None).unwrap());
        let contact = values(json!({
            "institution.contact_phone": "55 5555 0101", "institution.street": "Calle Ficticia", "institution.ext_number": "12",
            "institution.postal_code": "45010", "institution.state": "jal", "institution.municipality": "Zapopan",
        }));
        saved(save(&mut c, "institution.contact", contact, None).unwrap());
        let legal = values(json!({
            "institution.legal_name": "Asilo Ficticio, I.A.P.", "institution.purpose": "La asistencia a personas mayores.",
            "institution.legal_form": "iap", "institution.founded_year": 1987, "institution.junta_folio": "JAP-0001",
            "institution.legal_rfc": "afi870101ab1", "institution.tax_regime": "non_profit", "institution.authorized_donee": "yes",
            "institution.donee_category": "assistance", "institution.cluni": "no", "institution.cluni_key": "kept only with yes",
        }));
        saved(save(&mut c, "institution.legal", legal, None).unwrap());
        let capacity = values(json!({ "institution.capacity_total": 25, "institution.served_estimate": 18, "institution.notes": "Notas." }));
        let p = saved(save(&mut c, "institution.capacity", capacity, None).unwrap());

        // every window kept what the others saved
        let i = &p.input.institution;
        assert_eq!((i.name.as_str(), i.contact_phone.as_deref(), i.legal_rfc.as_deref()), ("Asilo Ficticio", Some("55 5555 0101"), Some("AFI870101AB1")));
        assert_eq!((p.input.capacity_total, p.input.served_estimate), (Some(25), Some(18)));
        let d = i.details.as_ref().unwrap();
        assert_eq!(d["institution.age_min"], json!(60));
        assert_eq!(d["institution.street"], json!("Calle Ficticia"));
        assert_eq!(d["institution.junta_folio"], json!("JAP-0001"));
        assert!(!d.contains_key("institution.cluni_key"), "it does not apply without a CLUNI");

        // what each window shows, and what is still missing in it
        assert!(get(&c, "institution.legal").unwrap().missing.is_empty());
        let contact = get(&c, "institution.contact").unwrap();
        assert_eq!(contact.values["institution.postal_code"], json!("45010"));
        assert!(contact.missing.is_empty());
        assert_eq!(get(&c, "institution.identity").unwrap().values["institution.services"], json!("Residencia y comedor."));

        // emptying a field of a window empties it, and only that one
        let contact = values(json!({ "institution.state": "jal", "institution.municipality": "Zapopan", "institution.street": "Calle Ficticia" }));
        saved(save(&mut c, "institution.contact", contact, None).unwrap());
        assert_eq!(get(&c, "institution.contact").unwrap().missing, vec!["institution.postal_code"]);
        assert_eq!(get(&c, "institution.legal").unwrap().values["institution.junta_folio"], json!("JAP-0001"));
        assert_eq!(get(&c, "institution.identity").unwrap().values["institution.age_min"], json!(60));
    }

    #[test]
    fn a_wrong_postal_code_rfc_date_or_age_range_is_refused() {
        let (_d, mut c) = conn();
        saved(save(&mut c, "institution.identity", values(json!({ "institution.name": "Casa" })), None).unwrap());
        assert_eq!(invalid(save(&mut c, "institution.contact", values(json!({ "institution.postal_code": "4501" })), None).unwrap()), vec![("postal_code_invalid", "institution.postal_code".into())]);
        let legal = values(json!({ "institution.legal_rfc": "AFI870101AB12", "institution.authorized_donee": "yes", "institution.donee_letter_date": "2023-02-30" }));
        assert_eq!(
            invalid(save(&mut c, "institution.legal", legal, None).unwrap()),
            vec![("rfc_moral_invalid", "institution.legal_rfc".into()), ("date_invalid", "institution.donee_letter_date".into())]
        );
        let ages = values(json!({ "institution.name": "Casa", "institution.populations": ["adults"], "institution.age_min": 70, "institution.age_max": 30 }));
        assert_eq!(invalid(save(&mut c, "institution.identity", ages, None).unwrap()), vec![("age_range", "institution.age_max".into())]);
    }

    /// The folio of the Junta is asked only of the legal forms a Junta watches over.
    #[test]
    fn the_junta_is_asked_only_of_an_iap_ibp_or_abp() {
        let (_d, mut c) = conn();
        saved(save(&mut c, "institution.identity", values(json!({ "institution.name": "Casa" })), None).unwrap());
        let ac = values(json!({ "institution.legal_form": "ac", "institution.junta_folio": "JAP-1" }));
        saved(save(&mut c, "institution.legal", ac, None).unwrap());
        let view = get(&c, "institution.legal").unwrap();
        assert!(!view.missing.contains(&"institution.junta_folio") && !view.values.contains_key("institution.junta_folio"));
        saved(save(&mut c, "institution.legal", values(json!({ "institution.legal_form": "iap" })), None).unwrap());
        assert!(get(&c, "institution.legal").unwrap().missing.contains(&"institution.junta_folio"));
    }

    /// The long texts that may reach the AI go through the scanner; the address, folios and keys do not.
    #[test]
    fn the_objeto_social_goes_through_the_scanner_and_the_address_does_not() {
        let (_d, mut c) = conn();
        saved(save(&mut c, "institution.identity", values(json!({ "institution.name": "Casa" })), None).unwrap());
        let legal = values(json!({ "institution.purpose": "Escribir a maria.lopez@example.com" }));
        assert!(matches!(save(&mut c, "institution.legal", legal, None).unwrap(), SaveProfileOutcome::Quarantine { .. }));
        let contact = values(json!({ "institution.street": "Calle de María López", "institution.contact_email": "contacto@casa.org" }));
        saved(save(&mut c, "institution.contact", contact, None).unwrap());
    }

    /// ADR-033 §4 and audit D6: every save leaves a line per datum that changed, with its origin; what is protected
    /// says only that it changed; saving the same again adds nothing; a save that does not happen leaves nothing.
    #[test]
    fn every_save_leaves_its_history_and_the_origin_of_each_datum() {
        use crate::core::history::changes;
        let (_d, mut c) = conn();
        saved(save(&mut c, "institution.identity", values(json!({ "institution.name": "Casa", "institution.mission": "Cuidar." })), None).unwrap());
        let contact = values(json!({ "institution.street": "Calle Ficticia", "institution.municipality": "Zapopan" }));
        saved(save(&mut c, "institution.contact", contact.clone(), None).unwrap());
        let lines = changes(&c, None, 50).unwrap();
        let line = |f: &str| lines.iter().find(|l| l.field == f).unwrap_or_else(|| panic!("no line for {f}: {lines:?}"));
        assert_eq!((line("institution.name").value.clone(), line("institution.name").origin.as_str()), (Some(json!("Casa")), "user"));
        assert_eq!((line("institution.street").value.clone(), line("institution.street").protected), (None, true));
        assert_eq!(line("institution.municipality").value, Some(json!("Zapopan")));
        let view = get(&c, "institution.contact").unwrap();
        assert_eq!(view.origins["institution.street"].origin, "user");
        assert!(view.origins["institution.street"].confirmed_at.is_some());

        // the same again: nothing new; a save the scanner holds back: nothing either
        let n = lines.len();
        saved(save(&mut c, "institution.contact", contact, None).unwrap());
        let held = values(json!({ "institution.name": "Casa", "institution.mission": "Llamar a maria.lopez@example.com" }));
        assert!(matches!(save(&mut c, "institution.identity", held, None).unwrap(), SaveProfileOutcome::Quarantine { .. }));
        assert_eq!(changes(&c, None, 50).unwrap().len(), n);
    }

    /// The datum and its line go together: if the history cannot be written, the datum is not saved either.
    #[test]
    fn without_its_history_a_datum_is_not_saved() {
        let (_d, mut c) = conn();
        saved(save(&mut c, "institution.identity", values(json!({ "institution.name": "Casa" })), None).unwrap());
        c.execute_batch("CREATE TEMP TRIGGER fail_history BEFORE INSERT ON core_change BEGIN SELECT RAISE(ABORT, 'no'); END;").unwrap();
        assert!(save(&mut c, "institution.identity", values(json!({ "institution.name": "Casa Nueva" })), None).is_err());
        assert_eq!(store::load_current(&c).unwrap().unwrap().input.institution.name, "Casa");
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
