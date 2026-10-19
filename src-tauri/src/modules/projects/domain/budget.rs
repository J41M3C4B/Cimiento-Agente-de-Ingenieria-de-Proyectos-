//! Budget of a project. The AI never calculates: every total, tax and percentage is computed here, from the
//! lines the person wrote (docs/05-modelo-datos.md, `budget_item`). Amounts are pesos; rounding is to cents per
//! line, so the total is always the sum of what the person sees.

use serde::{Deserialize, Serialize};

/// Value added tax in Mexico.
pub const VAT_RATE: f64 = 0.16;

/// Who pays a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Funder {
    /// The call (what is requested).
    Requested,
    /// The institution itself (its counterpart).
    Institution,
    /// Someone else (another donor, a partner).
    Other,
}

impl Funder {
    pub fn as_db(self) -> &'static str {
        match self {
            Funder::Requested => "requested",
            Funder::Institution => "institution",
            Funder::Other => "other",
        }
    }

    pub fn from_db(s: &str) -> Option<Funder> {
        match s {
            "requested" => Some(Funder::Requested),
            "institution" => Some(Funder::Institution),
            "other" => Some(Funder::Other),
            _ => None,
        }
    }
}

/// What the rules need of a budget line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    pub quantity: f64,
    pub unit_price: f64,
    /// The price already carries the tax.
    pub vat_included: bool,
    pub funded_by: Funder,
    /// An administrative or indirect expense (the call may cap them).
    pub administrative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LineTotal {
    pub subtotal: f64,
    pub vat: f64,
    pub total: f64,
}

pub fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Why a line cannot be saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemError {
    /// Quantity must be above zero.
    Quantity,
    /// Price cannot be negative.
    Price,
    /// A number that is not a number (NaN, infinity) or absurdly large.
    NotANumber,
}

const MAX_NUMBER: f64 = 1.0e12;

pub fn validate(quantity: f64, unit_price: f64) -> Result<(), ItemError> {
    if !quantity.is_finite() || !unit_price.is_finite() || quantity > MAX_NUMBER || unit_price > MAX_NUMBER {
        return Err(ItemError::NotANumber);
    }
    if quantity <= 0.0 {
        return Err(ItemError::Quantity);
    }
    if unit_price < 0.0 {
        return Err(ItemError::Price);
    }
    Ok(())
}

