//! The published types of the sample module (its `contract`, the ERP blueprint §4): the only part of it another module
//! may use. Changing them is changing the module's version.
//!
//! The contract does not reach into its module: it does not depend on it, so this does not compile.
//!
//! ```compile_fail
//! use muestra as _;
//! ```

use serde::{Deserialize, Serialize};

/// A reference to a note: what another module keeps, never a copy of the note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteRef(pub String);
