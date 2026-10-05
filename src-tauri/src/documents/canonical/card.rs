//! The card of a call: what a person needs to understand it quickly and decide whether it fits them (ADR-024).
//!
//! `summary.rs` says what the reading understood, group by group, for whoever wants to check a detail; this says what
//! matters, short. It is made of the same canonical document with no model and no rule about any funder, and the
//! words of the screen live in the app's texts: here there are only keys, the document's own words and the page
//! each came from. The AI-written «en pocas palabras» (`brief`) is stored apart and passes through untouched.

use super::summary::{single, source, text, CallSummary, SummaryAmount, SummaryGroup, SummaryItem};
use serde::Serialize;
use serde_json::Value;

/// How many points of a block the card shows; the rest is only counted.
const MAX_POINTS: usize = 3;
/// How long a point or a figure may be on the card.
const POINT_CHARS: usize = 140;
const FACT_CHARS: usize = 80;
const LEAD_CHARS: usize = 280;
/// How many missing data are named (the rest is counted).
const MISSING_NAMED: usize = 3;
/// Kinds of document that are not a call: the person is warned when the file looks like one of these.
const NOT_A_CALL: [&str; 5] = ["aviso", "guia", "formato", "anexo", "otro"];

/// A figure of the call, as the document writes it, with where it was read.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CardFact {
    /// `max_amount`, `min_amount`, `cofunding`, `admin_cap`, `duration`, `closing` (or `registration`, when the call
    /// gives no closing date), `modalities`.
    pub kind: &'static str,
    pub value: String,
    pub page: Option<u64>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CardPoint {
    pub text: String,
    /// Who it applies to, only when the document restricts it.
    pub applies_to: Option<String>,
    pub page: Option<u64>,
    pub file: Option<String>,
}

/// A few points of one topic, and how many more the detail has.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CardBlock {
    /// `who_can`, `supported`, `fundable`, `not_fundable`.
    pub key: &'static str,
    pub points: Vec<CardPoint>,
    pub more: usize,
}

/// Something the person should hear about. Only present when it is true.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CardAlert {
    /// `not_a_call`, `conflicts`, `missing`, `doubts`.
    pub kind: &'static str,
    pub count: usize,
    /// For `missing`: the first data not found (`section.field`), the screen words them.
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CallCard {
    pub title: Option<String>,
    pub funder: Option<String>,
    pub edition: Option<String>,
    /// The AI's «en pocas palabras» (3 or 4 sentences), when it has been written and checked.
    pub brief: Option<String>,
    /// What stands in for it until then: the first sentence of what the document says the call is for.
    pub lead: Option<String>,
    pub facts: Vec<CardFact>,
    pub blocks: Vec<CardBlock>,
    pub alerts: Vec<CardAlert>,
}

/// Cuts at a word, never in the middle of one.
pub(crate) fn short(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    let at_word = match cut.rfind(char::is_whitespace) {
        Some(i) if i > max / 2 => &cut[..i],
        _ => cut.as_str(),
    };
    format!("{}…", at_word.trim_end_matches(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | ':' | '.' | '(' | '-')))
}

fn first_sentence(s: &str) -> String {
    let end = [". ", ".\n"].iter().filter_map(|m| s.find(m)).min().map(|i| i + 1);
    short(&s[..end.unwrap_or(s.len())], LEAD_CHARS)
}

/// A reading sometimes leaves a bare number where a date should be («15»): that is no date to show. A date has a word
/// (a month, «a partir de abril») or enough digits for day, month and year.
fn looks_like_a_date(s: &str) -> bool {
    s.chars().any(char::is_alphabetic) || s.chars().filter(char::is_ascii_digit).count() >= 6
}

