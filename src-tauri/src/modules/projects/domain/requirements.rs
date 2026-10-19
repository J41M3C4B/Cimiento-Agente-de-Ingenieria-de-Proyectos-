//! What a call asks of a project, as typed data (ADR-018). Pure types and rules; `documents::canonical::requirements`
//! fills them from the confirmed reading of the call. Whatever the call does not say stays `None` or empty and the
//! person completes it by hand: nothing is guessed.

use serde::Serialize;

/// A value of the call with the place it was read from, so the person can check it against the original.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sourced<T> {
    pub value: T,
    pub page: Option<u64>,
    pub file: Option<String>,
}

/// A line of a list of the call (a document, a format, a criterion, a requirement of the proposal).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Line {
    pub text: String,
    /// The condition under which it applies, when the call restricts it («organizaciones nuevas»).
    pub applies_to: Option<String>,
    pub page: Option<u64>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CallRequirements {
    /// Amounts in pesos. An amount in another currency is not taken: `foreign_currency` says so.
    pub max_amount_mxn: Option<Sourced<f64>>,
    pub min_amount_mxn: Option<Sourced<f64>>,
    pub foreign_currency: bool,
    /// Counterpart the applicant must put, as a percentage of the project.
    pub cofunding_percent: Option<Sourced<f64>>,
    /// The most of the support that may go to administrative expenses, as a percentage.
    pub admin_cap_percent: Option<Sourced<f64>>,
    pub max_duration_months: Option<Sourced<u32>>,
    /// The latest closing date the call gives (`AAAA-MM-DD`).
    pub closing_date: Option<Sourced<String>>,
    pub required_docs: Vec<Line>,
    pub conditional_docs: Vec<Line>,
    pub optional_docs: Vec<Line>,
    pub formats: Vec<Line>,
    pub evaluation_criteria: Vec<Line>,
    /// What the proposal must include or follow (components, approach, structure, evidence).
    pub project_requirements: Vec<Line>,
    pub fundable: Vec<Line>,
    pub not_fundable: Vec<Line>,
    pub indicators: Vec<Line>,
    pub how_to_deliver: Vec<Line>,
    pub contact: Vec<Line>,
}

/// Words that name a project proposal in the list of documents a call demands.
const PROPOSAL_WORDS: [&str; 6] = ["propuesta", "proyecto", "descripcion del proyecto", "memoria", "documento tecnico", "protocolo"];

fn plain(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            c => c,
        })
        .collect()
}

impl CallRequirements {
    /// The code's proposal of whether the call asks for a project proposal as a document of its own: the call
    /// lists one among the documents it demands, or says what the proposal must include. The person confirms it.
    pub fn asks_for_proposal(&self) -> bool {
        !self.project_requirements.is_empty()
            || self.required_docs.iter().chain(&self.conditional_docs).any(|d| {
                let t = plain(&d.text);
                PROPOSAL_WORDS.iter().any(|w| t.contains(w))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(t: &str) -> Line {
        Line { text: t.into(), applies_to: None, page: None, file: None }
    }

    #[test]
    fn a_call_asks_for_a_proposal_when_it_lists_one_or_says_what_it_must_include() {
        assert!(!CallRequirements::default().asks_for_proposal());
        let docs = |t: &str| CallRequirements { required_docs: vec![line("Acta constitutiva"), line(t)], ..Default::default() };
        assert!(docs("Propuesta del proyecto").asks_for_proposal());
        assert!(docs("Descripción del proyecto firmada").asks_for_proposal());
        assert!(docs("Documento técnico").asks_for_proposal());
        assert!(!docs("Comprobante de domicilio").asks_for_proposal());
        let reqs = CallRequirements { project_requirements: vec![line("Debe incluir justificación y cronograma")], ..Default::default() };
        assert!(reqs.asks_for_proposal());
        // a document asked only in some cases still counts: the person confirms it anyway
        assert!(CallRequirements { conditional_docs: vec![line("Proyecto ejecutivo")], ..Default::default() }.asks_for_proposal());
        // a document that is merely optional does not
        assert!(!CallRequirements { optional_docs: vec![line("Proyecto ejecutivo")], ..Default::default() }.asks_for_proposal());
    }
}
