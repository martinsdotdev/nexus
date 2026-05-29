//! `nexus-core`: the hexagonal core for Nexus.
//!
//! Holds the Loro-backed workspace document schema, plain read-model types, the
//! validator/repair logic, and the ports. It depends on `loro` for the CRDT
//! document but has no IO, async runtime, network, clock, or randomness (those
//! arrive via the `Clock`/`Random` ports). See ADR-0005.

pub mod builtin_themes;
pub mod default_doc;
pub mod model;
pub mod schema;
pub mod validate;