/// The latest milestone of the given kinds: with a date the code normalized, the one that is latest; if none has
/// one, the first the document gives.
fn latest_milestone<'a>(doc: &'a Value, kinds: &[&str]) -> Option<&'a Value> {
    let hitos: Vec<&Value> = doc["temporalidad"]["hitos"]["hitos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|h| h["tipo"].as_str().is_some_and(|t| kinds.contains(&t)) && text(&h["inicio_texto"]).is_some_and(|t| looks_like_a_date(&t)))
        .collect();
    hitos
        .iter()
        .filter_map(|h| Some((h["normalizado"]["inicio"]["fecha"].as_str()?.to_string(), *h)))
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, h)| h)
        .or_else(|| hitos.first().copied())
}

/// When the person has to act. The latest closing date the call gives (a project can apply while any is ahead, so the
/// latest is the one to meet); a call that gives no closing date but says when to register or apply gives that.
fn when_to_apply(doc: &Value) -> Option<CardFact> {
    let (kind, hito) = match latest_milestone(doc, &["cierre"]) {
        Some(h) => ("closing", h),
        None => ("registration", latest_milestone(doc, &["registro", "apertura"])?),
    };
    let (page, file, _) = source(hito);
    Some(CardFact { kind, value: short(&text(&hito["inicio_texto"])?, FACT_CHARS), page, file })
}

fn facts(doc: &Value, summary: &CallSummary) -> Vec<CardFact> {
    let mut out = Vec::new();
    for kind in ["max_amount", "min_amount", "cofunding", "admin_cap"] {
        if let Some(a) = summary.amounts.iter().find(|a| a.kind == kind && !a.value.is_empty()) {
            out.push(CardFact { kind, value: short(&a.value, FACT_CHARS), page: a.page, file: a.file.clone() });
        }
    }
    if let Some((value, page, file)) = single(&doc["temporalidad"]["periodo_ejecucion"]) {
        out.push(CardFact { kind: "duration", value: short(&value, FACT_CHARS), page, file });
    }
    out.extend(when_to_apply(doc));
    // a call that gives its money by modality has no single amount: name the modalities
    if !out.iter().any(|f| matches!(f.kind, "max_amount" | "min_amount")) {
        let named: Vec<&SummaryAmount> = summary.amounts.iter().filter(|a| a.kind == "modality" && a.label.is_some()).collect();
        if let Some(first) = named.first() {
            let value = named.iter().take(MAX_POINTS).filter_map(|a| a.label.as_deref()).collect::<Vec<_>>().join(" · ");
            out.push(CardFact { kind: "modalities", value: short(&value, POINT_CHARS), page: first.page, file: first.file.clone() });
        }
    }
    out
}

fn point(item: &SummaryItem) -> CardPoint {
    CardPoint { text: short(&item.text, POINT_CHARS), applies_to: item.applies_to.as_deref().map(|a| short(a, FACT_CHARS)), page: item.page, file: item.file.clone() }
}

/// The items of the groups, in the order given; the first `MAX_POINTS` become points and the rest is counted.
fn block(key: &'static str, groups: &[&SummaryGroup]) -> Option<CardBlock> {
    // the same line under two groups of one topic (a requirement the reading also filed as who can take part) is said once
    let mut seen: Vec<String> = Vec::new();
    let items: Vec<&SummaryItem> = groups
        .iter()
        .flat_map(|g| g.items.iter())
        .filter(|i| {
            let key = i.text.to_lowercase();
            let new = !seen.contains(&key);
            seen.push(key);
            new
        })
        .collect();
    if items.is_empty() {
        return None;
    }
    Some(CardBlock { key, points: items.iter().take(MAX_POINTS).map(|i| point(i)).collect(), more: items.len().saturating_sub(MAX_POINTS) })
}

