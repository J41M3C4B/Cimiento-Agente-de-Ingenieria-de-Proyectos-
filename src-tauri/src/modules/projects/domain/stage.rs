//! Project stages and the rules to move between them (docs/02-flujo-funcional.md).
//!
//! Pure logic: the caller computes `StageFacts` from the database; this module
//! only decides. You can go back to any earlier stage, never skip ahead.

use serde::{Deserialize, Serialize};

/// A profile confirmed more than this many days ago must be reviewed again.
pub const PROFILE_MAX_AGE_DAYS: i64 = 365;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Stage {
    Profile,
    /// Confirm the call the project was born from (ADR-017).
    CallSelection,
    Diagnosis,
    Prioritization,
    Drafting,
    Review,
    Ready,
}

pub const ALL_STAGES: [Stage; 7] = [
    Stage::Profile,
    Stage::CallSelection,
    Stage::Diagnosis,
    Stage::Prioritization,
    Stage::Drafting,
    Stage::Review,
    Stage::Ready,
];

impl Stage {
    pub fn as_db(self) -> &'static str {
        match self {
            Stage::Profile => "PROFILE",
            Stage::Diagnosis => "DIAGNOSIS",
            Stage::Prioritization => "PRIORITIZATION",
            Stage::CallSelection => "CALL_SELECTION",
            Stage::Drafting => "DRAFTING",
            Stage::Review => "REVIEW",
            Stage::Ready => "READY",
        }
    }

    pub fn from_db(s: &str) -> Option<Stage> {
        ALL_STAGES.iter().copied().find(|st| st.as_db() == s)
    }

    pub fn next(self) -> Option<Stage> {
        ALL_STAGES.get(self.index() + 1).copied()
    }

    fn index(self) -> usize {
        ALL_STAGES.iter().position(|s| *s == self).unwrap()
    }
}

/// What is true about the project right now. Computed by storage code.
#[derive(Debug, Clone, Copy, Default)]
pub struct StageFacts {
    /// Days since the profile was last confirmed (`None` if never).
    pub profile_confirmed_days_ago: Option<i64>,
    /// The person confirmed the root cause the conversation found.
    pub root_cause_confirmed: bool,
    /// The person confirmed the diagnosis summary.
    pub diagnosis_summary_confirmed: bool,
    /// A need is marked as "the project".
    pub need_selected: bool,
    /// The call was read and the person confirmed it is the right one.
    pub call_confirmed: bool,
    /// Every required section has confirmed text.
    pub sections_confirmed: bool,
    /// The final checklist has no errors.
    pub checklist_clean: bool,
}

/// Why a project cannot advance. The UI turns each into a friendly sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Missing {
    ProfileNotConfirmed,
    ProfileTooOld,
    DiagnosisIncomplete,
    SummaryNotConfirmed,
    NoNeedSelected,
    CallNotReady,
    SectionsNotConfirmed,
    ChecklistHasErrors,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StageError {
    #[error("the project is not ready for the next stage: {0:?}")]
    NotReady(Missing),
    #[error("the project is already in the last stage")]
    AlreadyFinal,
    /// Going back only works toward an earlier stage.
    #[error("you can only go back to an earlier stage")]
    NotEarlier,
}

