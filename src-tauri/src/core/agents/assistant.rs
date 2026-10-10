//! The Assistant (ADR-034 §1 and §4, block IA3): the door to the AI from every screen. It knows where the person is
//! (the screen and, when the «?» of a field opened it, the field), reads the manual and the parts of the sheet it
//! needs with its tools, and answers in plain words with the manual entries it used and, maybe, a place to go.
//!
//! - **What the person writes goes through the scanner first,** as every text; with findings, nothing is asked
//!   until the person decides (quarantine).
//! - **Without the AI** (no key, no internet, no budget, or the errand did not end) the manual still answers: the
//!   entries that match the question, found with no AI and no cost.
//! - **The conversation lives only on the screen** and is gone when the app closes: the last turns come with each
//!   question and nothing of it is saved.

use crate::ai::agent::{self, Agent, AgentError, Errand, Tool};
use crate::ai::pipeline::SqliteLedger;
use crate::ai::{AiError, AiProvider, ModelTier};
use crate::core::access::domain::Role;
use crate::core::error::ServiceError;
use crate::scanner::guard::{Decision, QuarantineReport};
use crate::storage::SharedDb;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Its trade, versioned like a prompt: changing it is a new file.
const TRADE: &str = include_str!("assistant.v1.md");

/// Where the Assistant may send the person: a page, or a window of «Mi institución» (`profile.<window>`).
pub const GO_TO: &[&str] = &[
    "home", "profile", "profile.institution", "profile.contact", "profile.legal", "profile.capacity", "profile.board", "documents", "projects", "staff",
    "people", "facilities", "finance", "help",
];

/// The screens of the app: (page, its name, its entry in the manual).
const SCREENS: &[(&str, &str, Option<&str>)] = &[
    ("home", "Inicio", Some("screen.home")),
    ("profile", "Mi institución", Some("screen.institution")),
    ("documents", "Documentos", Some("screen.documents")),
    ("projects", "Proyectos", Some("screen.projects")),
    ("staff", "Personal", Some("screen.staff")),
    ("people", "Beneficiarios", Some("screen.people")),
    ("facilities", "Instalaciones", Some("screen.facilities")),
    ("finance", "Finanzas", Some("screen.finance")),
    ("ai", "Inteligencia artificial (ajustes)", None),
    ("security", "Seguridad", None),
    ("admin", "Administración", None),
    ("help", "Ayuda", None),
];

/// The turns of the conversation that come with a question, at most.
pub const HISTORY_TURNS: usize = 6;
/// The longest question, in characters.
pub const MAX_QUESTION_CHARS: usize = 1_000;
/// The entries of the manual shown when the AI cannot answer.
const MANUAL_HITS: usize = 4;

pub fn agent() -> Agent {
    Agent {
        task: "agent.assistant",
        name: "assistant",
        tier: ModelTier::Light,
        trade: TRADE,
        tools: &["manual_entry", "manual_search", "fill_state", "sheet_section"],
        max_tool_calls: 4,
        max_tokens: 40_000,
        max_output_tokens: 700,
        answer: json!({
            "type": "object",
            "properties": {
                "text": { "type": "string" },
                "go_to": { "anyOf": [{ "type": "string", "enum": GO_TO }, { "type": "null" }] },
                "sources": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["text", "go_to", "sources"],
            "additionalProperties": false
        }),
        knows_institution: true,
    }
}

/// One turn of the conversation, as the screen keeps it.
#[derive(Debug, Clone, Deserialize)]
pub struct Turn {
    /// `person` or `assistant`.
    pub from: String,
    pub text: String,
}

/// What the screen sends with a question.
#[derive(Debug, Clone, Deserialize)]
pub struct Ask {
    /// The page where the person is (`profile`).
    pub screen: String,
    /// The field whose «?» opened the question, if any (`institution.legal_rfc`).
    pub field: Option<String>,
    pub question: String,
    #[serde(default)]
    pub history: Vec<Turn>,
}

/// An entry of the manual, by its id and title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Source {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AssistantOutcome {
    /// The AI answered. `asked` is the question as it went (covered, if the person chose so).
    Answered { asked: String, text: String, go_to: Option<String>, sources: Vec<Source> },
    /// The AI could not answer (`reason`): the manual answers instead.
    Manual { asked: String, reason: &'static str, hits: Vec<Source> },
    /// The question has data of a person: nothing was asked.
    Quarantine { report: QuarantineReport },
}

/// Why the AI did not answer, in a word the screen puts into words.
fn reason(e: &AgentError) -> &'static str {
    match e {
        AgentError::Ai(AiError::NoApiKey) => "not_configured",
        AgentError::Ai(AiError::Offline) => "offline",
        AgentError::Ai(AiError::RateLimited | AiError::Timeout) => "busy",
        AgentError::Ai(AiError::BudgetExhausted) => "budget",
        AgentError::Ai(AiError::QuotaReached) => "quota",
        AgentError::Ai(AiError::Auth) => "key_rejected",
        _ => "unavailable",
    }
}

