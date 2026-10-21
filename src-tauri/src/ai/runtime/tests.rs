use std::cell::{Cell, RefCell};

use rusqlite::Connection;
use serde_json::{json, Value};

use super::outcome::{Again, Ask, Body, Next, Outcome};
use super::request::{Channel, Content, Request};
use super::task::{Limits, Reached, Task, TaskState};
use super::*;
use crate::capability::{Permission, Registry};

/// A module as any other would be, with the capabilities the tests need. It keeps a name in a table of its own.
mod trial {
    use crate::capability::{Call, CapabilityError, Limits};

    #[derive(serde::Deserialize, schemars::JsonSchema)]
    pub struct Nothing {}

    #[derive(serde::Serialize, schemars::JsonSchema)]
    pub struct Name {
        pub name: String,
    }

    #[derive(serde::Deserialize, schemars::JsonSchema)]
    pub struct Rename {
        pub name: String,
    }

    #[derive(serde::Serialize, schemars::JsonSchema)]
    pub struct Change {
        pub from: String,
        pub to: String,
    }

    #[derive(serde::Deserialize, schemars::JsonSchema)]
    pub struct Count {
        pub n: i64,
    }

    fn db(_: rusqlite::Error) -> CapabilityError {
        CapabilityError::Failed { reason: "the database did not answer" }
    }

    fn current(call: &Call) -> Result<String, CapabilityError> {
        call.conn.query_row("SELECT name FROM trial_name WHERE id = 1", [], |r| r.get(0)).map_err(db)
    }

    pub fn read(call: &Call, _: Nothing) -> Result<Name, CapabilityError> {
        Ok(Name { name: current(call)? })
    }

    pub fn preview(call: &Call, ask: Rename) -> Result<Change, CapabilityError> {
        if ask.name.trim().is_empty() {
            return Err(CapabilityError::NeedsPerson { what: "the new name" });
        }
        if ask.name.chars().count() > 40 {
            return Err(CapabilityError::Invalid { field: "name", reason: "longer than 40 letters" });
        }
        Ok(Change { from: current(call)?, to: ask.name })
    }

    pub fn rename(call: &Call, ask: Rename) -> Result<Change, CapabilityError> {
        let from = current(call)?;
        call.conn.execute("UPDATE trial_name SET name = ?1, task = ?2 WHERE id = 1", rusqlite::params![ask.name, call.task]).map_err(db)?;
        Ok(Change { from, to: ask.name })
    }

    /// Never has enough until the count reaches 100: a task that does not advance.
    pub fn count(_: &Call, ask: Count) -> Result<Name, CapabilityError> {
        if ask.n < 100 {
            return Err(CapabilityError::NeedsPerson { what: "a larger count" });
        }
        Ok(Name { name: ask.n.to_string() })
    }

    pub fn broken(_: &Call, _: Nothing) -> Result<Name, CapabilityError> {
        Err(CapabilityError::Failed { reason: "always fails" })
    }

    crate::capability! {
        static READ = "trial.read" {
            description: "Lee el nombre de prueba.",
            effect: Query,
            permission: Use,
            data: [Internal],
            approval: Never,
            limits: Limits::READ,
            run: read(read),
        }
    }

    crate::capability! {
        static RENAME = "trial.rename" {
            description: "Cambia el nombre de prueba, con la aprobación de la persona.",
            effect: Modify,
            permission: Use,
            data: [Public],
            approval: Required,
            limits: Limits::CHANGE,
            run: change(preview, rename),
        }
    }

    crate::capability! {
        static RENAME_NOW = "trial.rename_now" {
            description: "Cambia el nombre de prueba sin pedir aprobación.",
            effect: Modify,
            permission: Use,
            data: [Public],
            approval: Never,
            limits: Limits::CHANGE,
            run: change(preview, rename),
        }
    }

    crate::capability! {
        static ADMIN = "trial.admin" {
            description: "Una consulta solo para quien administra.",
            effect: Query,
            permission: Administer,
            data: [Internal],
            approval: Never,
            limits: Limits::READ,
            run: read(read),
        }
    }

