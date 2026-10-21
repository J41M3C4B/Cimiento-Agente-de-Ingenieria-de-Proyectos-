//! The kernel of the ERP (ADR-002, the ERP blueprint §5): what every module needs and none owns. It sits under the
//! modules and knows none of them, not even by name; what it needs of a module, the module declares in its manifest.
//!
//! In this first step it holds the skeleton: the manifest of a module, the registry of modules (K12), the migrations
//! of each owner of tables, every module and the AI platform (K4), and the vocabulary of permissions (K2).
//!
//! A module declares itself with `module!`, and the registry finds it when the program is linked, with no list
//! (compiled, not run: a documentation example does not link SQLCipher on Windows):
//!
//! ```no_run
//! kernel::module! {
//!     static NOTES = kernel::Manifest {
//!         id: "notas",
//!         name: "Notas",
//!         version: "0.1.0",
//!         depends_on: &[],
//!         permissions: &[kernel::Permission { id: "notas.escribir", description: "Escribir notas." }],
//!         migrations: &[kernel::Migration { version: 1, sql: "CREATE TABLE notas_note (id TEXT PRIMARY KEY);" }],
//!         on: true,
//!     };
//! }
//!
//! let modules = kernel::Modules::discover().unwrap();
//! assert!(modules.is_on("notas"));
//! ```
//!
//! The kernel cannot use a module: it does not depend on any, so this does not compile.
//!
//! ```compile_fail
//! use muestra as _;
//! ```

mod access;
mod conformance;
mod migrate;
mod module;
mod registry;

pub use access::Permission;
pub use conformance::{conform, Breach};
pub use migrate::{migrate, migrate_owner, MIGRATIONS_TABLE};
pub use module::{Manifest, Migration};
pub use registry::{Fault, ManifestError, Modules, MODULES};

// what `module!` writes needs this from here, wherever it is used
#[doc(hidden)]
pub use linkme;

/// Declares the manifest of a module, for the registry to find it when the program is linked.
#[macro_export]
macro_rules! module {
    (static $name:ident = $manifest:expr;) => {
        #[$crate::linkme::distributed_slice($crate::MODULES)]
        #[linkme(crate = $crate::linkme)]
        static $name: $crate::Manifest = $manifest;
    };
}
