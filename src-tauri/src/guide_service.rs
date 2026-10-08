//! The guide of a project in Word (ADR-018): what the conversation and the drafting concluded, ordered so the person
//! only has to copy it into the forms and documents of the donor. Cimiento does NOT fill the donor's formats.
//!
//! First what is indispensable of the call (facts and documents to hand in), then the project: its summary, the
//! proposal (the sections the person confirmed), the budget and the schedule, the indicators, the institution and
//! what is still pending. The code builds the file; the AI wrote only text that the person confirmed.

use crate::audit::{self, AuditKind};
use crate::scanner::PublicDocScanner;
use crate::conversation_service::conversation_view;
use crate::diagnosis_service::{lock, SharedDb};
use crate::documents::docx::{self, Block, TableData};
use crate::domain::budget::format_mxn;
use crate::domain::checklist::{self, Facts, Level, Report, SectionState};
use crate::domain::requirements::{CallRequirements, Line, Sourced};
use crate::domain::sections::SectionKind;
use crate::domain::stage::Stage;
use crate::drafting_service::{drafting_view, DraftingView, SectionStatus};
use crate::scanner::{RegexScanner, SensitiveScanner};
use crate::service::ServiceError;
use crate::storage::projects::{self as projects, ProjectRow, StoredSummary};
use crate::storage::profile as profile_store;
use rusqlite::Connection;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Everything the guide and the review are made of.
pub struct GuideData {
    pub project: ProjectRow,
    pub drafting: DraftingView,
    pub summary: Option<StoredSummary>,
    pub root: Option<String>,
    pub institution: Option<crate::domain::profile::ProfileInput>,
    /// The facilities, from their module (ADR-030).
    pub facilities: Vec<crate::modules::facilities::domain::aggregate::SiteSummary>,
    pub call_name: Option<String>,
    pub funder: Option<String>,
    pub year: Option<i64>,
    pub fit: Option<String>,
    pub today: String,
}

pub fn gather(conn: &Connection, project_id: &str) -> Result<GuideData, ServiceError> {
    let drafting = drafting_view(conn, project_id)?;
    let project = drafting.project.clone();
    let reading = match &project.call_reading_id {
        Some(id) => crate::storage::calls::get(conn, id)?,
        None => None,
    };
    Ok(GuideData {
        summary: projects::get_summary(conn, project_id)?,
        root: projects::get_root(conn, project_id)?.map(|r| r.text),
        institution: profile_store::load_current(conn)?.map(|p| p.input),
        facilities: crate::modules::facilities::api::summaries(conn)?,
        call_name: reading.as_ref().map(|r| r.name.clone()),
        funder: reading.as_ref().and_then(|r| r.funder.clone()),
        year: reading.as_ref().and_then(|r| r.year),
        fit: conversation_view(conn, project_id)?.fit.map(|f| f.fit),
        today: conn.query_row("SELECT date('now')", [], |r| r.get(0))?,
        drafting,
        project,
    })
}

const MONTHS: [&str; 12] = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

/// «3 de octubre de 2026» from `2026-10-03`; anything else is returned as it came.
pub fn spanish_date(iso: &str) -> String {
    let parts: Vec<&str> = iso.split('-').collect();
    match parts.as_slice() {
        [y, m, d] => match (m.parse::<usize>(), d.parse::<u32>()) {
            (Ok(m), Ok(d)) if (1..=12).contains(&m) && (1..=31).contains(&d) => format!("{d} de {} de {y}", MONTHS[m - 1]),
            _ => iso.to_string(),
        },
        _ => iso.to_string(),
    }
}

fn kind_words(kind: &str) -> &'static str {
    match kind {
        "elderly_home" => "Asilo o casa de adultos mayores",
        "children_home" => "Casa hogar para niñas, niños o adolescentes",
        _ => "Institución de asistencia",
    }
}

fn source_note(page: Option<u64>, file: &Option<String>) -> String {
    match (file, page) {
        (Some(f), Some(p)) => format!("{f}, página {p}"),
        (Some(f), None) => f.clone(),
        (None, Some(p)) => format!("página {p}"),
        (None, None) => String::new(),
    }
}

