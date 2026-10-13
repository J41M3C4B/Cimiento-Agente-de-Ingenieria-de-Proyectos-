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

/// Days from `from` to `to` (both `YYYY-MM-DD`); negative when `to` comes first.
pub fn days_between(from: &str, to: &str) -> Option<i64> {
    // the days of a civil date since a fixed day (H. Hinnant, «chrono-compatible low-level date algorithms»)
    fn days(s: &str) -> Option<i64> {
        if !date_is_valid(s) {
            return None;
        }
        let (y, m, d) = (s[..4].parse::<i64>().ok()?, s[5..7].parse::<i64>().ok()?, s[8..].parse::<i64>().ok()?);
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
        Some(era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy)
    }
    Some(days(to)? - days(from)?)
}

/// The date `n` days after `date` (`YYYY-MM-DD`; `n` may be negative): the reverse of `days_between`.
pub fn add_days(date: &str, n: i64) -> Option<String> {
    // the civil date of a count of days since 1970-01-01 (H. Hinnant, `civil_from_days`)
    let z = days_between("1970-01-01", date)? + n + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    let out = format!("{y:04}-{m:02}-{d:02}");
    date_is_valid(&out).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_number_of_days_after_a_date() {
        assert_eq!(add_days("2026-02-06", 30).as_deref(), Some("2026-03-08"));
        assert_eq!(add_days("2028-02-28", 1).as_deref(), Some("2028-02-29"), "a leap year");
        assert_eq!(add_days("2026-12-31", 1).as_deref(), Some("2027-01-01"));
        assert_eq!(add_days("2026-03-01", -1).as_deref(), Some("2026-02-28"));
        for (from, n) in [("2015-03-10", 4000), ("1999-12-31", 1), ("2026-10-13", -400)] {
            assert_eq!(days_between(from, &add_days(from, n).unwrap()), Some(n));
        }
        assert_eq!(add_days("2026-02-30", 1), None);
    }

    #[test]
    fn days_between_two_dates() {
        assert_eq!(days_between("2026-10-10", "2026-11-09"), Some(30));
        assert_eq!(days_between("2026-10-10", "2026-10-09"), Some(-1));
        assert_eq!(days_between("2028-02-28", "2028-03-01"), Some(2), "a leap year");
        assert_eq!(days_between("2026-12-31", "2027-01-01"), Some(1));
        assert_eq!(days_between("2026-02-30", "2026-03-01"), None);
    }


    #[test]
    fn real_dates_and_whole_years() {
        assert!(date_is_valid("2024-02-29") && !date_is_valid("2023-02-29") && !date_is_valid("2024-13-01") && !date_is_valid("24-01-01"));
        assert_eq!(years_between("1956-04-27", "2026-04-26"), Some(69));
        assert_eq!(years_between("1956-04-27", "2026-04-27"), Some(70));
    }
}
