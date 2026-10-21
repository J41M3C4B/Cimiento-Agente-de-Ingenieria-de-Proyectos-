//! What the sample module publishes (the ERP blueprint §4, `capabilities`): its capabilities, declared from its own
//! crate with the contract (ADR-001) and with a permission it defines in its manifest (ADR-003 A2). The platform
//! finds them in the registry and invokes them through its executor; the module never knows the platform.
//!
//! The one that changes a note comes with IA-7: a change goes through `kernel::apply` (ERP-9), and the contract does
//! not give it a connection it can open a transaction on yet.

use capability::{Call, CapabilityError, Limits};

use crate::contract::NoteRef;

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct WhichNote {
    /// The id of the note.
    pub id: String,
}

/// A note as the person reads it. Its phone is protected and never leaves this way.
#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub version: i64,
}

fn read(call: &Call, which: WhichNote) -> Result<Note, CapabilityError> {
    let note = crate::storage::find(call.conn, &NoteRef(which.id.clone()))
        .map_err(|_| CapabilityError::Failed { reason: "the note could not be read" })?
        .ok_or(CapabilityError::Invalid { field: "id", reason: "not_found" })?;
    Ok(Note { id: which.id, title: note.title, body: note.body, version: note.version })
}

capability::capability! {
    static READ_NOTE = "muestra.leer_nota" {
        description: "Lee una nota de la muestra.",
        effect: Query,
        permission: "muestra.notas.leer",
        data: [Internal],
        approval: Never,
        limits: Limits::READ,
        run: read(read),
    }
}
