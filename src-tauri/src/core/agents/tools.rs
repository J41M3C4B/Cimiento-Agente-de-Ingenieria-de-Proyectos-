//! The read-only tools of the core (ADR-034 §2): how far the data of the institution are filled in, and one part
//! of its sheet at a time. Each one checks the permission of whoever asks (ADR-028) and returns only what may reach
//! the AI: what is missing goes as words and field ids, never as values, and a part of the sheet obeys the rules of
//! the whole sheet (ADR-023). The tool is the frontier, not the prompt.

use crate::ai::agent::{Tool, ToolError};
use crate::core::access::domain::{Permission, Role};
use crate::core::ai_sheet::{self, SECTIONS};
use crate::core::overview::{self, Place};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// What every tool of the core needs: the base and who is asking.
#[derive(Clone)]
struct Ctx {
    db: Arc<Mutex<Connection>>,
    role: Option<Role>,
}

impl Ctx {
    /// The base, if this person may see the data of the institution on their screen.
    fn open(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ToolError> {
        if !self.role.is_some_and(|r| r.can(Permission::Use)) {
            return Err(ToolError::Denied);
        }
        self.db.lock().map_err(|_| ToolError::Failed)
    }
}

/// The tools of the core for one person (by the role of their session).
pub fn toolbox(db: Arc<Mutex<Connection>>, role: Option<Role>) -> Vec<Box<dyn Tool>> {
    let ctx = Ctx { db, role };
    vec![Box::new(FillState(ctx.clone())), Box::new(SheetSection(ctx))]
}

/// What is missing, said as the first start and «Mi institución» say it.
fn gap_words(code: &str) -> &'static str {
    match code {
        "name" => "El nombre",
        "mission" => "A qué se dedica",
        "populations" => "A quién atienden",
        "modalities" => "Cómo los atienden",
        "state" => "El estado",
        "municipality" => "El municipio",
        "contact" => "Un teléfono o un correo",
        "legal_form" => "La figura jurídica",
        "founded_year" => "El año de fundación",
        "authorized_donee" => "Si es donataria autorizada",
        "cluni" => "Si tiene CLUNI",
        "capacity_total" => "La capacidad",
        "served" => "Cuántas personas atienden",
        "staff" => "Cuántas personas trabajan y cuántas son voluntarias",
        "expenses" => "Cuánto gasta al año",
        "income" => "Al menos una fuente de ingreso con monto",
        "floors" => "Cuántos pisos tiene",
        "tenure" => "De quién es el inmueble",
        "staff_records" => "Registrar al personal",
        "served_records" => "Registrar a las personas que atienden",
        "spaces" => "Registrar los espacios del inmueble",
        _ => "Otro dato",
    }
}

fn place_words(p: Place) -> &'static str {
    match p {
        Place::Institution => "Mi institución, ventana «Su institución»",
        Place::Contact => "Mi institución, ventana «Ubicación y contacto»",
        Place::Legal => "Mi institución, ventana «Datos legales»",
        Place::Capacity => "Mi institución, ventana de capacidad y personas",
        Place::Finance => "el módulo de Finanzas",
        Place::Facilities => "el módulo de Instalaciones",
        Place::Staff => "el módulo de Personal",
        Place::People => "el módulo de Beneficiarios",
    }
}

/// How far the data are and what is missing, with where it is filled in.
struct FillState(Ctx);

