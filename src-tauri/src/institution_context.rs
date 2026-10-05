//! The sheet of the institution that the AI reads: everything «Mi institución» says, as aggregates, in plain Spanish.
//!
//! The AI knows the institution only through this text, so what it leaves out the AI cannot know and ends up
//! guessing. Hence three rules: (1) it carries every datum the person captured that is allowed to reach the AI;
//! (2) a section with nothing captured says so («no capturado»), because «no sé» is not «cero»; (3) the sums are
//! made here, by code. Never in it: names, contact data, RFC, pay or the amount any person pays (ADR-020), nor an
//! age range of a group of one person. It carries no incidental numbers either (versions, dates): the figure
//! check treats every number of this text as something the person said.

use crate::domain::profile::{Condition, ContractKind, DependencyLevel, InstitutionKind, ProfileInput};
use crate::service::ServiceError;
use crate::storage::profile::{self as profile_store, StoredProfile};
use rusqlite::Connection;

/// The sheet of the current profile (the latest version, confirmed or draft).
pub fn profile_context(conn: &Connection) -> Result<String, ServiceError> {
    Ok(match profile_store::load_current(conn)? {
        Some(p) => render(&p),
        None => "Perfil: sin datos. Todavía no hay nada capturado en «Mi institución»; de la institución solo se sabe lo que la persona diga.".into(),
    })
}

fn kind_text(k: InstitutionKind) -> &'static str {
    match k {
        InstitutionKind::ElderlyHome => "asilo",
        InstitutionKind::ChildrenHome => "casa hogar",
        InstitutionKind::Other => "institución de asistencia",
    }
}

fn condition_text(c: Condition) -> &'static str {
    match c {
        Condition::Good => "bueno",
        Condition::Fair => "regular",
        Condition::Poor => "malo",
        Condition::Critical => "crítico",
    }
}

fn dependency_text(d: DependencyLevel) -> &'static str {
    match d {
        DependencyLevel::Low => "baja",
        DependencyLevel::Medium => "media",
        DependencyLevel::High => "alta",
        DependencyLevel::Total => "total",
    }
}

fn contract_text(c: ContractKind) -> &'static str {
    match c {
        ContractKind::Permanent => "base",
        ContractKind::Temporary => "temporal",
        ContractKind::Fees => "honorarios",
    }
}

/// `1800000` -> `$1,800,000` (the figure check reads the commas as thousands).
fn mxn(n: i64) -> String {
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{}${out}", if n < 0 { "-" } else { "" })
}

fn text(o: &Option<String>) -> Option<&str> {
    o.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// The people served, one line per group and level of support (the fees that tell lines apart in the roster are
/// not shown): how many, how many pay a fee, and the ages when the group is of two or more people.
struct Group {
    label: String,
    dependency: Option<DependencyLevel>,
    count: i64,
    payers: i64,
    ages: Option<(i64, i64)>,
}

fn groups(i: &ProfileInput) -> Vec<Group> {
    let mut out: Vec<Group> = Vec::new();
    for g in i.population.iter().filter(|g| g.count > 0) {
        let ages = match (g.age_min, g.age_max) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        };
        match out.iter_mut().find(|x| x.label == g.label && x.dependency == g.dependency_level) {
            Some(x) => {
                x.count += g.count;
                x.payers += g.paying_count.unwrap_or(0);
                x.ages = match (x.ages, ages) {
                    (Some((a, b)), Some((c, d))) => Some((a.min(c), b.max(d))),
                    (a, b) => a.or(b),
                };
            }
            None => out.push(Group {
                label: g.label.clone(),
                dependency: g.dependency_level,
                count: g.count,
                payers: g.paying_count.unwrap_or(0),
                ages,
            }),
        }
    }
    out
}

