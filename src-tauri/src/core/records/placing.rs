//! A file the code is not sure of (ADR-042 §4): it is not kept on the document it was put on until the person says.
//! While it waits, the file already read stays in memory, so the answer does not read it again (a scan takes
//! minutes); like a reading, it does not outlive the program. One file waits per document: a new one replaces it.

use super::{background, keep, AttachOutcome, FileText};
use crate::audit::{self, AuditKind};
use crate::core::error::ServiceError;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// What the person answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Answer {
    /// «Sí es este documento»: kept where it was put, read with the rules of that document, and written in the log.
    Here,
    /// Kept on the document its words say it is.
    There,
    /// Not kept at all.
    Cancel,
}

/// A file waiting for the person: the document it was put on, where it is on this computer, what was read of it,
/// and the kind its words say, if any.
pub struct Waiting {
    kind: String,
    id: Option<String>,
    path: PathBuf,
    file: FileText,
    looks_like: Option<&'static str>,
}

static WAITING: Mutex<Option<HashMap<String, Waiting>>> = Mutex::new(None);

/// Leaves a file waiting under the key of its document.
pub(super) fn wait(kind: &str, id: Option<&str>, path: PathBuf, file: FileText, looks_like: Option<&'static str>) {
    let waiting = Waiting { kind: kind.to_string(), id: id.map(str::to_string), path, file, looks_like };
    if let Ok(mut all) = WAITING.lock() {
        all.get_or_insert_with(HashMap::new).insert(background::key(kind, id), waiting);
    }
}

/// Takes the file waiting on a document, if there is one.
pub fn take(key: &str) -> Option<Waiting> {
    WAITING.lock().ok()?.as_mut()?.remove(key)
}

/// Leaves a file waiting again, as it was: its document was busy when the person answered.
pub fn put_back(key: &str, waiting: Waiting) {
    if let Ok(mut all) = WAITING.lock() {
        all.get_or_insert_with(HashMap::new).entry(key.to_string()).or_insert(waiting);
    }
}

impl Waiting {
    /// The document it ends on with an answer, and its key: where it was put, or the one its words say.
    pub fn target(&self, answer: Answer) -> Option<(String, String)> {
        match answer {
            Answer::Here => Some((self.kind.clone(), background::key(&self.kind, self.id.as_deref()))),
            Answer::There => self.looks_like.map(|k| (k.to_string(), background::key(k, None))),
            Answer::Cancel => None,
        }
    }
}

/// Keeps a waiting file as the person answered. `Here` reads it with the rules of the document it was put on and
/// writes in the log that the person said so (which documents, never the file); `There` puts it on the document its
/// words say, which replaces the file that document had. `Cancel`, or `There` with no kind to go to, keeps nothing.
pub fn keep_as(conn: &mut Connection, w: Waiting, answer: Answer) -> Result<Option<AttachOutcome>, ServiceError> {
    match answer {
        Answer::Here => {
            let out = keep(conn, &w.kind, w.id.as_deref(), &w.path, w.file)?;
            let id = match &out {
                AttachOutcome::Attached { id, .. } => Some(id.as_str()),
                _ => None,
            };
            audit::record(conn, AuditKind::RecordKindConfirmed, Some("core_record"), id, json!({ "kind": w.kind, "looked_like": w.looks_like }))?;
            Ok(Some(out))
        }
        Answer::There => match w.looks_like {
            Some(other) => Ok(Some(keep(conn, other, None, &w.path, w.file)?)),
            None => Ok(None),
        },
        Answer::Cancel => Ok(None),
    }
}
