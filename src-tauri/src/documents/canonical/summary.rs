//! What a person sees of a reading: «this is what we understood of the call». Built from the canonical
//! document only, with no model and no rule about any funder. Every line carries the page and the file
//! it was copied from, so it can be checked against the original.
//!
//! Group names are keys (`who_can`, `docs_required`...): the words for them live in the app's texts.

use serde::Serialize;
use serde_json::Value;

/// How long a quote may be to stand in as the text of a line.
const MAX_LINE_CHARS: usize = 320;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryItem {
    pub text: String,
    /// Who or what the line applies to, only when the document restricts it («organizaciones nuevas»).
    pub applies_to: Option<String>,
    /// `obligatorio`, `deseable`, `informativo`; absent when the document does not say.
    pub requirement: Option<String>,
    pub page: Option<u64>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryGroup {
    pub key: &'static str,
    pub items: Vec<SummaryItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryDate {
    pub label: String,
    /// The date as the document writes it (a range reads «inicio al fin»).
    pub when: String,
    pub kind: String,
    pub page: Option<u64>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryAmount {
    /// `max_amount`, `min_amount`, `cofunding`, `admin_cap`, `modality`.
    pub kind: &'static str,
    pub label: Option<String>,
    pub value: String,
    pub page: Option<u64>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryConflict {
    pub field: String,
    pub note: String,
    pub versions: Vec<SummaryItem>,
}

/// What the document calls itself and the class the code reads from those words (`convocatoria`,
/// `reglas_de_operacion`, `lineamientos`, `aviso`, `guia`, `formato`, `anexo`, `otro`). Descriptive: it never
/// decides what is read.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryKind {
    pub words: String,
    pub class: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CallSummary {
    pub document_kind: Option<SummaryKind>,
    pub title: Option<String>,
    pub funder: Option<String>,
    pub edition: Option<String>,
    pub objective: Option<String>,
    pub dates: Vec<SummaryDate>,
    pub amounts: Vec<SummaryAmount>,
    pub groups: Vec<SummaryGroup>,
    pub conflicts: Vec<SummaryConflict>,
    pub doubts: Vec<String>,
    /// Fields the reading looked for and the documents do not say (`section.field`).
    pub missing: Vec<String>,
}

/// The groups that tell a conversation what the call is for, in the order they matter, with the words the AI reads.
const CONTEXT_GROUPS: [(&str, &str); 10] = [
    ("supported", "Qué apoya"),
    ("fundable", "Lo que sí se puede pagar"),
    ("not_fundable", "Lo que no se puede pagar"),
    ("target_population", "A quién debe atender el proyecto"),
    ("who_can", "Quién puede participar"),
    ("objectives", "Objetivos del proyecto"),
    ("expected_results", "Resultados esperados"),
    ("indicators", "Indicadores"),
    ("project_types", "Tipos de proyecto"),
    ("regions", "Dónde"),
];

fn amount_label(kind: &str) -> &'static str {
    match kind {
        "max_amount" => "Monto máximo por proyecto",
        "min_amount" => "Monto mínimo por proyecto",
        "cofunding" => "Lo que debe aportar la organización",
        "admin_cap" => "Tope para gastos administrativos",
        _ => "Tipo de apoyo",
    }
}

/// Adds the line if it fits whole: a context is cut between lines, never in the middle of one.
fn push_line(out: &mut String, line: &str, max_chars: usize) -> bool {
    if out.chars().count() + line.chars().count() + 1 > max_chars {
        return false;
    }
    out.push_str(line);
    out.push('\n');
    true
}

impl CallSummary {
    /// What the call is for, in plain lines for the conversation of the diagnosis: who gives it, what it
    /// supports and what it does not, for whom, with which indicators. Cut between lines to `max_chars`.
    pub fn context_text(&self, max_chars: usize) -> String {
        let mut out = String::new();
        let mut lines: Vec<String> = Vec::new();
        if let Some(t) = &self.title {
            lines.push(format!("Convocatoria: {t}"));
        }
        if let Some(f) = &self.funder {
            lines.push(format!("Quién convoca: {f}"));
        }
        if let Some(e) = &self.edition {
            lines.push(format!("Edición: {e}"));
        }
        if let Some(o) = &self.objective {
            lines.push(format!("Para qué es: {o}"));
        }
        for a in &self.amounts {
            let what = a.label.as_deref().map(|l| format!("{} ({l})", amount_label(a.kind))).unwrap_or_else(|| amount_label(a.kind).to_string());
            lines.push(format!("{what}: {}", a.value));
        }
        for line in &lines {
            if !push_line(&mut out, line, max_chars) {
                return out.trim_end().to_string();
            }
        }
        for (key, label) in CONTEXT_GROUPS {
            let Some(group) = self.groups.iter().find(|g| g.key == key) else { continue };
            let mut items = group.items.iter();
            let Some(first) = items.next() else { continue };
            // the title of a group only goes in if at least its first line does too
            let mut probe = out.clone();
            if !push_line(&mut probe, &format!("{label}:"), max_chars) || !push_line(&mut probe, &item_line(first), max_chars) {
                continue;
            }
            out = probe;
            for item in items {
                if !push_line(&mut out, &item_line(item), max_chars) {
                    break;
                }
            }
        }
        out.trim_end().to_string()
    }
}

fn item_line(item: &SummaryItem) -> String {
    match &item.applies_to {
        Some(who) => format!("- {} (aplica a: {who})", item.text),
        None => format!("- {}", item.text),
    }
}

pub(super) fn text(v: &Value) -> Option<String> {
    v.as_str().map(|s| s.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|s| !s.is_empty())
}

fn clip(s: &str) -> String {
    if s.chars().count() <= MAX_LINE_CHARS {
        return s.to_string();
    }
    let cut: String = s.chars().take(MAX_LINE_CHARS).collect();
    format!("{}…", cut.trim_end())
}

/// The first quote of a node: where its line comes from.
pub(super) fn source(node: &Value) -> (Option<u64>, Option<String>, Option<String>) {
    let e = &node["evidencias"][0];
    (e["pagina"].as_u64(), text(&e["documento"]), text(&e["cita"]))
}

pub(super) fn single(field: &Value) -> Option<(String, Option<u64>, Option<String>)> {
    if field["estado"].as_str() == Some("no_aparece") {
        return None;
    }
    let (page, file, quote) = source(field);
    text(&field["valor_texto"]).or(quote).map(|t| (clip(&t), page, file))
}

fn items_of(list: &Value, key: &str) -> Vec<SummaryItem> {
    list[key]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|it| {
            let (page, file, quote) = source(it);
            let line = text(&it["titulo"]).or(quote)?;
            Some(SummaryItem {
                text: clip(&line),
                applies_to: text(&it["aplica_a"]),
                requirement: it["obligatoriedad"].as_str().filter(|o| matches!(*o, "obligatorio" | "deseable" | "informativo")).map(String::from),
                page,
                file,
            })
        })
        .collect()
}

fn criteria(list: &Value) -> Vec<SummaryItem> {
    list["criterios"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|it| {
            let (page, file, quote) = source(it);
            let title = text(&it["titulo"]).or(quote)?;
            let line = match text(&it["ponderacion"]).or_else(|| it["ponderacion"].as_f64().map(|n| n.to_string())) {
                Some(w) => format!("{} ({w})", clip(&title)),
                None => clip(&title),
            };
            Some(SummaryItem { text: line, applies_to: None, requirement: None, page, file })
        })
        .collect()
}

fn dates(doc: &Value) -> Vec<SummaryDate> {
    doc["temporalidad"]["hitos"]["hitos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| {
            let start = text(&h["inicio_texto"])?;
            let when = match text(&h["fin_texto"]) {
                Some(end) => format!("{start} al {end}"),
                None => start,
            };
            let (page, file, quote) = source(h);
            Some(SummaryDate { label: text(&h["etiqueta"]).or(quote).map(|t| clip(&t)).unwrap_or_default(), when, kind: h["tipo"].as_str().unwrap_or("otro").to_string(), page, file })
        })
        .collect()
}

fn amounts(doc: &Value) -> Vec<SummaryAmount> {
    let f = &doc["financiamiento"];
    let mut out = Vec::new();
    for (kind, field) in [("max_amount", "monto_maximo"), ("min_amount", "monto_minimo"), ("cofunding", "porcentaje_cofinanciamiento"), ("admin_cap", "tope_gastos_administrativos")] {
        if let Some((value, page, file)) = single(&f[field]) {
            out.push(SummaryAmount { kind, label: None, value, page, file });
        }
    }
    for m in f["modalidades"]["modalidades"].as_array().into_iter().flatten() {
        let parts: Vec<String> = [("monto_minimo", "mínimo"), ("monto_maximo", "máximo"), ("porcentaje_cofinanciamiento", "contrapartida"), ("duracion_maxima", "duración")]
            .iter()
            .filter_map(|(k, name)| single(&m[*k]).map(|(v, _, _)| format!("{name} {v}")))
            .collect();
        let (page, file, _) = source(m);
        out.push(SummaryAmount { kind: "modality", label: text(&m["nombre"]), value: parts.join(" · "), page, file });
    }
    out
}

/// Group key, section and field, in the order a person reads a call: who, what, how much, how it is judged.
const GROUPS: [(&str, &str, &str); 24] = [
    ("supported", "identidad", "que_apoya"),
    ("who_can", "elegibilidad", "quienes_pueden_participar"),
    ("who_cannot", "elegibilidad", "quienes_no_pueden"),
    ("org_requirements", "elegibilidad", "requisitos_organizacion"),
    ("regions", "elegibilidad", "regiones"),
    ("target_population", "elegibilidad", "poblacion_objetivo"),
    ("participation_limits", "elegibilidad", "limites_participacion"),
    ("fundable", "financiamiento", "conceptos_financiables"),
    ("not_fundable", "financiamiento", "conceptos_no_financiables"),
    ("support_conditions", "financiamiento", "condiciones_del_apoyo"),
    ("project_types", "proyecto", "tipos"),
    ("objectives", "proyecto", "objetivos"),
    ("expected_results", "proyecto", "resultados_esperados"),
    ("indicators", "proyecto", "indicadores"),
    ("project_requirements", "proyecto", "requisitos_del_proyecto"),
    ("docs_required", "documentacion", "obligatoria"),
    ("docs_conditional", "documentacion", "condicional"),
    ("docs_optional", "documentacion", "opcional"),
    ("formats", "documentacion", "formatos_a_utilizar"),
    ("priorities", "evaluacion", "prioridades"),
    ("disqualification", "evaluacion", "causas_descalificacion"),
    ("evaluation_process", "evaluacion", "proceso_evaluacion"),
    ("how_to_deliver", "entrega", "instrucciones"),
    ("contact", "entrega", "contacto"),
];

/// Fields whose absence a person should hear about: the ones a decision to take part rests on.
const KEY_FIELDS: [(&str, &str); 8] = [
    ("identidad", "nombre"),
    ("identidad", "convocante"),
    ("financiamiento", "monto_maximo"),
    ("financiamiento", "porcentaje_cofinanciamiento"),
    ("temporalidad", "periodo_ejecucion"),
    ("entrega", "medio_o_lugar"),
    ("evaluacion", "criterios"),
    ("documentacion", "obligatoria"),
];

pub fn summarize(doc: &Value) -> CallSummary {
    let mut groups = Vec::new();
    for (key, section, field) in GROUPS {
        let items = items_of(&doc[section][field], "elementos");
        if !items.is_empty() {
            groups.push(SummaryGroup { key, items });
        }
    }
    let criteria = criteria(&doc["evaluacion"]["criterios"]);
    if !criteria.is_empty() {
        groups.push(SummaryGroup { key: "criteria", items: criteria });
    }
    let conflicts = doc["conflictos"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| SummaryConflict {
            field: c["campo"].as_str().unwrap_or("").to_string(),
            note: text(&c["nota"]).unwrap_or_default(),
            versions: c["versiones"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| {
                    let e = &v["evidencia"];
                    let line = text(&v["valor_texto"]).or_else(|| text(&e["cita"]))?;
                    Some(SummaryItem { text: clip(&line), applies_to: None, requirement: None, page: e["pagina"].as_u64(), file: text(&e["documento"]) })
                })
                .collect(),
        })
        .collect();
    let missing = KEY_FIELDS
        .iter()
        .filter(|(s, f)| {
            let node = &doc[*s][*f];
            node["estado"].as_str().map_or(true, |e| e == "no_aparece") && node["elementos"].as_array().map_or(true, Vec::is_empty) && node["criterios"].as_array().map_or(true, Vec::is_empty)
        })
        .map(|(s, f)| format!("{s}.{f}"))
        .collect();
    let document_kind = single(&doc["identidad"]["tipo_de_documento"]).map(|(words, _, _)| SummaryKind {
        words,
        class: doc["identidad"]["tipo_de_documento"]["normalizado"]["clase"].as_str().unwrap_or("otro").to_string(),
    });
    CallSummary {
        document_kind,
        title: single(&doc["identidad"]["nombre"]).map(|x| x.0),
        funder: single(&doc["identidad"]["convocante"]).map(|x| x.0),
        edition: single(&doc["identidad"]["edicion"]).map(|x| x.0),
        objective: single(&doc["identidad"]["objetivo_general"]).map(|x| x.0),
        dates: dates(doc),
        amounts: amounts(doc),
        groups,
        conflicts,
        doubts: doc["dudas"].as_array().into_iter().flatten().filter_map(|d| text(&d["texto"])).collect(),
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ev(cita: &str, pagina: u64, documento: &str) -> Value {
        json!([{ "cita": cita, "pagina": pagina, "documento": documento }])
    }

    fn doc() -> Value {
        json!({
            "identidad": {
                "tipo_de_documento": { "estado": "encontrado", "valor_texto": "Convocatoria", "evidencias": ev("CONVOCATORIA EJEMPLO 2027", 1, "bases.pdf"), "normalizado": { "clase": "convocatoria" } },
                "nombre": { "estado": "encontrado", "valor_texto": "Convocatoria Ejemplo 2027", "evidencias": ev("CONVOCATORIA EJEMPLO 2027", 1, "bases.pdf") },
                "convocante": { "estado": "encontrado", "valor_texto": "Fundación Ficticia", "evidencias": ev("Fundación Ficticia A.C.", 1, "bases.pdf") },
                "edicion": { "estado": "no_aparece", "valor_texto": null, "evidencias": [] },
                "que_apoya": { "estado": "encontrado", "elementos": [{ "titulo": null, "evidencias": ev("Proyectos de\nalimentación.", 2, "bases.pdf"), "aplica_a": null, "obligatoriedad": "no_indicado" }] }
            },
            "temporalidad": { "hitos": { "estado": "encontrado", "hitos": [
                { "tipo": "cierre", "etiqueta": "Cierre de postulación", "inicio_texto": "23 de mayo de 2027", "fin_texto": null, "evidencias": ev("Cierre: 23 de mayo de 2027", 3, "bases.pdf") },
                { "tipo": "registro", "etiqueta": "Registro", "inicio_texto": "agosto", "fin_texto": "septiembre 2026", "evidencias": ev("Registro agosto y septiembre 2026", 3, "bases.pdf") }
            ] } },
            "elegibilidad": {
                "quienes_pueden_participar": { "estado": "encontrado", "elementos": [{ "titulo": "Asociaciones civiles", "evidencias": ev("Podrán participar las asociaciones civiles.", 2, "bases.pdf"), "aplica_a": "con dos años de operación", "obligatoriedad": "obligatorio" }] }
            },
            "financiamiento": {
                "monto_maximo": { "estado": "encontrado", "valor_texto": "$250,000", "evidencias": ev("Monto máximo: $250,000 pesos", 2, "bases.pdf") },
                "monto_minimo": { "estado": "no_aparece", "valor_texto": null, "evidencias": [] },
                "modalidades": { "estado": "encontrado", "modalidades": [{ "nombre": "Seguridad alimentaria", "evidencias": ev("Seguridad alimentaria hasta 70%", 4, "bases.pdf"),
                    "monto_minimo": { "estado": "no_aparece" }, "monto_maximo": { "estado": "no_aparece" },
                    "porcentaje_cofinanciamiento": { "estado": "encontrado", "valor_texto": "30%", "evidencias": ev("contrapartida mínima del 30%", 4, "bases.pdf") },
                    "duracion_maxima": { "estado": "encontrado", "valor_texto": "12 meses", "evidencias": ev("hasta 12 meses", 4, "bases.pdf") } }] }
            },
            "evaluacion": { "criterios": { "estado": "encontrado", "criterios": [{ "titulo": "Claridad del problema", "ponderacion": "20 puntos", "evidencias": ev("1. Claridad del problema (20 puntos)", 7, "bases.pdf") }] } },
            "entrega": {},
            "conflictos": [{ "campo": "contrapartida", "nota": "Dicen 30 % y 20 %.", "versiones": [
                { "valor_texto": "30%", "evidencia": { "cita": "contrapartida mínima del 30%", "pagina": 4, "documento": "bases.pdf" } },
                { "valor_texto": "20%", "evidencia": { "cita": "contrapartida mínima del 20%", "pagina": 14, "documento": "formato.docx" } }] }],
            "dudas": [{ "texto": "No se dice cuántos proyectos se apoyarán.", "evidencias": [] }]
        })
    }

    #[test]
    fn the_summary_reads_the_canonical_document_without_a_model() {
        let s = summarize(&doc());
        assert_eq!((s.title.as_deref(), s.funder.as_deref(), s.edition), (Some("Convocatoria Ejemplo 2027"), Some("Fundación Ficticia"), None));
        assert_eq!(s.document_kind, Some(SummaryKind { words: "Convocatoria".into(), class: "convocatoria".into() }));
        assert_eq!(s.dates.iter().map(|d| (d.label.as_str(), d.when.as_str(), d.page)).collect::<Vec<_>>(), vec![("Cierre de postulación", "23 de mayo de 2027", Some(3)), ("Registro", "agosto al septiembre 2026", Some(3))]);
        assert_eq!(s.amounts.iter().map(|a| (a.kind, a.value.as_str())).collect::<Vec<_>>(), vec![("max_amount", "$250,000"), ("modality", "contrapartida 30% · duración 12 meses")]);
        assert_eq!(s.amounts[1].label.as_deref(), Some("Seguridad alimentaria"));
        let who = s.groups.iter().find(|g| g.key == "who_can").unwrap();
        assert_eq!((who.items[0].text.as_str(), who.items[0].applies_to.as_deref(), who.items[0].requirement.as_deref()), ("Asociaciones civiles", Some("con dos años de operación"), Some("obligatorio")));
        // a line without a title is the quote itself, on one line, and «no_indicado» says nothing
        let supported = s.groups.iter().find(|g| g.key == "supported").unwrap();
        assert_eq!((supported.items[0].text.as_str(), supported.items[0].requirement.as_deref(), supported.items[0].file.as_deref()), ("Proyectos de alimentación.", None, Some("bases.pdf")));
        assert_eq!(s.groups.iter().find(|g| g.key == "criteria").unwrap().items[0].text, "Claridad del problema (20 puntos)");
        assert!(s.groups.iter().all(|g| g.key != "who_cannot"), "a group with nothing in it is not shown");
        assert_eq!((s.conflicts.len(), s.conflicts[0].versions.len(), s.conflicts[0].versions[1].file.as_deref()), (1, 2, Some("formato.docx")));
        assert_eq!(s.doubts, vec!["No se dice cuántos proyectos se apoyarán."]);
        assert!(s.missing.contains(&"entrega.medio_o_lugar".to_string()) && !s.missing.contains(&"financiamiento.monto_maximo".to_string()));
    }

    #[test]
    fn a_very_long_quote_is_cut_and_an_empty_document_gives_an_empty_summary() {
        let long = "palabra ".repeat(100);
        let d = json!({ "identidad": { "que_apoya": { "estado": "encontrado", "elementos": [{ "titulo": null, "evidencias": ev(&long, 1, "a.pdf"), "aplica_a": null, "obligatoriedad": "no_indicado" }] } } });
        let item = &summarize(&d).groups[0].items[0];
        assert!(item.text.chars().count() <= MAX_LINE_CHARS + 1 && item.text.ends_with('…'));
        let e = summarize(&json!({}));
        assert_eq!((e.title, e.dates.len(), e.groups.len(), e.conflicts.len()), (None, 0, 0, 0));
        assert_eq!(e.missing.len(), KEY_FIELDS.len());
    }

    #[test]
    fn the_context_for_the_conversation_names_the_call_what_it_supports_and_is_cut_between_lines() {
        let s = summarize(&doc());
        let ctx = s.context_text(3000);
        assert!(ctx.contains("Quién convoca: Fundación Ficticia"), "{ctx}");
        assert!(ctx.contains("Monto máximo por proyecto: $250,000"), "{ctx}");
        assert!(ctx.contains("Qué apoya:\n- Proyectos de alimentación."), "{ctx}");
        assert!(ctx.contains("- Asociaciones civiles (aplica a: con dos años de operación)"), "{ctx}");
        // a short budget keeps whole lines only and never a title with nothing under it
        let short = s.context_text(120);
        assert!(short.chars().count() <= 120, "{short}");
        assert!(short.lines().all(|l| !l.ends_with(':') || short.lines().count() > 1), "{short}");
        assert!(ctx.starts_with(&short), "the short one is the beginning of the long one");
        assert_eq!(s.context_text(0), "");
    }
}
