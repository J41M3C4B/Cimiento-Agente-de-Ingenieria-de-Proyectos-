//! The guided conversation of the diagnosis (ADR-017). Pure rules, no I/O.
//!
//! The AI leads the conversation with the person; the CODE guarantees that it ends. The routine is always the
//! same: an opening question (idea, obstacle, future benefit measured with the call), up to five «whys» that
//! look for the root cause, a proposal of that root cause and the person's explicit confirmation. The AI
//! words the next question and proposes; the code decides which step comes, when a proposal is allowed, when
//! it is forced and when the person's own words are taken as they are.

use crate::documents::text::norm;
use serde::Serialize;

/// The most «whys» asked. The root cause usually shows up in the third or fourth.
pub const MAX_WHYS: u8 = 5;
/// The earliest «why» after whose answer the AI may propose the root cause.
pub const MIN_ROOT_LEVEL: u8 = 3;
/// The most times the root cause is proposed; after a second «no» the person's own words are taken.
pub const MAX_ROOT_ROUNDS: u8 = 2;
/// Vague answers in a row after which the next question comes with closed options.
pub const VAGUE_STREAK_FOR_OPTIONS: u8 = 2;
/// Vague answers in a row after which the code stops asking «why» and proposes what it has.
pub const VAGUE_STREAK_TO_PROPOSE: u8 = 3;
const MIN_WORDS: usize = 4;
const SHORT_ANSWER_WORDS: usize = 9;
const MAX_OPTIONS: usize = 4;
const MAX_OPTION_CHARS: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Assistant,
    Person,
}

impl Role {
    pub fn as_db(self) -> &'static str {
        match self {
            Role::Assistant => "assistant",
            Role::Person => "person",
        }
    }

    pub fn from_db(s: &str) -> Option<Role> {
        match s {
            "assistant" => Some(Role::Assistant),
            "person" => Some(Role::Person),
            _ => None,
        }
    }
}

/// What a turn is. A turn of the person has the kind of what it answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Opening,
    Why,
    RootProposal,
    RootReply,
}

impl Kind {
    pub fn as_db(self) -> &'static str {
        match self {
            Kind::Opening => "opening",
            Kind::Why => "why",
            Kind::RootProposal => "root_proposal",
            Kind::RootReply => "root_reply",
        }
    }

    pub fn from_db(s: &str) -> Option<Kind> {
        match s {
            "opening" => Some(Kind::Opening),
            "why" => Some(Kind::Why),
            "root_proposal" => Some(Kind::RootProposal),
            "root_reply" => Some(Kind::RootReply),
            _ => None,
        }
    }
}

/// What the rules need to know of a stored turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnFacts {
    pub role: Role,
    pub kind: Kind,
    /// Which «why» the assistant asks, or the person answers.
    pub level: Option<u8>,
    /// The person's answer says nothing to build on (`is_vague`). Always false for the assistant.
    pub vague: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Nothing asked yet: the opening is owed.
    NeedsOpening,
    /// The assistant asked and the person has to answer.
    AwaitingAnswer,
    /// The person answered and the AI owes the next message (it failed or was interrupted): «Reintentar».
    AwaitingAi,
    /// The root cause was proposed; the person confirms it or says what is different.
    RootProposed,
    /// The root cause is confirmed.
    Closed,
}

pub fn phase(turns: &[TurnFacts], root_confirmed: bool) -> Phase {
    if root_confirmed {
        return Phase::Closed;
    }
    match turns.last() {
        None => Phase::NeedsOpening,
        Some(t) if t.role == Role::Person => Phase::AwaitingAi,
        Some(t) if t.kind == Kind::RootProposal => Phase::RootProposed,
        Some(_) => Phase::AwaitingAnswer,
    }
}

/// An answer with nothing to build on: too short, or a «no sé» with hardly anything else.
pub fn is_vague(text: &str) -> bool {
    let words = text.split_whitespace().count();
    if words < MIN_WORDS {
        return true;
    }
    let plain: String = norm(text)
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            c => c,
        })
        .collect();
    let doesnt_know = ["no se", "ni idea", "no lo se", "no tengo idea", "no sabria", "no sabemos", "quien sabe"];
    words < SHORT_ANSWER_WORDS && doesnt_know.iter().any(|p| plain.contains(p))
}

