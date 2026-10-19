//! Schedule of a project: activities placed in months. The code validates the duration against what the call
//! allows (docs/02-flujo-funcional.md); the AI never does this.

/// A reasonable bound: no project lasts more than fifty years.
pub const MAX_MONTH: u32 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityError {
    /// Months start at 1.
    StartBeforeOne,
    /// The end cannot come before the start.
    EndBeforeStart,
    TooLong,
}

pub fn validate(start_month: i64, end_month: i64) -> Result<(), ActivityError> {
    if start_month < 1 {
        return Err(ActivityError::StartBeforeOne);
    }
    if end_month < start_month {
        return Err(ActivityError::EndBeforeStart);
    }
    if end_month > MAX_MONTH as i64 {
        return Err(ActivityError::TooLong);
    }
    Ok(())
}

/// How many months the project lasts: the end of its last activity.
pub fn duration_months(activities: &[(u32, u32)]) -> u32 {
    activities.iter().map(|(_, end)| *end).max().unwrap_or(0)
}

/// The activities that go beyond the months the call allows.
pub fn beyond(activities: &[(u32, u32)], max_months: u32) -> Vec<usize> {
    activities.iter().enumerate().filter(|(_, (_, end))| *end > max_months).map(|(i, _)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_activity_starts_at_month_one_or_later_and_does_not_end_before_it_starts() {
        assert_eq!(validate(1, 1), Ok(()));
        assert_eq!(validate(3, 6), Ok(()));
        assert_eq!(validate(0, 2), Err(ActivityError::StartBeforeOne));
        assert_eq!(validate(-4, 2), Err(ActivityError::StartBeforeOne));
        assert_eq!(validate(5, 4), Err(ActivityError::EndBeforeStart));
        assert_eq!(validate(1, 601), Err(ActivityError::TooLong));
    }

    #[test]
    fn the_project_lasts_until_its_last_activity_ends() {
        assert_eq!(duration_months(&[]), 0);
        assert_eq!(duration_months(&[(1, 3), (2, 12), (4, 6)]), 12);
    }

    #[test]
    fn activities_beyond_the_allowed_months_are_pointed_out() {
        assert_eq!(beyond(&[(1, 3), (2, 12), (10, 14)], 12), vec![2]);
        assert!(beyond(&[(1, 3)], 12).is_empty());
    }
}
