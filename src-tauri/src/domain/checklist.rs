//! The automatic review before a project is ready (docs/02-flujo-funcional.md). Pure: the service gathers the
//! facts, this decides. No AI takes part. A check is an error (the project cannot go on) or a warning (it is said
//! and nothing is blocked). The screen words every check in plain language from its `code` and `args`.

use super::budget::{format_mxn, Totals};
use super::requirements::CallRequirements;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    pub code: &'static str,
    pub level: Level,
    /// Values for the words of the check, already written the way a person reads them.
    pub args: Vec<String>,
    /// What fixes it: a section key, or `budget`, `schedule`, `call`.
    pub target: Option<String>,
    /// The check in plain Spanish, ready to show or to put in the guide.
    pub text: String,
}

/// A section of the guide as the review sees it.
#[derive(Debug, Clone)]
pub struct SectionState {
    pub key: String,
    pub title: String,
    pub required: bool,
    pub confirmed: bool,
}

pub struct Facts<'a> {
    pub requirements: &'a CallRequirements,
    pub totals: &'a Totals,
    pub budget_lines: usize,
    pub budget_confirmed: bool,
    pub schedule_activities: usize,
    pub duration_months: u32,
    pub schedule_confirmed: bool,
    pub sections: &'a [SectionState],
    /// Findings of the scanner over everything that goes into the guide.
    pub scanner_findings: usize,
    /// `fits`, `partial` or `mismatch`, as judged in the conversation.
    pub fit: Option<&'a str>,
    /// Today, `AAAA-MM-DD`.
    pub today: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub checks: Vec<Check>,
    pub errors: usize,
    pub warnings: usize,
}

impl Report {
    /// The project can go on: nothing is an error.
    pub fn clean(&self) -> bool {
        self.errors == 0
    }
}

fn check(code: &'static str, level: Level, args: Vec<String>, target: Option<&str>) -> Check {
    let text = describe(code, &args);
    Check { code, level, args, target: target.map(String::from), text }
}

/// The words of every check, in the plain language of docs/08-estilo-redaccion.md.
pub fn describe(code: &str, a: &[String]) -> String {
    let arg = |i: usize| a.get(i).cloned().unwrap_or_default();
    match code {
        "no_budget" => "Falta el presupuesto: agregue al menos una partida.".into(),
        "amount_over_max" => format!("Lo que pide ({}) es más de lo que permite la convocatoria ({}). Baje lo que pide o pase partidas a lo que aporta la institución.", arg(0), arg(1)),
        "amount_under_min" => format!("Lo que pide ({}) es menos de lo que la convocatoria exige como mínimo ({}).", arg(0), arg(1)),
        "cofunding_low" => format!("La institución y otras fuentes aportan el {} % del proyecto y la convocatoria pide al menos el {} %.", arg(0), arg(1)),
        "admin_over_cap" => format!("Los gastos administrativos son el {} % de lo que se pide y la convocatoria permite hasta el {} %.", arg(0), arg(1)),
        "totals_mismatch" => "Los totales del presupuesto no cuadran. Revise las partidas.".into(),
        "budget_not_confirmed" => "Falta confirmar el presupuesto.".into(),
        "amount_unknown" => "La convocatoria no dice un monto máximo. Confirme con la convocatoria cuánto se puede pedir.".into(),
        "foreign_currency" => "La convocatoria da sus montos en otra moneda y no se pudieron comparar con el presupuesto. Revíselos usted.".into(),
        "no_schedule" => "Falta el cronograma: agregue al menos una actividad.".into(),
        "duration_over_max" => format!("El proyecto dura {} meses y la convocatoria permite hasta {}.", arg(0), arg(1)),
        "schedule_not_confirmed" => "Falta confirmar el cronograma.".into(),
        "section_not_confirmed" => format!("Falta confirmar la sección «{}».", arg(0)),
        "scanner_findings" => format!("Hay {} datos que parecen de una persona en los textos. Quítelos o táptelos antes de seguir.", arg(0)),
        "call_closed" => format!("La fecha de cierre de la convocatoria ({}) ya pasó. Confirme si todavía puede participar.", arg(0)),
        "fit_mismatch" => "La idea del proyecto no parece encajar con lo que apoya la convocatoria.".into(),
        "fit_partial" => "La idea del proyecto solo encaja en parte con lo que apoya la convocatoria.".into(),
        "docs_to_gather" => format!("La convocatoria pide {} documentos obligatorios. Reúnalos antes de entregar.", arg(0)),
        _ => "Hay algo por revisar.".into(),
    }
}

