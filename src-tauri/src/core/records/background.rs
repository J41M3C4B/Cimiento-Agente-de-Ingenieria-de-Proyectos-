//! The documents of the Expediente being read in the background (ADR-038): a file is read while the person goes on
//! working, even with its window closed, and only one reading of a document runs at a time. Kept in memory only: a
//! reading does not outlive the program (decision of the institution, 2026-10-13).

use std::collections::HashSet;
use std::sync::Mutex;

static READING: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// What names a document while it is read: its type of the catalog, or `other:<id>` for one the institution named.
pub fn key(kind: &str, id: Option<&str>) -> String {
    match (kind, id) {
        ("other", Some(id)) => format!("other:{id}"),
        _ => kind.to_string(),
    }
}

/// A reading under way: the document is free again when this is dropped, whatever happens to the reading.
#[derive(Debug)]
pub struct Reading(String);

impl Drop for Reading {
    fn drop(&mut self) {
        if let Ok(mut set) = READING.lock() {
            if let Some(set) = set.as_mut() {
                set.remove(&self.0);
            }
        }
    }
}

/// Starts reading a document, or `None` when it is already being read.
pub fn start(key: &str) -> Option<Reading> {
    let mut set = READING.lock().ok()?;
    set.get_or_insert_with(HashSet::new).insert(key.to_string()).then(|| Reading(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_is_read_once_at_a_time_and_is_free_again_when_it_ends() {
        assert_eq!(key("articles_of_incorporation", None), "articles_of_incorporation");
        assert_eq!(key("other", Some("rec_1")), "other:rec_1");
        let first = start("test_reading_power_of_attorney").expect("free");
        assert!(start("test_reading_power_of_attorney").is_none(), "already being read");
        assert!(start("test_reading_other_kind").is_some(), "another document is read at the same time");
        drop(first);
        assert!(start("test_reading_power_of_attorney").is_some(), "free again");
    }
}
