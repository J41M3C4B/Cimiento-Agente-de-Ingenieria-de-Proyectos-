//! The capability contract and the registry (ADR-001). A module publishes what it offers as capabilities, each one
//! declared with `capability!` next to the function that does the work; the registry finds them all when the program
//! is linked. Nothing here knows the AI exists, and no module uses the AI: the platform uses the capabilities.
//!
//! A contract is whole or it does not compile. This one has every field:
//!
//! ```no_run
//! use sociai_lib::capability::{Call, CapabilityError, Limits};
//!
//! #[derive(serde::Deserialize, schemars::JsonSchema)]
//! struct Ask {}
//!
//! #[derive(serde::Serialize, schemars::JsonSchema)]
//! struct Missing { fields: Vec<String> }
//!
//! fn missing(_: &Call, _: Ask) -> Result<Missing, CapabilityError> {
//!     Ok(Missing { fields: vec![] })
//! }
//!
//! sociai_lib::capability! {
//!     static MISSING = "institution.missing_fields" {
//!         description: "Qué datos le faltan a la institución, por ventana.",
//!         effect: Query,
//!         permission: Use,
//!         data: [Internal],
//!         approval: Never,
//!         limits: Limits::READ,
//!         run: read(missing),
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! The same without the permission does not:
//!
//! ```compile_fail
//! use sociai_lib::capability::{Call, CapabilityError, Limits};
//! # #[derive(serde::Deserialize, schemars::JsonSchema)]
//! # struct Ask {}
//! # #[derive(serde::Serialize, schemars::JsonSchema)]
//! # struct Missing { fields: Vec<String> }
//! # fn missing(_: &Call, _: Ask) -> Result<Missing, CapabilityError> { Ok(Missing { fields: vec![] }) }
//!
//! sociai_lib::capability! {
//!     static MISSING = "institution.missing_fields" {
//!         description: "Qué datos le faltan a la institución, por ventana.",
//!         effect: Query,
//!         data: [Internal],
//!         approval: Never,
//!         limits: Limits::READ,
//!         run: read(missing),
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! Nor without the effect:
//!
//! ```compile_fail
//! use sociai_lib::capability::{Call, CapabilityError, Limits};
//! # #[derive(serde::Deserialize, schemars::JsonSchema)]
//! # struct Ask {}
//! # #[derive(serde::Serialize, schemars::JsonSchema)]
//! # struct Missing { fields: Vec<String> }
//! # fn missing(_: &Call, _: Ask) -> Result<Missing, CapabilityError> { Ok(Missing { fields: vec![] }) }
//!
//! sociai_lib::capability! {
//!     static MISSING = "institution.missing_fields" {
//!         description: "Qué datos le faltan a la institución, por ventana.",
//!         permission: Use,
//!         data: [Internal],
//!         approval: Never,
//!         limits: Limits::READ,
//!         run: read(missing),
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! And a change whose `apply` takes another input than its `prepare` does not either:
//!
//! ```compile_fail
//! use sociai_lib::capability::{Call, CapabilityError, Limits};
//! # #[derive(serde::Deserialize, schemars::JsonSchema)]
//! # struct Name { name: String }
//! # #[derive(serde::Deserialize, schemars::JsonSchema)]
//! # struct Other { other: String }
//! # #[derive(serde::Serialize, schemars::JsonSchema)]
//! # struct Done {}
//! # fn preview(_: &Call, _: Name) -> Result<Done, CapabilityError> { Ok(Done {}) }
//! # fn save(_: &Call, _: Other) -> Result<Done, CapabilityError> { Ok(Done {}) }
//!
//! sociai_lib::capability! {
//!     static RENAME = "institution.rename" {
//!         description: "Cambia el nombre de la institución.",
//!         effect: Modify,
//!         permission: Use,
//!         data: [Public],
//!         approval: Required,
//!         limits: Limits::CHANGE,
//!         run: change(preview, save),
//!     }
//! }
//! # fn main() {}
//! ```

mod contract;
mod registry;
#[cfg(test)]
mod registry_tests;

pub use contract::*;
pub use registry::*;

// what `capability!` writes needs these from here, wherever it is used
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use serde_json::Value;

/// Declares a capability of a module (ADR-001 §5): every field, in this order, and the functions that do the work.
/// A query or transformation runs with `read(f)`; a modification or sensitive action with `change(prepare, apply)`.
/// The schemas of the input and the output come from the types of those functions.
#[macro_export]
macro_rules! capability {
    (
        static $name:ident = $id:literal {
            description: $description:literal,
            effect: $effect:ident,
            permission: $permission:ident,
            data: [$($class:ident),* $(,)?],
            approval: $approval:ident,
            limits: $limits:expr,
            run: read($read:path) $(,)?
        }
    ) => {
        $crate::capability!(@declare $name, $id, $description, $effect, $permission, [$($class),*], $approval, $limits,
            $crate::capability::Run::Read($crate::capability!(@step $read)), $read);
    };
    (
        static $name:ident = $id:literal {
            description: $description:literal,
            effect: $effect:ident,
            permission: $permission:ident,
            data: [$($class:ident),* $(,)?],
            approval: $approval:ident,
            limits: $limits:expr,
            run: change($prepare:path, $apply:path) $(,)?
        }
    ) => {
        const _: fn() = || $crate::capability::same_input($prepare, $apply);
        $crate::capability!(@declare $name, $id, $description, $effect, $permission, [$($class),*], $approval, $limits,
            $crate::capability::Run::Change {
                prepare: $crate::capability!(@step $prepare),
                apply: $crate::capability!(@step $apply),
            },
            $prepare);
    };
    (@step $f:path) => {
        $crate::capability::Step {
            call: {
                fn call(
                    call: &$crate::capability::Call,
                    input: $crate::capability::Value,
                ) -> ::std::result::Result<$crate::capability::Value, $crate::capability::CapabilityError> {
                    $crate::capability::adapt(call, input, $f)
                }
                call
            },
            output: {
                fn output() -> $crate::capability::Value {
                    $crate::capability::output_schema($f)
                }
                output
            },
        }
    };
    (@declare $name:ident, $id:literal, $description:literal, $effect:ident, $permission:ident, [$($class:ident),*],
        $approval:ident, $limits:expr, $run:expr, $first:path) => {
        #[$crate::capability::linkme::distributed_slice($crate::capability::CAPABILITIES)]
        #[linkme(crate = $crate::capability::linkme)]
        static $name: $crate::capability::Capability = $crate::capability::Capability {
            id: $id,
            description: $description,
            effect: $crate::capability::Effect::$effect,
            permission: $crate::capability::Permission::$permission,
            data: &[$($crate::capability::DataClass::$class),*],
            approval: $crate::capability::Approval::$approval,
            limits: $limits,
            input: {
                fn input() -> $crate::capability::Value {
                    $crate::capability::input_schema($first)
                }
                input
            },
            run: $run,
        };
    };
}
