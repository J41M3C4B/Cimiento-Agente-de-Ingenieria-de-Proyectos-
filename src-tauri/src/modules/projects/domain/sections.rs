//! The sections of the project's guide and how the code plans them (ADR-018).
//!
//! First what is indispensable of the call (written by the code), then the proposal of the project. If the call
//! asks for a project proposal as a document of its own, the proposal follows what the call says it must include;
//! if not, the same information goes in a base structure: what, why, for whom, where, when, how, how much, results,
//! sustainability. The person confirms which of the two applies.

use super::requirements::CallRequirements;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    /// Written by the code from the call (facts and documents to hand in): nothing to draft or confirm.
    Data,
    /// Written by the AI from the conversation, edited and confirmed by the person.
    Text,
    /// The budget table: the person captures it, the code adds it up.
    Budget,
    /// The schedule: the person captures it, the code validates it.
    Schedule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The fixed base structure.
    Base,
    /// A requirement of the call for the proposal.
    Call,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionSpec {
    pub key: String,
    pub title: String,
    /// What the section must say (for the AI and for the person).
    pub guidance: String,
    pub kind: SectionKind,
    /// It has to be confirmed before the project moves on.
    pub required: bool,
    pub source: Source,
}

pub const KEY_CALL_DATA: &str = "call_data";
pub const KEY_DELIVERABLES: &str = "deliverables";
pub const KEY_BUDGET: &str = "budget";
pub const KEY_SCHEDULE: &str = "schedule";

struct Base {
    key: &'static str,
    title: &'static str,
    guidance: &'static str,
    /// Words (without accents) that, found in a requirement of the call, say the call already asks for this.
    covered_by: &'static [&'static str],
}

const BASE: [Base; 9] = [
    Base { key: "what", title: "Qué se va a hacer", guidance: "La idea del proyecto y su objetivo, en pocas líneas y en palabras sencillas.", covered_by: &["objetivo", "descripcion", "que se va a hacer", "resumen"] },
    Base { key: "why", title: "Por qué hace falta", guidance: "El problema, su causa de fondo y lo que pasa hoy por no resolverlo.", covered_by: &["justificacion", "problema", "diagnostico", "antecedentes", "por que"] },
    Base { key: "who", title: "A quién beneficia", guidance: "Quiénes son las personas beneficiadas y cuántas son (cifras, nunca nombres).", covered_by: &["beneficiari", "poblacion", "a quien", "personas atendidas"] },
    Base { key: "where", title: "Dónde se hará", guidance: "El lugar donde se hará: la institución, sus instalaciones y la zona.", covered_by: &["ubicacion", "donde", "cobertura", "zona"] },
    Base { key: "when", title: "Cuándo se hará", guidance: "Los periodos y las etapas del proyecto, de acuerdo con el cronograma.", covered_by: &["cronograma", "calendario", "cuando", "periodo", "plazo"] },
    Base { key: "how", title: "Cómo se hará", guidance: "Las actividades, quién las hace y la alternativa que se eligió para resolver la causa de fondo.", covered_by: &["metodologia", "actividades", "como se", "estrategia", "plan de trabajo"] },
    Base { key: "how_much", title: "Cuánto cuesta", guidance: "El costo total, lo que se pide y lo que aporta la institución, de acuerdo con el presupuesto.", covered_by: &["presupuesto", "costo", "cuanto", "financiamiento"] },
    Base { key: "results", title: "Resultados e indicadores", guidance: "Los resultados que se esperan y los indicadores para saber si funcionó, alineados con los de la convocatoria.", covered_by: &["resultado", "indicador", "impacto", "evaluacion", "metas"] },
    Base { key: "sustainability", title: "Cómo se mantendrá", guidance: "Quién mantendrá lo logrado después del apoyo y cuánto cuesta mantenerlo.", covered_by: &["sostenibilidad", "continuidad", "mantenimiento", "despues del apoyo"] },
];

fn plain(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

/// The first sentence or the first words of a requirement, as the title of its section.
pub fn short_title(text: &str) -> String {
    let t = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let first = t.split(['.', ':', ';']).next().unwrap_or(&t).trim();
    let first = if first.is_empty() { t.as_str() } else { first };
    if first.chars().count() <= 70 {
        return first.trim_end_matches([',', ' ']).to_string();
    }
    let cut: String = first.chars().take(70).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.clone(), |(head, _)| head.to_string());
    format!("{}…", cut.trim_end_matches([',', ' ']))
}

/// A key that stays the same while the requirement says the same thing.
fn call_key(text: &str) -> String {
    let normalized = plain(text).split_whitespace().collect::<Vec<_>>().join(" ");
    format!("req_{}", &hex::encode(Sha256::digest(normalized.as_bytes()))[..8])
}

