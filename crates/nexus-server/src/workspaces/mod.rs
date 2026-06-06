//! Workspaces + memberships (ADR-0009): the data layer (`store`), the workspace-authorization
//! seam (`access`: `resolve_membership` + the role extractors shared by the routes and the
//! `/sync` gate), the cloud-mode HTTP routes (`routes`), and the overlay read-only token
//! (`overlay_token`; its `revoke` has no route yet, so it is allowed to read as dead code
//! until an overlay-token management UI lands).

pub mod access;
pub mod routes;
pub mod store;

#[allow(dead_code)]
pub mod overlay_token;