fn line_item(l: &Line) -> String {
    match &l.applies_to {
        Some(who) => format!("{} (solo si: {who})", l.text),
        None => l.text.clone(),
    }
}

fn facts_table(r: &CallRequirements, data: &GuideData) -> TableData {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut add = |what: &str, value: String, from: String| {
        if !value.trim().is_empty() {
            rows.push(vec![what.to_string(), value, from]);
        }
    };
    add("Convocatoria", data.call_name.clone().unwrap_or_default(), String::new());
    add("Quién la convoca", data.funder.clone().unwrap_or_default(), String::new());
    add("Año", data.year.map(|y| y.to_string()).unwrap_or_default(), String::new());
    let money = |s: &Option<Sourced<f64>>| s.as_ref().map(|v| (format_mxn(v.value), source_note(v.page, &v.file)));
    let percent = |s: &Option<Sourced<f64>>| s.as_ref().map(|v| (format!("{} %", v.value), source_note(v.page, &v.file)));
    for (what, v) in [("Monto máximo por proyecto", money(&r.max_amount_mxn)), ("Monto mínimo por proyecto", money(&r.min_amount_mxn)), ("Lo que debe aportar la organización", percent(&r.cofunding_percent)), ("Tope de gastos administrativos", percent(&r.admin_cap_percent))] {
        if let Some((value, from)) = v {
            add(what, value, from);
        }
    }
    if let Some(d) = &r.max_duration_months {
        add("Duración máxima del proyecto", format!("{} meses", d.value), source_note(d.page, &d.file));
    }
    if let Some(d) = &r.closing_date {
        add("Fecha de cierre", spanish_date(&d.value), source_note(d.page, &d.file));
    }
    if !r.how_to_deliver.is_empty() {
        add("Cómo y dónde se entrega", r.how_to_deliver.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join("\n\n"), r.how_to_deliver.first().map(|l| source_note(l.page, &l.file)).unwrap_or_default());
    }
    if !r.contact.is_empty() {
        add("Para dudas", r.contact.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join("\n\n"), r.contact.first().map(|l| source_note(l.page, &l.file)).unwrap_or_default());
    }
    TableData { header: vec!["Dato".into(), "Lo que dice la convocatoria".into(), "Dónde lo dice".into()], rows }
}

