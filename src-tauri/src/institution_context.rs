//! The sheet of the institution that the AI reads: everything «Mi institución» says, as aggregates, in plain Spanish.
//!
//! The AI knows the institution only through this text, so what it leaves out the AI cannot know and ends up
//! guessing. Hence three rules: (1) it carries every datum the person captured that is allowed to reach the AI;
//! (2) a section with nothing captured says so («no capturado»), because «no sé» is not «cero»; (3) the sums are
//! made here, by code. Never in it: names, contact data, RFC, pay or the amount any person pays (ADR-020), nor an
//! age range of a group of one person. Neither the payroll nor the fees of the roster go as a figure, not even
//! added up: with one paid person the total is that person's pay. That is also why the balance goes as words
//! (whether the income covers the expenses), never as a figure the payroll could be worked out from (ADR-026).
//! It carries no incidental numbers either (versions, dates): the figure check treats every number of this text
//! as something the person said.

use crate::domain::finances::{ExpenseBasis, BENEFICIARY_FEES, EXPENSE};
use crate::domain::profile::{Condition, DependencyLevel, InstitutionKind, IncomeKind, Period, ProfileInput};
use crate::hr::api::{Count, MIN_GROUP};
use crate::hr::domain::aggregate::StaffSummary;
use crate::service::ServiceError;
use crate::storage::profile::{self as profile_store, StoredProfile};
use rusqlite::Connection;

/// The sheet of the current profile (the latest version, confirmed or draft), with the staff as the staff module
/// tells it now (ADR-027).
pub fn profile_context(conn: &Connection) -> Result<String, ServiceError> {
    Ok(match profile_store::load_current(conn)? {
        Some(p) => render(&p, &crate::hr::api::ai_summary(conn)?),
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

fn relation_text(code: &str) -> &'static str {
    match code {
        "employee" => "con sueldo",
        "fees" => "por honorarios o asimilados",
        "religious" => "religiosas o religiosos de la congregación",
        "volunteer" => "de voluntariado",
        "trainee" => "en servicio social o prácticas",
        _ => "de una empresa externa",
    }
}

fn area_text(code: &str) -> &'static str {
    match code {
        "care" => "cuidado",
        "health" => "salud",
        "kitchen" => "cocina",
        "cleaning" => "limpieza",
        "laundry" => "lavandería",
        "administration" => "administración",
        "social_work" => "trabajo social",
        "psychology" => "psicología",
        "rehabilitation" => "rehabilitación",
        "education" => "educación",
        "pastoral" => "pastoral",
        "maintenance" => "mantenimiento",
        "security" => "vigilancia",
        _ => "otra área",
    }
}

fn schedule_text(code: &str) -> &'static str {
    match code {
        "full_time" => "tiempo completo",
        "part_time" => "medio tiempo",
        "hourly" => "por horas",
        _ => "fines de semana",
    }
}

fn shift_text(code: &str) -> &'static str {
    match code {
        "morning" => "matutino",
        "afternoon" => "vespertino",
        "night" => "nocturno",
        "rotating" => "por turnos",
        _ => "24 horas",
    }
}

fn education_text(code: &str) -> &'static str {
    match code {
        "basic" => "hasta secundaria",
        "high_school_or_technical" => "con preparatoria o carrera técnica",
        _ => "con licenciatura o posgrado",
    }
}

fn seniority_text(code: &str) -> &'static str {
    match code {
        "under_1" => "con menos de 1 año",
        "1_to_4" => "de 1 a 4 años",
        "5_to_9" => "de 5 a 9 años",
        _ => "de 10 años o más",
    }
}

fn people(n: i64) -> String {
    format!("{n} {}", if n == 1 { "persona" } else { "personas" })
}

/// «4 con sueldo, 3 de voluntariado».
fn listed(counts: &[Count], words: fn(&str) -> &'static str) -> String {
    counts.iter().map(|c| format!("{} {}", c.count, words(c.code))).collect::<Vec<_>>().join(", ")
}

