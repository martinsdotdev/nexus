//! Container names for the Loro workspace document. Shared by the writer
//! ([`crate::default_doc`]) and the reader ([`crate::model`]) so the two never
//! drift. CSS-style magic strings live here only.

/// Root `LoroTree` holding the `layout -> scene -> widget` hierarchy.
pub const TREE: &str = "tree";

/// Root `LoroMap` holding workspace-level pointers (`activeLayoutId`, ...).
pub const WORKSPACE: &str = "workspace";

/// Root `LoroMap` of user-authored custom themes (ADR-0006), keyed by a
/// client-minted unique id. Each value is a map of `{ name, base, tokens }`.
pub const THEMES: &str = "themes";
