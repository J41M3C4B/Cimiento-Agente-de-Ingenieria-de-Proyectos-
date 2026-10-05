//! Reads what a call asks of a project from the canonical document of its reading (ADR-015), with the page and
//! file of every value. The document was already checked quote by quote; here only typed values are picked out:
//! the numbers come from what the code normalized, never from the model.

use super::summary::summarize;
use crate::domain::requirements::{CallRequirements, Line, Sourced};
use serde_json::Value;

/// The first quote of a node: where its value comes from.
fn source(node: &Value) -> (Option<u64>, Option<String>) {
    let e = &node["evidencias"][0];
    (e["pagina"].as_u64(), e["documento"].as_str().map(str::to_string))
}

fn present(node: &Value) -> bool {
    node["estado"].as_str().is_some_and(|e| e != "no_aparece")
}

/// A number the code normalized from the quote of the field (`path` inside `normalizado`).
fn number(node: &Value, key: &str) -> Option<Sourced<f64>> {
    if !present(node) {
        return None;
    }
    let value = node["normalizado"][key].as_f64()?;
    let (page, file) = source(node);
    Some(Sourced { value, page, file })
}

fn months(node: &Value) -> Option<Sourced<u32>> {
    if !present(node) {
        return None;
    }
    let n = &node["normalizado"];
    let v = n["valor"].as_f64()?;
    let months = match n["unidad"].as_str()? {
        "meses" => v,
        "anios" => v * 12.0,
        "dias" => (v / 30.0).ceil(),
        _ => return None,
    };
    let (page, file) = source(node);
    (months >= 1.0 && months <= 600.0).then(|| Sourced { value: months.round() as u32, page, file })
}

/// An amount in pesos. `XXX` is how the code marks a figure with no currency mark: the document is in pesos.
fn amount(node: &Value, foreign: &mut bool) -> Option<Sourced<f64>> {
    if !present(node) {
        return None;
    }
    match node["normalizado"]["moneda"].as_str() {
        Some("MXN") | Some("XXX") | None => number(node, "cantidad"),
        Some(_) => {
            *foreign = true;
            None
        }
    }
}

