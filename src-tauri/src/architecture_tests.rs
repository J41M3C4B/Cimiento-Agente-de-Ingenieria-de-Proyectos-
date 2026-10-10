//! The layers of the app (ADR-032) and the one test that keeps them. Every file belongs to a layer, and a layer only
//! looks down: the base knows no business; a module (staff, people served, facilities, finances) uses only the base;
//! the core uses the base and the `api`, `service` and `domain` of the modules; the projects use the base and
//! `core::api`; the commands use anything.
//!
//! What breaks the rule today is written down in `DEBT`, block by block of ADR-032, and the list may only shrink:
//! the test also fails when a line of it is no longer needed.

use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    Base,
    Module(&'static str),
    Core,
    Projects,
    Shell,
}

use Layer::*;

/// Where each piece lives, by module path; the longest prefix wins. As the blocks of ADR-032 move files into
/// `core/` and `modules/`, the lines of the old places go away.
const PLACES: &[(&str, Layer)] = &[
    // base
    ("ai", Base),
    ("audit", Base),
    ("common", Base),
    ("documents", Base),
    ("scanner", Base),
    ("storage", Base),
    // modules
    // only its mod.rs, which declares them; a new module needs its own line
    ("modules", Base),
    ("modules::hr", Module("hr")),
    ("modules::care", Module("care")),
    ("modules::facilities", Module("facilities")),
    ("modules::finance", Module("finance")),
    // core
    ("core", Core),
    // projects
    ("modules::projects", Projects),
    // shell
    ("", Shell),
    ("commands", Shell),
    ("error", Shell),
    ("main", Shell),
];

/// What breaks the rule today: (the file, as a module path; what it uses). Each block of ADR-032 empties its part.
const DEBT: &[(&str, &str)] = &[];

fn segments(path: &str) -> Vec<&str> {
    path.split("::").filter(|s| !s.is_empty()).collect()
}

/// The layer of a module path, and what comes after the place that matched.
fn place(path: &str) -> Option<(Layer, Vec<String>)> {
    let segs = segments(path);
    // a type of the root (`crate::Db`) belongs to the shell
    if segs.first().is_some_and(|s| s.starts_with(|c: char| c.is_ascii_uppercase())) {
        return Some((Shell, segs.iter().map(|s| s.to_string()).collect()));
    }
    PLACES
        .iter()
        .filter(|(p, _)| {
            let ps = segments(p);
            // the root (`lib.rs`) is only itself, never a prefix of everything
            if ps.is_empty() {
                return segs.is_empty();
            }
            ps.len() <= segs.len() && ps.iter().zip(&segs).all(|(a, b)| a == b)
        })
        .max_by_key(|(p, _)| segments(p).len())
        .map(|(p, layer)| (*layer, segs[segments(p).len()..].iter().map(|s| s.to_string()).collect()))
}

/// Whether `from` may use `target` (whose layer is `to` and whose path after its place is `rest`).
fn allowed(from_path: &str, from: Layer, to: Layer, rest: &[String]) -> bool {
    if from == to {
        return true;
    }
    let first = rest.first().map(String::as_str).unwrap_or("");
    let a_type = first.starts_with(|c: char| c.is_ascii_uppercase());
    match (from, to) {
        (Shell, _) => true,
        (_, Base) => true,
        // a move of old data into a module is a migration (ADR-027, 029, 030)
        (Base, Module(_)) => from_path.starts_with("storage::migrations") && (first == "legacy" || a_type),
        (Core, Module(_)) => a_type || matches!(first, "api" | "service" | "domain"),
        (Projects, Core) => first == "api",
        _ => false,
    }
}

/// Every `crate::…` path a line names; `crate::a::{b, c::d}` names `a::b` and `a::c::d`.
fn paths_in(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let ident = |s: &str| s.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').count();
    let mut rest = line;
    while let Some(at) = rest.find("crate::") {
        let mut tail = &rest[at + 7..];
        let mut path = Vec::new();
        loop {
            let n = ident(tail);
            if n == 0 {
                break;
            }
            path.push(&tail[..n]);
            tail = &tail[n..];
            if let Some(t) = tail.strip_prefix("::") {
                tail = t;
            } else {
                break;
            }
        }
        if let Some(group) = tail.strip_prefix('{') {
            let inner = group.split('}').next().unwrap_or("");
            for item in inner.split(',') {
                let item = item.trim().split(" as ").next().unwrap_or("").trim();
                if !item.is_empty() && item != "self" {
                    out.push(format!("{}::{}", path.join("::"), item));
                }
            }
        }
        if !path.is_empty() {
            out.push(path.join("::"));
        }
        rest = tail;
    }
    out
}