    crate::capability! {
        static COUNT = "trial.count" {
            description: "Pide una cuenta mayor hasta llegar a 100.",
            effect: Query,
            permission: Use,
            data: [Internal],
            approval: Never,
            limits: Limits::READ,
            run: read(count),
        }
    }

    crate::capability! {
        static BROKEN = "trial.broken" {
            description: "Siempre falla.",
            effect: Query,
            permission: Use,
            data: [Internal],
            approval: Never,
            limits: Limits::READ,
            run: read(broken),
        }
    }
}

struct Clock(Cell<i64>);

impl super::Clock for Clock {
    fn now_ms(&self) -> i64 {
        // a millisecond passes on each look, so the trail keeps its order
        self.0.set(self.0.get() + 1);
        self.0.get()
    }
}

impl Clock {
    fn advance(&self, ms: i64) {
        self.0.set(self.0.get() + ms);
    }
}

/// The session of the tests: «ana» uses the app; nobody administers, unless a test says so.
struct Session {
    permissions: RefCell<Vec<Permission>>,
}

impl Authorize for Session {
    fn may(&self, who: &Who, permission: Permission) -> bool {
        who.account == "ana" && self.permissions.borrow().contains(&permission)
    }
}

#[derive(Default)]
struct Window(RefCell<Vec<Outcome>>);

impl Deliver for Window {
    fn deliver(&self, outcome: &Outcome) {
        self.0.borrow_mut().push(outcome.clone());
    }
}

struct World {
    conn: Connection,
    registry: Registry,
    clock: Clock,
    session: Session,
    window: Window,
}

const ANA: Who = Who { account: "ana", organization: "org" };

fn trial_table(conn: &Connection) {
    conn.execute_batch("CREATE TABLE trial_name (id INTEGER PRIMARY KEY, name TEXT NOT NULL, task TEXT); INSERT INTO trial_name (id, name) VALUES (1, 'Casa Hogar');")
        .unwrap();
}

fn world() -> World {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate(&mut conn).unwrap();
    trial_table(&conn);
    World::on(conn)
}

impl World {
    fn on(conn: Connection) -> World {
        World {
            conn,
            registry: Registry::discover().unwrap(),
            clock: Clock(Cell::new(1_000_000)),
            session: Session { permissions: RefCell::new(vec![Permission::Use]) },
            window: Window::default(),
        }
    }

    fn submit(&self, who: &Who, task: Option<&str>, content: Content) -> Outcome {
        let runtime = Runtime { registry: &self.registry, clock: &self.clock, authorize: &self.session, deliver: &self.window };
        let request = Request { channel: Channel::Screen { place: "institution".into() }, task: task.map(String::from), content };
        runtime.submit(&self.conn, who, request).unwrap()
    }

    fn invoke(&self, task: Option<&str>, capability: &str, input: Value) -> Outcome {
        self.submit(&ANA, task, Content::Invoke { capability: capability.into(), input })
    }

    fn answer(&self, task: &str, values: Value) -> Outcome {
        self.submit(&ANA, Some(task), Content::Answer { values })
    }

    fn decide(&self, task: &str, fingerprint: &str, approve: bool) -> Outcome {
        self.submit(&ANA, Some(task), Content::Decision { fingerprint: fingerprint.into(), approve })
    }

    fn name(&self) -> String {
        self.conn.query_row("SELECT name FROM trial_name WHERE id = 1", [], |r| r.get(0)).unwrap()
    }

    fn task(&self, id: &str) -> Task {
        store::load_task(&self.conn, id).unwrap().unwrap()
    }

    /// The trail of a task: (event, capability, code).
    fn trail(&self, id: &str) -> Vec<(String, Option<String>, Option<String>)> {
        self.conn
            .prepare("SELECT event, capability, code FROM ai_event WHERE task_id = ?1 ORDER BY at_ms, rowid")
            .unwrap()
            .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }
}

fn fingerprint_of(o: &Outcome) -> String {
    match &o.body {
        Body::Proposal { fingerprint, .. } => fingerprint.clone(),
        other => panic!("not a proposal: {other:?}"),
    }
}

fn line(event: &str, capability: Option<&str>, code: &str) -> (String, Option<String>, Option<String>) {
    (event.into(), capability.map(String::from), Some(code.into()))
}

