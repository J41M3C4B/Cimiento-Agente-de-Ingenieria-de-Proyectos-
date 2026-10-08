//! Amounts as people write them, and what they are worth in a year.

use serde::{Deserialize, Serialize};

/// The largest amount a person can write (one hundred thousand million pesos): more is a typing mistake, and it
/// keeps every sum far from overflowing.
pub const MAX_MXN: i64 = 100_000_000_000;

/// Whether an amount is written per month or per year. The code turns it into a year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    Monthly,
    #[default]
    Annual,
}

impl Period {
    pub fn as_db(self) -> &'static str {
        match self {
            Period::Monthly => "monthly",
            Period::Annual => "annual",
        }
    }
    pub fn from_db(s: &str) -> Self {
        if s == "monthly" { Period::Monthly } else { Period::Annual }
    }
}

/// Where an income written by the person comes from. The stay fees of the people served are not one of these: the
/// core adds them up from their module and hands them over (`balance::Derived`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IncomeKind {
    /// What the people served pay, written by hand while their records have no fees.
    FeeEstimate,
    /// People or companies that give the same amount every month or year.
    RecurringDonor,
    /// People or companies that give when they want.
    OccasionalDonation,
    /// Money won with a project (a call, a foundation).
    ProjectGrant,
    #[default]
    Other,
}

impl IncomeKind {
    pub const ALL: [IncomeKind; 5] =
        [IncomeKind::FeeEstimate, IncomeKind::RecurringDonor, IncomeKind::OccasionalDonation, IncomeKind::ProjectGrant, IncomeKind::Other];
    pub fn as_db(self) -> &'static str {
        match self {
            IncomeKind::FeeEstimate => "fee_estimate",
            IncomeKind::RecurringDonor => "recurring_donor",
            IncomeKind::OccasionalDonation => "occasional_donation",
            IncomeKind::ProjectGrant => "project_grant",
            IncomeKind::Other => "other",
        }
    }
    pub fn from_db(s: &str) -> Self {
        IncomeKind::ALL.into_iter().find(|k| k.as_db() == s).unwrap_or_default()
    }
}

/// Reads an amount the way a person writes it: `1800000`, `1,800,000`, `$1 800 000` or `1.800.000`. Separators
/// are accepted only between groups of three digits; anything else (letters, cents, signs) is not an amount.
#[cfg_attr(not(test), allow(dead_code))] // the screen reads amounts with the same rule (`parsePesos`)
pub fn parse_pesos(s: &str) -> Option<i64> {
    let s = s.trim();
    let s = s.strip_prefix('$').unwrap_or(s).trim_start();
    if s.is_empty() {
        return None;
    }
    let groups: Vec<&str> = s.split([',', ' ', '.']).collect();
    let first = groups[0];
    let ok = !first.is_empty()
        && first.bytes().all(|b| b.is_ascii_digit())
        && (groups.len() == 1 || first.len() <= 3)
        && groups[1..].iter().all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit()));
    if !ok {
        return None;
    }
    groups.concat().parse().ok()
}

/// What an amount is worth in a year.
pub fn annual(amount: i64, period: Period) -> i64 {
    match period {
        Period::Monthly => amount * 12,
        Period::Annual => amount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amounts_are_read_as_people_write_them() {
        for (text, n) in [("1800000", 1_800_000), ("1,800,000", 1_800_000), ("$1 800 000", 1_800_000), ("$ 950", 950), ("1.800.000", 1_800_000), (" 0 ", 0)] {
            assert_eq!(parse_pesos(text), Some(n), "{text}");
        }
        for text in ["", "abc", "9 mil", "1,80,000", "1800,000", "-5", "12.50", "1,8000", "$", "1,800,"] {
            assert_eq!(parse_pesos(text), None, "{text}");
        }
    }

    #[test]
    fn a_monthly_amount_is_twelve_in_a_year() {
        assert_eq!(annual(5_000, Period::Monthly), 60_000);
        assert_eq!(annual(5_000, Period::Annual), 5_000);
    }
}