/// How many of the person's last answers in a row were vague.
pub fn vague_streak(turns: &[TurnFacts]) -> u8 {
    turns.iter().rev().filter(|t| t.role == Role::Person).take_while(|t| t.vague).count().min(u8::MAX as usize) as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tactic {
    /// An open question.
    Open,
    /// Closed options, because the person is stuck.
    Options,
}

pub fn tactic(turns: &[TurnFacts]) -> Tactic {
    if vague_streak(turns) >= VAGUE_STREAK_FOR_OPTIONS {
        Tactic::Options
    } else {
        Tactic::Open
    }
}

/// What the AI is asked for next, decided by the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The opening question.
    Opening,
    /// The person answered the opening: judge the fit with the call and ask the first «why».
    FirstWhy { tactic: Tactic },
    /// The person answered «why» number `answered`: ask the next or propose the root cause.
    Why { answered: u8, tactic: Tactic, may_propose: bool, must_propose: bool },
    /// The person said the proposed root cause is not it: propose again with their correction.
    Reconsider,
    /// The person turned the proposal down a second time: their own words are the root cause. No AI.
    TakeTheirWords,
}

/// The step that follows the turns stored so far, or `None` when nothing is owed (the person has to answer, or
/// the conversation is over).
pub fn step_after(turns: &[TurnFacts]) -> Option<Step> {
    let Some(last) = turns.last() else { return Some(Step::Opening) };
    if last.role != Role::Person {
        return None;
    }
    let streak = vague_streak(turns);
    match last.kind {
        Kind::Opening => Some(Step::FirstWhy { tactic: tactic(turns) }),
        Kind::Why => {
            let answered = last.level.unwrap_or(1);
            Some(Step::Why {
                answered,
                tactic: tactic(turns),
                may_propose: answered >= MIN_ROOT_LEVEL || streak >= VAGUE_STREAK_TO_PROPOSE,
                must_propose: answered >= MAX_WHYS || streak >= VAGUE_STREAK_TO_PROPOSE,
            })
        }
        Kind::RootReply => {
            let rounds = turns.iter().filter(|t| t.role == Role::Assistant && t.kind == Kind::RootProposal).count();
            Some(if rounds >= MAX_ROOT_ROUNDS as usize { Step::TakeTheirWords } else { Step::Reconsider })
        }
        Kind::RootProposal => None,
    }
}

/// A cause the person gave, as the AI wrote it with the literal quote that backs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cause {
    pub text: String,
    /// What the AI quoted of the person's answer to back it.
    pub quote: String,
    /// The quote was found in what the person wrote (checked by `quote_found`).
    pub verified: bool,
}

/// The quote has to be in what the person wrote (ignoring case, accents of the model and punctuation), and it has
/// to be a phrase, not a word.
pub fn quote_found(person_text: &str, quote: &str) -> bool {
    let q = norm(quote);
    q.split_whitespace().count() >= 2 && norm(person_text).contains(&q)
}

/// What the AI answered for the turn, already checked against the person's text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reply {
    pub message: String,
    pub cause: Option<Cause>,
    pub root_hypothesis: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Ask the person the «why» number `level`.
    Ask { level: u8, message: String, options: Vec<String> },
    /// Propose this as the root cause; `forced` when the code made the proposal instead of the AI.
    Propose { hypothesis: String, forced: bool },
}

fn clean(s: &str) -> Option<String> {
    let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
    (!t.is_empty()).then_some(t)
}

/// The closed options worth showing: not empty, short, no more than four, no repeats.
pub fn clean_options(options: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for o in options {
        if let Some(t) = clean(o).filter(|t| t.chars().count() <= MAX_OPTION_CHARS) {
            if !out.iter().any(|x| x.to_lowercase() == t.to_lowercase()) {
                out.push(t);
            }
        }
        if out.len() == MAX_OPTIONS {
            break;
        }
    }
    out
}