/// The whole path of a change asked from a screen, with no router and no model: it is prepared, waits for the
/// approval of the person, is applied, and leaves its trail. Run it with `-- --nocapture` to read it.
#[test]
fn a_change_from_a_screen_walks_the_whole_path() {
    let w = world();
    println!("\nLa persona cambia el nombre en la pantalla y pulsa «Guardar».");
    let asked = w.invoke(None, "trial.rename", json!({ "name": "Casa Hogar San José" }));
    let task = asked.task.clone();
    println!("  tarea {task}: {:?}, {:?}", asked.state, asked.body);
    assert_eq!(asked.state, TaskState::Paused);
    let Body::Proposal { proposal, again: None, .. } = &asked.body else { panic!("{asked:?}") };
    assert_eq!(proposal, &json!({ "from": "Casa Hogar", "to": "Casa Hogar San José" }));
    assert_eq!(w.name(), "Casa Hogar", "nothing changes before the approval");

    println!("La persona revisa la propuesta y la aprueba.");
    let done = w.decide(&task, &fingerprint_of(&asked), true);
    println!("  tarea {}: {:?}, {:?}", done.task, done.state, done.body);
    assert_eq!(done.task, task, "the same task, resumed");
    assert_eq!(done.state, TaskState::Done);
    assert!(matches!(&done.body, Body::Data { capability, .. } if capability == "trial.rename"));
    let (name, by_task): (String, String) = w.conn.query_row("SELECT name, task FROM trial_name", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    assert_eq!((name.as_str(), by_task.as_str()), ("Casa Hogar San José", task.as_str()), "the change carries the id of its task");

    println!("Lo que quedó en la auditoría de la tarea:");
    let trail = w.trail(&task);
    for (event, capability, code) in &trail {
        println!("  {event:<9} {:<13} {}", capability.as_deref().unwrap_or(""), code.as_deref().unwrap_or(""));
    }
    assert_eq!(
        trail,
        vec![
            line("open", None, "screen"),
            line("step", Some("trial.rename"), "ok"),
            line("approval", Some("trial.rename"), "asked"),
            line("pause", None, "approval"),
            line("approval", Some("trial.rename"), "approved"),
            line("step", Some("trial.rename"), "ok"),
            line("close", None, "done"),
        ]
    );
    let delivered = w.window.0.borrow();
    assert_eq!(delivered.len(), 2, "each outcome is also handed to the window");
    assert!(delivered.iter().all(|o| o.task == task));
}

#[test]
fn a_query_hands_its_data_without_approval() {
    let w = world();
    let o = w.invoke(None, "trial.read", json!({}));
    assert_eq!(o.state, TaskState::Done);
    assert_eq!(o.body, Body::Data { capability: "trial.read".into(), value: json!({ "name": "Casa Hogar" }) });
    assert_eq!(o.step, 1);
}

#[test]
fn a_change_that_needs_no_approval_is_prepared_and_applied_at_once() {
    let w = world();
    let o = w.invoke(None, "trial.rename_now", json!({ "name": "Otra" }));
    assert_eq!(o.state, TaskState::Done);
    assert_eq!(w.name(), "Otra");
    assert_eq!(w.task(&o.task).steps, 2, "prepare and apply");
}

#[test]
fn every_close_is_recorded_with_its_reason_and_no_personal_data() {
    let w = world();
    let done = w.invoke(None, "trial.read", json!({}));
    let denied = w.invoke(None, "trial.admin", json!({}));
    let error = w.invoke(None, "trial.broken", json!({}));
    let paused = w.invoke(None, "trial.rename", json!({ "name": "Rosa Hernández López" }));
    let limit = w.invoke(None, "trial.count", json!({ "n": 1 }));
    let limit = w.answer(&limit.task, json!({ "n": 1 }));

    let closes: Vec<(TaskState, Option<String>)> = [&done, &denied, &error, &paused, &limit].iter().map(|o| {
        let t = w.task(&o.task);
        (t.state, t.reason)
    }).collect();
    assert_eq!(
        closes,
        vec![
            (TaskState::Done, Some("done".into())),
            (TaskState::Denied, Some("no_permission".into())),
            (TaskState::Error, Some("capability_failed".into())),
            (TaskState::Paused, Some("approval".into())),
            (TaskState::Limit, Some("limit_repeated".into())),
        ]
    );
    assert_eq!(denied.body, Body::Error { code: "no_permission".into(), next: Next::AskAdministrator });
    assert!(w.trail(&denied.task).contains(&line("denied", Some("trial.admin"), "no_permission")));
    assert!(w.trail(&paused.task).contains(&line("pause", None, "approval")));

    // the trail holds ids and fixed codes: never the name the person wrote
    let trail: Vec<String> = w.conn.prepare("SELECT event || ' ' || ifnull(capability, '') || ' ' || ifnull(code, '') FROM ai_event").unwrap()
        .query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    assert!(trail.iter().all(|l| l.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c == '.' || c == ' ')), "{trail:?}");
    assert!(!trail.join(" ").contains("Rosa"));
}

