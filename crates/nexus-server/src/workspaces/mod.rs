//! Workspaces + memberships (ADR-0009): the cloud-mode data layer the `/sync` gate
//! and the workspace HTTP routes consume. Unused in production until the gate lands
//! in the next increment, so the store is allowed to read as dead code until then.

#[allow(dead_code)]
pub mod store;

// Overlay read-only tokens. `validate` backs the `/sync?token=` path; `mint`/`revoke`
// have no production caller until the workspace HTTP routes land, so allow until then.
#[allow(dead_code)]
pub mod overlay_token;
