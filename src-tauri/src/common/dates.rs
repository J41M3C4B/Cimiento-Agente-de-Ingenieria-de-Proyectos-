//! Dates as the app keeps them (`YYYY-MM-DD`), shared by the modules (ADR-029).

/// `YYYY-MM-DD` that exists on the calendar.
pub fn date_is_valid(s: &str) -> bool {
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 3 || p[0].len() != 4 || p[1].len() != 2 || p[2].len() != 2 {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (p[0].parse::<i64>(), p[1].parse::<u32>(), p[2].parse::<u32>()) else { return false };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1900..=2100).contains(&y) && (1..=days).contains(&d)
}

/// Whole years from `from` to `to` (both `YYYY-MM-DD`).
pub fn years_between(from: &str, to: &str) -> Option<i64> {
    if !date_is_valid(from) || !date_is_valid(to) {
        return None;
    }
    let y = |s: &str| s[..4].parse::<i64>().ok();
    let md = |s: &str| (&s[5..]).to_string();
    let mut years = y(to)? - y(from)?;
    if md(to) < md(from) {
        years -= 1;
    }
    Some(years)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_dates_and_whole_years() {
        assert!(date_is_valid("2024-02-29") && !date_is_valid("2023-02-29") && !date_is_valid("2024-13-01") && !date_is_valid("24-01-01"));
        assert_eq!(years_between("1956-04-27", "2026-04-26"), Some(69));
        assert_eq!(years_between("1956-04-27", "2026-04-27"), Some(70));
    }
}