#[test]
fn a_task_that_does_not_advance_is_cut_by_its_steps() {
    let w = world();
    let mut o = w.invoke(None, "trial.count", json!({ "n": 1 }));
    let limit = Limits::of(&Channel::Screen { place: String::new() }).steps;
    for n in 2..=limit + 1 {
        assert_eq!(o.state, TaskState::Paused);
        o = w.answer(&o.task, json!({ "n": n }));
    }
    assert_eq!(o.state, TaskState::Limit);
    assert_eq!(o.body, Body::Error { code: Reached::Steps.code().into(), next: Next::StartAgain });
    assert_eq!(w.task(&o.task).steps, limit);
}

#[test]
fn the_same_capability_with_the_same_input_twice_closes_the_task() {
    let w = world();
    let o = w.invoke(None, "trial.count", json!({ "n": 5 }));
    let o = w.answer(&o.task, json!({}));
    assert_eq!((o.state, w.task(&o.task).reason), (TaskState::Limit, Some("limit_repeated".into())));
    assert_eq!(w.task(&o.task).steps, 1, "the repeated step never ran");
}

#[test]
fn corrections_have_a_limit() {
    let w = world();
    let long = |i: usize| format!("{}{i}", "x".repeat(50));
    let mut o = w.invoke(None, "trial.rename", json!({ "name": long(0) }));
    assert!(matches!(&o.body, Body::Clarification { ask: Ask::Correct { field, .. }, .. } if field == "name"));
    for i in 1..=3 {
        o = w.answer(&o.task, json!({ "name": long(i) }));
        assert_eq!(o.state, TaskState::Paused, "correction {i}");
    }
    let o = w.answer(&o.task, json!({ "name": long(4) }));
    assert_eq!((o.state, w.task(&o.task).reason), (TaskState::Limit, Some("limit_corrections".into())));
}

#[test]
fn an_input_that_does_not_fit_its_contract_never_runs() {
    let w = world();
    let o = w.invoke(None, "trial.rename", json!({ "nombre": "Otra" }));
    assert_eq!(o.body, Body::Clarification { capability: "trial.rename".into(), ask: Ask::Correct { field: "name".into(), reason: "does_not_fit".into() } });
    assert_eq!(w.task(&o.task).steps, 0);
    // corrected, it goes on
    let o = w.answer(&o.task, json!({ "name": "Otra" }));
    assert!(matches!(o.body, Body::Proposal { .. }));
}

#[test]
fn a_pause_runs_out() {
    let w = world();
    let o = w.invoke(None, "trial.rename", json!({ "name": "" }));
    assert_eq!(o.body, Body::Clarification { capability: "trial.rename".into(), ask: Ask::Missing { what: "the new name".into() } });
    w.clock.advance(Limits::of(&o.channel).pause_ms + 1);
    let o = w.answer(&o.task, json!({ "name": "Otra" }));
    assert_eq!((o.state, w.task(&o.task).reason), (TaskState::Limit, Some("limit_pause".into())));
    assert_eq!(w.name(), "Casa Hogar");
}

