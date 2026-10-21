//! The sample module: the living reference of the shape of a module (the ERP blueprint §4). It is linked only in the
//! tests. Its manifest is what the registry reads; its contract (`muestra-contract`) is the only part of it another
//! module may use; its tables start with `muestra_`, and only it writes SQL on them.
//!
//! It grows with the kernel: ERP-9 prepares and applies its changes, ERP-2 adds its part of the dictionary. Its first
//! capability came with IA-5, a query with a permission of its own; IA-7 adds the one that changes a note through the
//! kernel, and ERP-1 adds them to the conformance test.

pub mod capabilities;
pub mod service;
pub mod storage;

pub use muestra_contract as contract;

use kernel::{Manifest, Migration, Permission};

kernel::module! {
    static MANIFEST = Manifest {
        id: "muestra",
        name: "Muestra",
        version: env!("CARGO_PKG_VERSION"),
        depends_on: &[],
        permissions: &[
            Permission { id: "muestra.notas.leer", description: "Leer las notas de la muestra." },
            Permission { id: "muestra.notas.escribir", description: "Escribir las notas de la muestra." },
        ],
        migrations: &[
            Migration { version: 1, sql: include_str!("../migrations/0001_note.sql") },
            Migration { version: 2, sql: include_str!("../migrations/0002_note_version.sql") },
        ],
        on: true,
    };
}
