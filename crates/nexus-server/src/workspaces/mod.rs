//! Workspaces + memberships (ADR-0009): the data layer (`store`) and the cloud-mode HTTP
//! routes for listing, creating, and inviting to workspaces (`routes`), plus the overlay
//! read-only token (`overlay_token`; its `mint`/`revoke` have no route yet, so it is
//! allowed to read as dead code until an overlay-token management UI lands).

pub mod routes;
pub mod store;

#[allow(dead_code)]
pub mod overlay_token;