fn lock(db: &SharedDb) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, ServiceError> {
    db.lock().map_err(|_| ServiceError::Internal("lock".into()))
}

fn source(id: &str) -> Option<Source> {
    crate::core::manual::entry(id).map(|e| Source { id: e.entry.id, title: e.entry.title })
}

/// Where a field of the catalog is filled in, as a place of `GO_TO`: the code knows it, the AI does not guess it.
pub fn place_of_field(field: &str) -> Option<&'static str> {
    use crate::core::institution::forms::{FORMS, LIST_FORMS};
    let form = FORMS.iter().chain(LIST_FORMS).find(|f| f.field(field).is_some())?;
    Some(match form.id {
        "institution.identity" => "profile.institution",
        "institution.contact" => "profile.contact",
        "institution.legal" => "profile.legal",
        "institution.capacity" => "profile.capacity",
        "institution.board_member" | "institution.bank_account" => "profile.board",
        _ => "profile",
    })
}

/// What the Assistant is told besides the question: where the person is, the field and what the manual says of it,
/// and the last turns of the conversation. Ids, never the content of a record.
pub fn context(ask: &Ask) -> Vec<String> {
    let mut out = Vec::new();
    let screen = SCREENS.iter().find(|(p, _, _)| *p == ask.screen);
    let mut place = format!("La persona está en la pantalla «{}»", screen.map_or("(otra)", |s| s.1));
    if let Some(id) = screen.and_then(|s| s.2) {
        place.push_str(&format!(" (su entrada en el manual: `{id}`)"));
    }
    place.push('.');
    out.push(place);
    if let Some(e) = ask.field.as_deref().and_then(crate::core::manual::entry) {
        let mut s = format!("Preguntó desde el campo «{}» (`{}`).", e.entry.title, e.entry.id);
        if let Some(place) = place_of_field(&e.entry.id) {
            s.push_str(&format!(" Ese campo se llena en el lugar `{place}`."));
        }
        s.push_str(&format!(" El manual dice de él:\n{}", e.entry.paragraphs.join("\n")));
        out.push(s);
    }
    let turns: Vec<&Turn> = ask.history.iter().rev().take(HISTORY_TURNS).collect::<Vec<_>>().into_iter().rev().collect();
    if !turns.is_empty() {
        let lines: String = turns
            .iter()
            .map(|t| format!("{}: {}\n", if t.from == "assistant" { "Asistente" } else { "Persona" }, t.text.trim()))
            .collect();
        out.push(format!("La conversación hasta ahora:\n{lines}"));
    }
    out
}

/// The manual alone: the entries that match the question, and the one of the field if it came from one.
fn manual(db: &SharedDb, ask: &Ask, asked: &str, why: &'static str) -> Result<AssistantOutcome, ServiceError> {
    let conn = lock(db)?;
    let mut hits: Vec<Source> = ask.field.as_deref().and_then(source).into_iter().collect();
    for h in crate::core::manual::search(&conn, asked, MANUAL_HITS)? {
        if !hits.iter().any(|x| x.id == h.id) {
            hits.push(Source { id: h.id, title: h.title });
        }
    }
    hits.truncate(MANUAL_HITS);
    Ok(AssistantOutcome::Manual { asked: asked.to_string(), reason: why, hits })
}

/// Answers a question of the person. `role` is the one of their session: the tools give only what they may see.
pub async fn ask(db: &SharedDb, provider: Option<&dyn AiProvider>, role: Option<Role>, ask: Ask, decision: Option<Decision>) -> Result<AssistantOutcome, ServiceError> {
    let question = ask.question.trim();
    if question.is_empty() {
        return Err(ServiceError::EmptyText);
    }
    if question.chars().count() > MAX_QUESTION_CHARS {
        return Err(ServiceError::TextTooLarge);
    }
    let (asked, scanner) = {
        let conn = lock(db)?;
        let fields = vec![("question".to_string(), question.to_string())];
        match crate::core::screen::guard_texts(&conn, "assistant_question", &fields, decision)? {
            Ok(mut t) => (t.remove(0), crate::core::screen::scanner_for(&conn)?),
            Err(report) => return Ok(AssistantOutcome::Quarantine { report }),
        }
    };
    let Some(provider) = provider else { return manual(db, &ask, &asked, "not_configured") };

    let boxed = crate::core::agents::tools::toolbox(db.clone(), role);
    let tools: Vec<&dyn Tool> = boxed.iter().map(|t| t.as_ref()).collect();
    let ledger = SqliteLedger(db.clone());
    let errand = Errand { context: context(&ask), request: asked.clone() };
    match agent::run(provider, &scanner, &ledger, &agent(), &tools, errand).await {
        Ok(run) => Ok(answered(&run.answer, asked)),
        Err(e) => manual(db, &ask, &asked, reason(&e)),
    }
}

