//! The loop of an agent (ADR-034 §1-2). An agent has a trade, a tier, a closed list of tools, a cap of tool requests
//! and of text per errand, and the schema of its answer; the code defines it, never the AI. In each step the model
//! answers JSON (`{"tool", "args", "answer"}`) through the usual pipeline (scanner, schema, pace, `ai_usage`): it
//! asks for a tool or it answers. Rust checks that the agent may use the tool and that the arguments fit, runs it and
//! hands the result back in the next step. The tools only read; what the AI wants changed is a proposal the person
//! accepts. Nothing here knows the institution: the core registers its tools by their `api`.

// the assistant (IA3) is its first user in the app; until then the tools of the core and the tests use it
#![cfg_attr(not(test), allow(dead_code))]

use super::pipeline::{self, AiCall, Custom, Ledger, RunRecord};
use super::{prompts, AiError, AiProvider, AiTask, ModelTier};
use crate::scanner::SensitiveScanner;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// What went wrong with a tool. The model is told in words and goes on; nothing of it is an error of the errand.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ToolError {
    /// The person who asks could not see this on their screen (ADR-028).
    #[error("the person may not see this")]
    Denied,
    #[error("bad arguments: {0}")]
    BadArgs(String),
    #[error("the tool failed")]
    Failed,
}

/// A read-only tool an agent may ask for. Whoever registers it decides what it returns, and that is the frontier:
/// only what may reach the AI (aggregates, groups of three or more, never contact, names, RFC, accounts or pay).
pub trait Tool: Send + Sync {
    /// Stable name, also what the model writes in `tool` (`fill_state`).
    fn name(&self) -> &'static str;
    /// What it gives, in Spanish, for the prompt (fixed text, so the prompt stays cacheable).
    fn description(&self) -> &'static str;
    /// The JSON Schema of its arguments: an object with at least one property, every property required (null when
    /// it does not apply) and no others, so it works as structured output in every provider.
    fn args_schema(&self) -> Value;
    fn run(&self, args: &Value) -> Result<Value, ToolError>;
}

/// An agent: who it is and how far it may go. Defined by the code.
pub struct Agent {
    /// Short name, without spaces (`assistant`); the task in `ai_usage` is `agent.<name>`.
    pub task: &'static str,
    pub name: &'static str,
    pub tier: ModelTier,
    /// Its trade, in Spanish: who it is, what it does and how it answers.
    pub trade: &'static str,
    /// The tools it may ask for, by name. Any other is refused.
    pub tools: &'static [&'static str],
    /// The most tool requests in one errand; then it must answer with what it has.
    pub max_tool_calls: u32,
    /// The most text (input and output) one errand may spend before it is cut.
    pub max_tokens: u64,
    pub max_output_tokens: u32,
    /// The JSON Schema of its final answer (an object).
    pub answer: Value,
    /// Whether it gets the sheet of the institution, and so the rules about how to treat it.
    pub knows_institution: bool,
}