impl Tool for FillState {
    fn name(&self) -> &'static str {
        "fill_state"
    }
    fn description(&self) -> &'static str {
        "Dice qué tan completos están los datos de la institución (porcentaje), qué falta y dónde se llena cada cosa, y qué campos de cada formulario siguen vacíos. No devuelve ningún valor capturado. `form` es el id de un formulario para ver solo ese, o null para todo."
    }
    fn args_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "form": { "anyOf": [{ "type": "string" }, { "type": "null" }] } },
            "required": ["form"],
            "additionalProperties": false
        })
    }
    fn run(&self, args: &Value) -> Result<Value, ToolError> {
        let conn = self.0.open()?;
        let wanted = args["form"].as_str();
        let forms = crate::core::institution::forms::FORMS;
        if let Some(id) = wanted {
            if !forms.iter().any(|f| f.id == id) {
                return Err(ToolError::BadArgs(format!("unknown form {id}")));
            }
        }
        let mut per_form = Vec::new();
        for f in forms.iter().filter(|f| wanted.is_none_or(|w| w == f.id)) {
            let view = crate::core::profile::forms::get(&conn, f.id).map_err(|_| ToolError::Failed)?;
            per_form.push(json!({ "form": f.id, "missing_fields": view.missing }));
        }
        if wanted.is_some() {
            return Ok(json!({ "forms": per_form }));
        }
        let o = overview::institution_overview(&conn).map_err(|_| ToolError::Failed)?;
        let missing: Vec<Value> = o.completion.gaps.iter().map(|g| json!({ "what": gap_words(g.code), "where": place_words(g.place) })).collect();
        Ok(json!({ "percent": o.completion.percent, "missing": missing, "forms": per_form }))
    }
}

/// One part of the sheet of the institution.
struct SheetSection(Ctx);

