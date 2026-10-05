//! The golden case (`fixtures/caso-dorado-bano.md`) as data: what the person writes in the guided conversation
//! and the criteria of the summary, shared by the dry run and the live run so both judge by exactly the same.

use crate::scanner::{RegexScanner, SensitiveScanner};
use serde_json::Value;

/// What the person writes to the opening question (idea, obstacle, future benefit), as in the fixture.
pub const OPENING_ANSWER: &str = "Queremos remodelar el baño de la planta baja con regadera a ras de piso y barras de apoyo, porque hoy el piso resbala y no hemos tenido dinero para hacerlo; así los 14 abuelitos con movilidad reducida podrían bañarse con seguridad y habría menos caídas.";

/// What the person answers to each «why», in order. The conversation ends when the root cause is proposed and
/// confirmed, which may be before the last one is needed.
pub const WHY_ANSWERS: [&str; 5] = [
    "Porque el edificio es de los años 70 y no se pensó para personas mayores; no hay barras ni regadera a ras de piso. Este año hubo 3 caídas, una con fractura de cadera, y 6 de ellos van en silla de ruedas.",
    "No se ha hecho nada más por falta de dinero; solo se pusieron tapetes antiderrapantes, pero se mueven.",
    "Porque no tenemos un plan ni un fondo para adecuar la casa; solo se arregla cuando algo falla. Bañar a una persona en silla de ruedas requiere a 2 personas y tarda casi 40 minutos, y en la noche solo hay 1 enfermera.",
    "El mantenimiento lo cubre la institución y no hay un responsable que revise la casa.",
    "No habíamos pedido ayuda antes porque no sabíamos que existían convocatorias para esto.",
];

pub struct Criterion {
    pub name: &'static str,
    pub ok: bool,
    /// A hard requirement fails the run; the rest are judged by a person reading the summary.
    pub hard: bool,
}

pub fn collect_strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| collect_strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| collect_strings(x, out)),
        Value::Number(n) => out.push(n.to_string()),
        _ => {}
    }
}

/// The criteria of the fixture, applied to a summary. `unsupported_figures` comes from the
/// code's own check of the summary against what the person said.
pub fn criteria(summary: &Value, unsupported_figures: &[String]) -> Vec<Criterion> {
    let mut texts = Vec::new();
    collect_strings(summary, &mut texts);
    let all = texts.join(" ").to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| all.contains(w));
    let c = |name, ok, hard| Criterion { name, ok, hard };
    vec![
        c("replantea como seguridad/accesibilidad", has(&["segur", "accesib"]), false),
        c("cifras 14, 6 y 3", ["14", "6", "3"].iter().all(|n| all.contains(n)), false),
        c("carga del personal (2 personas, 40 min, 1 enfermera)", has(&["40"]) && has(&["enfermera", "personal"]), false),
        c("regadera a ras de piso", has(&["ras de piso"]), false),
        c("barras de apoyo", has(&["barra"]), false),
        c("piso antiderrapante", has(&["antiderrap"]), false),
        // "silla de ruedas" is in the person's own answer, so a bare "silla" proves nothing
        c("silla de baño/regadera", has(&["silla de baño", "silla de regadera", "silla para baño", "silla para regadera", "silla de ducha", "silla de aseo"]), false),
        c("puerta amplia", has(&["puerta"]), false),
        c("capacitación del personal", has(&["capacit"]), false),
        c("indicador: caídas", has(&["caída", "caida"]), false),
        c("indicador: tiempo por baño", has(&["tiempo", "minutos"]), false),
        c("open_questions: cotizaciones/medidas/mantenimiento", has(&["cotiza", "medida", "mantenimiento"]), false),
        c("sin cifras inventadas", unsupported_figures.is_empty(), true),
        c("sin datos personales (escáner)", RegexScanner::new(Default::default()).scan(&texts.join("\n")).is_clean(), true),
    ]
}

pub fn print(criteria: &[Criterion]) {
    println!("\nCriterios del caso dorado:");
    for c in criteria {
        println!("  [{}] {}{}", if c.ok { "x" } else { " " }, c.name, if c.hard { " (obligatorio)" } else { "" });
    }
}

pub fn hard_requirements_met(criteria: &[Criterion]) -> bool {
    criteria.iter().filter(|c| c.hard).all(|c| c.ok)
}