/// One errand: what the person asks and the minimum context (the sheet, the screen where they are).
pub struct Errand {
    pub context: Vec<String>,
    pub request: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentRun {
    pub answer: Value,
    pub model_calls: u32,
    pub tool_calls: u32,
    pub tools: BTreeMap<&'static str, u32>,
    pub rejected: u32,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AgentError {
    #[error(transparent)]
    Ai(#[from] AiError),
    /// It kept asking for tools after its last one.
    #[error("the agent asked for more tools than it may")]
    StepLimit,
    /// The errand spent its whole allowance of text before answering.
    #[error("the errand used up its text")]
    TokenLimit,
}

impl AgentError {
    pub fn kind(&self) -> String {
        match self {
            AgentError::Ai(e) => e.kind(),
            AgentError::StepLimit => "step_limit".into(),
            AgentError::TokenLimit => "token_limit".into(),
        }
    }
}

/// The tools of the box this agent may use, in the order of its list (an allowed name the box lacks is skipped).
fn usable<'a>(agent: &Agent, toolbox: &'a [&'a dyn Tool]) -> Vec<&'a dyn Tool> {
    agent.tools.iter().filter_map(|n| toolbox.iter().copied().find(|t| t.name() == *n)).collect()
}

/// The fixed prompt of the agent: its trade, the protocol and each of its tools with its arguments.
pub fn system(agent: &Agent, toolbox: &[&dyn Tool]) -> String {
    let tools: String = usable(agent, toolbox)
        .iter()
        .map(|t| format!("- `{}`: {} Argumentos: {}\n", t.name(), t.description().trim(), t.args_schema()))
        .collect();
    let tools = if tools.is_empty() { "(ninguna)".to_string() } else { tools };
    prompts::agent_system(agent.trade, &tools, agent.knows_institution)
}

/// The schema of one step: a tool with its arguments, or the answer. The tool is any text (an unknown one is refused
/// by the code and the model is told), the arguments one of the shapes of its tools.
pub fn step_schema(agent: &Agent, toolbox: &[&dyn Tool]) -> Value {
    let mut args: Vec<Value> = usable(agent, toolbox).iter().map(|t| t.args_schema()).collect();
    args.push(json!({ "type": "null" }));
    json!({
        "type": "object",
        "properties": {
            "tool": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
            "args": { "anyOf": args },
            "answer": { "anyOf": [agent.answer, { "type": "null" }] }
        },
        "required": ["tool", "args", "answer"],
        "additionalProperties": false
    })
}

/// What happened with one request of a tool, as the next step tells it.
struct Done {
    tool: String,
    args: Value,
    outcome: Result<Value, String>,
}

fn step_text(errand: &Errand, done: &[Done], left: u32) -> String {
    let mut s = format!("Encargo:\n{}\n", errand.request.trim());
    if !done.is_empty() {
        s.push_str("\nLo que ya pidió:\n");
        for (i, d) in done.iter().enumerate() {
            match &d.outcome {
                Ok(v) => s.push_str(&format!("{}. `{}` con {} devolvió: {}\n", i + 1, d.tool, d.args, v)),
                Err(why) => s.push_str(&format!("{}. `{}` con {}: {}\n", i + 1, d.tool, d.args, why)),
            }
        }
    }
    s.push_str(&if left == 0 {
        "\nYa no puede pedir herramientas: responda ahora en `answer` con lo que tiene.".to_string()
    } else {
        format!("\nLe quedan {left} pedidos de herramientas. Pida una o responda en `answer`.")
    });
    s
}

fn validate_args(schema: &Value, args: &Value) -> Result<(), String> {
    let validator = jsonschema::validator_for(schema).map_err(|e| format!("bad schema: {e}"))?;
    validator.validate(args).map_err(|e| e.to_string())
}

/// Runs one errand to its answer. Every step goes through the pipeline; the end, whatever it is, leaves `ai.run`.
pub async fn run(
    provider: &dyn AiProvider,
    scanner: &dyn SensitiveScanner,
    ledger: &dyn Ledger,
    agent: &Agent,
    toolbox: &[&dyn Tool],
    errand: Errand,
) -> Result<AgentRun, AgentError> {
    let task = AiTask::Agent { name: agent.task, tier: agent.tier };
    let system = system(agent, toolbox);
    let schema = step_schema(agent, toolbox);
    let mut record = RunRecord { agent: agent.name, model_calls: 0, tool_calls: 0, tools: BTreeMap::new(), rejected: 0, result: String::new() };
    let mut done: Vec<Done> = Vec::new();
    let mut tokens = 0u64;

    let result = loop {
        let left = agent.max_tool_calls.saturating_sub(record.tool_calls);
        let call = AiCall { task, context: errand.context.clone(), user: step_text(&errand, &done, left), project_id: None };
        let custom = Custom { schema: schema.clone(), max_output_tokens: agent.max_output_tokens, system: Some(system.clone()) };
        record.model_calls += 1;
        let step = match pipeline::run_measured(provider, scanner, ledger, call, Some(custom)).await {
            Ok(m) => {
                tokens += m.tokens;
                m.value
            }
            Err(e) => break Err(AgentError::Ai(e)),
        };
        if !step["answer"].is_null() {
            break Ok(step["answer"].clone());
        }
        let Some(name) = step["tool"].as_str().map(str::to_string) else {
            // neither a tool nor an answer: it counts as a request, so the loop still ends
            record.tool_calls += 1;
            record.rejected += 1;
            done.push(Done { tool: "(ninguna)".into(), args: Value::Null, outcome: Err("no pidió herramienta ni dio respuesta.".into()) });
            continue;
        };
        if left == 0 {
            break Err(AgentError::StepLimit);
        }
        if tokens >= agent.max_tokens {
            break Err(AgentError::TokenLimit);
        }
        record.tool_calls += 1;
        let args = step["args"].clone();
        let tool = usable(agent, toolbox).into_iter().find(|t| t.name() == name);
        let outcome = match tool {
            None => Err("se rechazó: no es una de sus herramientas.".to_string()),
            Some(t) => match validate_args(&t.args_schema(), &args) {
                Err(_) => Err("se rechazó: los argumentos no tienen la forma pedida.".to_string()),
                Ok(()) => match t.run(&args) {
                    Ok(v) => {
                        *record.tools.entry(t.name()).or_insert(0) += 1;
                        Ok(v)
                    }
                    Err(ToolError::Denied) => Err("se rechazó: la persona no tiene permiso de ver esto.".to_string()),
                    Err(ToolError::BadArgs(_)) => Err("se rechazó: los argumentos no sirven.".to_string()),
                    Err(ToolError::Failed) => Err("falló; siga sin este dato.".to_string()),
                },
            },
        };
        if outcome.is_err() {
            record.rejected += 1;
        }
        done.push(Done { tool: name, args, outcome });
    };

    record.result = match &result {
        Ok(_) => "answered".into(),
        Err(e) => e.kind(),
    };
    ledger.record_run(&record)?;
    result.map(|answer| AgentRun { answer, model_calls: record.model_calls, tool_calls: record.tool_calls, tools: record.tools, rejected: record.rejected })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::mock::MockProvider;
    use crate::ai::pipeline::SqliteLedger;
    use crate::ai::{AiResponse, Usage};
    use crate::scanner::{RegexScanner, ScannerConfig};
    use crate::storage::open_encrypted;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn ledger() -> (tempfile::TempDir, SqliteLedger, Arc<Mutex<Connection>>) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(open_encrypted(&dir.path().join("t.db"), KEY).unwrap()));
        (dir, SqliteLedger(conn.clone()), conn)
    }

    /// A tool that answers what it was given, and says when it ran.
    struct Echo {
        name: &'static str,
        ran: Mutex<u32>,
        result: Result<Value, ToolError>,
    }

    impl Echo {
        fn new(name: &'static str, result: Result<Value, ToolError>) -> Self {
            Echo { name, ran: Mutex::new(0), result }
        }
        fn times(&self) -> u32 {
            *self.ran.lock().unwrap()
        }
    }

    impl Tool for Echo {
        fn name(&self) -> &'static str {
            self.name
        }
        fn description(&self) -> &'static str {
            "Devuelve un dato de prueba."
        }
        fn args_schema(&self) -> Value {
            json!({ "type": "object", "properties": { "what": { "type": "string" } }, "required": ["what"], "additionalProperties": false })
        }
        fn run(&self, _args: &Value) -> Result<Value, ToolError> {
            *self.ran.lock().unwrap() += 1;
            self.result.clone()
        }
    }