pub fn line_total(i: &Item) -> LineTotal {
    let gross = round2(i.quantity * i.unit_price);
    if i.vat_included {
        // the price already carries the tax: the tax is the part of it that is tax
        let subtotal = round2(gross / (1.0 + VAT_RATE));
        LineTotal { subtotal, vat: round2(gross - subtotal), total: gross }
    } else {
        let vat = round2(gross * VAT_RATE);
        LineTotal { subtotal: gross, vat, total: round2(gross + vat) }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct Totals {
    pub subtotal: f64,
    pub vat: f64,
    pub total: f64,
    pub requested: f64,
    pub institution: f64,
    pub other: f64,
    /// Administrative expenses inside what is requested.
    pub administrative_requested: f64,
    /// What the institution and others put, as a percentage of the whole project.
    pub counterpart_percent: f64,
    /// Administrative expenses as a percentage of what is requested.
    pub administrative_percent: f64,
}

pub fn totals(items: &[Item]) -> Totals {
    let mut t = Totals::default();
    for i in items {
        let l = line_total(i);
        t.subtotal += l.subtotal;
        t.vat += l.vat;
        t.total += l.total;
        match i.funded_by {
            Funder::Requested => {
                t.requested += l.total;
                if i.administrative {
                    t.administrative_requested += l.total;
                }
            }
            Funder::Institution => t.institution += l.total,
            Funder::Other => t.other += l.total,
        }
    }
    let percent = |part: f64, whole: f64| if whole > 0.0 { round2(part / whole * 100.0) } else { 0.0 };
    t.counterpart_percent = percent(t.institution + t.other, t.total);
    t.administrative_percent = percent(t.administrative_requested, t.requested);
    Totals {
        subtotal: round2(t.subtotal),
        vat: round2(t.vat),
        total: round2(t.total),
        requested: round2(t.requested),
        institution: round2(t.institution),
        other: round2(t.other),
        administrative_requested: round2(t.administrative_requested),
        ..t
    }
}

/// «$250,000.00»: pesos as a person reads them.
pub fn format_mxn(x: f64) -> String {
    let cents = (x.abs() * 100.0).round() as u64;
    let (whole, frac) = (cents / 100, cents % 100);
    let digits = whole.to_string();
    let mut grouped = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    format!("{}${grouped}.{frac:02}", if x < 0.0 { "-" } else { "" })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(q: f64, p: f64, vat_included: bool, funded_by: Funder, administrative: bool) -> Item {
        Item { quantity: q, unit_price: p, vat_included, funded_by, administrative }
    }

    #[test]
    fn a_price_without_tax_gets_the_tax_added_and_one_with_tax_has_it_taken_apart() {
        let without = line_total(&item(2.0, 1000.0, false, Funder::Requested, false));
        assert_eq!(without, LineTotal { subtotal: 2000.0, vat: 320.0, total: 2320.0 });
        let with = line_total(&item(2.0, 1160.0, true, Funder::Requested, false));
        assert_eq!(with, LineTotal { subtotal: 2000.0, vat: 320.0, total: 2320.0 });
        // the parts always add up to the total, whatever the rounding
        let odd = line_total(&item(3.0, 99.99, true, Funder::Requested, false));
        assert!((odd.subtotal + odd.vat - odd.total).abs() < 1e-9, "{odd:?}");
    }

    #[test]
    fn totals_split_by_who_pays_and_count_the_administrative_part_of_what_is_requested() {
        let items = [
            item(1.0, 80_000.0, false, Funder::Requested, false),
            item(1.0, 5_000.0, false, Funder::Requested, true),
            item(10.0, 1_000.0, true, Funder::Institution, false),
            item(1.0, 4_640.0, true, Funder::Other, true),
        ];
        let t = totals(&items);
        assert_eq!(t.requested, 98_600.0); // (80,000 + 5,000) * 1.16
        assert_eq!(t.institution, 10_000.0);
        assert_eq!(t.other, 4_640.0);
        assert_eq!(t.total, 113_240.0);
        assert!((t.requested + t.institution + t.other - t.total).abs() < 0.005);
        assert_eq!(t.administrative_requested, 5_800.0, "only what is requested counts toward the administrative cap");
        assert_eq!(t.administrative_percent, 5.88);
        assert_eq!(t.counterpart_percent, 12.93); // 14,640 / 113,240
    }

    #[test]
    fn an_empty_budget_has_no_percentages_to_divide() {
        let t = totals(&[]);
        assert_eq!((t.total, t.counterpart_percent, t.administrative_percent), (0.0, 0.0, 0.0));
    }

    #[test]
    fn a_line_needs_a_quantity_above_zero_and_a_price_that_is_not_negative() {
        assert_eq!(validate(1.0, 0.0), Ok(()), "a free item is allowed");
        assert_eq!(validate(0.0, 10.0), Err(ItemError::Quantity));
        assert_eq!(validate(-1.0, 10.0), Err(ItemError::Quantity));
        assert_eq!(validate(1.0, -5.0), Err(ItemError::Price));
        assert_eq!(validate(f64::NAN, 1.0), Err(ItemError::NotANumber));
        assert_eq!(validate(1.0, f64::INFINITY), Err(ItemError::NotANumber));
        assert_eq!(validate(1.0, 2.0e12), Err(ItemError::NotANumber));
    }

    #[test]
    fn pesos_are_written_the_way_a_person_reads_them() {
        assert_eq!(format_mxn(250_000.0), "$250,000.00");
        assert_eq!(format_mxn(0.0), "$0.00");
        assert_eq!(format_mxn(1_234_567.891), "$1,234,567.89");
        assert_eq!(format_mxn(999.5), "$999.50");
        assert_eq!(format_mxn(-1_000.0), "-$1,000.00");
    }

    #[test]
    fn the_names_in_the_database_round_trip() {
        for f in [Funder::Requested, Funder::Institution, Funder::Other] {
            assert_eq!(Funder::from_db(f.as_db()), Some(f));
        }
        assert_eq!(Funder::from_db("nadie"), None);
    }
}