#[test]
fn a_turn_has_a_time_limit_and_spending_is_reserved_first() {
    let mut task = Task::open("t".into(), "org", "ana", Channel::Chat, 0);
    let limits = task.limits();
    assert_eq!(task.may_step(limits.turn_ms), Ok(()));
    assert_eq!(task.may_step(limits.turn_ms + 1), Err(Reached::Time));

    // two calls at once cannot pass the limit together: the second one is stopped by what the first reserved
    let half = limits.spend_mxn / 2.0;
    assert_eq!(task.reserve(half + 0.1), Ok(()));
    assert_eq!(task.reserve(half), Err(Reached::Spend));
    task.settle(half + 0.1, 0.3);
    assert_eq!(task.reserve(half), Ok(()));
    assert_eq!(task.reserve(limits.spend_mxn), Err(Reached::Spend));
    // a screen spends nothing on models
    let mut screen = Task::open("s".into(), "org", "ana", Channel::Screen { place: String::new() }, 0);
    assert_eq!(screen.reserve(0.01), Err(Reached::Spend));
}

#[test]
fn new_parameters_make_the_earlier_approval_stale() {
    let w = world();
    let first = w.invoke(None, "trial.rename", json!({ "name": "Uno" }));
    let task = first.task.clone();
    let second = w.invoke(Some(&task), "trial.rename", json!({ "name": "Dos" }));
    assert!(matches!(second.body, Body::Proposal { again: Some(Again::Changed), .. }), "{second:?}");

    // approving what was shown first does not apply it: the proposal waiting is shown again
    let o = w.decide(&task, &fingerprint_of(&first), true);
    assert!(matches!(o.body, Body::Proposal { again: Some(Again::Outdated), .. }), "{o:?}");
    assert_eq!(fingerprint_of(&o), fingerprint_of(&second));
    assert_eq!(w.name(), "Casa Hogar");

    let o = w.decide(&task, &fingerprint_of(&second), true);
    assert_eq!(o.state, TaskState::Done);
    assert_eq!(w.name(), "Dos");
    let states: Vec<String> = w.conn.prepare("SELECT state FROM ai_approval ORDER BY asked_ms").unwrap()
        .query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(states, vec!["stale", "approved"]);
}

#[test]
fn the_same_proposal_asked_again_is_shown_again_without_running() {
    let w = world();
    let first = w.invoke(None, "trial.rename", json!({ "name": "Uno" }));
    let again = w.invoke(Some(&first.task), "trial.rename", json!({ "name": "Uno" }));
    assert_eq!(fingerprint_of(&again), fingerprint_of(&first));
    assert_eq!(w.task(&first.task).steps, 1);
}

#[test]
fn rejecting_a_proposal_changes_nothing() {
    let w = world();
    let asked = w.invoke(None, "trial.rename", json!({ "name": "Otra" }));
    let o = w.decide(&asked.task, &fingerprint_of(&asked), false);
    assert_eq!(o.state, TaskState::Done);
    assert_eq!(o.body, Body::State { reason: "rejected".into() });
    assert_eq!(w.name(), "Casa Hogar");
    assert!(w.trail(&asked.task).contains(&line("approval", Some("trial.rename"), "rejected")));
}

#[test]
fn the_permission_is_checked_again_before_applying() {
    let w = world();
    let asked = w.invoke(None, "trial.rename", json!({ "name": "Otra" }));
    w.session.permissions.borrow_mut().clear();
    let o = w.decide(&asked.task, &fingerprint_of(&asked), true);
    assert_eq!(o.state, TaskState::Denied);
    assert_eq!(w.name(), "Casa Hogar");
}

#[test]
fn an_apply_that_never_ended_is_not_run_again() {
    let w = world();
    let asked = w.invoke(None, "trial.rename", json!({ "name": "Otra" }));
    // the app closed while the change ran: the step stayed «started»
    store::start_step(&w.conn, &asked.task, 2, "trial.rename", super::task::Phase::Apply, "h", 0).unwrap();
    let o = w.decide(&asked.task, &fingerprint_of(&asked), true);
    assert_eq!(o.state, TaskState::Error);
    assert_eq!(o.body, Body::Error { code: "uncertain_change".into(), next: Next::CheckHistory });
    assert_eq!(w.name(), "Casa Hogar", "it is never repeated blindly");
    let result: String = w.conn.query_row("SELECT result FROM ai_step WHERE task_id = ?1 AND n = 2", [&asked.task], |r| r.get(0)).unwrap();
    assert_eq!(result, "uncertain");
}