/// The guide as blocks. `pending` are the notes of the review that are not errors (warnings), put at the end.
pub fn guide_blocks(data: &GuideData, pending: &[String]) -> Vec<Block> {
    let d = &data.drafting;
    let r = &d.requirements;
    let mut b: Vec<Block> = Vec::new();

    b.push(Block::Title(format!("Guía del proyecto: {}", data.project.title)));
    let institution = data.institution.as_ref().map(|i| i.institution.name.clone()).unwrap_or_default();
    b.push(Block::Paragraph(format!("Institución: {institution}\nConvocatoria: {}\nPreparada el {}", data.call_name.clone().unwrap_or_default(), spanish_date(&data.today))));
    b.push(Block::Note("Esta guía reúne lo que se concluyó en el proyecto para que usted lo pase a los formularios y documentos que pide la convocatoria. No sustituye los formatos del donante.".into()));

    // 1. what is indispensable of the call
    b.push(Block::Heading(1, "1. Datos de la convocatoria".into()));
    b.push(Block::Table(facts_table(r, data)));

    b.push(Block::Heading(1, "2. Qué hay que entregar".into()));
    let mut any = false;
    for (title, lines, conditional) in [("Documentos obligatorios", &r.required_docs, false), ("Documentos que piden solo en ciertos casos", &r.conditional_docs, true), ("Documentos opcionales", &r.optional_docs, false), ("Formatos que hay que usar", &r.formats, false)] {
        if lines.is_empty() {
            continue;
        }
        any = true;
        b.push(Block::Heading(2, title.into()));
        b.push(Block::Checklist(lines.iter().map(|l| if conditional { line_item(l) } else { l.text.clone() }).collect()));
    }
    if !any {
        b.push(Block::Paragraph("La convocatoria no menciona documentos ni formatos. Revise sus bases.".into()));
    }

    // 3. the project in short
    b.push(Block::Heading(1, "3. Resumen del proyecto".into()));
    if let Some(o) = &d.objective {
        b.push(Block::Paragraph(format!("Objetivo: {o}")));
    }
    if let Some(root) = &data.root {
        b.push(Block::Paragraph(format!("Causa de fondo que se quiere atacar: {root}")));
    }
    if let Some(s) = data.summary.as_ref().map(|s| &s.summary) {
        if let Some(p) = s["problem_statement"].as_str().filter(|t| !t.trim().is_empty()) {
            b.push(Block::Paragraph(format!("El problema: {p}")));
        }
        let affected = &s["affected"];
        let who = affected["group"].as_str().unwrap_or("");
        let count = affected["count"].as_u64().map(|c| format!(" ({c} personas)")).unwrap_or_default();
        let desc = affected["description"].as_str().unwrap_or("");
        if !who.is_empty() || !desc.is_empty() {
            b.push(Block::Paragraph(format!("A quién beneficia: {who}{count}. {desc}").trim().to_string()));
        }
    }

    // 4. the proposal
    b.push(Block::Heading(1, "4. Propuesta del proyecto".into()));
    b.push(Block::Paragraph(if d.asks_for_proposal {
        "La convocatoria pide una propuesta del proyecto en un documento aparte. Esta es la estructura que sigue lo que la convocatoria pide que incluya.".into()
    } else {
        "La convocatoria no pide una propuesta aparte. Esta información, ordenada, le sirve para llenar los formularios.".into()
    }));
    let mut wrote = false;
    for s in d.sections.iter().filter(|s| s.spec.kind == SectionKind::Text && s.status == SectionStatus::Confirmed) {
        wrote = true;
        b.push(Block::Heading(2, s.spec.title.clone()));
        b.push(Block::Paragraph(s.content.clone()));
    }
    if !wrote {
        b.push(Block::Paragraph("Todavía no hay secciones confirmadas.".into()));
    }

    // 5. budget
    b.push(Block::Heading(1, "5. Presupuesto".into()));
    if d.budget.items.is_empty() {
        b.push(Block::Paragraph("Todavía no hay partidas.".into()));
    } else {
        let paid = |f: crate::domain::budget::Funder| match f {
            crate::domain::budget::Funder::Requested => "Lo pide a la convocatoria",
            crate::domain::budget::Funder::Institution => "La institución",
            crate::domain::budget::Funder::Other => "Otra fuente",
        };
        b.push(Block::Table(TableData {
            header: ["Concepto", "Categoría", "Cantidad", "Precio unitario", "Total con IVA", "Quién lo paga"].iter().map(|s| s.to_string()).collect(),
            rows: d.budget.items.iter().map(|i| {
                let unit = i.row.unit.as_deref().map(|u| format!(" {u}")).unwrap_or_default();
                vec![i.row.description.clone(), i.row.category.clone(), format!("{}{unit}", i.row.quantity), format_mxn(i.row.unit_price_mxn), format_mxn(i.line.total), paid(i.row.funded_by).to_string()]
            }).collect(),
        }));
        let t = &d.budget.totals;
        b.push(Block::Table(TableData {
            header: vec!["Resumen".into(), "Monto".into()],
            rows: vec![
                vec!["Total del proyecto".into(), format_mxn(t.total)],
                vec!["IVA incluido en el total".into(), format_mxn(t.vat)],
                vec!["Lo que se pide a la convocatoria".into(), format_mxn(t.requested)],
                vec!["Lo que aporta la institución".into(), format_mxn(t.institution)],
                vec!["Lo que aportan otras fuentes".into(), format_mxn(t.other)],
                vec!["Contrapartida (institución y otras fuentes)".into(), format!("{} %", t.counterpart_percent)],
            ],
        }));
    }

    // 6. schedule
    b.push(Block::Heading(1, "6. Cronograma".into()));
    if d.schedule.activities.is_empty() {
        b.push(Block::Paragraph("Todavía no hay actividades.".into()));
    } else {
        b.push(Block::Table(TableData {
            header: vec!["Actividad".into(), "Del mes".into(), "Al mes".into()],
            rows: d.schedule.activities.iter().map(|a| vec![a.title.clone(), a.start_month.to_string(), a.end_month.to_string()]).collect(),
        }));
        b.push(Block::Paragraph(format!("Duración del proyecto: {} meses.", d.schedule.duration_months)));
    }

    // 7. indicators
    b.push(Block::Heading(1, "7. Indicadores y resultados".into()));
    let ours: Vec<String> = data.summary.as_ref().and_then(|s| s.summary["suggested_indicators"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())).unwrap_or_default();
    if !ours.is_empty() {
        b.push(Block::Heading(2, "Cómo sabremos que funcionó".into()));
        b.push(Block::Bullets(ours));
    }
    if !r.indicators.is_empty() {
        b.push(Block::Heading(2, "Indicadores que pide la convocatoria".into()));
        b.push(Block::Bullets(r.indicators.iter().map(|l| l.text.clone()).collect()));
    }

    // 8. the institution
    b.push(Block::Heading(1, "8. Datos de la institución".into()));
    if let Some(i) = &data.institution {
        let inst = &i.institution;
        let mut lines = vec![format!("Nombre: {}", inst.name), format!("Tipo: {}", kind_words(inst.kind.as_db()))];
        for (label, v) in [("A qué se dedica", &inst.mission), ("RFC", &inst.legal_rfc), ("Teléfono", &inst.contact_phone), ("Correo", &inst.contact_email), ("Representante legal", &inst.legal_rep_name)] {
            if let Some(v) = v.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
                lines.push(format!("{label}: {v}"));
            }
        }
        if let Some(c) = i.capacity_total {
            lines.push(format!("Capacidad total: {c} personas"));
        }
        b.push(Block::Paragraph(lines.join("\n")));
        if !i.population.is_empty() {
            b.push(Block::Heading(2, "Población que se atiende".into()));
            b.push(Block::Bullets(i.population_by_label().into_iter().map(|(label, n)| format!("{label}: {n} personas")).collect()));
        }
    }
    let facility_lines: Vec<String> = data
        .facilities
        .iter()
        .flat_map(|s| {
            crate::domain::facility_text::site_lines(&s.site)
                .into_iter()
                .chain(s.spaces.iter().map(crate::domain::facility_text::space_line))
                .chain(s.equipment.iter().map(crate::domain::facility_text::equipment_line))
        })
        .collect();
    if !facility_lines.is_empty() {
        b.push(Block::Heading(2, "Instalaciones".into()));
        b.push(Block::Bullets(facility_lines));
    }

    // 9. what is pending
    let mut notes: Vec<String> = Vec::new();
    for s in d.sections.iter().filter(|s| s.status == SectionStatus::Confirmed) {
        notes.extend(s.open_points.iter().map(|p| format!("{}: {p}", s.spec.title)));
    }
    if let Some(a) = data.summary.as_ref().and_then(|s| s.summary["open_questions"].as_array()) {
        notes.extend(a.iter().filter_map(|x| x.as_str().map(String::from)));
    }
    notes.extend(pending.iter().cloned());
    b.push(Block::Heading(1, "9. Pendientes por revisar".into()));
    if notes.is_empty() {
        b.push(Block::Paragraph("No quedaron pendientes.".into()));
    } else {
        b.push(Block::Checklist(notes));
    }
    b
}

