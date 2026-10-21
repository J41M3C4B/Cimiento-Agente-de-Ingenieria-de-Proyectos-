//! What a capability declares (ADR-001 §2 to §4): its id, what it does, its effect, the permission it needs, the
//! classes of data it touches, whether a person approves it, its limits and how it runs. The rules a contract must
//! keep are checked by the registry.

use std::time::Duration;

use rusqlite::Connection;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// What running a capability does to the institution's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Reads, writes nothing.
    Query,
    /// Works on its input (a text, a file) and writes nothing.
    Transform,
    /// Changes data of the ERP: it is prepared first, then applied.
    Modify,
    /// Sensitive, external or irreversible: it always needs the person's approval.
    Sensitive,
}

impl Effect {
    /// Whether it changes something, and so runs as `Run::Change`.
    pub fn changes(self) -> bool {
        matches!(self, Effect::Modify | Effect::Sensitive)
    }
}

/// The class of a datum. Provisional: the ADR of IA-3 sets the final classes and their policy, without changing the
/// shape of the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClass {
    /// What the institution says of itself in public.
    Public,
    /// The institution's own, not personal: its figures, its plans.
    Internal,
    /// The institution's own and protected: its bank accounts, its contracts.
    Confidential,
    /// A datum of a person.
    Personal,
    /// A datum of a person that the law protects more: health, beliefs, origin.
    SensitivePersonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Approval {
    Never,
    /// The person approves the concrete action and its input before it is applied.
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The longest one run may take.
    pub timeout: Duration,
    /// How many times it may be tried again after a correctable error.
    pub retries: u8,
    /// Whether running it twice with the same input gives the same as once.
    pub idempotent: bool,
}

impl Limits {
    /// A read of the database: quick, and safe to try again.
    pub const READ: Limits = Limits { timeout: Duration::from_secs(10), retries: 2, idempotent: true };
    /// A change that must not run twice: one try.
    pub const CHANGE: Limits = Limits { timeout: Duration::from_secs(10), retries: 0, idempotent: false };
}

/// What a capability receives when it runs. Only the platform makes one (`Call::new`); the modules only read it, so
/// a new field never touches their code.
pub struct Call<'a> {
    /// The database, open and unlocked.
    pub conn: &'a Connection,
    /// The id of the account that asks.
    pub by: &'a str,
    /// The id of the task this run belongs to: it goes with everything the run leaves behind.
    pub task: &'a str,
}

impl<'a> Call<'a> {
    pub fn new(conn: &'a Connection, by: &'a str, task: &'a str) -> Self {
        Call { conn, by, task }
    }
}

/// Why a run did not give its output, which says what to do next (blueprints §3.2). The texts are fixed, so a datum
/// of a person can never travel inside an error, nor from there into the audit log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CapabilityError {
    /// An argument that can be corrected: the run may be tried again with a corrected input.
    #[error("invalid {field}: {reason}")]
    Invalid { field: &'static str, reason: &'static str },
    /// Something only the person knows is missing: the task pauses and asks.
    #[error("needs from the person: {what}")]
    NeedsPerson { what: &'static str },
    /// Nothing to correct nor to ask: the task ends with an error.
    #[error("failed: {reason}")]
    Failed { reason: &'static str },
}

/// One function of a capability, over JSON, with the schema of what it gives.
#[derive(Clone, Copy)]
pub struct Step {
    pub call: fn(&Call, Value) -> Result<Value, CapabilityError>,
    pub output: fn() -> Value,
}

/// How a capability runs; the registry checks that it fits the effect.
#[derive(Clone, Copy)]
pub enum Run {
    /// A query or a transformation.
    Read(Step),
    /// A modification or a sensitive action: `prepare` says what will change without writing, and `apply` makes the
    /// change with the same input, once it is approved when the contract asks for it.
    Change { prepare: Step, apply: Step },
}

/// The contract of a capability. A module declares it with `capability!`, never by hand.
pub struct Capability {
    /// `module.name`: lowercase letters, digits and `_`, with one dot.
    pub id: &'static str,
    /// What it does, in Spanish, in a line or two: the router reads it to propose it.
    pub description: &'static str,
    pub effect: Effect,
    /// The id of the permission it needs (ADR-003 A2): one of the kernel's (`kernel.usar`) or one its module defines
    /// (`contabilidad.polizas.registrar`). The catalog, the roles and the decision are the kernel's (K2).
    pub permission: &'static str,
    pub data: &'static [DataClass],
    pub approval: Approval,
    pub limits: Limits,
    /// The JSON schema of the input, taken from its Rust type.
    pub input: fn() -> Value,
    pub run: Run,
}

impl Capability {
    /// The module that publishes it: what comes before the dot of its id.
    pub fn module(&self) -> &'static str {
        self.id.split('.').next().unwrap_or("")
    }
}

// What `capability!` writes around the typed functions of a module. Not for anyone else.

#[doc(hidden)]
pub fn adapt<I, O, F>(call: &Call, input: Value, f: F) -> Result<Value, CapabilityError>
where
    I: DeserializeOwned,
    O: Serialize,
    F: Fn(&Call, I) -> Result<O, CapabilityError>,
{
    // serde's message may quote the value it got: it never leaves here
    let input = serde_json::from_value(input)
        .map_err(|_| CapabilityError::Invalid { field: "input", reason: "does not fit the schema of the capability" })?;
    let output = f(call, input)?;
    serde_json::to_value(output).map_err(|_| CapabilityError::Failed { reason: "the output could not be written as JSON" })
}

#[doc(hidden)]
pub fn input_schema<I, O, F>(_: F) -> Value
where
    I: JsonSchema,
    F: Fn(&Call, I) -> Result<O, CapabilityError>,
{
    schemars::schema_for!(I).to_value()
}

#[doc(hidden)]
pub fn output_schema<I, O, F>(_: F) -> Value
where
    O: JsonSchema,
    F: Fn(&Call, I) -> Result<O, CapabilityError>,
{
    schemars::schema_for!(O).to_value()
}

/// Only type-checked: `prepare` and `apply` take the same input.
#[doc(hidden)]
pub fn same_input<I, P, O, F, G>(_: F, _: G)
where
    F: Fn(&Call, I) -> Result<P, CapabilityError>,
    G: Fn(&Call, I) -> Result<O, CapabilityError>,
{
}