pub fn render(p: &StoredProfile) -> String {
    let i = &p.input;
    let t = i.totals();
    let mut s = String::new();
    let mut missing: Vec<&str> = Vec::new();

    s.push_str(if p.confirmed_at.is_some() {
        "Datos de «Mi institución», confirmados por la persona.\n"
    } else {
        "Datos de «Mi institución». Es un BORRADOR: la persona todavía no lo confirma.\n"
    });
    s.push_str(&format!("Institución: {} ({}).\n", i.institution.name, kind_text(i.institution.kind)));
    match text(&i.institution.mission) {
        Some(m) => s.push_str(&format!("A qué se dedica: {m}\n")),
        None => missing.push("a qué se dedica"),
    }
    match i.capacity_total {
        Some(c) => s.push_str(&format!("Capacidad total: {c} personas.\n")),
        None => missing.push("capacidad total"),
    }
    match i.annual_budget_mxn {
        Some(b) => s.push_str(&format!("Presupuesto anual: {} pesos.\n", mxn(b))),
        None => missing.push("presupuesto anual"),
    }

    if i.income.is_empty() {
        missing.push("de dónde vienen sus ingresos");
    } else {
        for inc in &i.income {
            match inc.annual_amount_mxn {
                Some(a) => s.push_str(&format!("Ingreso: {} — {} pesos al año.\n", inc.label, mxn(a))),
                None => s.push_str(&format!("Ingreso: {} — monto no capturado.\n", inc.label)),
            }
        }
        if t.income_annual_mxn > 0 {
            s.push_str(&format!("Suma de los ingresos capturados: {} pesos al año.\n", mxn(t.income_annual_mxn)));
        }
    }

    // people served: only how many, never who
    let people = groups(i);
    if people.is_empty() {
        missing.push("a quiénes atiende");
    } else {
        for g in &people {
            let mut line = format!("Población: {} — {} {}", g.label, g.count, if g.count == 1 { "persona" } else { "personas" });
            if let Some(d) = g.dependency {
                line.push_str(&format!("; nivel de dependencia {}", dependency_text(d)));
            }
            if let (Some((a, b)), true) = (g.ages, g.count >= 2) {
                line.push_str(&format!("; edades de {a} a {b} años"));
            }
            if g.payers > 0 {
                line.push_str(&format!("; {} pagan cuota de estancia", g.payers));
            }
            s.push_str(&line);
            s.push_str(".\n");
        }
        s.push_str(&format!("Total de personas atendidas: {}.\n", t.population));
        if t.fee_payers > 0 {
            s.push_str(&format!("De ellas, {} pagan cuota de estancia.\n", t.fee_payers));
        }
    }

    // staff: one line per kind of position; pay never goes to the AI
    let mut jobs: Vec<(&str, bool, Option<&str>, Option<ContractKind>, i64)> = Vec::new();
    for st in &i.staff {
        let shift = text(&st.shift);
        match jobs.iter_mut().find(|j| j.0 == st.role.as_str() && j.1 == st.paid && j.2 == shift && j.3 == st.contract) {
            Some(j) => j.4 += st.count,
            None => jobs.push((st.role.as_str(), st.paid, shift, st.contract, st.count)),
        }
    }
    if jobs.is_empty() {
        missing.push("el personal");
    } else {
        for (role, paid, shift, contract, count) in jobs {
            let mut details = vec![if paid { "con sueldo".to_string() } else { "voluntariado".to_string() }];
            if let Some(sh) = shift {
                details.push(format!("turno: {sh}"));
            }
            if let Some(c) = contract {
                details.push(format!("contrato: {}", contract_text(c)));
            }
            s.push_str(&format!("Personal: {role} — {count} ({}).\n", details.join("; ")));
        }
        s.push_str(&format!("Total del personal: {} con sueldo y {} de voluntariado.\n", t.staff_paid, t.staff_volunteer));
    }

    if i.facilities.is_empty() {
        missing.push("las instalaciones");
    } else {
        for f in &i.facilities {
            let mut details = vec![format!("estado: {}", f.condition.map(condition_text).unwrap_or("sin indicar"))];
            match f.accessible {
                Some(true) => details.push("accesible: sí".into()),
                Some(false) => details.push("accesible: no".into()),
                None => {}
            }
            if let Some(n) = text(&f.notes) {
                details.push(format!("nota: {n}"));
            }
            s.push_str(&format!("Instalación: {} ×{} ({}).\n", f.kind, f.count, details.join("; ")));
        }
    }

    if let Some(n) = text(&i.notes) {
        s.push_str(&format!("Notas de la institución: {n}\n"));
    }
    if !missing.is_empty() {
        s.push_str(&format!(
            "No capturado en «Mi institución»: {}. «No capturado» quiere decir que no se sabe, no que sea cero.\n",
            missing.join(", ")
        ));
    }
    s
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::domain::profile::*;
    use crate::storage::open_encrypted;

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        (dir, c)
    }

    /// A profile with every kind of datum, for the tests that check the AI gets all of it.
    pub(crate) fn rich() -> ProfileInput {
        ProfileInput {
            institution: InstitutionInput {
                name: "Asilo Ficticio".into(),
                kind: InstitutionKind::ElderlyHome,
                mission: Some("Un hogar digno para adultos mayores.".into()),
                legal_rfc: Some("AFI200101AB1".into()),
                contact_phone: Some("55 5555 0101".into()),
                legal_rep_name: Some("Rosa Representante".into()),
                ..Default::default()
            },
            capacity_total: Some(25),
            annual_budget_mxn: Some(1_800_000),
            notes: Some("Perfil ficticio para pruebas.".into()),
            population: vec![
                PopulationGroupInput { label: "Adultos mayores".into(), count: 8, age_min: Some(70), age_max: Some(95), dependency_level: Some(DependencyLevel::High), paying_count: Some(5), monthly_fee_mxn: Some(3_333), ..Default::default() },
                PopulationGroupInput { label: "Adultos mayores".into(), count: 3, age_min: Some(66), age_max: Some(80), dependency_level: Some(DependencyLevel::High), ..Default::default() },
                PopulationGroupInput { label: "Hombres".into(), count: 1, age_min: Some(93), age_max: Some(93), dependency_level: Some(DependencyLevel::Total), ..Default::default() },
            ],
            staff: vec![
                StaffGroupInput { role: "Cuidadora".into(), count: 4, paid: true, monthly_salary_mxn: Some(7_777), shift: Some("noche".into()), contract: Some(ContractKind::Permanent), ..Default::default() },
                StaffGroupInput { role: "Voluntaria".into(), count: 3, paid: false, ..Default::default() },
            ],
            facilities: vec![
                FacilityInput { kind: "Baño".into(), count: 3, condition: Some(Condition::Poor), accessible: Some(false), notes: Some("Piso resbaloso, sin barras.".into()) },
                FacilityInput { kind: "Cocina".into(), count: 1, condition: Some(Condition::Good), accessible: None, notes: None },
            ],
            income: vec![
                IncomeSourceInput { label: "Cuotas de recuperación".into(), annual_amount_mxn: Some(720_000) },
                IncomeSourceInput { label: "Donativos".into(), annual_amount_mxn: Some(680_000) },
            ],
        }
    }

    fn saved(input: &ProfileInput, confirm: bool) -> (tempfile::TempDir, Connection) {
        let (d, mut c) = db();
        profile_store::save(&mut c, input).unwrap();
        if confirm {
            profile_store::confirm(&mut c).unwrap();
        }
        (d, c)
    }

    /// What the sheet must carry, whichever stage asks: one line of the sheet per fact of «Mi institución».
    pub(crate) const RICH_FACTS: &[&str] = &[
        "Asilo Ficticio (asilo)",
        "A qué se dedica: Un hogar digno para adultos mayores.",
        "Capacidad total: 25 personas.",
        "Presupuesto anual: $1,800,000 pesos.",
        "Ingreso: Cuotas de recuperación — $720,000 pesos al año.",
        "Suma de los ingresos capturados: $1,400,000 pesos al año.",
        "Población: Adultos mayores — 11 personas; nivel de dependencia alta; edades de 66 a 95 años; 5 pagan cuota de estancia.",
        "Población: Hombres — 1 persona; nivel de dependencia total.",
        "Total de personas atendidas: 12.",
        "Personal: Cuidadora — 4 (con sueldo; turno: noche; contrato: base).",
        "Personal: Voluntaria — 3 (voluntariado).",
        "Total del personal: 4 con sueldo y 3 de voluntariado.",
        "Instalación: Baño ×3 (estado: malo; accesible: no; nota: Piso resbaloso, sin barras.).",
        "Instalación: Cocina ×1 (estado: bueno).",
        "Notas de la institución: Perfil ficticio para pruebas.",
    ];

    #[test]
    fn the_sheet_carries_every_fact_of_the_profile_in_plain_spanish() {
        let (_d, c) = saved(&rich(), true);
        let ctx = profile_context(&c).unwrap();
        for fact in RICH_FACTS {
            assert!(ctx.contains(fact), "missing «{fact}» in:\n{ctx}");
        }
        assert!(ctx.starts_with("Datos de «Mi institución», confirmados por la persona."));
        assert!(!ctx.contains("No capturado"), "nothing is missing:\n{ctx}");
        for raw in ["poor", "good", "high", "total", "fees", "permanent", "elderly_home"] {
            assert!(!ctx.contains(&format!("{raw})")) && !ctx.contains(&format!(": {raw}")), "raw value «{raw}» in:\n{ctx}");
        }
    }

    #[test]
    fn what_must_not_reach_the_ai_does_not() {
        let (_d, c) = saved(&rich(), true);
        let ctx = profile_context(&c).unwrap();
        // pay, the fee a person pays, contact data, RFC and the representative
        for secret in ["7777", "7,777", "3333", "3,333", "AFI200101", "55 5555", "Rosa Representante"] {
            assert!(!ctx.contains(secret), "«{secret}» leaked:\n{ctx}");
        }
        // the age of a group of one person would be that person's age
        assert!(ctx.contains("Población: Hombres — 1 persona; nivel de dependencia total."), "{ctx}");
        assert!(!ctx.contains("93"), "{ctx}");
    }

    #[test]
    fn a_section_with_nothing_captured_says_so_instead_of_staying_silent() {
        let input = ProfileInput { institution: InstitutionInput { name: "Casa Vacía".into(), ..Default::default() }, ..Default::default() };
        let (_d, c) = saved(&input, false);
        let ctx = profile_context(&c).unwrap();
        assert!(ctx.contains("BORRADOR"), "{ctx}");
        assert!(ctx.contains("Institución: Casa Vacía (institución de asistencia)."), "{ctx}");
        for gap in ["a qué se dedica", "capacidad total", "presupuesto anual", "de dónde vienen sus ingresos", "a quiénes atiende", "el personal", "las instalaciones"] {
            assert!(ctx.contains(gap), "the gap «{gap}» is not named:\n{ctx}");
        }
        assert!(ctx.contains("no que sea cero"), "{ctx}");
        assert!(!ctx.contains("Total de personas atendidas"), "no invented zero totals:\n{ctx}");
    }

    #[test]
    fn without_any_profile_the_sheet_says_there_is_nothing() {
        let (_d, c) = db();
        let ctx = profile_context(&c).unwrap();
        assert!(ctx.starts_with("Perfil: sin datos."), "{ctx}");
    }

    #[test]
    fn the_sheet_adds_no_incidental_numbers_to_what_the_figure_check_trusts() {
        let (_d, c) = saved(&rich(), true);
        let ctx = profile_context(&c).unwrap();
        // every number of the sheet is a fact of the profile or a sum made by code
        let allowed = ["25", "1800000", "720000", "680000", "1400000", "11", "8", "3", "5", "66", "95", "12", "4", "1", "70", "80"];
        for n in crate::domain::figures::digit_numbers(&ctx) {
            assert!(allowed.contains(&n.as_str()), "unexpected number {n} in the sheet:\n{ctx}");
        }
    }

    #[test]
    fn the_padron_reaches_the_ai_only_as_counts() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        let input: ProfileInput = serde_json::from_str(include_str!("../../fixtures/institucion-asilo.json")).unwrap();
        crate::service::save_profile(&mut c, input.clone(), None).unwrap();
        crate::roster_service::seed_roster(&mut c, include_str!("../../fixtures/padron-asilo.json")).unwrap();
        crate::service::save_profile(&mut c, input, None).unwrap();

        let stored = profile_store::load_current(&c).unwrap().unwrap();
        let ctx = render(&stored);
        assert!(ctx.contains("Personal: "), "{ctx}");
        assert!(ctx.contains("Población: "), "{ctx}");
        assert!(ctx.contains("Total de personas atendidas: "), "{ctx}");
        assert!(ctx.contains("Piso resbaloso, sin barras de apoyo"), "the notes of the facilities reach the AI:\n{ctx}");
        assert!(ctx.contains("Presupuesto anual: $1,800,000 pesos."), "{ctx}");
        assert!(!ctx.contains("Esperanza Robles") && !ctx.contains("Vázquez"), "no name:\n{ctx}");
        for st in &stored.input.staff {
            if let Some(pay) = st.monthly_salary_mxn {
                assert!(!ctx.contains(&pay.to_string()), "a salary ({pay}) leaked:\n{ctx}");
            }
        }
        for g in &stored.input.population {
            if let Some(fee) = g.monthly_fee_mxn {
                assert!(!ctx.contains(&fee.to_string()), "a fee ({fee}) leaked:\n{ctx}");
            }
        }
    }
}
