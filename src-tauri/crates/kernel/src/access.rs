//! The vocabulary of permissions (K2): each one by its id, declared by the module it belongs to, in its manifest, and
//! named after it (`contabilidad.polizas.registrar`). A capability names the permission it needs by that id
//! (ADR-003, A2), so the contract does not depend on the kernel. The roles that group permissions, the classes of
//! data each role sees and the decision to authorize come later, also in K2 (ERP-10).

/// A permission a module defines, in its manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permission {
    /// `module.what[.more]`: the id of the module, then lowercase parts.
    pub id: &'static str,
    /// What it lets a person do, in Spanish, for whoever gives the roles.
    pub description: &'static str,
}