    fn agent(max_tool_calls: u32) -> Agent {
        Agent {
            task: "agent.test",
            name: "test",
            tier: ModelTier::Light,
            trade: "Usted es un agente de prueba. Responde con usted.",
            tools: &["first", "second"],
            max_tool_calls,
            max_tokens: 100_000,
            max_output_tokens: 400,
            answer: json!({ "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"], "additionalProperties": false }),
            knows_institution: false,
        }
    }

    fn reply(v: Value, tokens: u64) -> Result<AiResponse, AiError> {
        Ok(AiResponse { value: v, model: "mock".into(), usage: Usage { input_tokens: tokens, output_tokens: 10, ..Default::default() } })
    }
    fn ask(tool: &str) -> Result<AiResponse, AiError> {
        reply(json!({ "tool": tool, "args": { "what": "x" }, "answer": null }), 100)
    }
    fn answer(text: &str) -> Result<AiResponse, AiError> {
        reply(json!({ "tool": null, "args": null, "answer": { "text": text } }), 100)
    }

    fn scanner() -> RegexScanner {
        RegexScanner::new(ScannerConfig::default())
    }

    fn errand() -> Errand {
        Errand { context: vec!["Pantalla: Mi institución".into()], request: "¿Qué me falta?".into() }
    }

    fn run_details(conn: &Arc<Mutex<Connection>>) -> Value {
        let d: String = conn.lock().unwrap().query_row("SELECT details_json FROM audit_log WHERE event='ai.run'", [], |r| r.get(0)).unwrap();
        serde_json::from_str(&d).unwrap()
    }

    #[tokio::test]
    async fn an_agent_asks_for_two_tools_and_answers() {
        let (_d, l, conn) = ledger();
        let first = Echo::new("first", Ok(json!({ "falta": "misión" })));
        let second = Echo::new("second", Ok(json!({ "avance": 80 })));
        let p = MockProvider::new(vec![ask("first"), ask("second"), answer("Le falta la misión.")]);
        let run = run(&p, &scanner(), &l, &agent(4), &[&first, &second], errand()).await.unwrap();
        assert_eq!(run.answer["text"], "Le falta la misión.");
        assert_eq!((run.model_calls, run.tool_calls, run.rejected), (3, 2, 0));
        assert_eq!((first.times(), second.times()), (1, 1));
        // the result of each tool reaches the next step, and the count of what is left goes down
        let reqs = p.requests();
        assert!(reqs[1].user.contains("`first` con {\"what\":\"x\"} devolvió: {\"falta\":\"misión\"}"), "{}", reqs[1].user);
        assert!(reqs[2].user.contains("devolvió: {\"avance\":80}") && reqs[2].user.contains("Le quedan 2 pedidos"));
        // the prompt is the same in every step (cacheable) and lists the tools with their arguments
        assert!(reqs.iter().all(|r| r.system == reqs[0].system));
        assert!(reqs[0].system.contains("`first`: Devuelve un dato de prueba.") && reqs[0].system.contains("Reglas que aplican siempre"));
        assert_eq!(reqs[0].task.as_str(), "agent.test");
        // every step went through the pipeline: three calls logged
        let calls: i64 = conn.lock().unwrap().query_row("SELECT count(*) FROM ai_usage WHERE task='agent.test' AND success=1", [], |r| r.get(0)).unwrap();
        assert_eq!(calls, 3);
        // the audit log keeps counts, never content
        let d = run_details(&conn);
        assert_eq!(d, json!({ "agent": "test", "model_calls": 3, "tool_calls": 2, "tools": { "first": 1, "second": 1 }, "rejected": 0, "result": "answered" }));
    }

    #[tokio::test]
    async fn a_tool_the_agent_may_not_use_is_refused_and_never_runs() {
        let (_d, l, conn) = ledger();
        let first = Echo::new("first", Ok(json!(1)));
        // `delete_all` is in the box (another agent may have it), not in the list of this one
        let other = Echo::new("delete_all", Ok(json!("borrado")));
        let p = MockProvider::new(vec![ask("delete_all"), ask("invented"), answer("No pude.")]);
        let run = run(&p, &scanner(), &l, &agent(4), &[&first, &other], errand()).await.unwrap();
        assert_eq!(other.times(), 0, "a tool outside its list never runs");
        assert_eq!((run.tool_calls, run.rejected), (2, 2));
        let reqs = p.requests();
        assert!(reqs[1].user.contains("`delete_all` con {\"what\":\"x\"}: se rechazó: no es una de sus herramientas."), "{}", reqs[1].user);
        assert!(!reqs[0].system.contains("delete_all"), "the prompt only lists its own tools");
        assert_eq!(run_details(&conn)["rejected"], 2);
    }

    #[tokio::test]
    async fn bad_arguments_and_a_denied_tool_are_told_to_the_model() {
        let (_d, l, _c) = ledger();
        let first = Echo::new("first", Err(ToolError::Denied));
        // the shape fits, the value does not (a section that does not exist): the tool says so
        let second = Echo::new("second", Err(ToolError::BadArgs("unknown section".into())));
        let p = MockProvider::new(vec![ask("first"), ask("second"), answer("Listo.")]);
        let r = run(&p, &scanner(), &l, &agent(4), &[&first, &second], errand()).await.unwrap();
        assert_eq!(r.rejected, 2);
        assert!(r.tools.is_empty(), "only what ran counts as used");
        let last = p.requests().last().unwrap().user.clone();
        assert!(last.contains("no tiene permiso de ver esto") && last.contains("los argumentos no sirven"), "{last}");

        // arguments of no tool's shape: the pipeline refuses the step and asks again, and the tool never runs
        let (_d, l, _c) = ledger();
        let second = Echo::new("second", Ok(json!(1)));
        let out_of_shape = reply(json!({ "tool": "second", "args": { "nope": 1 }, "answer": null }), 100);
        let p = MockProvider::new(vec![out_of_shape, answer("Listo.")]);
        run(&p, &scanner(), &l, &agent(4), &[&second], errand()).await.unwrap();
        assert_eq!(second.times(), 0);
        assert!(p.requests()[1].user.contains("no cumplió el formato"));
    }

    #[tokio::test]
    async fn the_cap_of_tool_requests_cuts_the_loop() {
        let (_d, l, conn) = ledger();
        let first = Echo::new("first", Ok(json!(1)));
        // it asks and asks: after two it is told to answer, and asking again ends the errand
        let p = MockProvider::new(vec![ask("first"), ask("first"), ask("first"), answer("nunca llega")]);
        let r = run(&p, &scanner(), &l, &agent(2), &[&first], errand()).await;
        assert!(matches!(r, Err(AgentError::StepLimit)), "{r:?}");
        assert_eq!(first.times(), 2);
        assert_eq!(p.requests().len(), 3, "no call after the cap");
        assert!(p.requests()[2].user.contains("Ya no puede pedir herramientas"));
        let d = run_details(&conn);
        assert_eq!((d["result"].as_str(), d["tool_calls"].as_u64()), (Some("step_limit"), Some(2)));
    }

    #[tokio::test]
    async fn with_no_requests_left_it_still_answers() {
        let (_d, l, _c) = ledger();
        let first = Echo::new("first", Ok(json!(1)));
        let p = MockProvider::new(vec![ask("first"), answer("Con lo que tengo.")]);
        let run = run(&p, &scanner(), &l, &agent(1), &[&first], errand()).await.unwrap();
        assert_eq!(run.answer["text"], "Con lo que tengo.");
    }

    #[tokio::test]
    async fn the_cap_of_text_cuts_the_loop() {
        let (_d, l, conn) = ledger();
        let first = Echo::new("first", Ok(json!(1)));
        let mut a = agent(10);
        a.max_tokens = 1_500;
        let big = || reply(json!({ "tool": "first", "args": { "what": "x" }, "answer": null }), 1_000);
        let p = MockProvider::new(vec![big(), big(), answer("nunca llega")]);
        let r = run(&p, &scanner(), &l, &a, &[&first], errand()).await;
        assert!(matches!(r, Err(AgentError::TokenLimit)), "{r:?}");
        assert_eq!(first.times(), 1, "the second request came after the text was used up");
        assert_eq!(run_details(&conn)["result"], "token_limit");
    }

    #[tokio::test]
    async fn what_a_tool_returns_goes_through_the_scanner_before_the_next_call() {
        let (_d, l, conn) = ledger();
        let leaky = Echo::new("first", Ok(json!({ "nota": "CURP LOPM800101MDFRZN09" })));
        let p = MockProvider::new(vec![ask("first"), answer("Listo.")]);
        run(&p, &scanner(), &l, &agent(4), &[&leaky], errand()).await.unwrap();
        let second = &p.requests()[1].user;
        assert!(!second.contains("LOPM8") && second.contains("[CURP OCULTA]"), "{second}");
        let n: i64 = conn.lock().unwrap().query_row("SELECT count(*) FROM audit_log WHERE event='scanner.leak_prevented'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }

    #[tokio::test]
    async fn an_error_of_the_service_ends_the_errand_and_is_recorded() {
        let (_d, l, conn) = ledger();
        let first = Echo::new("first", Ok(json!(1)));
        let p = MockProvider::new(vec![ask("first"), Err(AiError::Auth)]);
        let r = run(&p, &scanner(), &l, &agent(4), &[&first], errand()).await;
        assert!(matches!(r, Err(AgentError::Ai(AiError::Auth))));
        assert_eq!(run_details(&conn)["result"], "auth");
    }

    #[test]
    fn the_step_schema_takes_a_tool_or_an_answer_and_the_arguments_of_its_tools() {
        let first = Echo::new("first", Ok(json!(1)));
        let s = step_schema(&agent(4), &[&first]);
        let v = jsonschema::validator_for(&s).unwrap();
        assert!(v.is_valid(&json!({ "tool": "first", "args": { "what": "x" }, "answer": null })));
        assert!(v.is_valid(&json!({ "tool": null, "args": null, "answer": { "text": "t" } })));
        assert!(v.is_valid(&json!({ "tool": "unknown", "args": null, "answer": null })), "an unknown tool is refused by the code, not the schema");
        assert!(!v.is_valid(&json!({ "tool": null, "args": null, "answer": { "nope": 1 } })), "the answer keeps its schema");
        assert!(!v.is_valid(&json!({ "tool": "first" })));
    }
}