/// The lines of a file that count: the code that is not a test. A `#[cfg(test)]` at the start of a line takes the
/// next item out (one line if it ends in `;`, else everything until its closing brace at the start of a line).
fn code_lines(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut lines = text.lines().enumerate().peekable();
    while let Some((n, line)) = lines.next() {
        if line.trim_start() == "#[cfg(test)]" {
            let indent = line.len() - line.trim_start().len();
            let Some((_, next)) = lines.next() else { break };
            if next.trim_end().ends_with(';') {
                continue;
            }
            let close = format!("{}}}", " ".repeat(indent));
            for (_, l) in lines.by_ref() {
                if l.trim_end() == close {
                    break;
                }
            }
            continue;
        }
        out.push((n + 1, line));
    }
    out
}

fn module_path(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap().with_extension("");
    let mut segs: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    if matches!(segs.last().map(String::as_str), Some("mod" | "lib")) {
        segs.pop();
    }
    segs.join("::")
}

fn is_test_file(file: &Path) -> bool {
    let name = file.file_stem().unwrap().to_string_lossy();
    name == "tests" || name.ends_with("_tests") || name == "test_support"
}

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Every use that breaks the rule: (file, what it uses, where).
fn violations() -> (BTreeSet<(String, String)>, Vec<String>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut found = BTreeSet::new();
    let mut lost = Vec::new();
    for file in files.iter().filter(|f| !is_test_file(f)) {
        let from_path = module_path(&root, file);
        let Some((from, _)) = place(&from_path) else {
            lost.push(format!("{} has no layer: add it to PLACES", file.display()));
            continue;
        };
        let text = std::fs::read_to_string(file).unwrap();
        for (_, line) in code_lines(&text) {
            for target in paths_in(line) {
                let Some((to, rest)) = place(&target) else {
                    lost.push(format!("{from_path} uses crate::{target}, which has no layer"));
                    continue;
                };
                if !allowed(&from_path, from, to, &rest) {
                    // the debt names the place used and, inside a module, which part of it
                    let segs = segments(&target);
                    let mut cut = segs.len() - rest.len();
                    if matches!(to, Module(_)) && rest.first().is_some_and(|r| r.starts_with(|c: char| c.is_ascii_lowercase())) {
                        cut += 1;
                    }
                    found.insert((from_path.clone(), segs[..cut].join("::")));
                }
            }
        }
    }
    (found, lost)
}

#[test]
fn every_layer_only_looks_down() {
    let (found, lost) = violations();
    assert!(lost.is_empty(), "{lost:#?}");
    let debt: BTreeSet<(String, String)> = DEBT.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    let new: Vec<_> = found.difference(&debt).collect();
    let paid: Vec<_> = debt.difference(&found).collect();
    let listing: String = found.iter().map(|(a, b)| format!("    (\"{a}\", \"{b}\"),\n")).collect();
    assert!(new.is_empty(), "a layer looks up or sideways (ADR-032): {new:#?}\n\nthe whole list today:\n{listing}");
    assert!(paid.is_empty(), "this debt is paid, take it out of DEBT: {paid:#?}");
}

/// The tables of the core and of each module (`docs/05-modelo-datos.md`, «Dueño de cada tabla»). Only their owner
/// writes SQL on them; the rest ask through its `api` (audit D2: the projects read the profile with SQL and no `use`
/// showed it). The tables of a module go by their prefix.
const CORE_TABLES: &[&str] = &[
    "institution", "institution_profile", "population_group", "staff_group", "document", "document_chunk",
    "document_chunk_fts", "app_user", "access_request", "roster_field", "roster_entry", "income_source", "expense_item",
    "ai_proposal",
];
const MODULE_PREFIXES: &[(&str, &str)] = &[("hr_", "hr"), ("care_", "care"), ("fac_", "facilities"), ("fin_", "finance")];

/// What SQL on someone else's tables is still allowed today: (the file, the table). It may only shrink, like `DEBT`.
const TABLE_DEBT: &[(&str, &str)] = &[];

fn table_owner(table: &str) -> Option<Layer> {
    if CORE_TABLES.contains(&table) {
        return Some(Core);
    }
    MODULE_PREFIXES.iter().find(|(p, _)| table.starts_with(p)).map(|(_, m)| Module(m))
}

