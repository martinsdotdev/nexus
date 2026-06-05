//! Axum application wiring: shared state, the `/sync` WebSocket route, the cloud-mode
//! auth routes, and optional static asset serving for the built SvelteKit UI.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};

use crate::auth::session::SessionStore;
use crate::runtime::WorkspaceRuntime;
use crate::ws::sync_handler;

/// Shared application state. `runtime` is always present (the canonical Loro replica);
/// `sessions` is `Some` only in cloud mode (`NEXUS_DATABASE_URL` set), where it backs
/// authentication. Cheap to clone (both fields are reference-counted).
#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<WorkspaceRuntime>,
    pub sessions: Option<SessionStore>,
}

/// Build the axum router. `static_dir`, when set, serves the built UI. Unmatched paths
/// fall back to the SPA shell (`200.html`) so clean URLs like `/edit` (not just
/// `/edit.html`) resolve and client-side routing renders the page. The auth routes
/// (`/auth/*`) are added in cloud mode.
pub fn build_app(state: AppState, static_dir: Option<PathBuf>) -> Router {
    let mut app = Router::new().route("/sync", get(sync_handler));

    if state.sessions.is_some() {
        app = app.merge(crate::auth::routes::router());
    }

    if let Some(dir) = static_dir {
        let spa = ServeFile::new(dir.join("200.html"));
        app = app.fallback_service(ServeDir::new(dir).fallback(spa));
    }

    app.with_state(state)
}
