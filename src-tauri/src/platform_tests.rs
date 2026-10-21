//! The AI platform with the modules the app links (ADR-003): the capabilities of a module, declared from its own crate
//! with a permission it defines, go through the runtime from end to end, and the kernel decides on each step (K2).

use std::cell::Cell;

use capability::Registry;
use kernel::Modules;
use platform::outcome::Body;
use platform::request::{Channel, Content};
use platform::task::TaskState;
use platform::{Accounts, Clock, Deliver, Outcome, Request, Runtime, Who};
use rusqlite::Connection;
use serde_json::json;

const ANA: &str = "usr_01JABCDEFGHJKMNPQRSTVWXYZ0";

/// Ana, with the session open and the role a test gives her.
struct Ana(Cell<&'static str>);

impl Accounts for Ana {
    fn now(&self, who: &Who) -> Option<kernel::Account> {
        (who.account == ANA).then(|| kernel::Account {
            id: ANA.into(),
            organization: "org".into(),
            role: self.0.get().into(),
            active: true,
            must_change_password: false,
        })
    }
}

struct Ticks(Cell<i64>);

impl Clock for Ticks {
    fn now_ms(&self) -> i64 {
        self.0.set(self.0.get() + 1);
        self.0.get()
    }
}

struct Nowhere;

impl Deliver for Nowhere {
    fn deliver(&self, _: &Outcome) {}
}

/// Every capability the app links asks for a permission the kernel or a module defines: the app starts.
#[test]
fn every_capability_asks_for_a_permission_that_exists() {
    let registry = Registry::discover().unwrap();
    let modules = Modules::discover().unwrap();
    assert!(registry.get("muestra.leer_nota").is_some(), "the sample module publishes from its own crate");
    assert_eq!(platform::unknown_permissions(&registry, &modules), vec![]);
}

#[test]
fn a_capability_of_a_module_runs_from_end_to_end() {
    let mut conn = Connection::open_in_memory().unwrap();
    let modules = Modules::discover().unwrap();
    kernel::migrate(&mut conn, &modules).unwrap();
    platform::migrate(&mut conn).unwrap();
    let registry = Registry::discover().unwrap();
    let ana = Ana(Cell::new("manager"));
    let runtime = Runtime { registry: &registry, modules: &modules, accounts: &ana, clock: &Ticks(Cell::new(0)), deliver: &Nowhere };
    let who = Who { account: ANA, organization: "org" };

    // a note the person saved on the screen, through the kernel (ERP-9)
    let id = kernel::new_id("nota");
    let note = muestra::contract::NoteInput { id: id.clone(), title: "Primera nota".into(), body: None, phone: Some("55 1234 5678".into()) };
    muestra::service::apply(&mut conn, &note, None, &kernel::Origin::Person { account: ANA.into() }).unwrap();
    let read = || Content::Invoke { capability: "muestra.leer_nota".into(), input: json!({ "id": id }) };
    let submit = |task: Option<&str>, content: Content| {
        let request = Request { channel: Channel::Screen { place: "muestra".into() }, task: task.map(String::from), content };
        runtime.submit(&conn, &who, request).unwrap()
    };

    // the direction does not have the permission the module defines: the kernel says no
    let denied = submit(None, read());
    assert_eq!(denied.state, TaskState::Denied);
    assert!(matches!(&denied.body, Body::Error { code, .. } if code == "access_denied"), "{denied:?}");

    // the administration does: the module reads the note, without its protected phone
    ana.0.set("admin");
    let done = submit(None, read());
    assert_eq!(done.state, TaskState::Done, "{done:?}");
    let Body::Data { capability, value } = &done.body else { panic!("{done:?}") };
    assert_eq!(capability, "muestra.leer_nota");
    assert_eq!(value, &json!({ "id": id, "title": "Primera nota", "body": null, "version": 1 }));

    // and the kernel's audit store tells the life of both tasks, for whom and why
    let moments: Vec<(String, Option<String>)> = conn
        .prepare("SELECT event, reason FROM kernel_audit WHERE correlation_id IN (?1, ?2) ORDER BY rowid")
        .unwrap()
        .query_map([&denied.task, &done.task], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert!(moments.contains(&("task.denied".into(), Some("access_denied".into()))), "{moments:?}");
    assert!(moments.contains(&("task.step".into(), None)), "{moments:?}");
}