/// The tables a line of SQL names after FROM, JOIN, INTO, UPDATE or TABLE.
fn tables_in(line: &str) -> Vec<String> {
    let words: Vec<&str> = line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).filter(|w| !w.is_empty()).collect();
    words
        .windows(2)
        .filter(|w| matches!(w[0], "FROM" | "JOIN" | "INTO" | "UPDATE" | "TABLE"))
        .map(|w| w[1].to_string())
        .collect()
}

/// Every SQL that touches a table of another layer: (file, table). The base storage (migrations, backup, the scan of
/// the whole base) and the shell may touch any.
fn foreign_tables() -> BTreeSet<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut found = BTreeSet::new();
    for file in files.iter().filter(|f| !is_test_file(f)) {
        let from_path = module_path(&root, file);
        let Some((from, _)) = place(&from_path) else { continue };
        if from == Shell || from_path.starts_with("storage") {
            continue;
        }
        let text = std::fs::read_to_string(file).unwrap();
        for (_, line) in code_lines(&text) {
            for table in tables_in(line) {
                if table_owner(&table).is_some_and(|owner| owner != from) {
                    found.insert((from_path.clone(), table));
                }
            }
        }
    }
    found
}

#[test]
fn only_the_owner_of_a_table_writes_sql_on_it() {
    let found = foreign_tables();
    let debt: BTreeSet<(String, String)> = TABLE_DEBT.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    let new: Vec<_> = found.difference(&debt).collect();
    let paid: Vec<_> = debt.difference(&found).collect();
    let listing: String = found.iter().map(|(a, b)| format!("    (\"{a}\", \"{b}\"),
")).collect();
    assert!(new.is_empty(), "SQL on a table of another layer (ADR-032): {new:#?}

the whole list today:
{listing}");
    assert!(paid.is_empty(), "this debt is paid, take it out of TABLE_DEBT: {paid:#?}");
}

#[test]
fn tables_are_read_from_a_line() {
    assert_eq!(tables_in("SELECT a FROM institution_profile p JOIN hr_job j ON 1"), vec!["institution_profile", "hr_job"]);
    assert_eq!(tables_in("\"INSERT INTO document (id) VALUES (?1)\","), vec!["document"]);
    assert!(tables_in("let from = 1;").is_empty());
    assert_eq!(table_owner("care_person"), Some(Module("care")));
    assert_eq!(table_owner("project"), None);
}

#[test]
fn paths_are_read_from_a_line() {
    assert_eq!(paths_in("use crate::modules::hr::api::{Count, MIN_GROUP};"), vec!["modules::hr::api::Count", "modules::hr::api::MIN_GROUP", "modules::hr::api"]);
    assert_eq!(paths_in("let x = crate::modules::care::api::waiting(conn)?;"), vec!["modules::care::api::waiting"]);
    assert_eq!(paths_in("use crate::domain::{stage, profile::ProfileInput as P};"), vec!["domain::stage", "domain::profile::ProfileInput", "domain"]);
    assert!(paths_in("let a = 1;").is_empty());
}

#[test]
fn the_layer_rules() {
    let rest = |s: &str| segments(s).iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert!(allowed("hr::service", Module("hr"), Base, &rest("audit")));
    assert!(!allowed("hr::service", Module("hr"), Module("care"), &rest("api")));
    assert!(!allowed("hr::service", Module("hr"), Core, &rest("")));
    assert!(allowed("core::home", Core, Module("care"), &rest("api::indicators")));
    assert!(allowed("core::home", Core, Module("care"), &rest("CareError")));
    assert!(!allowed("core::home", Core, Module("care"), &rest("storage::Group")));
    assert!(allowed("modules::projects::guide", Projects, Core, &rest("api::sheet")));
    assert!(!allowed("modules::projects::guide", Projects, Core, &rest("institution")));
    assert!(!allowed("modules::projects::guide", Projects, Module("facilities"), &rest("api")));
    assert!(!allowed("scanner::guard", Base, Core, &rest("")));
    assert!(allowed("storage::migrations", Base, Module("hr"), &rest("legacy")));
    assert!(!allowed("storage::backup", Base, Module("hr"), &rest("legacy")));
    assert!(allowed("commands::hr", Shell, Module("hr"), &rest("storage")));
    assert_eq!(place("core::profile::storage::load_current").map(|p| p.0), Some(Core));
    assert_eq!(place("storage::open_encrypted").map(|p| p.0), Some(Base));
    assert_eq!(place("modules::finance::api").map(|p| p.0), Some(Module("finance")));
}
