//! Container names for the Loro workspace document. Shared by the writer
//! ([`crate::default_doc`]) and the reader ([`crate::model`]) so the two never
//! drift. CSS-style magic strings live here only.

/// Root `LoroTree` holding the `layout -> scene -> widget` hierarchy.
pub const TREE: &str = "tree";

/// Root `LoroMap` holding workspace-level pointers (`activeLayoutId`, ...).
pub const WORKSPACE: &str = "workspace";

/// Root `LoroMap` of the theme registry (ADR-0007): the relay seeds the built-ins
/// as `protected` entries with stable ids (`cozy`/`cyber`/`editorial`/`sticker`),
/// and user themes are added with client-minted `theme-<uuid>` ids. Each value is
/// a map of `{ name, base, protected, tokens }`.
pub const THEMES: &str = "themes";