/// What to do with the AI's answer. `last_cause` is the latest verified cause of the conversation and
/// `person_text` what the person just wrote: the last resort when a proposal is forced.
pub fn decide(step: Step, reply: &Reply, last_cause: Option<&str>, person_text: &str) -> Outcome {
    let hypothesis = reply.root_hypothesis.as_deref().and_then(clean);
    let verified_cause = reply.cause.as_ref().filter(|c| c.verified).map(|c| c.text.clone());
    let fallback = || {
        hypothesis
            .clone()
            .or_else(|| verified_cause.clone())
            .or_else(|| last_cause.and_then(clean))
            .or_else(|| reply.cause.as_ref().and_then(|c| clean(&c.text)))
            .or_else(|| clean(person_text).map(|t| t.chars().take(200).collect()))
            .unwrap_or_default()
    };
    match step {
        Step::Opening => Outcome::Ask { level: 0, message: reply.message.clone(), options: vec![] },
        Step::FirstWhy { tactic } => Outcome::Ask {
            level: 1,
            message: reply.message.clone(),
            options: if tactic == Tactic::Options { clean_options(&reply.options) } else { vec![] },
        },
        Step::Why { answered, tactic, may_propose, must_propose } => {
            // a proposal needs a hypothesis and a cause the person really said
            if may_propose && hypothesis.is_some() && verified_cause.is_some() {
                return Outcome::Propose { hypothesis: hypothesis.unwrap(), forced: false };
            }
            if must_propose {
                return Outcome::Propose { hypothesis: fallback(), forced: true };
            }
            Outcome::Ask {
                level: (answered + 1).min(MAX_WHYS),
                message: reply.message.clone(),
                options: if tactic == Tactic::Options { clean_options(&reply.options) } else { vec![] },
            }
        }
        Step::Reconsider => Outcome::Propose { hypothesis: fallback(), forced: hypothesis.is_none() },
        Step::TakeTheirWords => Outcome::Propose { hypothesis: clean(person_text).unwrap_or_default(), forced: true },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(role: Role, kind: Kind, level: Option<u8>, vague: bool) -> TurnFacts {
        TurnFacts { role, kind, level, vague }
    }
    fn ai(kind: Kind, level: Option<u8>) -> TurnFacts {
        t(Role::Assistant, kind, level, false)
    }
    fn me(kind: Kind, level: Option<u8>) -> TurnFacts {
        t(Role::Person, kind, level, false)
    }
    fn vague(kind: Kind, level: Option<u8>) -> TurnFacts {
        t(Role::Person, kind, level, true)
    }

    fn reply(hypothesis: Option<&str>, cause: Option<(&str, bool)>) -> Reply {
        Reply {
            message: "¿Y por qué pasa eso?".into(),
            cause: cause.map(|(t, v)| Cause { text: t.into(), quote: String::new(), verified: v }),
            root_hypothesis: hypothesis.map(String::from),
            options: vec![],
        }
    }

    #[test]
    fn the_phase_follows_who_spoke_last() {
        assert_eq!(phase(&[], false), Phase::NeedsOpening);
        assert_eq!(phase(&[ai(Kind::Opening, None)], false), Phase::AwaitingAnswer);
        assert_eq!(phase(&[ai(Kind::Opening, None), me(Kind::Opening, None)], false), Phase::AwaitingAi);
        assert_eq!(phase(&[ai(Kind::RootProposal, None)], false), Phase::RootProposed);
        assert_eq!(phase(&[ai(Kind::Why, Some(1))], false), Phase::AwaitingAnswer);
        // a confirmed root cause ends it whatever was said last
        assert_eq!(phase(&[ai(Kind::RootProposal, None)], true), Phase::Closed);
    }

    #[test]
    fn vague_answers_are_short_or_a_bare_i_dont_know() {
        assert!(is_vague("no sé"));
        assert!(is_vague("pues sí pasa"));
        assert!(is_vague("La verdad no sé por qué pasa esto"));
        assert!(is_vague("Ni idea, nunca lo hemos visto"));
        assert!(!is_vague("Porque el tubo del agua está viejo y nadie lo revisa"));
        // a long answer that happens to say «no sé» somewhere is not vague
        assert!(!is_vague("No sé el monto exacto pero el techo gotea desde hace dos años en el dormitorio principal"));
    }

    #[test]
    fn two_vague_answers_in_a_row_bring_closed_options() {
        let base = vec![ai(Kind::Opening, None), me(Kind::Opening, None), ai(Kind::Why, Some(1))];
        assert_eq!(tactic(&base), Tactic::Open);
        let one = [base.clone(), vec![vague(Kind::Why, Some(1))]].concat();
        assert_eq!((vague_streak(&one), tactic(&one)), (1, Tactic::Open));
        let two = [one.clone(), vec![ai(Kind::Why, Some(2)), vague(Kind::Why, Some(2))]].concat();
        assert_eq!((vague_streak(&two), tactic(&two)), (2, Tactic::Options));
        // a good answer breaks the streak
        let broken = [two, vec![ai(Kind::Why, Some(3)), me(Kind::Why, Some(3))]].concat();
        assert_eq!(vague_streak(&broken), 0);
    }

    #[test]
    fn nothing_is_owed_while_the_person_has_to_answer() {
        assert_eq!(step_after(&[]), Some(Step::Opening));
        assert_eq!(step_after(&[ai(Kind::Opening, None)]), None);
        assert_eq!(step_after(&[ai(Kind::RootProposal, None)]), None);
    }

    #[test]
    fn the_steps_after_each_answer() {
        let t0 = [ai(Kind::Opening, None), me(Kind::Opening, None)];
        assert_eq!(step_after(&t0), Some(Step::FirstWhy { tactic: Tactic::Open }));

        let why = |n: u8| [ai(Kind::Why, Some(n)), me(Kind::Why, Some(n))];
        let after2 = [&t0[..], &why(1)[..], &why(2)[..]].concat();
        assert_eq!(step_after(&after2), Some(Step::Why { answered: 2, tactic: Tactic::Open, may_propose: false, must_propose: false }));
        let after3 = [&after2[..], &why(3)[..]].concat();
        assert_eq!(step_after(&after3), Some(Step::Why { answered: 3, tactic: Tactic::Open, may_propose: true, must_propose: false }));
        let after5 = [&after3[..], &why(4)[..], &why(5)[..]].concat();
        assert_eq!(step_after(&after5), Some(Step::Why { answered: 5, tactic: Tactic::Open, may_propose: true, must_propose: true }));
    }

    #[test]
    fn three_vague_answers_make_the_code_stop_asking_why() {
        let turns = vec![
            ai(Kind::Opening, None), me(Kind::Opening, None),
            ai(Kind::Why, Some(1)), vague(Kind::Why, Some(1)),
            ai(Kind::Why, Some(2)), vague(Kind::Why, Some(2)),
            ai(Kind::Why, Some(3)), vague(Kind::Why, Some(3)),
        ];
        assert_eq!(step_after(&turns), Some(Step::Why { answered: 3, tactic: Tactic::Options, may_propose: true, must_propose: true }));
    }

    #[test]
    fn a_second_no_to_the_root_cause_takes_the_persons_own_words() {
        let first = [ai(Kind::RootProposal, None), me(Kind::RootReply, None)];
        assert_eq!(step_after(&first), Some(Step::Reconsider));
        let second = [&first[..], &[ai(Kind::RootProposal, None), me(Kind::RootReply, None)][..]].concat();
        assert_eq!(step_after(&second), Some(Step::TakeTheirWords));
    }

    #[test]
    fn the_quote_has_to_be_a_phrase_that_the_person_wrote() {
        let said = "Porque nadie revisa el tubo del agua, está muy viejo.";
        assert!(quote_found(said, "nadie revisa el tubo del agua"));
        assert!(quote_found(said, "Nadie REVISA el tubo"));
        assert!(!quote_found(said, "el techo se cae"));
        assert!(!quote_found(said, "tubo"), "one word is not a quote");
    }

    #[test]
    fn a_proposal_needs_the_third_why_a_hypothesis_and_a_cause_the_person_said() {
        let step = |answered, may, must| Step::Why { answered, tactic: Tactic::Open, may_propose: may, must_propose: must };
        let good = reply(Some("No hay quien dé mantenimiento"), Some(("nadie da mantenimiento", true)));
        // too early: the hypothesis is ignored and the next «why» is asked
        assert!(matches!(decide(step(2, false, false), &good, None, "x"), Outcome::Ask { level: 3, .. }));
        // from the third on
        assert_eq!(decide(step(3, true, false), &good, None, "x"), Outcome::Propose { hypothesis: "No hay quien dé mantenimiento".into(), forced: false });
        // a cause the person did not really say does not count: keep asking
        let invented = reply(Some("No hay quien dé mantenimiento"), Some(("nadie da mantenimiento", false)));
        assert!(matches!(decide(step(3, true, false), &invented, None, "x"), Outcome::Ask { level: 4, .. }));
        // no hypothesis: keep asking
        let none = reply(None, Some(("nadie da mantenimiento", true)));
        assert!(matches!(decide(step(4, true, false), &none, None, "x"), Outcome::Ask { level: 5, .. }));
    }

    #[test]
    fn at_the_fifth_why_the_code_makes_the_proposal_with_the_best_it_has() {
        let forced = |r: &Reply, last: Option<&str>| decide(Step::Why { answered: 5, tactic: Tactic::Open, may_propose: true, must_propose: true }, r, last, "Pues porque sí, así es la casa");
        assert_eq!(forced(&reply(Some("H"), Some(("c", false))), None), Outcome::Propose { hypothesis: "H".into(), forced: true });
        assert_eq!(forced(&reply(None, Some(("la causa", true))), None), Outcome::Propose { hypothesis: "la causa".into(), forced: true });
        assert_eq!(forced(&reply(None, None), Some("lo último verificado")), Outcome::Propose { hypothesis: "lo último verificado".into(), forced: true });
        assert_eq!(forced(&reply(None, None), None), Outcome::Propose { hypothesis: "Pues porque sí, así es la casa".into(), forced: true });
    }

    #[test]
    fn closed_options_come_only_with_that_tactic_clean_and_few() {
        let mut r = reply(None, None);
        r.options = vec!["  Falta dinero ".into(), "falta dinero".into(), "".into(), "Falta personal".into(), "No sé".into(), "Otra cosa".into(), "Una más".into()];
        let open = Step::FirstWhy { tactic: Tactic::Open };
        let closed = Step::FirstWhy { tactic: Tactic::Options };
        assert!(matches!(decide(open, &r, None, "x"), Outcome::Ask { options, .. } if options.is_empty()));
        match decide(closed, &r, None, "x") {
            Outcome::Ask { options, .. } => assert_eq!(options, vec!["Falta dinero", "Falta personal", "No sé", "Otra cosa"]),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn turning_the_proposal_down_proposes_again_and_the_second_time_takes_their_words() {
        let r = reply(Some("Falta un plan de mantenimiento"), None);
        assert_eq!(decide(Step::Reconsider, &r, None, "x"), Outcome::Propose { hypothesis: "Falta un plan de mantenimiento".into(), forced: false });
        assert_eq!(
            decide(Step::TakeTheirWords, &reply(None, None), None, "  Es que el patronato no aprueba   gastos "),
            Outcome::Propose { hypothesis: "Es que el patronato no aprueba gastos".into(), forced: true }
        );
    }

    #[test]
    fn the_conversation_always_ends_within_the_bound() {
        // worst case: every answer is a good one and the AI never proposes: the fifth «why» forces it
        let mut turns = vec![ai(Kind::Opening, None), me(Kind::Opening, None)];
        let mut asked = 0;
        loop {
            match step_after(&turns) {
                Some(Step::FirstWhy { .. }) => {
                    turns.push(ai(Kind::Why, Some(1)));
                    asked += 1;
                    turns.push(me(Kind::Why, Some(1)));
                }
                Some(Step::Why { answered, must_propose, .. }) => {
                    if must_propose {
                        break;
                    }
                    turns.push(ai(Kind::Why, Some(answered + 1)));
                    asked += 1;
                    turns.push(me(Kind::Why, Some(answered + 1)));
                }
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(asked, MAX_WHYS);
    }
}