/// All the text of the blocks, for the scanner.
pub fn blocks_text(blocks: &[Block]) -> String {
    blocks.iter().map(Block::text).collect::<Vec<_>>().join("\n")
}

/// How many data of a person the text carries. A call is public and so is the institution's own contact: only what
/// identifies a person (CURP, personal RFC, voter key, bank accounts...) counts, as in the reading of a call.
pub fn findings_in(conn: &Connection, text: &str) -> Result<usize, ServiceError> {
    let scanner = PublicDocScanner::new(RegexScanner::new(profile_store::scanner_config(conn)?));
    Ok(scanner.scan(text).findings.len())
}

/// The review of the project as the person sees it: what the code checked, with the state of the sections.
pub fn review_report(conn: &Connection, data: &GuideData) -> Result<Report, ServiceError> {
    let d = &data.drafting;
    let sections: Vec<SectionState> = d
        .sections
        .iter()
        .filter(|s| s.spec.kind == SectionKind::Text)
        .map(|s| SectionState { key: s.spec.key.clone(), title: s.spec.title.clone(), required: s.spec.required, confirmed: s.status == SectionStatus::Confirmed })
        .collect();
    let findings = findings_in(conn, &blocks_text(&guide_blocks(data, &[])))?;
    let months: Vec<(u32, u32)> = d.schedule.activities.iter().map(|a| (a.start_month, a.end_month)).collect();
    Ok(checklist::run(&Facts {
        requirements: &d.requirements,
        totals: &d.budget.totals,
        budget_lines: d.budget.items.len(),
        budget_confirmed: d.budget.confirmed,
        schedule_activities: d.schedule.activities.len(),
        duration_months: crate::domain::schedule::duration_months(&months),
        schedule_confirmed: d.schedule.confirmed,
        sections: &sections,
        scanner_findings: findings,
        fit: data.fit.as_deref(),
        today: &data.today,
    }))
}