fn blocks(summary: &CallSummary) -> Vec<CardBlock> {
    let group = |key: &str| summary.groups.iter().find(|g| g.key == key);
    let pick = |keys: &[&str]| -> Vec<&SummaryGroup> { keys.iter().filter_map(|k| group(k)).collect() };
    let supported = if group("supported").is_some() { pick(&["supported"]) } else { pick(&["objectives"]) };
    let mut out: Vec<CardBlock> = Vec::new();
    for b in [
        block("who_can", &pick(&["who_can", "org_requirements"])),
        block("supported", &supported),
        block("fundable", &pick(&["fundable"])),
        block("not_fundable", &pick(&["not_fundable"])),
    ]
    .into_iter()
    .flatten()
    {
        // a reading sometimes files the same lines under two topics («qué apoya» and «lo que se puede pagar»):
        // saying the same thing twice is noise, so the later block is left to the detail
        let same = |o: &CardBlock| o.points.iter().map(|p| p.text.to_lowercase()).eq(b.points.iter().map(|p| p.text.to_lowercase()));
        if !out.iter().any(same) {
            out.push(b);
        }
    }
    out
}

fn alerts(summary: &CallSummary) -> Vec<CardAlert> {
    let mut out = Vec::new();
    if summary.document_kind.as_ref().is_some_and(|k| NOT_A_CALL.contains(&k.class.as_str())) {
        out.push(CardAlert { kind: "not_a_call", count: 1, fields: vec![] });
    }
    if !summary.conflicts.is_empty() {
        out.push(CardAlert { kind: "conflicts", count: summary.conflicts.len(), fields: vec![] });
    }
    if !summary.missing.is_empty() {
        out.push(CardAlert { kind: "missing", count: summary.missing.len(), fields: summary.missing.iter().take(MISSING_NAMED).cloned().collect() });
    }
    if !summary.doubts.is_empty() {
        out.push(CardAlert { kind: "doubts", count: summary.doubts.len(), fields: vec![] });
    }
    out
}