/// Moves to the next stage if its condition holds. Never skips.
pub fn advance(current: Stage, f: &StageFacts) -> Result<Stage, StageError> {
    let next = current.next().ok_or(StageError::AlreadyFinal)?;
    let missing = match current {
        Stage::Profile => match f.profile_confirmed_days_ago {
            None => Some(Missing::ProfileNotConfirmed),
            Some(d) if d > PROFILE_MAX_AGE_DAYS => Some(Missing::ProfileTooOld),
            Some(_) => None,
        },
        Stage::CallSelection => (!f.call_confirmed).then_some(Missing::CallNotReady),
        Stage::Diagnosis => {
            if !f.root_cause_confirmed {
                Some(Missing::DiagnosisIncomplete)
            } else if !f.diagnosis_summary_confirmed {
                Some(Missing::SummaryNotConfirmed)
            } else {
                None
            }
        }
        Stage::Prioritization => (!f.need_selected).then_some(Missing::NoNeedSelected),
        Stage::Drafting => (!f.sections_confirmed).then_some(Missing::SectionsNotConfirmed),
        Stage::Review => (!f.checklist_clean).then_some(Missing::ChecklistHasErrors),
        Stage::Ready => unreachable!("Ready has no next stage"),
    };
    match missing {
        Some(m) => Err(StageError::NotReady(m)),
        None => Ok(next),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackOutcome {
    pub new_stage: Stage,
    /// Stages whose work depended on the one we returned to: mark as "por revisar", never delete.
    pub needs_review: Vec<Stage>,
}

/// Goes back to an earlier stage.
pub fn go_back(current: Stage, target: Stage) -> Result<BackOutcome, StageError> {
    if target >= current {
        return Err(StageError::NotEarlier);
    }
    let needs_review = ALL_STAGES
        .iter()
        .copied()
        .filter(|s| *s > target && *s <= current)
        .collect();
    Ok(BackOutcome { new_stage: target, needs_review })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_facts() -> StageFacts {
        StageFacts {
            profile_confirmed_days_ago: Some(10),
            root_cause_confirmed: true,
            diagnosis_summary_confirmed: true,
            need_selected: true,
            call_confirmed: true,
            sections_confirmed: true,
            checklist_clean: true,
        }
    }

    #[test]
    fn happy_path_goes_through_every_stage_in_order() {
        let f = ready_facts();
        let mut s = Stage::Profile;
        let mut seen = vec![s];
        while let Ok(n) = advance(s, &f) {
            s = n;
            seen.push(s);
        }
        assert_eq!(seen, ALL_STAGES.to_vec());
        assert_eq!(advance(Stage::Ready, &f), Err(StageError::AlreadyFinal));
    }

    #[test]
    fn each_stage_blocks_for_its_own_reason() {
        let none = StageFacts::default();
        assert_eq!(advance(Stage::Profile, &none), Err(StageError::NotReady(Missing::ProfileNotConfirmed)));
        let old = StageFacts { profile_confirmed_days_ago: Some(366), ..none };
        assert_eq!(advance(Stage::Profile, &old), Err(StageError::NotReady(Missing::ProfileTooOld)));
        let edge = StageFacts { profile_confirmed_days_ago: Some(365), ..none };
        assert_eq!(advance(Stage::Profile, &edge), Ok(Stage::CallSelection));

        // the call has to be read and confirmed by the person before the diagnosis starts
        assert_eq!(advance(Stage::CallSelection, &none), Err(StageError::NotReady(Missing::CallNotReady)));
        let confirmed = StageFacts { call_confirmed: true, ..none };
        assert_eq!(advance(Stage::CallSelection, &confirmed), Ok(Stage::Diagnosis));

        let no_root = StageFacts { root_cause_confirmed: false, diagnosis_summary_confirmed: true, ..none };
        assert_eq!(advance(Stage::Diagnosis, &no_root), Err(StageError::NotReady(Missing::DiagnosisIncomplete)));
        let no_summary = StageFacts { root_cause_confirmed: true, ..none };
        assert_eq!(advance(Stage::Diagnosis, &no_summary), Err(StageError::NotReady(Missing::SummaryNotConfirmed)));
        assert_eq!(advance(Stage::Prioritization, &none), Err(StageError::NotReady(Missing::NoNeedSelected)));
        assert_eq!(advance(Stage::Drafting, &none), Err(StageError::NotReady(Missing::SectionsNotConfirmed)));
        assert_eq!(advance(Stage::Review, &none), Err(StageError::NotReady(Missing::ChecklistHasErrors)));
    }

    #[test]
    fn one_step_at_a_time_even_if_everything_is_ready() {
        assert_eq!(advance(Stage::Profile, &ready_facts()), Ok(Stage::CallSelection));
        assert_eq!(advance(Stage::CallSelection, &ready_facts()), Ok(Stage::Diagnosis));
        assert_eq!(advance(Stage::Diagnosis, &ready_facts()), Ok(Stage::Prioritization));
    }

    #[test]
    fn going_back_marks_dependents_for_review() {
        let out = go_back(Stage::Drafting, Stage::Diagnosis).unwrap();
        assert_eq!(out.new_stage, Stage::Diagnosis);
        assert_eq!(out.needs_review, vec![Stage::Prioritization, Stage::Drafting]);
        // one step back marks only the stage we left
        let out = go_back(Stage::Review, Stage::Drafting).unwrap();
        assert_eq!(out.needs_review, vec![Stage::Review]);
    }

    #[test]
    fn cannot_go_back_to_same_or_later_stage() {
        assert_eq!(go_back(Stage::Diagnosis, Stage::Diagnosis), Err(StageError::NotEarlier));
        assert_eq!(go_back(Stage::Diagnosis, Stage::Drafting), Err(StageError::NotEarlier));
    }

    #[test]
    fn every_stage_can_go_back_to_every_earlier_one() {
        for (i, cur) in ALL_STAGES.iter().enumerate() {
            for tgt in &ALL_STAGES[..i] {
                assert!(go_back(*cur, *tgt).is_ok(), "{cur:?} -> {tgt:?}");
            }
        }
    }

    #[test]
    fn db_names_round_trip() {
        for s in ALL_STAGES {
            assert_eq!(Stage::from_db(s.as_db()), Some(s));
        }
        assert_eq!(Stage::from_db("NOPE"), None);
    }
}