fn pct(x: f64) -> String {
    format!("{}", (x * 100.0).round() / 100.0)
}

pub fn run(f: &Facts) -> Report {
    use Level::*;
    let r = f.requirements;
    let t = f.totals;
    let mut c: Vec<Check> = Vec::new();

    // the budget
    if f.budget_lines == 0 {
        c.push(check("no_budget", Error, vec![], Some("budget")));
    } else {
        if let Some(max) = &r.max_amount_mxn {
            if t.requested > max.value + 0.005 {
                c.push(check("amount_over_max", Error, vec![format_mxn(t.requested), format_mxn(max.value)], Some("budget")));
            }
        }
        if let Some(min) = &r.min_amount_mxn {
            if t.requested + 0.005 < min.value {
                c.push(check("amount_under_min", Error, vec![format_mxn(t.requested), format_mxn(min.value)], Some("budget")));
            }
        }
        if let Some(co) = &r.cofunding_percent {
            if t.counterpart_percent + 0.005 < co.value {
                c.push(check("cofunding_low", Error, vec![pct(t.counterpart_percent), pct(co.value)], Some("budget")));
            }
        }
        if let Some(cap) = &r.admin_cap_percent {
            if t.administrative_percent > cap.value + 0.005 {
                c.push(check("admin_over_cap", Error, vec![pct(t.administrative_percent), pct(cap.value)], Some("budget")));
            }
        }
        if (t.requested + t.institution + t.other - t.total).abs() > 0.01 {
            c.push(check("totals_mismatch", Error, vec![], Some("budget")));
        }
        if !f.budget_confirmed {
            c.push(check("budget_not_confirmed", Error, vec![], Some("budget")));
        }
        if r.max_amount_mxn.is_none() && !r.foreign_currency {
            c.push(check("amount_unknown", Info, vec![], Some("budget")));
        }
    }
    if r.foreign_currency {
        c.push(check("foreign_currency", Warn, vec![], Some("budget")));
    }

    // the schedule
    if f.schedule_activities == 0 {
        c.push(check("no_schedule", Error, vec![], Some("schedule")));
    } else {
        if let Some(max) = &r.max_duration_months {
            if f.duration_months > max.value {
                c.push(check("duration_over_max", Error, vec![f.duration_months.to_string(), max.value.to_string()], Some("schedule")));
            }
        }
        if !f.schedule_confirmed {
            c.push(check("schedule_not_confirmed", Error, vec![], Some("schedule")));
        }
    }

    // the sections
    for s in f.sections {
        // an optional section nobody confirmed is simply left out of the guide
        if s.required && !s.confirmed {
            c.push(check("section_not_confirmed", Error, vec![s.title.clone()], Some(&s.key)));
        }
    }

    // what goes out
    if f.scanner_findings > 0 {
        c.push(check("scanner_findings", Error, vec![f.scanner_findings.to_string()], None));
    }

    // the call
    if let Some(d) = &r.closing_date {
        if d.value.as_str() < f.today {
            c.push(check("call_closed", Warn, vec![d.value.clone()], Some("call")));
        }
    }
    match f.fit {
        Some("mismatch") => c.push(check("fit_mismatch", Warn, vec![], Some("call"))),
        Some("partial") => c.push(check("fit_partial", Warn, vec![], Some("call"))),
        _ => {}
    }
    let docs = r.required_docs.len();
    if docs > 0 {
        c.push(check("docs_to_gather", Info, vec![docs.to_string()], None));
    }

    let errors = c.iter().filter(|x| x.level == Error).count();
    let warnings = c.iter().filter(|x| x.level == Warn).count();
    Report { checks: c, errors, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::requirements::{Line, Sourced};

    fn sourced<T>(value: T) -> Option<Sourced<T>> {
        Some(Sourced { value, page: Some(2), file: Some("bases.pdf".into()) })
    }

    fn reqs() -> CallRequirements {
        CallRequirements {
            max_amount_mxn: sourced(250_000.0),
            min_amount_mxn: sourced(50_000.0),
            cofunding_percent: sourced(30.0),
            admin_cap_percent: sourced(10.0),
            max_duration_months: sourced(12),
            closing_date: sourced("2027-05-23".to_string()),
            required_docs: vec![Line { text: "Acta constitutiva".into(), applies_to: None, page: None, file: None }],
            ..Default::default()
        }
    }

    fn good_totals() -> Totals {
        Totals { subtotal: 150_000.0, vat: 24_000.0, total: 200_000.0, requested: 140_000.0, institution: 60_000.0, other: 0.0, administrative_requested: 7_000.0, counterpart_percent: 30.0, administrative_percent: 5.0 }
    }

    fn sections(confirmed: bool) -> Vec<SectionState> {
        vec![
            SectionState { key: "what".into(), title: "Qué se va a hacer".into(), required: true, confirmed },
            SectionState { key: "sustainability".into(), title: "Cómo se mantendrá".into(), required: false, confirmed: false },
        ]
    }

    fn facts<'a>(r: &'a CallRequirements, t: &'a Totals, s: &'a [SectionState]) -> Facts<'a> {
        Facts { requirements: r, totals: t, budget_lines: 4, budget_confirmed: true, schedule_activities: 3, duration_months: 10, schedule_confirmed: true, sections: s, scanner_findings: 0, fit: Some("fits"), today: "2027-03-01" }
    }

    fn codes(rep: &Report) -> Vec<&'static str> {
        rep.checks.iter().map(|c| c.code).collect()
    }

    #[test]
    fn a_project_that_meets_everything_is_clean_and_only_reminds_what_to_gather() {
        let (r, t, s) = (reqs(), good_totals(), sections(true));
        let rep = run(&facts(&r, &t, &s));
        assert!(rep.clean() && rep.warnings == 0, "{:?}", rep.checks);
        assert_eq!(codes(&rep), vec!["docs_to_gather"]);
        assert_eq!(rep.checks[0].args, vec!["1".to_string()]);
    }

    #[test]
    fn each_figure_outside_what_the_call_allows_is_an_error_that_says_both_numbers() {
        let r = reqs();
        let s = sections(true);
        let t = Totals { requested: 300_000.0, total: 360_000.0, institution: 60_000.0, counterpart_percent: 16.67, ..good_totals() };
        let rep = run(&facts(&r, &t, &s));
        let over = rep.checks.iter().find(|c| c.code == "amount_over_max").unwrap();
        assert_eq!(over.args, vec!["$300,000.00".to_string(), "$250,000.00".to_string()]);
        assert!(codes(&rep).contains(&"cofunding_low"));
        let low = Totals { requested: 10_000.0, total: 70_000.0, ..good_totals() };
        assert!(codes(&run(&facts(&r, &low, &s))).contains(&"amount_under_min"));
        let admin = Totals { administrative_percent: 12.5, ..good_totals() };
        let rep = run(&facts(&r, &admin, &s));
        assert_eq!(rep.checks.iter().find(|c| c.code == "admin_over_cap").unwrap().args, vec!["12.5".to_string(), "10".to_string()]);
        assert!(!rep.clean());
    }

    #[test]
    fn totals_that_do_not_add_up_are_an_error_even_if_the_call_says_nothing() {
        let r = CallRequirements::default();
        let s = sections(true);
        let t = Totals { requested: 100.0, institution: 50.0, other: 0.0, total: 200.0, ..Default::default() };
        let rep = run(&facts(&r, &t, &s));
        assert!(codes(&rep).contains(&"totals_mismatch"));
        // and with nothing to compare against the missing amount is only mentioned
        let ok = good_totals();
        assert!(codes(&run(&facts(&r, &ok, &s))).contains(&"amount_unknown"));
        assert!(run(&facts(&r, &ok, &s)).clean());
    }

    #[test]
    fn a_budget_and_a_schedule_are_needed_and_the_duration_has_to_fit() {
        let (r, t, s) = (reqs(), good_totals(), sections(true));
        let mut f = facts(&r, &t, &s);
        f.budget_lines = 0;
        f.schedule_activities = 0;
        assert_eq!(codes(&run(&f)).iter().filter(|c| matches!(**c, "no_budget" | "no_schedule")).count(), 2);
        let mut f = facts(&r, &t, &s);
        f.duration_months = 14;
        let rep = run(&f);
        assert_eq!(rep.checks.iter().find(|c| c.code == "duration_over_max").unwrap().args, vec!["14".to_string(), "12".to_string()]);
        let mut f = facts(&r, &t, &s);
        f.budget_confirmed = false;
        f.schedule_confirmed = false;
        assert!(codes(&run(&f)).contains(&"budget_not_confirmed") && codes(&run(&f)).contains(&"schedule_not_confirmed"));
    }

    #[test]
    fn a_required_section_that_is_not_confirmed_blocks_and_points_to_it_but_an_optional_one_does_not() {
        let (r, t) = (reqs(), good_totals());
        let s = sections(false);
        let rep = run(&facts(&r, &t, &s));
        let c = rep.checks.iter().find(|c| c.code == "section_not_confirmed").unwrap();
        assert_eq!((c.args.clone(), c.target.as_deref()), (vec!["Qué se va a hacer".to_string()], Some("what")));
        assert_eq!(rep.checks.iter().filter(|c| c.code == "section_not_confirmed").count(), 1, "the optional one is not asked for");
    }

    #[test]
    fn anything_the_scanner_finds_blocks_and_the_call_and_the_fit_only_warn() {
        let (r, t, s) = (reqs(), good_totals(), sections(true));
        let mut f = facts(&r, &t, &s);
        f.scanner_findings = 2;
        let rep = run(&f);
        assert_eq!(rep.checks.iter().find(|c| c.code == "scanner_findings").unwrap().args, vec!["2".to_string()]);
        assert!(!rep.clean());

        let mut f = facts(&r, &t, &s);
        f.today = "2027-06-01";
        f.fit = Some("mismatch");
        let rep = run(&f);
        assert_eq!(rep.checks.iter().find(|c| c.code == "call_closed").unwrap().args, vec!["2027-05-23".to_string()]);
        assert!(codes(&rep).contains(&"fit_mismatch"));
        assert_eq!((rep.errors, rep.warnings), (0, 2));
        assert!(rep.clean(), "warnings never block");
        f.fit = Some("partial");
        assert!(codes(&run(&f)).contains(&"fit_partial"));
    }

    #[test]
    fn an_amount_in_another_currency_is_a_warning_because_it_could_not_be_compared() {
        let mut r = reqs();
        r.max_amount_mxn = None;
        r.foreign_currency = true;
        let (t, s) = (good_totals(), sections(true));
        let rep = run(&facts(&r, &t, &s));
        assert!(codes(&rep).contains(&"foreign_currency") && !codes(&rep).contains(&"amount_unknown"));
        assert!(rep.clean());
    }

    #[test]
    fn every_check_has_plain_words_with_its_numbers() {
        let (r, t, s) = (reqs(), good_totals(), sections(false));
        let mut f = facts(&r, &t, &s);
        f.scanner_findings = 3;
        f.today = "2027-06-01";
        f.fit = Some("partial");
        let over = Totals { requested: 300_000.0, total: 360_000.0, counterpart_percent: 16.67, administrative_percent: 12.0, ..good_totals() };
        let mut all = run(&f).checks;
        all.extend(run(&facts(&r, &over, &s)).checks);
        assert!(all.iter().any(|c| c.text.contains("3 datos que parecen de una persona")));
        assert!(all.iter().any(|c| c.text.contains("$300,000.00") && c.text.contains("$250,000.00")));
        assert!(all.iter().all(|c| !c.text.is_empty() && c.text != "Hay algo por revisar."), "{:?}", all.iter().map(|c| (&c.code, &c.text)).collect::<Vec<_>>());
        // every code the rules can produce is worded
        for code in ["no_budget", "amount_under_min", "totals_mismatch", "budget_not_confirmed", "amount_unknown", "foreign_currency", "no_schedule", "duration_over_max", "schedule_not_confirmed", "fit_mismatch"] {
            assert_ne!(describe(code, &[]), "Hay algo por revisar.", "{code}");
        }
        assert_eq!(describe("nada", &[]), "Hay algo por revisar.");
    }
}