/// The card of a call from its canonical document and the summary of it, with the AI's brief if there is one.
pub fn card_of(doc: &Value, summary: &CallSummary, brief: Option<String>) -> CallCard {
    CallCard {
        title: summary.title.clone(),
        funder: summary.funder.clone(),
        edition: summary.edition.clone(),
        brief: brief.map(|b| b.trim().to_string()).filter(|b| !b.is_empty()),
        lead: summary.objective.as_deref().map(first_sentence).filter(|l| !l.is_empty()),
        facts: facts(doc, summary),
        blocks: blocks(summary),
        alerts: alerts(summary),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::canonical::summary::summarize;
    use serde_json::json;

    fn ev(cita: &str, pagina: u64) -> Value {
        json!([{ "cita": cita, "pagina": pagina, "documento": "bases.pdf" }])
    }

    fn items(n: usize, what: &str) -> Value {
        let elementos: Vec<Value> = (0..n).map(|i| json!({ "titulo": format!("{what} número {i} con una descripción larga que pide cuidado para no cortarse a la mitad de una palabra cuando la ficha la acorta para la persona que la lee"), "evidencias": ev("cita", 2), "aplica_a": null, "obligatoriedad": "obligatorio" })).collect();
        json!({ "estado": "encontrado", "elementos": elementos })
    }

    fn doc() -> Value {
        json!({
            "identidad": {
                "tipo_de_documento": { "estado": "encontrado", "valor_texto": "Convocatoria", "evidencias": ev("CONVOCATORIA", 1), "normalizado": { "clase": "convocatoria" } },
                "nombre": { "estado": "encontrado", "valor_texto": "Convocatoria Ejemplo 2027", "evidencias": ev("CONVOCATORIA EJEMPLO 2027", 1) },
                "convocante": { "estado": "encontrado", "valor_texto": "Fundación Ficticia", "evidencias": ev("Fundación Ficticia A.C.", 1) },
                "edicion": { "estado": "no_aparece", "valor_texto": null, "evidencias": [] },
                "objetivo_general": { "estado": "encontrado", "valor_texto": "Apoyar a casas hogar y asilos en su alimentación. Con recursos de la fundación.", "evidencias": ev("Apoyar a casas hogar", 2) },
                "que_apoya": { "estado": "encontrado", "elementos": [{ "titulo": "Proyectos de alimentación", "evidencias": ev("Proyectos de alimentación.", 2), "aplica_a": null, "obligatoriedad": "no_indicado" }] }
            },
            "temporalidad": {
                "periodo_ejecucion": { "estado": "encontrado", "valor_texto": "12 meses", "evidencias": ev("hasta 12 meses", 3), "normalizado": { "valor": 12.0, "unidad": "meses" } },
                "hitos": { "estado": "encontrado", "hitos": [
                    { "tipo": "cierre", "etiqueta": "Cierre de registro", "inicio_texto": "15 de abril de 2027", "evidencias": ev("Cierre de registro: 15 de abril de 2027", 3), "normalizado": { "inicio": { "fecha": "2027-04-15", "precision": "dia" } } },
                    { "tipo": "cierre", "etiqueta": "Cierre de postulación", "inicio_texto": "23 de mayo de 2027", "evidencias": ev("Cierre: 23 de mayo de 2027", 4), "normalizado": { "inicio": { "fecha": "2027-05-23", "precision": "dia" } } }
                ] }
            },
            "elegibilidad": {
                "quienes_pueden_participar": { "estado": "encontrado", "elementos": [{ "titulo": "Asociaciones civiles", "evidencias": ev("Podrán participar las asociaciones civiles.", 2), "aplica_a": "con dos años de operación", "obligatoriedad": "obligatorio" }] },
                "requisitos_organizacion": { "estado": "encontrado", "elementos": [{ "titulo": "Estar al corriente con el SAT", "evidencias": ev("Estar al corriente con el SAT", 2), "aplica_a": null, "obligatoriedad": "obligatorio" }] }
            },
            "financiamiento": {
                "monto_maximo": { "estado": "encontrado", "valor_texto": "$250,000", "evidencias": ev("Monto máximo: $250,000 pesos", 2) },
                "monto_minimo": { "estado": "no_aparece", "valor_texto": null, "evidencias": [] },
                "porcentaje_cofinanciamiento": { "estado": "encontrado", "valor_texto": "20%", "evidencias": ev("contrapartida del 20%", 3) }
            },
            "entrega": {},
            "conflictos": [{ "campo": "contrapartida", "nota": "Dicen 30 % y 20 %.", "versiones": [] }],
            "dudas": [{ "texto": "No se dice cuántos proyectos se apoyarán.", "evidencias": [] }, { "texto": "No se dice si se puede repetir.", "evidencias": [] }]
        })
    }

    fn card(doc: &Value, brief: Option<&str>) -> CallCard {
        card_of(doc, &summarize(doc), brief.map(String::from))
    }

    #[test]
    fn the_figures_are_the_documents_own_words_with_their_page_and_the_latest_closing_is_the_one_shown() {
        let c = card(&doc(), None);
        let f = |k: &str| c.facts.iter().find(|f| f.kind == k).map(|f| (f.value.as_str(), f.page, f.file.as_deref()));
        assert_eq!(f("max_amount"), Some(("$250,000", Some(2), Some("bases.pdf"))));
        assert_eq!(f("cofunding"), Some(("20%", Some(3), Some("bases.pdf"))));
        assert_eq!(f("duration"), Some(("12 meses", Some(3), Some("bases.pdf"))));
        assert_eq!(f("closing"), Some(("23 de mayo de 2027", Some(4), Some("bases.pdf"))), "the later of the two closings");
        assert_eq!(f("min_amount"), None, "what the document does not say is not shown");
        assert_eq!(c.facts.iter().map(|f| f.kind).collect::<Vec<_>>(), ["max_amount", "cofunding", "duration", "closing"]);
        assert_eq!((c.title.as_deref(), c.funder.as_deref(), c.edition), (Some("Convocatoria Ejemplo 2027"), Some("Fundación Ficticia"), None));
    }

    #[test]
    fn a_call_with_no_closing_date_gives_when_to_register_instead_and_an_info_session_is_not_a_deadline() {
        let mut d = doc();
        d["temporalidad"]["hitos"]["hitos"] = json!([
            { "tipo": "sesion_informativa", "etiqueta": "Sesión informativa", "inicio_texto": "7 abril 2026", "evidencias": ev("Sesión 7 abril", 3) },
            { "tipo": "registro", "etiqueta": "Formulario", "inicio_texto": "8 de abril 2026", "evidencias": ev("Formulario 8 de abril", 4), "normalizado": { "inicio": { "fecha": "2026-04-08", "precision": "dia" } } },
            { "tipo": "registro", "etiqueta": "Postulación", "inicio_texto": "ABRIL", "evidencias": ev("Postulación ABRIL", 4) }
        ]);
        let c = card(&d, None);
        let f = c.facts.iter().find(|f| f.kind == "registration").expect("the registration stands in for the closing");
        assert_eq!((f.value.as_str(), f.page), ("8 de abril 2026", Some(4)), "the one with a date the code could read");
        assert!(c.facts.iter().all(|f| f.kind != "closing"));
        // nothing that says when to act: no date at all, nothing made up
        d["temporalidad"]["hitos"]["hitos"] = json!([{ "tipo": "sesion_informativa", "etiqueta": "Sesión", "inicio_texto": "7 abril 2026", "evidencias": ev("x", 3) }]);
        assert!(card(&d, None).facts.iter().all(|f| f.kind != "closing" && f.kind != "registration"));
    }

    #[test]
    fn a_bare_number_where_a_date_should_be_is_not_shown_as_one() {
        assert!(looks_like_a_date("23 de mayo de 2027") && looks_like_a_date("ABRIL") && looks_like_a_date("15/04/2027"));
        assert!(!looks_like_a_date("15") && !looks_like_a_date("2027"));
        let mut d = doc();
        d["temporalidad"]["hitos"]["hitos"] = json!([{ "tipo": "registro", "etiqueta": "Registro", "inicio_texto": "15", "evidencias": ev("15", 3) }]);
        assert!(card(&d, None).facts.iter().all(|f| f.kind != "registration"));
    }

    #[test]
    fn a_line_filed_under_two_groups_of_the_same_topic_is_said_once_and_only_the_distinct_ones_are_counted() {
        let mut d = doc();
        let same = json!({ "estado": "encontrado", "elementos": [{ "titulo": "Ser una organización sin fines de lucro", "evidencias": ev("c", 2), "aplica_a": null, "obligatoriedad": "obligatorio" }, { "titulo": "Estar al corriente con el SAT", "evidencias": ev("c", 2), "aplica_a": null, "obligatoriedad": "obligatorio" }] });
        d["elegibilidad"]["quienes_pueden_participar"] = same.clone();
        d["elegibilidad"]["requisitos_organizacion"] = same;
        let who = card(&d, None).blocks.into_iter().find(|b| b.key == "who_can").unwrap();
        assert_eq!(who.points.len(), 2);
        assert_eq!(who.more, 0, "the repeats are not counted as «y más»");
    }

    #[test]
    fn the_same_lines_filed_under_two_topics_are_shown_once() {
        let mut d = doc();
        d["identidad"]["que_apoya"] = items(5, "Gasto");
        d["financiamiento"]["conceptos_financiables"] = items(5, "Gasto");
        d["financiamiento"]["conceptos_no_financiables"] = items(2, "Exclusión");
        let c = card(&d, None);
        let keys: Vec<&str> = c.blocks.iter().map(|b| b.key).collect();
        assert_eq!(keys, ["who_can", "supported", "not_fundable"], "«lo que sí se puede pagar» repeated «qué apoya» and is left to the detail");
    }

    #[test]
    fn who_can_takes_part_comes_first_then_what_the_organization_must_meet_and_the_rest_is_counted() {
        let c = card(&doc(), None);
        let who = c.blocks.iter().find(|b| b.key == "who_can").unwrap();
        assert_eq!(who.points.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(), ["Asociaciones civiles", "Estar al corriente con el SAT"]);
        assert_eq!(who.points[0].applies_to.as_deref(), Some("con dos años de operación"));
        assert_eq!(who.more, 0);

        let mut d = doc();
        d["elegibilidad"]["quienes_pueden_participar"] = items(5, "Participante");
        let who = card(&d, None).blocks.into_iter().find(|b| b.key == "who_can").unwrap();
        assert_eq!((who.points.len(), who.more), (3, 3), "5 + 1 requirement = 6 lines: three shown, three counted");
        assert!(card(&doc(), None).blocks.iter().all(|b| b.key != "fundable"), "a topic with nothing in it is not shown");
    }

    #[test]
    fn what_it_supports_falls_back_to_the_objectives_of_the_project() {
        let mut d = doc();
        d["identidad"]["que_apoya"] = json!({ "estado": "no_aparece", "elementos": [] });
        d["proyecto"] = json!({ "objetivos": { "estado": "encontrado", "elementos": [{ "titulo": "Mejorar la nutrición", "evidencias": ev("Mejorar la nutrición", 5), "aplica_a": null, "obligatoriedad": "no_indicado" }] } });
        let c = card(&d, None);
        assert_eq!(c.blocks.iter().find(|b| b.key == "supported").unwrap().points[0].text, "Mejorar la nutrición");
    }

    #[test]
    fn a_point_is_cut_at_a_word_never_in_the_middle_of_one() {
        let mut d = doc();
        d["elegibilidad"]["quienes_pueden_participar"] = items(1, "Participante");
        let whole = summarize(&d).groups.iter().find(|g| g.key == "who_can").unwrap().items[0].text.clone();
        let who = card(&d, None).blocks.into_iter().find(|b| b.key == "who_can").unwrap();
        let t = &who.points[0].text;
        assert!(t.chars().count() <= POINT_CHARS + 1 && t.ends_with('…'), "{t}");
        let body = t.trim_end_matches('…');
        assert!(whole.starts_with(body) && whole[body.len()..].starts_with(' '), "cut in the middle of a word: {t}");
        assert_eq!(short("corto", 10), "corto");
        assert_eq!(short("una frase de varias palabras largas", 20), "una frase de varias…");
        assert_eq!(short("palabrasinespacios".repeat(3).as_str(), 20).chars().count(), 21, "a word longer than the room is cut as a last resort");
    }

    #[test]
    fn the_lead_is_the_first_sentence_of_the_objective_and_the_brief_passes_through() {
        let c = card(&doc(), Some("  Esta convocatoria apoya la alimentación.  "));
        assert_eq!(c.lead.as_deref(), Some("Apoyar a casas hogar y asilos en su alimentación."));
        assert_eq!(c.brief.as_deref(), Some("Esta convocatoria apoya la alimentación."));
        assert_eq!(card(&doc(), Some("   ")).brief, None, "a blank brief is no brief");
        let mut d = doc();
        d["identidad"]["objetivo_general"] = json!({ "estado": "no_aparece", "valor_texto": null, "evidencias": [] });
        assert_eq!(card(&d, None).lead, None);
    }

    #[test]
    fn alerts_are_only_there_when_they_are_true() {
        let c = card(&doc(), None);
        let a = |k: &str| c.alerts.iter().find(|a| a.kind == k);
        assert_eq!(a("conflicts").map(|a| a.count), Some(1));
        assert_eq!(a("doubts").map(|a| a.count), Some(2));
        let missing = a("missing").unwrap();
        assert!(missing.count >= 1 && missing.fields.len() <= MISSING_NAMED && missing.fields.contains(&"entrega.medio_o_lugar".to_string()), "{missing:?}");
        assert!(a("not_a_call").is_none());

        let mut d = doc();
        d["identidad"]["tipo_de_documento"]["normalizado"]["clase"] = json!("aviso");
        d["conflictos"] = json!([]);
        d["dudas"] = json!([]);
        let c = card(&d, None);
        assert!(c.alerts.iter().any(|a| a.kind == "not_a_call") && c.alerts.iter().all(|a| a.kind != "conflicts" && a.kind != "doubts"));
    }

    #[test]
    fn a_call_that_pays_by_modality_names_the_modalities_when_it_has_no_single_amount() {
        let mut d = doc();
        d["financiamiento"] = json!({ "modalidades": { "estado": "encontrado", "modalidades": [
            { "nombre": "Seguridad alimentaria", "evidencias": ev("Seguridad alimentaria", 4), "monto_maximo": { "estado": "encontrado", "valor_texto": "$100,000", "evidencias": ev("hasta $100,000", 4) } },
            { "nombre": "Infraestructura", "evidencias": ev("Infraestructura", 5) }
        ] } });
        let c = card(&d, None);
        let m = c.facts.iter().find(|f| f.kind == "modalities").unwrap();
        assert_eq!((m.value.as_str(), m.page), ("Seguridad alimentaria · Infraestructura", Some(4)));
        assert!(card(&doc(), None).facts.iter().all(|f| f.kind != "modalities"), "with a single amount the modalities stay in the detail");
    }

    #[test]
    fn an_empty_document_gives_an_empty_card_that_only_says_what_is_missing() {
        let c = card(&json!({}), None);
        assert!(c.facts.is_empty() && c.blocks.is_empty() && c.brief.is_none() && c.lead.is_none());
        assert_eq!(c.alerts.iter().map(|a| a.kind).collect::<Vec<_>>(), ["missing"]);
    }

    /// The card is short whatever the call: this is what keeps it from turning into the list it replaces.
    #[test]
    fn the_card_stays_short_even_for_a_call_with_everything_and_a_lot_of_it() {
        let mut d = doc();
        d["elegibilidad"]["quienes_pueden_participar"] = items(40, "Participante");
        d["elegibilidad"]["requisitos_organizacion"] = items(40, "Requisito");
        d["identidad"]["que_apoya"] = items(40, "Apoyo");
        d["financiamiento"]["conceptos_financiables"] = items(40, "Concepto");
        d["financiamiento"]["conceptos_no_financiables"] = items(40, "Exclusión");
        d["conflictos"] = json!((0..10).map(|i| json!({ "campo": format!("c{i}"), "nota": "x", "versiones": [] })).collect::<Vec<_>>());
        let c = card(&d, Some(&"Una frase del resumen. ".repeat(20)));
        assert!(c.blocks.len() <= 4 && c.blocks.iter().all(|b| b.points.len() <= MAX_POINTS));
        assert_eq!(c.blocks.iter().find(|b| b.key == "who_can").unwrap().more, 40 + 40 - MAX_POINTS);
        let size = serde_json::to_string(&c).unwrap().chars().count();
        assert!(size < 4500, "the card grew to {size} characters");
    }

    /// Prints the card of a real call's canonical document, to look at it by eye. No calls are made.
    #[test]
    #[ignore = "reads a file named by CIMIENTO_CANON_FILE; prints, asserts nothing"]
    fn print_card_of_a_real_call() {
        let path = std::env::var("CIMIENTO_CANON_FILE").expect("CIMIENTO_CANON_FILE names a .canonico.json");
        let doc: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let c = card(&doc, None);
        eprintln!("{}", serde_json::to_string_pretty(&c).unwrap());
        eprintln!("({} characters)", serde_json::to_string(&c).unwrap().chars().count());
    }
}
