//! The pay rules of the law, by code (ADR-026, moved here by ADR-027): a day of pay, the aguinaldo and the vacation
//! premium, and how a weekly or biweekly pay becomes a month.

/// Paid vacation days for a year of service (Ley Federal del Trabajo, art. 76, reformed in 2023): 12 the first
/// year, two more each year up to 20 in the fifth, then two more every five years. `years` below 1 counts as 1.
pub fn vacation_days(years: i64) -> i64 {
    let n = years.max(1);
    if n <= 5 {
        12 + 2 * (n - 1)
    } else {
        20 + 2 * ((n - 5 + 4) / 5)
    }
}

/// Christmas bonus by law: at least 15 days of pay.
pub const AGUINALDO_DAYS: i64 = 15;

/// What the benefits of one person with a monthly pay add in a year: the aguinaldo (15 days) and the vacation
/// premium (25 % of the vacation days). A day of pay is the month divided by 30; rounded to whole pesos.
pub fn annual_benefits(monthly_salary: i64, years_of_service: i64) -> i64 {
    let aguinaldo = (monthly_salary * AGUINALDO_DAYS + 15) / 30;
    let premium = (monthly_salary * vacation_days(years_of_service) + 60) / 120;
    aguinaldo + premium
}

/// A pay as a month, from the day of pay: a week is 7 days, a fortnight 15 and a month 30. Rounded to pesos.
pub fn monthly_equivalent(amount: i64, period: &str) -> i64 {
    match period {
        "weekly" => (amount * 30 + 3) / 7,
        "biweekly" => amount * 2,
        _ => amount,
    }
}

/// The weekly hours of the law for a day shift (art. 61): more is overtime.
pub const LEGAL_WEEKLY_HOURS: i64 = 48;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vacation_days_follow_the_2023_law() {
        let days: Vec<i64> = [0, 1, 2, 3, 4, 5, 6, 10, 11, 15, 16, 20, 21, 30].iter().map(|&y| vacation_days(y)).collect();
        assert_eq!(days, vec![12, 12, 14, 16, 18, 20, 22, 22, 24, 24, 26, 26, 28, 30]);
    }

    #[test]
    fn benefits_are_aguinaldo_and_vacation_premium() {
        // 9,000 a month: a day is 300; aguinaldo 15 days = 4,500; first year 12 days, premium 25 % = 900
        assert_eq!(annual_benefits(9_000, 1), 5_400);
        // eighth year: 22 days, premium 1,650
        assert_eq!(annual_benefits(9_000, 8), 6_150);
    }

    #[test]
    fn weekly_and_biweekly_pay_become_a_month() {
        assert_eq!(monthly_equivalent(2_100, "weekly"), 9_000); // 300 a day
        assert_eq!(monthly_equivalent(4_500, "biweekly"), 9_000);
        assert_eq!(monthly_equivalent(9_000, "monthly"), 9_000);
    }
}
