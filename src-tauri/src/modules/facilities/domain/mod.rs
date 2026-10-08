//! The rules of the facilities module, without input or output.

pub mod aggregate;
pub mod catalog;
pub mod group;
pub mod site;

use serde::Serialize;

/// Something the person should look at. The screen turns `code` into words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub code: &'static str,
    pub field: String,
    /// `true` blocks saving; `false` is only a heads-up.
    pub blocking: bool,
}

pub(crate) fn code_ok(value: &Option<String>, allowed: &[&str]) -> bool {
    value.as_deref().is_none_or(|v| v.is_empty() || allowed.contains(&v))
}

pub(crate) fn text(o: &Option<String>) -> Option<&str> {
    o.as_deref().map(str::trim).filter(|s| !s.is_empty())
}