// ------------------------------------------------------------------ the file

#[derive(Debug, Clone, Serialize)]
pub struct Exported {
    pub file_name: String,
    pub path: String,
}

/// A name that is safe in any folder of Windows.
fn safe_name(s: &str) -> String {
    let cleaned: String = s.chars().map(|c| if c.is_control() || "\\/:*?\"<>|".contains(c) { ' ' } else { c }).collect();
    let t = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let t: String = t.chars().take(80).collect();
    let t = t.trim_matches([' ', '.']).to_string();
    if t.is_empty() { "proyecto".into() } else { t }
}

/// The first name that does not exist yet: «guía.docx», «guía (2).docx»…
fn free_path(dir: &Path, stem: &str) -> PathBuf {
    let mut p = dir.join(format!("{stem}.docx"));
    let mut n = 2;
    while p.exists() {
        p = dir.join(format!("{stem} ({n}).docx"));
        n += 1;
    }
    p
}

/// Writes the guide in `dir`. It only exists when the project is `READY` and the review is clean, and nothing that
/// identifies a person goes into it. The log keeps only counts.
pub fn export_guide(db: &SharedDb, project_id: &str, dir: &Path) -> Result<Exported, ServiceError> {
    let conn = lock(db)?;
    let data = gather(&conn, project_id)?;
    if data.project.stage != Stage::Ready {
        return Err(ServiceError::WrongStage);
    }
    let report = review_report(&conn, &data)?;
    if !report.clean() {
        return Err(ServiceError::Stage(crate::domain::stage::StageError::NotReady(crate::domain::stage::Missing::ChecklistHasErrors)));
    }
    let pending: Vec<String> = report.checks.iter().filter(|c| c.level == Level::Warn).map(|c| c.text.clone()).collect();
    let blocks = guide_blocks(&data, &pending);
    if findings_in(&conn, &blocks_text(&blocks))? > 0 {
        return Err(ServiceError::GuideHasPersonalData);
    }
    let institution = data.institution.as_ref().map(|i| i.institution.name.clone()).unwrap_or_else(|| "Cimiento".into());
    let created: String = conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')", [], |r| r.get(0))?;
    let bytes = docx::build(&blocks, &format!("Guía del proyecto: {}", data.project.title), &institution, &created).map_err(|e| ServiceError::Internal(e.to_string()))?;
    std::fs::create_dir_all(dir).map_err(|e| ServiceError::Internal(e.to_string()))?;
    let path = free_path(dir, &format!("{} - guía {}", safe_name(&data.project.title), data.today));
    std::fs::write(&path, bytes).map_err(|e| ServiceError::Internal(e.to_string()))?;
    let tables = blocks.iter().filter(|b| matches!(b, Block::Table(_))).count();
    audit::record(&conn, AuditKind::ExportCreated, Some("project"), Some(project_id), serde_json::json!({ "format": "docx", "blocks": blocks.len(), "tables": tables }))?;
    Ok(Exported { file_name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), path: path.to_string_lossy().into_owned() })
}

#[cfg(test)]
#[path = "guide_service_tests.rs"]
mod tests;
