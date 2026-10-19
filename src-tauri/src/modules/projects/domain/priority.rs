//! Need prioritisation. The AI proposes needs; the SCORE IS COMPUTED HERE.
//! Each criterion is rated 1-5 by the manager (docs/02-flujo-funcional.md).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weights {
    pub beneficiaries: u32,
    pub severity: u32,
    pub mission: u32,
    pub feasibility: u32,
    pub sustainability: u32,
}

impl Default for Weights {
    /// Initial weights from the docs: 25 / 25 / 20 / 15 / 15.
    fn default() -> Self {
        Weights { beneficiaries: 25, severity: 25, mission: 20, feasibility: 15, sustainability: 15 }
    }
}

impl Weights {
    fn sum(&self) -> u32 {
        self.beneficiaries + self.severity + self.mission + self.feasibility + self.sustainability
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scores {
    pub beneficiaries: u8,
    pub severity: u8,
    pub mission: u8,
    pub feasibility: u8,
    pub sustainability: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriorityError {
    /// A rating outside 1..=5.
    ScoreOutOfRange(&'static str),
    /// All weights are zero.
    NoWeights,
}

impl Scores {
    fn check(&self) -> Result<(), PriorityError> {
        for (name, v) in [
            ("beneficiaries", self.beneficiaries),
            ("severity", self.severity),
            ("mission", self.mission),
            ("feasibility", self.feasibility),
            ("sustainability", self.sustainability),
        ] {
            if !(1..=5).contains(&v) {
                return Err(PriorityError::ScoreOutOfRange(name));
            }
        }
        Ok(())
    }
}

/// Weighted score from 20 (all ones) to 100 (all fives), one decimal.
pub fn total_score(scores: &Scores, weights: &Weights) -> Result<f64, PriorityError> {
    scores.check()?;
    let wsum = weights.sum();
    if wsum == 0 {
        return Err(PriorityError::NoWeights);
    }
    let weighted = scores.beneficiaries as u32 * weights.beneficiaries
        + scores.severity as u32 * weights.severity
        + scores.mission as u32 * weights.mission
        + scores.feasibility as u32 * weights.feasibility
        + scores.sustainability as u32 * weights.sustainability;
    // integer maths up to the last step, so results are exact and repeatable
    let tenths = (weighted * 1000 + wsum * 5 / 2) / (wsum * 5); // percent x10, rounded
    Ok(tenths as f64 / 10.0)
}

/// Suggested rating for "people benefited" from their share of the population.
pub fn suggest_beneficiaries_score(affected: u32, population: u32) -> Option<u8> {
    if population == 0 {
        return None;
    }
    let pct = affected as f64 * 100.0 / population as f64;
    Some(match pct {
        p if p <= 10.0 => 1,
        p if p <= 25.0 => 2,
        p if p <= 50.0 => 3,
        p if p <= 75.0 => 4,
        _ => 5,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ranked {
    pub index: usize,
    pub score: f64,
}

/// Orders needs by score (highest first). Ties go to higher severity, then more beneficiaries,
/// then the original order, so the ranking is always the same for the same inputs.
pub fn rank(needs: &[Scores], weights: &Weights) -> Result<Vec<Ranked>, PriorityError> {
    let mut out = Vec::with_capacity(needs.len());
    for (index, s) in needs.iter().enumerate() {
        out.push(Ranked { index, score: total_score(s, weights)? });
    }
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap()
            .then(needs[b.index].severity.cmp(&needs[a.index].severity))
            .then(needs[b.index].beneficiaries.cmp(&needs[a.index].beneficiaries))
            .then(a.index.cmp(&b.index))
    });
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(b: u8, sev: u8, m: u8, f: u8, su: u8) -> Scores {
        Scores { beneficiaries: b, severity: sev, mission: m, feasibility: f, sustainability: su }
    }

    #[test]
    fn extremes_and_default_weights() {
        let w = Weights::default();
        assert_eq!(total_score(&s(5, 5, 5, 5, 5), &w).unwrap(), 100.0);
        assert_eq!(total_score(&s(1, 1, 1, 1, 1), &w).unwrap(), 20.0);
        assert_eq!(total_score(&s(3, 3, 3, 3, 3), &w).unwrap(), 60.0);
    }

    #[test]
    fn weights_change_the_result() {
        // beneficiaries 5, everything else 1: (5*25 + 25 + 20 + 15 + 15) / 500 = 40 %
        let sc = s(5, 1, 1, 1, 1);
        assert_eq!(total_score(&sc, &Weights::default()).unwrap(), 40.0);
        let only_beneficiaries = Weights { beneficiaries: 100, severity: 0, mission: 0, feasibility: 0, sustainability: 0 };
        assert_eq!(total_score(&sc, &only_beneficiaries).unwrap(), 100.0);
    }

    #[test]
    fn rejects_bad_scores_and_weights() {
        let w = Weights::default();
        assert_eq!(total_score(&s(0, 3, 3, 3, 3), &w), Err(PriorityError::ScoreOutOfRange("beneficiaries")));
        assert_eq!(total_score(&s(3, 3, 3, 3, 6), &w), Err(PriorityError::ScoreOutOfRange("sustainability")));
        let zero = Weights { beneficiaries: 0, severity: 0, mission: 0, feasibility: 0, sustainability: 0 };
        assert_eq!(total_score(&s(3, 3, 3, 3, 3), &zero), Err(PriorityError::NoWeights));
    }

    #[test]
    fn rounding_is_to_one_decimal() {
        let w = Weights { beneficiaries: 1, severity: 1, mission: 1, feasibility: 0, sustainability: 0 };
        // (4+4+5)/(3*5) = 86.666..% -> 86.7
        assert_eq!(total_score(&s(4, 4, 5, 1, 1), &w).unwrap(), 86.7);
    }

    #[test]
    fn suggestion_from_share_of_population() {
        assert_eq!(suggest_beneficiaries_score(2, 25), Some(1)); // 8 %
        assert_eq!(suggest_beneficiaries_score(10, 100), Some(1)); // exactly 10 %
        assert_eq!(suggest_beneficiaries_score(5, 25), Some(2)); // 20 %
        assert_eq!(suggest_beneficiaries_score(10, 25), Some(3)); // 40 %
        assert_eq!(suggest_beneficiaries_score(14, 25), Some(4)); // 56 %
        assert_eq!(suggest_beneficiaries_score(25, 25), Some(5)); // 100 %
        assert_eq!(suggest_beneficiaries_score(3, 0), None);
    }

    #[test]
    fn ranking_is_stable_and_tie_broken() {
        let w = Weights::default();
        // scores: 60.0, 100.0, 60.0, 60.0 (index 3: 2*25+4*25+3*20+3*15+3*15 = 300 -> 60.0)
        let needs = [s(3, 3, 3, 3, 3), s(5, 5, 5, 5, 5), s(3, 3, 3, 3, 3), s(2, 4, 3, 3, 3)];
        let order: Vec<usize> = rank(&needs, &w).unwrap().iter().map(|r| r.index).collect();
        // 1 wins; among the 60.0 ties, higher severity (3) goes first, then original order (0 before 2)
        assert_eq!(order, vec![1, 3, 0, 2]);
        // and the same inputs always give the same ranking
        assert_eq!(order, rank(&needs, &w).unwrap().iter().map(|r| r.index).collect::<Vec<_>>());
    }
}