/// The staff, by position: how many, their relation, schedule and shift, the seats and what the position does.
/// Schooling and seniority go only for groups of `MIN_GROUP` people or more.
fn render_staff(s: &mut String, staff: &StaffSummary, missing: &mut Vec<&str>) {
    if staff.total == 0 && staff.positions.is_empty() {
        missing.push("el personal");
        return;
    }
    for p in &staff.positions {
        let area = p.area.as_deref().map(|a| format!(" ({})", area_text(a))).unwrap_or_default();
        if p.people == 0 {
            let seats = p.authorized_seats.unwrap_or(0);
            s.push_str(&format!(
                "Puesto sin cubrir: {}{area} — {seats} {} y ninguna cubierta.\n",
                p.title,
                if seats == 1 { "plaza autorizada" } else { "plazas autorizadas" }
            ));
        } else {
            let mut details = vec![listed(&p.by_relation, relation_text)];
            if !p.schedules.is_empty() {
                details.push(format!("jornada: {}", listed(&p.schedules, schedule_text)));
            }
            if !p.shifts.is_empty() {
                details.push(format!("turno: {}", listed(&p.shifts, shift_text)));
            }
            s.push_str(&format!("Personal: {}{area} — {} ({}).", p.title, people(p.people), details.join("; ")));
            if let Some(seats) = p.authorized_seats {
                s.push_str(&format!(" Plazas autorizadas: {seats}; sin cubrir: {}.", p.vacancies));
            }
            s.push('\n');
        }
        if let Some(d) = &p.duties {
            s.push_str(&format!("Funciones del puesto {}: {d}\n", p.title));
        }
    }
    if staff.total > 0 {
        s.push_str(&format!("Total del personal: {}: {}.\n", people(staff.total), listed(&staff.by_relation, relation_text)));
    }
    if !staff.education.is_empty() {
        s.push_str(&format!("Escolaridad del personal (solo grupos de {MIN_GROUP} o más personas): {}.\n", listed(&staff.education, education_text)));
    }
    if !staff.seniority.is_empty() {
        s.push_str(&format!("Antigüedad en la institución (solo grupos de {MIN_GROUP} o más personas): {}.\n", listed(&staff.seniority, seniority_text)));
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

fn income_kind_text(k: IncomeKind) -> &'static str {
    match k {
        IncomeKind::FeeEstimate => "cuotas de los beneficiarios (aproximado escrito a mano)",
        IncomeKind::RecurringDonor => "donante fijo",
        IncomeKind::OccasionalDonation => "donativo ocasional",
        IncomeKind::ProjectGrant => "donativo ganado con un proyecto",
        IncomeKind::Other => "otro ingreso",
    }
}

/// `$5,000 al mes ($60,000 al año)` or `$680,000 al año`; the yearly figure is made here.
fn amount_text(amount: Option<i64>, period: Period) -> String {
    match (amount, period) {
        (None, _) => "monto no capturado".into(),
        (Some(a), Period::Monthly) => format!("{} pesos al mes ({} al año)", mxn(a), mxn(a * 12)),
        (Some(a), Period::Annual) => format!("{} pesos al año", mxn(a)),
    }
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

pub fn render(p: &StoredProfile, staff: &StaffSummary) -> String {
    let i = &p.input;
    let t = i.totals(p.as_of_year);
    let money = i.finances(p.as_of_year);
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

    // income: what the person wrote, by kind; the roster fees only as how many pay
    let roster_fees = money.income.iter().any(|l| l.kind == BENEFICIARY_FEES);
    if roster_fees {
        s.push_str(&format!(
            "Ingreso — cuotas de los beneficiarios (del padrón): las pagan {} {}; el monto no se comparte.\n",
            t.fee_payers,
            if t.fee_payers == 1 { "persona" } else { "personas" }
        ));
    }
    let written: Vec<_> = money.income.iter().filter(|l| l.counted && l.index.is_some()).collect();
    for l in &written {
        let inc = &i.income[l.index.unwrap_or_default()];
        s.push_str(&format!("Ingreso — {}: {} — {}.\n", income_kind_text(inc.kind), inc.label, amount_text(inc.amount_mxn, inc.period)));
    }
    let written_sum: i64 = written.iter().filter_map(|l| l.annual_mxn).sum();
    if written_sum > 0 {
        s.push_str(&format!(
            "Suma de los ingresos escritos a mano: {} pesos al año{}.\n",
            mxn(written_sum),
            if roster_fees { " (sin las cuotas del padrón)" } else { "" }
        ));
    }
    if !roster_fees && written.is_empty() {
        missing.push("de dónde vienen sus ingresos");
    }

    // expenses: the approximate figure, the list, and the payroll only as words
    if let Some(b) = i.annual_budget_mxn {
        s.push_str(&format!("Gasto anual aproximado (cifra a ojo de la persona, todo incluido): {} pesos.\n", mxn(b)));
    }
    let listed: Vec<_> = money.expenses.iter().filter(|l| l.kind == EXPENSE).collect();
    for l in &listed {
        let e = &i.expenses[l.index.unwrap_or_default()];
        s.push_str(&format!("Egreso: {} — {}.\n", e.label, amount_text(e.amount_mxn, e.period)));
    }
    if t.payroll_cost_annual_mxn > 0 {
        s.push_str(match money.expenses_basis {
            ExpenseBasis::Estimate => "La nómina del personal (con aguinaldo y prima vacacional) va dentro del gasto aproximado; el monto no se comparte.\n",
            _ => "Egreso: nómina del personal con aguinaldo y prima vacacional (del padrón); el monto no se comparte.\n",
        });
    }
    if t.staff_support_annual_mxn > 0 {
        s.push_str("Egreso: aportaciones a la congregación y apoyos de servicio social (del padrón); el monto no se comparte.\n");
    }
    if t.external_staff_annual_mxn > 0 {
        s.push_str("Egreso: personal de empresas externas (del padrón); el monto no se comparte.\n");
    }
    let listed_sum: i64 = listed.iter().filter_map(|l| l.annual_mxn).sum();
    if listed_sum > 0 {
        s.push_str(&format!(
            "Suma de los egresos escritos a mano: {} pesos al año{}.\n",
            mxn(listed_sum),
            if t.payroll_cost_annual_mxn > 0 { " (sin la nómina)" } else { "" }
        ));
    }
    if money.expenses_basis == ExpenseBasis::Unknown {
        missing.push("cuánto gasta al año");
    }
    if let Some(b) = money.balance_annual_mxn {
        let against = if money.expenses_basis == ExpenseBasis::List { "la lista de egresos" } else { "el gasto anual aproximado" };
        let verdict = match b {
            b if b < 0 => "NO alcanzan a cubrir los egresos: hay déficit",
            0 => "alcanzan justo para cubrir los egresos",
            _ => "alcanzan a cubrir los egresos y sobra algo",
        };
        s.push_str(&format!("Balance del año, calculado con lo capturado y {against}: los ingresos {verdict}.\n"));
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

    // staff: from the staff module, as positions and counts; never a person, a pay or a date (ADR-027)
    render_staff(&mut s, staff, &mut missing);

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
                StaffGroupInput { role: "Cuidadora".into(), count: 4, paid: true, monthly_salary_mxn: Some(7_777), shift: Some("noche".into()), contract: Some(ContractKind::Permanent), start_year: Some(2019), ..Default::default() },
                StaffGroupInput { role: "Voluntaria".into(), count: 3, paid: false, ..Default::default() },
            ],
            facilities: vec![
                FacilityInput { kind: "Baño".into(), count: 3, condition: Some(Condition::Poor), accessible: Some(false), notes: Some("Piso resbaloso, sin barras.".into()) },
                FacilityInput { kind: "Cocina".into(), count: 1, condition: Some(Condition::Good), accessible: None, notes: None },
            ],
            income: vec![
                IncomeSourceInput { label: "Padrinos".into(), kind: IncomeKind::RecurringDonor, amount_mxn: Some(10_000), period: Period::Monthly },
                IncomeSourceInput { label: "Donativos".into(), kind: IncomeKind::OccasionalDonation, amount_mxn: Some(680_000), period: Period::Annual },
                // the roster has the real fees: this one would count them twice and is left out
                IncomeSourceInput { label: "Cuotas aprox.".into(), kind: IncomeKind::FeeEstimate, amount_mxn: Some(450_000), period: Period::Annual },
            ],
            expenses: vec![
                ExpenseItemInput { label: "Alimentos".into(), amount_mxn: Some(15_000), period: Period::Monthly },
                ExpenseItemInput { label: "Luz y agua".into(), amount_mxn: Some(36_000), period: Period::Annual },
            ],
        }
    }

    /// The staff of `rich()` in the staff module: 4 night caregivers with pay and 3 volunteers.
    pub(crate) fn seed_rich_staff(c: &mut Connection) {
        use crate::hr::domain::person::PersonData;
        use crate::hr::domain::position::PositionInput;
        use crate::hr::storage as hr;
        let carer = hr::insert_position(c, &PositionInput {
            title: "Cuidadora".into(), area: Some("care".into()), duties: Some("Atiende a los residentes de noche.".into()), authorized_seats: Some(5), ..Default::default()
        }).unwrap();
        let volunteer = hr::insert_position(c, &PositionInput { title: "Voluntaria".into(), ..Default::default() }).unwrap();
        let year: i64 = hr::today(c).unwrap()[..4].parse().unwrap();
        for i in 0..4 {
            let d = PersonData {
                first_names: format!("Cuidadora Secreta {i}"), position_id: Some(carer.clone()), modality: "indefinite".into(), status: "active".into(),
                pay_amount_mxn: Some(7_777), pay_period: Some("monthly".into()), shift: Some("night".into()), schedule: Some("full_time".into()),
                start_date: Some(format!("{}-01-01", year - 7)), education: Some("high_school".into()), curp: Some("HEGG560427MVZRRL04".into()),
                ..Default::default()
            };
            hr::save_person(c, None, &d, false).unwrap();
        }
        for i in 0..3 {
            let d = PersonData { first_names: format!("Voluntaria Oculta {i}"), position_id: Some(volunteer.clone()), modality: "volunteer".into(), status: "active".into(), ..Default::default() };
            hr::save_person(c, None, &d, false).unwrap();
        }
    }

    fn saved(input: &ProfileInput, confirm: bool) -> (tempfile::TempDir, Connection) {
        let (d, mut c) = db();
        seed_rich_staff(&mut c);
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
        "Gasto anual aproximado (cifra a ojo de la persona, todo incluido): $1,800,000 pesos.",
        "Ingreso — cuotas de los beneficiarios (del padrón): las pagan 5 personas; el monto no se comparte.",
        "Ingreso — donante fijo: Padrinos — $10,000 pesos al mes ($120,000 al año).",
        "Ingreso — donativo ocasional: Donativos — $680,000 pesos al año.",
        "Suma de los ingresos escritos a mano: $800,000 pesos al año (sin las cuotas del padrón).",
        "Egreso: Alimentos — $15,000 pesos al mes ($180,000 al año).",
        "Egreso: Luz y agua — $36,000 pesos al año.",
        "Egreso: nómina del personal con aguinaldo y prima vacacional (del padrón); el monto no se comparte.",
        "Suma de los egresos escritos a mano: $216,000 pesos al año (sin la nómina).",
        "Balance del año, calculado con lo capturado y la lista de egresos: los ingresos alcanzan a cubrir los egresos y sobra algo.",
        "Población: Adultos mayores — 11 personas; nivel de dependencia alta; edades de 66 a 95 años; 5 pagan cuota de estancia.",
        "Población: Hombres — 1 persona; nivel de dependencia total.",
        "Total de personas atendidas: 12.",
        "Personal: Cuidadora (cuidado) — 4 personas (4 con sueldo; jornada: 4 tiempo completo; turno: 4 nocturno). Plazas autorizadas: 5; sin cubrir: 1.",
        "Funciones del puesto Cuidadora: Atiende a los residentes de noche.",
        "Personal: Voluntaria — 3 personas (3 de voluntariado).",
        "Total del personal: 7 personas: 4 con sueldo, 3 de voluntariado.",
        "Escolaridad del personal (solo grupos de 3 o más personas): 4 con preparatoria o carrera técnica.",
        "Antigüedad en la institución (solo grupos de 3 o más personas): 4 de 5 a 9 años.",
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
        // not even added up: payroll, its benefits, its cost, and the fees of the roster; nor the exact balance
        let stored = profile_store::load_current(&c).unwrap().unwrap();
        let t = stored.input.totals(stored.as_of_year);
        let f = stored.input.finances(stored.as_of_year);
        let numbers = crate::domain::figures::digit_numbers(&ctx);
        for hidden in [t.payroll_monthly_mxn, t.payroll_annual_mxn, t.payroll_benefits_annual_mxn, t.payroll_cost_annual_mxn,
                       t.fees_monthly_mxn, t.fees_annual_mxn, f.income_annual_mxn, f.expenses_annual_mxn.unwrap(), f.balance_annual_mxn.unwrap()] {
            assert!(!numbers.contains(&hidden.abs().to_string()), "«{hidden}» leaked:\n{ctx}");
        }
        assert!(!ctx.contains("Cuotas aprox."), "a fee estimate left out of the sums is not shown either:\n{ctx}");
        // pay, the fee a person pays, contact data, RFC and the representative; names and identifiers of the staff
        for secret in ["7777", "7,777", "3333", "3,333", "AFI200101", "55 5555", "Rosa Representante", "Secreta", "Oculta", "HEGG"] {
            assert!(!ctx.contains(secret), "«{secret}» leaked:\n{ctx}");
        }
        // the age of a group of one person would be that person's age
        assert!(ctx.contains("Población: Hombres — 1 persona; nivel de dependencia total."), "{ctx}");
        assert!(!ctx.contains("93"), "{ctx}");
    }

    #[test]
    fn a_section_with_nothing_captured_says_so_instead_of_staying_silent() {
        let input = ProfileInput { institution: InstitutionInput { name: "Casa Vacía".into(), ..Default::default() }, ..Default::default() };
        let (_d, mut c) = db();
        profile_store::save(&mut c, &input).unwrap();
        let ctx = profile_context(&c).unwrap();
        assert!(ctx.contains("BORRADOR"), "{ctx}");
        assert!(ctx.contains("Institución: Casa Vacía (institución de asistencia)."), "{ctx}");
        for gap in ["a qué se dedica", "capacidad total", "cuánto gasta al año", "de dónde vienen sus ingresos", "a quiénes atiende", "el personal", "las instalaciones"] {
            assert!(ctx.contains(gap), "the gap «{gap}» is not named:\n{ctx}");
        }
        assert!(ctx.contains("no que sea cero"), "{ctx}");
        assert!(!ctx.contains("Total de personas atendidas"), "no invented zero totals:\n{ctx}");
    }

    #[test]
    fn with_only_the_approximate_expense_the_payroll_goes_inside_it_and_a_deficit_is_said_in_words() {
        let mut p = rich();
        p.expenses.clear();
        p.annual_budget_mxn = Some(900_000); // income is the roster fees plus 800,000 written: more than this
        let (_d, c) = saved(&p, true);
        let ctx = profile_context(&c).unwrap();
        assert!(ctx.contains("La nómina del personal (con aguinaldo y prima vacacional) va dentro del gasto aproximado; el monto no se comparte."), "{ctx}");
        assert!(ctx.contains("Balance del año, calculado con lo capturado y el gasto anual aproximado: los ingresos alcanzan a cubrir los egresos y sobra algo."), "{ctx}");
        assert!(!ctx.contains("Egreso:"), "{ctx}");

        let mut p = rich();
        p.expenses.clear();
        p.annual_budget_mxn = Some(5_000_000);
        let (_d, c) = saved(&p, true);
        let ctx = profile_context(&c).unwrap();
        assert!(ctx.contains("los ingresos NO alcanzan a cubrir los egresos: hay déficit."), "{ctx}");
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
        let allowed = ["25", "1800000", "10000", "120000", "680000", "800000", "15000", "180000", "36000", "216000", "11", "8", "3", "5", "66", "95", "12", "4", "1", "70", "80", "7", "9"];
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
        let ctx = render(&stored, &crate::hr::api::ai_summary(&c).unwrap());
        assert!(ctx.contains("Personal: "), "{ctx}");
        assert!(ctx.contains("Población: "), "{ctx}");
        assert!(ctx.contains("Total de personas atendidas: "), "{ctx}");
        assert!(ctx.contains("Piso resbaloso, sin barras de apoyo"), "the notes of the facilities reach the AI:\n{ctx}");
        assert!(ctx.contains("Gasto anual aproximado (cifra a ojo de la persona, todo incluido): $1,800,000 pesos."), "{ctx}");
        assert!(ctx.contains("Ingreso — cuotas de los beneficiarios (del padrón)"), "{ctx}");
        let t = stored.input.totals(stored.as_of_year);
        for hidden in [t.payroll_monthly_mxn, t.payroll_annual_mxn, t.payroll_cost_annual_mxn, t.fees_monthly_mxn, t.fees_annual_mxn] {
            assert!(!crate::domain::figures::digit_numbers(&ctx).contains(&hidden.to_string()), "a sum of pay or fees ({hidden}) leaked:\n{ctx}");
        }
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