#[test]
fn a_task_of_another_account_is_not_touched() {
    let w = world();
    let asked = w.invoke(None, "trial.rename", json!({ "name": "Otra" }));
    let luis = Who { account: "luis", organization: "org" };
    let o = w.submit(&luis, Some(&asked.task), Content::Decision { fingerprint: fingerprint_of(&asked), approve: true });
    assert_ne!(o.task, asked.task);
    assert_eq!(o.state, TaskState::Denied);
    assert_eq!(w.task(&asked.task).state, TaskState::Paused);
    let runtime = Runtime { registry: &w.registry, clock: &w.clock, authorize: &w.session, deliver: &w.window };
    assert!(runtime.task(&w.conn, &luis, &asked.task).unwrap().is_none(), "nor can it be read");
    assert_eq!(runtime.task(&w.conn, &ANA, &asked.task).unwrap(), Some(asked));
}

#[test]
fn a_closed_task_does_not_move() {
    let w = world();
    let done = w.invoke(None, "trial.read", json!({}));
    let again = w.invoke(Some(&done.task), "trial.rename_now", json!({ "name": "Otra" }));
    assert_eq!(again, done);
    assert_eq!(w.name(), "Casa Hogar");
}

#[test]
fn the_chat_without_a_model_says_how_to_go_on() {
    let w = world();
    let o = w.submit(&ANA, None, Content::Text { text: "¿Qué me falta?".into() });
    assert_eq!(o.body, Body::Error { code: "without_ai".into(), next: Next::UseScreen });
    assert!(o.state.is_closed());
}

#[test]
fn a_paused_task_survives_reopening_the_app() {
    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.db");
    let (task, fp) = {
        let mut conn = crate::storage::open_encrypted(&path, KEY).unwrap();
        migrate(&mut conn).unwrap();
        trial_table(&conn);
        let w = World::on(conn);
        let asked = w.invoke(None, "trial.rename", json!({ "name": "Otra" }));
        (asked.task.clone(), fingerprint_of(&asked))
    };
    // the app opens again: a new connection, a new runtime, the same task
    let mut conn = crate::storage::open_encrypted(&path, KEY).unwrap();
    migrate(&mut conn).unwrap();
    let w = World::on(conn);
    assert_eq!(w.task(&task).state, TaskState::Paused);
    let o = w.decide(&task, &fp, true);
    assert_eq!(o.state, TaskState::Done);
    assert_eq!(w.name(), "Otra");
}

#[test]
fn the_tables_of_the_platform_keep_the_shape_of_an_owner() {
    let mut conn = Connection::open_in_memory().unwrap();
    assert_eq!(kernel::conform(store::OWNER, store::MIGRATIONS, &mut conn), vec![]);
}

#[test]
fn the_fingerprint_does_not_depend_on_the_order_of_the_keys() {
    let a = super::executor::fingerprint("m.c", &json!({ "a": 1, "b": { "x": [1, 2], "y": null } }));
    let b = super::executor::fingerprint("m.c", &serde_json::from_str::<Value>(r#"{"b":{"y":null,"x":[1,2]},"a":1}"#).unwrap());
    assert_eq!(a, b);
    assert_ne!(a, super::executor::fingerprint("m.c", &json!({ "a": 2, "b": { "x": [1, 2], "y": null } })));
    assert_ne!(a, super::executor::fingerprint("m.d", &json!({ "a": 1, "b": { "x": [1, 2], "y": null } })));
}

#[test]
fn a_request_and_an_outcome_read_as_the_screens_write_them() {
    let r: Request = serde_json::from_value(json!({
        "channel": { "kind": "screen", "place": "institution" },
        "content": { "kind": "invoke", "capability": "trial.read", "input": {} }
    }))
    .unwrap();
    assert_eq!(r.channel, Channel::Screen { place: "institution".into() });
    assert!(r.task.is_none());
    let o = world().invoke(None, "trial.rename", json!({ "name": "Otra" }));
    let v = serde_json::to_value(&o).unwrap();
    assert_eq!(v["state"], "paused");
    assert_eq!(v["body"]["kind"], "proposal");
    assert_eq!(v["body"]["again"], Value::Null);
    assert_eq!(v["channel"], json!({ "kind": "screen", "place": "institution" }));
}