/// The answer as the screen gets it: a place to go only if it is one of `GO_TO`, and the sources only if they are
/// entries of the manual (with their titles, from the code).
fn answered(a: &Value, asked: String) -> AssistantOutcome {
    let text = a["text"].as_str().unwrap_or_default().trim().to_string();
    let go_to = a["go_to"].as_str().filter(|g| GO_TO.contains(g)).map(str::to_string);
    let mut sources: Vec<Source> = Vec::new();
    for id in a["sources"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        if let Some(s) = source(id).filter(|s| !sources.contains(s)) {
            sources.push(s);
        }
    }
    AssistantOutcome::Answered { asked, text, go_to, sources }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::mock::MockProvider;
    use crate::ai::{AiResponse, Usage};
    use crate::core::ai_sheet::tests::{rich, rich_money, seed_rich_board, seed_rich_facilities, seed_rich_people, seed_rich_staff};
    use std::sync::{Arc, Mutex};

    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    fn rich_db() -> (tempfile::TempDir, SharedDb) {
        let dir = tempfile::tempdir().unwrap();
        let mut c = crate::storage::open_encrypted(&dir.path().join("t.db"), KEY).unwrap();
        seed_rich_staff(&mut c);
        seed_rich_people(&mut c);
        seed_rich_facilities(&c);
        seed_rich_board(&mut c);
        crate::core::profile::storage::save(&mut c, &rich()).unwrap();
        crate::modules::finance::storage::save(&mut c, &rich_money()).unwrap();
        crate::core::profile::storage::confirm(&mut c).unwrap();
        (dir, Arc::new(Mutex::new(c)))
    }

    fn reply(v: Value) -> Result<AiResponse, AiError> {
        Ok(AiResponse { value: v, model: "mock".into(), usage: Usage { input_tokens: 900, output_tokens: 80, ..Default::default() } })
    }
    fn tool(name: &str, args: Value) -> Result<AiResponse, AiError> {
        reply(json!({ "tool": name, "args": args, "answer": null }))
    }
    fn answer(text: &str, go_to: Option<&str>, sources: &[&str]) -> Result<AiResponse, AiError> {
        reply(json!({ "tool": null, "args": null, "answer": { "text": text, "go_to": go_to, "sources": sources } }))
    }

    fn question(screen: &str, field: Option<&str>, q: &str) -> Ask {
        Ask { screen: screen.into(), field: field.map(str::to_string), question: q.into(), history: vec![] }
    }

    /// The battery of IA3 (ADR-034): one simulated question per kind of doubt and screen. Each one reads what it
    /// needs with its tools and answers with its sources and, maybe, a place to go.
    #[tokio::test]
    async fn the_assistant_answers_each_kind_of_doubt_from_its_screen() {
        struct Case {
            ask: Ask,
            steps: Vec<Result<AiResponse, AiError>>,
            /// what the tool results must carry to the next step
            seen: &'static str,
            go_to: Option<&'static str>,
            sources: &'static [&'static str],
        }
        let cases = vec![
            // «¿qué pongo aquí?» from the «?» of a field: the entry of the field comes in the context
            Case {
                ask: question("profile", Some("institution.junta_folio"), "¿Qué pongo aquí?"),
                steps: vec![answer("Escriba el folio de su constancia de registro ante la Junta.", Some("profile.legal"), &["institution.junta_folio"])],
                seen: "",
                go_to: Some("profile.legal"),
                sources: &["institution.junta_folio"],
            },
            // «¿cómo hago…?»: it searches the manual
            Case {
                ask: question("home", None, "¿Cómo hago un respaldo?"),
                steps: vec![tool("manual_search", json!({ "query": "respaldo" })), answer("En Seguridad, use «Hacer un respaldo».", None, &["howto.backup"])],
                seen: "howto.backup",
                go_to: None,
                sources: &["howto.backup"],
            },
            // «¿qué me falta?»: the fill state, never a value
            Case {
                ask: question("profile", None, "¿Qué me falta por llenar?"),
                steps: vec![tool("fill_state", json!({ "form": null })), answer("Tiene todo completo.", None, &[])],
                seen: "\"percent\":",
                go_to: None,
                sources: &[],
            },
            // a doubt about the institution: the part of the sheet that matters
            Case {
                ask: question("finance", None, "¿Nos alcanza el dinero este año?"),
                steps: vec![tool("sheet_section", json!({ "section": "money" })), answer("Sí, sus ingresos alcanzan a cubrir los egresos.", Some("finance"), &[])],
                seen: "Balance del año",
                go_to: Some("finance"),
                sources: &[],
            },
            // «¿dónde está…?»: the screen entry and a place to go
            Case {
                ask: question("people", None, "¿Dónde registro al patronato?"),
                steps: vec![tool("manual_entry", json!({ "id": "institution.board_member" })), answer("En Mi institución, apartado «Patronato y cuentas».", Some("profile.board"), &["institution.board_member"])],
                seen: "Patronato u órgano de gobierno",
                go_to: Some("profile.board"),
                sources: &["institution.board_member"],
            },
        ];
        for case in cases {
            let (_d, db) = rich_db();
            let p = MockProvider::new(case.steps);
            let out = ask(&db, Some(&p), Some(Role::Manager), case.ask.clone(), None).await.unwrap();
            let AssistantOutcome::Answered { go_to, sources, text, .. } = out else { panic!("{:?}: {out:?}", case.ask.question) };
            assert!(!text.is_empty());
            assert_eq!(go_to.as_deref(), case.go_to, "{}", case.ask.question);
            assert_eq!(sources.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), case.sources, "{}", case.ask.question);
            let reqs = p.requests();
            assert!(reqs[0].system.contains("Usted es el Asistente"), "its trade");
            if case.ask.field.is_some() {
                assert!(reqs[0].context.iter().any(|c| c.contains("El manual dice de él") && c.contains("Junta de Asistencia Privada de su estado")), "{:?}", reqs[0].context);
                assert!(reqs[0].context.iter().any(|c| c.contains("se llena en el lugar `profile.legal`")), "the code says where the field is");
            }
            if !case.seen.is_empty() {
                assert!(reqs.last().unwrap().user.contains(case.seen), "{}: {}", case.ask.question, reqs.last().unwrap().user);
            }
            // nothing protected reaches the AI by any of its tools
            for r in &reqs {
                let all = format!("{} {}", r.context.join(" "), r.user);
                for secret in ["Presidenta Ficticia", "002010077777777771", "AFI200101"] {
                    assert!(!all.contains(secret), "«{secret}» reached the AI");
                }
            }
        }
    }

    /// The screen and the last turns come with the question; older turns stay out.
    #[test]
    fn the_context_says_where_the_person_is_and_what_was_said() {
        let mut a = question("staff", None, "¿Y eso?");
        a.history = (0..10).map(|i| Turn { from: if i % 2 == 0 { "person".into() } else { "assistant".into() }, text: format!("turno {i}") }).collect();
        let c = context(&a);
        assert!(c[0].contains("«Personal»") && c[0].contains("`screen.staff`"), "{c:?}");
        assert!(c[1].contains("turno 9") && c[1].contains("turno 4") && !c[1].contains("turno 3"), "{c:?}");
        assert!(context(&question("nowhere", None, "x"))[0].contains("(otra)"));
    }

    /// Without the AI the manual answers; a question with data of a person waits for the person.
    #[tokio::test]
    async fn without_the_ai_the_manual_answers_and_personal_data_waits() {
        let (_d, db) = rich_db();
        let out = ask(&db, None, Some(Role::Manager), question("profile", Some("institution.legal_rfc"), "¿Cómo hago un respaldo?"), None).await.unwrap();
        let AssistantOutcome::Manual { reason, hits, .. } = out else { panic!("{out:?}") };
        assert_eq!(reason, "not_configured");
        assert_eq!(hits[0].id, "institution.legal_rfc", "the field first");
        assert!(hits.iter().any(|h| h.id == "howto.backup"), "{hits:?}");

        // the AI fails: the manual answers, and the reason says why
        let p = MockProvider::new(vec![Err(AiError::Offline), Err(AiError::Offline)]);
        let out = ask(&db, Some(&p), Some(Role::Manager), question("home", None, "respaldo"), None).await.unwrap();
        assert!(matches!(out, AssistantOutcome::Manual { reason: "offline", .. }), "{out:?}");

        // a question with a CURP is held; covered, it goes on without it
        let held = question("people", None, "¿Dónde pongo la CURP LOPM800101MDFRZN09?");
        let p = MockProvider::new(vec![answer("En la ficha de la persona.", Some("people"), &[])]);
        assert!(matches!(ask(&db, Some(&p), Some(Role::Manager), held.clone(), None).await.unwrap(), AssistantOutcome::Quarantine { .. }));
        assert!(p.requests().is_empty(), "nothing went to the AI");
        let out = ask(&db, Some(&p), Some(Role::Manager), held, Some(Decision::Redact)).await.unwrap();
        let AssistantOutcome::Answered { asked, .. } = out else { panic!("{out:?}") };
        assert!(!asked.contains("LOPM800101") && !p.requests()[0].user.contains("LOPM800101"));

        assert!(matches!(ask(&db, None, Some(Role::Manager), question("home", None, "  "), None).await, Err(ServiceError::EmptyText)));
    }

    /// The code keeps only places it knows and sources that are entries of the manual.
    #[test]
    fn a_place_or_a_source_the_code_does_not_know_is_dropped() {
        let a = json!({ "text": " Hola ", "go_to": "somewhere", "sources": ["institution.name", "invented.entry", "institution.name"] });
        let AssistantOutcome::Answered { text, go_to, sources, .. } = answered(&a, "q".into()) else { panic!() };
        assert_eq!((text.as_str(), go_to), ("Hola", None));
        assert_eq!(sources, vec![Source { id: "institution.name".into(), title: "Nombre de la institución".into() }]);
    }

    /// Every field of the catalog has its place, and it is one the Assistant may offer.
    #[test]
    fn every_field_has_the_place_where_it_is_filled_in() {
        use crate::core::institution::forms::{FORMS, LIST_FORMS};
        for f in FORMS.iter().chain(LIST_FORMS).flat_map(|f| f.fields()) {
            let place = place_of_field(f.id).unwrap();
            assert!(GO_TO.contains(&place) && place != "profile", "{}: {place}", f.id);
        }
        assert_eq!(place_of_field("institution.junta_folio"), Some("profile.legal"));
        assert_eq!(place_of_field("institution.bank.clabe"), Some("profile.board"));
        assert_eq!(place_of_field("nope"), None);
    }

    /// Every place the trade offers is one the code knows, and the other way round.
    #[test]
    fn the_trade_names_every_place() {
        for g in GO_TO {
            assert!(TRADE.contains(&format!("`{g}`:")), "{g} is not in the trade");
        }
        assert!(agent().tools.iter().all(|t| crate::core::agents::tools::toolbox(Arc::new(Mutex::new(rusqlite::Connection::open_in_memory().unwrap())), None).iter().any(|b| b.name() == *t)));
    }

    /// The real run of IA3, opt-in and capped: three questions from three screens with the provider and key saved in
    /// the app (at most 5 steps each). It prints each answer, its steps and what it cost.
    ///   cargo test assistant_live -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn assistant_live() {
        let (_d, db) = rich_db();
        let provider = {
            let c = db.lock().unwrap();
            let cfg = crate::ai::settings::load(&c).unwrap();
            let key = crate::storage::get_api_key(cfg.provider.as_str()).unwrap().expect("save the key in the app first");
            println!("Proveedor: {}  modelo ligero: {}", cfg.provider.as_str(), cfg.model_for(ModelTier::Light));
            crate::ai::build_provider(&cfg, key)
        };
        let questions = [
            question("profile", Some("institution.junta_folio"), "¿Qué pongo aquí?"),
            question("home", None, "¿Qué me falta por llenar?"),
            question("finance", None, "¿Nos alcanza el dinero este año?"),
        ];
        let spend = |db: &SharedDb| -> (i64, i64, i64, f64) {
            db.lock()
                .unwrap()
                .query_row("SELECT count(*), coalesce(sum(input_tokens),0), coalesce(sum(output_tokens),0), coalesce(sum(estimated_cost_mxn),0) FROM ai_usage", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                .unwrap()
        };
        for q in questions {
            let before = spend(&db);
            let started = std::time::Instant::now();
            let out = ask(&db, Some(provider.as_ref()), Some(Role::Manager), q.clone(), None).await.unwrap();
            let after = spend(&db);
            println!(
                "\n«{}» ({}): {out:#?}\nLlamadas: {}  entrada: {}  salida: {}  costo: ${:.4} MXN  tiempo: {:.1} s",
                q.question,
                q.screen,
                after.0 - before.0,
                after.1 - before.1,
                after.2 - before.2,
                after.3 - before.3,
                started.elapsed().as_secs_f64()
            );
        }
        let runs: Vec<String> =
            db.lock().unwrap().prepare("SELECT details_json FROM audit_log WHERE event='ai.run'").unwrap().query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        println!("\nBitácora: {runs:#?}");
    }
}