/// The latest closing date the call gives. A project can still apply while any of them is ahead, so the latest
/// is the one the person has to meet.
fn closing_date(doc: &Value) -> Option<Sourced<String>> {
    doc["temporalidad"]["hitos"]["hitos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|h| h["tipo"].as_str() == Some("cierre"))
        .filter_map(|h| {
            let date = h["normalizado"]["inicio"]["fecha"].as_str()?.to_string();
            let (page, file) = source(h);
            Some(Sourced { value: date, page, file })
        })
        .max_by(|a, b| a.value.cmp(&b.value))
}

/// What the call asks of a project, from the canonical document of its reading.
pub fn call_requirements(doc: &Value) -> CallRequirements {
    let s = summarize(doc);
    let list = |key: &str| -> Vec<Line> {
        s.groups
            .iter()
            .find(|g| g.key == key)
            .map(|g| g.items.iter().map(|i| Line { text: i.text.clone(), applies_to: i.applies_to.clone(), page: i.page, file: i.file.clone() }).collect())
            .unwrap_or_default()
    };
    let f = &doc["financiamiento"];
    let mut foreign = false;
    let max_amount_mxn = amount(&f["monto_maximo"], &mut foreign);
    let min_amount_mxn = amount(&f["monto_minimo"], &mut foreign);
    CallRequirements {
        max_amount_mxn,
        min_amount_mxn,
        foreign_currency: foreign,
        cofunding_percent: number(&f["porcentaje_cofinanciamiento"], "valor").filter(|p| (0.0..=100.0).contains(&p.value)),
        admin_cap_percent: number(&f["tope_gastos_administrativos"], "valor").filter(|p| (0.0..=100.0).contains(&p.value)),
        max_duration_months: months(&doc["temporalidad"]["periodo_ejecucion"]),
        closing_date: closing_date(doc),
        required_docs: list("docs_required"),
        conditional_docs: list("docs_conditional"),
        optional_docs: list("docs_optional"),
        formats: list("formats"),
        evaluation_criteria: list("criteria"),
        project_requirements: list("project_requirements"),
        fundable: list("fundable"),
        not_fundable: list("not_fundable"),
        indicators: list("indicators"),
        how_to_deliver: list("how_to_deliver"),
        contact: list("contact"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ev(cita: &str, pagina: u64) -> Value {
        json!([{ "cita": cita, "pagina": pagina, "documento": "bases.pdf" }])
    }

    fn doc() -> Value {
        json!({
            "temporalidad": {
                "periodo_ejecucion": { "estado": "encontrado", "valor_texto": "12 meses", "evidencias": ev("hasta 12 meses", 3), "normalizado": { "valor": 12.0, "unidad": "meses" } },
                "hitos": { "estado": "encontrado", "hitos": [
                    { "tipo": "cierre", "etiqueta": "Cierre de registro", "inicio_texto": "15 de abril de 2027", "evidencias": ev("Cierre de registro: 15 de abril de 2027", 3), "normalizado": { "inicio": { "fecha": "2027-04-15", "precision": "dia" }, "fin": null } },
                    { "tipo": "cierre", "etiqueta": "Cierre de postulación", "inicio_texto": "23 de mayo de 2027", "evidencias": ev("Cierre: 23 de mayo de 2027", 4), "normalizado": { "inicio": { "fecha": "2027-05-23", "precision": "dia" }, "fin": null } },
                    { "tipo": "resultados", "etiqueta": "Resultados", "inicio_texto": "junio", "evidencias": ev("Resultados en junio", 4), "normalizado": { "inicio": { "fecha": null, "precision": "mes" }, "fin": null } }
                ] }
            },
            "financiamiento": {
                "monto_maximo": { "estado": "encontrado", "valor_texto": "$250,000", "evidencias": ev("Monto máximo: $250,000 pesos", 2), "normalizado": { "cantidad": 250000.0, "moneda": "MXN" } },
                "monto_minimo": { "estado": "no_aparece", "valor_texto": null, "evidencias": [] },
                "porcentaje_cofinanciamiento": { "estado": "encontrado", "valor_texto": "30%", "evidencias": ev("contrapartida mínima del 30%", 4), "normalizado": { "valor": 30.0 } },
                "tope_gastos_administrativos": { "estado": "encontrado", "valor_texto": "10%", "evidencias": ev("hasta el 10% en gastos administrativos", 5), "normalizado": { "valor": 10.0 } }
            },
            "documentacion": {
                "obligatoria": { "estado": "encontrado", "elementos": [
                    { "titulo": "Acta constitutiva", "evidencias": ev("Acta constitutiva", 6), "aplica_a": null, "obligatoriedad": "obligatorio" },
                    { "titulo": "Propuesta del proyecto", "evidencias": ev("Propuesta del proyecto", 6), "aplica_a": null, "obligatoriedad": "obligatorio" }
                ] },
                "condicional": { "estado": "encontrado", "elementos": [{ "titulo": "Estados financieros", "evidencias": ev("Estados financieros", 6), "aplica_a": "organizaciones nuevas", "obligatoriedad": "obligatorio" }] }
            },
            "proyecto": {
                "requisitos_del_proyecto": { "estado": "encontrado", "elementos": [{ "titulo": "Justificación del problema", "evidencias": ev("Justificación del problema", 7), "aplica_a": null, "obligatoriedad": "obligatorio" }] }
            },
            "evaluacion": { "criterios": { "estado": "encontrado", "criterios": [{ "titulo": "Claridad del problema", "ponderacion": "20 puntos", "evidencias": ev("1. Claridad del problema (20 puntos)", 8) }] } }
        })
    }

    #[test]
    fn typed_values_come_from_what_the_code_normalized_with_their_page() {
        let r = call_requirements(&doc());
        assert_eq!(r.max_amount_mxn, Some(Sourced { value: 250000.0, page: Some(2), file: Some("bases.pdf".into()) }));
        assert_eq!(r.min_amount_mxn, None, "what the call does not say stays empty");
        assert_eq!((r.cofunding_percent.map(|p| p.value), r.admin_cap_percent.map(|p| p.value)), (Some(30.0), Some(10.0)));
        assert_eq!(r.max_duration_months.map(|d| d.value), Some(12));
        assert!(!r.foreign_currency);
    }

    #[test]
    fn the_closing_date_is_the_latest_one_and_a_month_without_a_day_is_not_a_date() {
        let r = call_requirements(&doc());
        assert_eq!(r.closing_date.map(|d| (d.value, d.page)), Some(("2027-05-23".to_string(), Some(4))));
        assert_eq!(call_requirements(&json!({})).closing_date, None);
    }

    #[test]
    fn the_lists_keep_the_condition_and_say_whether_a_proposal_is_asked_for() {
        let r = call_requirements(&doc());
        assert_eq!(r.required_docs.iter().map(|d| d.text.as_str()).collect::<Vec<_>>(), vec!["Acta constitutiva", "Propuesta del proyecto"]);
        assert_eq!((r.conditional_docs[0].text.as_str(), r.conditional_docs[0].applies_to.as_deref()), ("Estados financieros", Some("organizaciones nuevas")));
        assert_eq!(r.project_requirements[0].text, "Justificación del problema");
        assert_eq!(r.evaluation_criteria[0].text, "Claridad del problema (20 puntos)");
        assert!(r.asks_for_proposal());
    }

    #[test]
    fn durations_in_years_and_days_become_months_and_other_currencies_are_not_taken_as_pesos() {
        let years = json!({ "temporalidad": { "periodo_ejecucion": { "estado": "encontrado", "evidencias": [], "normalizado": { "valor": 2.0, "unidad": "anios" } } } });
        assert_eq!(call_requirements(&years).max_duration_months.map(|d| d.value), Some(24));
        let days = json!({ "temporalidad": { "periodo_ejecucion": { "estado": "encontrado", "evidencias": [], "normalizado": { "valor": 45.0, "unidad": "dias" } } } });
        assert_eq!(call_requirements(&days).max_duration_months.map(|d| d.value), Some(2));
        let dollars = json!({ "financiamiento": { "monto_maximo": { "estado": "encontrado", "evidencias": [], "normalizado": { "cantidad": 10000.0, "moneda": "USD" } } } });
        let r = call_requirements(&dollars);
        assert_eq!((r.max_amount_mxn, r.foreign_currency), (None, true));
        // a figure with no currency mark is taken as pesos
        let plain = json!({ "financiamiento": { "monto_maximo": { "estado": "encontrado", "evidencias": [], "normalizado": { "cantidad": 40000.0, "moneda": "XXX" } } } });
        assert_eq!(call_requirements(&plain).max_amount_mxn.map(|a| a.value), Some(40000.0));
    }

    #[test]
    fn an_empty_reading_asks_nothing() {
        let r = call_requirements(&json!({}));
        assert_eq!(r, CallRequirements::default());
        assert!(!r.asks_for_proposal());
    }
}