/// The sections of the guide, in order. `asks_for_proposal` is what the person confirmed.
pub fn plan_sections(req: &CallRequirements, asks_for_proposal: bool) -> Vec<SectionSpec> {
    let mut out = vec![
        SectionSpec { key: KEY_CALL_DATA.into(), title: "Datos de la convocatoria".into(), guidance: "Lo indispensable de la convocatoria.".into(), kind: SectionKind::Data, required: false, source: Source::Base },
        SectionSpec { key: KEY_DELIVERABLES.into(), title: "Qué hay que entregar".into(), guidance: "Los documentos y formatos que pide la convocatoria.".into(), kind: SectionKind::Data, required: false, source: Source::Base },
    ];
    let base = |b: &Base, required: bool| SectionSpec { key: b.key.into(), title: b.title.into(), guidance: b.guidance.into(), kind: SectionKind::Text, required, source: Source::Base };
    if asks_for_proposal && !req.project_requirements.is_empty() {
        // the proposal follows what the call says it must include
        let mut seen = Vec::new();
        for line in &req.project_requirements {
            let key = call_key(&line.text);
            if seen.contains(&key) {
                continue;
            }
            seen.push(key.clone());
            out.push(SectionSpec { key, title: short_title(&line.text), guidance: line.text.clone(), kind: SectionKind::Text, required: true, source: Source::Call });
        }
        // what the call does not ask for stays available, but nobody has to write it
        let asked = plain(&req.project_requirements.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join(" "));
        for b in &BASE {
            if !b.covered_by.iter().any(|w| asked.contains(w)) {
                out.push(base(b, false));
            }
        }
    } else {
        out.extend(BASE.iter().map(|b| base(b, true)));
    }
    out.push(SectionSpec { key: KEY_BUDGET.into(), title: "Presupuesto".into(), guidance: "Las partidas, lo que se pide y lo que aporta la institución.".into(), kind: SectionKind::Budget, required: true, source: Source::Base });
    out.push(SectionSpec { key: KEY_SCHEDULE.into(), title: "Cronograma".into(), guidance: "Las actividades y los meses en que se hacen.".into(), kind: SectionKind::Schedule, required: true, source: Source::Base });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::projects::domain::requirements::Line;

    fn line(t: &str) -> Line {
        Line { text: t.into(), applies_to: None, page: None, file: None }
    }

    fn keys(s: &[SectionSpec]) -> Vec<&str> {
        s.iter().map(|x| x.key.as_str()).collect()
    }

    #[test]
    fn without_a_proposal_of_its_own_the_same_information_goes_in_the_base_structure() {
        let s = plan_sections(&CallRequirements::default(), false);
        assert_eq!(keys(&s), vec!["call_data", "deliverables", "what", "why", "who", "where", "when", "how", "how_much", "results", "sustainability", "budget", "schedule"]);
        assert!(s.iter().filter(|x| x.kind == SectionKind::Text).all(|x| x.required && x.source == Source::Base));
        assert!(s.iter().filter(|x| x.kind == SectionKind::Data).all(|x| !x.required), "the code writes those: nothing to confirm");
        // a call that names a proposal but gives no structure also gets the base one
        let named_only = CallRequirements { required_docs: vec![line("Propuesta del proyecto")], ..Default::default() };
        assert_eq!(keys(&plan_sections(&named_only, true)), keys(&s));
        // and when the person says the call asks for none, what the call listed is not used
        let listed = CallRequirements { project_requirements: vec![line("Justificación del problema")], ..Default::default() };
        assert_eq!(keys(&plan_sections(&listed, false)), keys(&s));
    }

    #[test]
    fn with_a_proposal_of_its_own_the_sections_follow_what_the_call_asks_and_the_rest_is_optional() {
        let req = CallRequirements {
            project_requirements: vec![
                line("Justificación del problema que se atiende. Debe incluir cifras."),
                line("Cronograma de actividades por mes"),
                line("Justificación del problema que se atiende. Debe incluir cifras."), // repeated: one section
            ],
            ..Default::default()
        };
        let s = plan_sections(&req, true);
        let calls: Vec<_> = s.iter().filter(|x| x.source == Source::Call).collect();
        assert_eq!(calls.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(), vec!["Justificación del problema que se atiende", "Cronograma de actividades por mes"]);
        assert!(calls.iter().all(|x| x.required && x.kind == SectionKind::Text));
        assert_eq!(calls[0].guidance, "Justificación del problema que se atiende. Debe incluir cifras.");
        let base_text: Vec<_> = s.iter().filter(|x| x.source == Source::Base && x.kind == SectionKind::Text).map(|x| x.key.as_str()).collect();
        // «justificación» covers why, «cronograma» covers when: they are not repeated; the others are optional
        assert!(!base_text.contains(&"why") && !base_text.contains(&"when"), "{base_text:?}");
        assert!(base_text.contains(&"who") && base_text.contains(&"sustainability"));
        assert!(s.iter().filter(|x| x.source == Source::Base && x.kind == SectionKind::Text).all(|x| !x.required));
        // the budget and the schedule are always there and always required
        assert!(s.iter().any(|x| x.key == "budget" && x.required) && s.iter().any(|x| x.key == "schedule" && x.required));
    }

    #[test]
    fn the_key_of_a_requirement_does_not_change_while_it_says_the_same() {
        let a = call_key("Justificación del problema");
        assert_eq!(a, call_key("  justificacion   DEL problema "));
        assert_ne!(a, call_key("Cronograma"));
        assert!(a.starts_with("req_") && a.len() == 12);
    }

    #[test]
    fn a_long_requirement_becomes_a_short_title() {
        assert_eq!(short_title("Describir el contexto. Incluir datos."), "Describir el contexto");
        let long = "Descripción detallada de las actividades, los responsables, los materiales y el lugar donde se realizarán todas";
        let t = short_title(long);
        assert!(t.chars().count() <= 71 && t.ends_with('…'), "{t}");
        assert!(long.starts_with(t.trim_end_matches('…')));
        assert_eq!(short_title("   "), "");
    }
}