impl Tool for SheetSection {
    fn name(&self) -> &'static str {
        "sheet_section"
    }
    fn description(&self) -> &'static str {
        "Devuelve una parte de la ficha de la institución en palabras, ya sumada por el programa y sin datos de ninguna persona: `institution` (quién es, a quién y cómo atiende, figura jurídica, capacidad), `money` (ingresos, egresos y si alcanzan), `people` (personas atendidas en grupos), `staff` (puestos y conteos del personal) o `facilities` (inmueble, espacios y equipo)."
    }
    fn args_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "section": { "type": "string", "enum": SECTIONS } },
            "required": ["section"],
            "additionalProperties": false
        })
    }
    fn run(&self, args: &Value) -> Result<Value, ToolError> {
        let conn = self.0.open()?;
        let section = args["section"].as_str().unwrap_or_default();
        match ai_sheet::section_context(&conn, section).map_err(|_| ToolError::Failed)? {
            Some(text) => Ok(json!({ "section": section, "text": text })),
            None => Err(ToolError::BadArgs(format!("unknown section {section}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent::{self, Agent, Errand};
    use crate::ai::mock::MockProvider;
    use crate::ai::pipeline::SqliteLedger;
    use crate::ai::{AiResponse, ModelTier, Usage};
    use crate::core::ai_sheet::tests::{rich, rich_money, seed_rich_facilities, seed_rich_people, seed_rich_staff};
    use crate::scanner::{RegexScanner, ScannerConfig};

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    /// A base with the rich institution of the sheet tests, confirmed.
    fn rich_db() -> (tempfile::TempDir, Arc<Mutex<Connection>>) {
        let dir = tempfile::tempdir().unwrap();
        let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        seed_rich_staff(&mut c);
        seed_rich_people(&mut c);
        seed_rich_facilities(&c);
        crate::core::profile::storage::save(&mut c, &rich()).unwrap();
        crate::modules::finance::storage::save(&mut c, &rich_money()).unwrap();
        crate::core::profile::storage::confirm(&mut c).unwrap();
        (dir, Arc::new(Mutex::new(c)))
    }

    fn tool<'a>(tools: &'a [Box<dyn Tool>], name: &str) -> &'a dyn Tool {
        tools.iter().find(|t| t.name() == name).unwrap().as_ref()
    }

    #[test]
    fn every_tool_has_arguments_that_work_as_structured_output() {
        let (_d, db) = rich_db();
        for t in toolbox(db, Some(Role::Manager)) {
            let s = t.args_schema();
            assert_eq!(s["additionalProperties"], false, "{}", t.name());
            let props = s["properties"].as_object().unwrap();
            assert!(!props.is_empty(), "{}: at least one property", t.name());
            let required: Vec<&str> = s["required"].as_array().unwrap().iter().filter_map(Value::as_str).collect();
            assert!(props.keys().all(|k| required.contains(&k.as_str())), "{}: every property required", t.name());
            jsonschema::validator_for(&s).unwrap();
        }
    }

    #[test]
    fn the_fill_state_says_what_is_missing_and_where_without_any_value() {
        let (_d, db) = rich_db();
        let tools = toolbox(db, Some(Role::Manager));
        let all = tool(&tools, "fill_state").run(&json!({ "form": null })).unwrap();
        assert_eq!((all["percent"].as_u64(), all["missing"].as_array().map(Vec::len)), (Some(100), Some(0)), "{all}");
        assert!(!all.to_string().contains("Asilo Ficticio"), "no values: {all}");

        // an institution with nothing yet: what is missing, in words, with where it is filled in
        let dir = tempfile::tempdir().unwrap();
        let empty = Arc::new(Mutex::new(crate::storage::open_encrypted(&dir.path().join("e.db"), KEY).unwrap()));
        let tools_empty = toolbox(empty, Some(Role::Manager));
        let none = tool(&tools_empty, "fill_state").run(&json!({ "form": null })).unwrap();
        assert!(none["percent"].as_u64().unwrap() < 100);
        assert_eq!(none["missing"][0], json!({ "what": "El nombre", "where": "Mi institución, ventana «Su institución»" }));
        assert!(none["missing"].as_array().unwrap().iter().all(|m| m["what"] != "Otro dato"), "every gap has its words: {none}");
        assert_eq!(none["forms"][0]["form"], "institution.identity");
        assert!(none["forms"][0]["missing_fields"].as_array().unwrap().contains(&json!("institution.mission")));

        let one = tool(&tools, "fill_state").run(&json!({ "form": "institution.identity" })).unwrap();
        assert!(one.get("percent").is_none() && one["forms"].as_array().unwrap().len() == 1);
        assert!(matches!(tool(&tools, "fill_state").run(&json!({ "form": "nope" })), Err(ToolError::BadArgs(_))));
    }

    #[test]
    fn a_part_of_the_sheet_never_carries_what_must_not_reach_the_ai() {
        let (_d, db) = rich_db();
        let tools = toolbox(db, Some(Role::Manager));
        for section in SECTIONS {
            let out = tool(&tools, "sheet_section").run(&json!({ "section": section })).unwrap();
            let text = out["text"].as_str().unwrap();
            assert!(text.len() > 60, "{section}: {text}");
            for secret in ["AFI200101AB1", "5555 0101", "Rosa Representante", "7,777", "7777", "3,333", "3333", "Secreta", "Oculta", "Reservada", "Discreto", "HEGG560427"] {
                assert!(!text.contains(secret), "{section} leaks «{secret}»:\n{text}");
            }
        }
        assert!(matches!(tool(&tools, "sheet_section").run(&json!({ "section": "salaries" })), Err(ToolError::BadArgs(_))));
    }

    #[test]
    fn without_a_session_that_may_use_the_app_nothing_is_read() {
        let (_d, db) = rich_db();
        for t in toolbox(db, None) {
            let args = if t.name() == "sheet_section" { json!({ "section": "money" }) } else { json!({ "form": null }) };
            assert_eq!(t.run(&args), Err(ToolError::Denied), "{}", t.name());
        }
    }

    fn errand_agent() -> Agent {
        Agent {
            task: "agent.dry",
            name: "dry",
            tier: ModelTier::Light,
            trade: "Usted ayuda a una directiva a saber qué le falta por llenar y cómo va la institución. Responde con usted, en dos o tres frases.",
            tools: &["fill_state", "sheet_section"],
            max_tool_calls: 3,
            max_tokens: 40_000,
            max_output_tokens: 600,
            answer: json!({ "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"], "additionalProperties": false }),
            knows_institution: true,
        }
    }

    /// The real run of IA1, opt-in and capped (3 tools, so at most 4 steps): the same errand with the provider and
    /// key saved in the app. It prints the answer, the steps and what they cost.
    ///   cargo test agent_tools_live -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn agent_tools_live() {
        let (_d, db) = rich_db();
        let (provider, light) = {
            let c = db.lock().unwrap();
            let cfg = crate::ai::settings::load(&c).unwrap();
            let key = crate::storage::get_api_key(cfg.provider.as_str()).unwrap().expect("save the key in the app first");
            (crate::ai::build_provider(&cfg, key), cfg.model_for(ModelTier::Light))
        };
        println!("Proveedor: {}  modelo ligero: {light}", provider.name());
        let ledger = SqliteLedger(db.clone());
        let boxed = toolbox(db.clone(), Some(Role::Manager));
        let tools: Vec<&dyn Tool> = boxed.iter().map(|t| t.as_ref()).collect();
        let scanner = RegexScanner::new(ScannerConfig::default());
        let started = std::time::Instant::now();
        let r = agent::run(provider.as_ref(), &scanner, &ledger, &errand_agent(), &tools, Errand { context: vec![], request: "¿Qué me falta por llenar y nos alcanza el dinero?".into() }).await;
        let secs = started.elapsed().as_secs_f64();
        let c = db.lock().unwrap();
        let (calls, input, output, cost): (i64, i64, i64, f64) = c
            .query_row("SELECT count(*), coalesce(sum(input_tokens),0), coalesce(sum(output_tokens),0), coalesce(sum(estimated_cost_mxn),0) FROM ai_usage", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap();
        let audit: String = c.query_row("SELECT details_json FROM audit_log WHERE event='ai.run'", [], |r| r.get(0)).unwrap();
        println!("Resultado: {r:#?}\nBitácora: {audit}\nLlamadas: {calls}  entrada: {input}  salida: {output}  costo: ${cost:.4} MXN  tiempo: {secs:.1} s");
        assert!(r.is_ok(), "the errand did not end in an answer");
    }

    /// The dry run of IA1: an agent asks for the two tools of the core, gets real data of the base and answers.
    #[tokio::test]
    async fn an_agent_reads_the_core_with_its_tools_and_answers() {
        let (_d, db) = rich_db();
        let ledger = SqliteLedger(db.clone());
        let boxed = toolbox(db.clone(), Some(Role::Manager));
        let tools: Vec<&dyn Tool> = boxed.iter().map(|t| t.as_ref()).collect();
        let reply = |v: Value| Ok(AiResponse { value: v, model: "mock".into(), usage: Usage { input_tokens: 800, output_tokens: 60, ..Default::default() } });
        let p = MockProvider::new(vec![
            reply(json!({ "tool": "fill_state", "args": { "form": null }, "answer": null })),
            reply(json!({ "tool": "sheet_section", "args": { "section": "money" }, "answer": null })),
            reply(json!({ "tool": null, "args": null, "answer": { "text": "Sus ingresos alcanzan; le falta completar unos datos." } })),
        ]);
        let a = errand_agent();
        let scanner = RegexScanner::new(ScannerConfig::default());
        let run = agent::run(&p, &scanner, &ledger, &a, &tools, Errand { context: vec![], request: "¿Qué me falta y cómo vamos de dinero?".into() }).await.unwrap();
        assert_eq!((run.model_calls, run.tool_calls), (3, 2));
        let reqs = p.requests();
        assert!(reqs[1].user.contains("\"percent\":"), "{}", reqs[1].user);
        assert!(reqs[2].user.contains("Balance del año"), "{}", reqs[2].user);
        assert!(reqs[0].system.contains("única fuente de hechos sobre la institución"), "it gets the rules about the institution");
        let d: String = db.lock().unwrap().query_row("SELECT details_json FROM audit_log WHERE event='ai.run'", [], |r| r.get(0)).unwrap();
        assert!(d.contains("\"fill_state\":1") && d.contains("\"sheet_section\":1") && !d.contains("Balance"), "{d}");
    }
}
