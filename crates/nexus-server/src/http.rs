//! Axum application wiring: shared state, the `/sync` WebSocket route, the cloud-mode
//! auth routes, and optional static asset serving for the built SvelteKit UI.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};

use crate::auth::email::EmailStore;
use crate::auth::email_sender::EmailSender;
use crate::auth::session::SessionStore;
use crate::registry::WorkspaceRegistry;
use crate::ws::sync_handler;

/// The cloud-mode authentication subsystem, present together or not at all (one "cloud
/// auth is on" union rather than parallel `Option`s that must agree). Built in `main`
/// when `NEXUS_DATABASE_URL` is set; absent in local file mode. Cheap to clone.
#[derive(Clone)]
pub struct CloudAuth {
    pub sessions: SessionStore,
    pub emails: EmailStore,
    pub sender: EmailSender,
}

/// Shared application state. `registry` is always present (the relay's live workspace
/// documents, loaded on demand); `cloud` is `Some` only in cloud mode, where it backs
/// authentication. Cheap to clone (every field is reference-counted).
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<WorkspaceRegistry>,
    pub cloud: Option<CloudAuth>,
}

/// Build the axum router. `static_dir`, when set, serves the built UI. Unmatched paths
/// fall back to the SPA shell (`200.html`) so clean URLs like `/edit` (not just
/// `/edit.html`) resolve and client-side routing renders the page. The auth routes
/// (`/auth/*`) are added in cloud mode.
pub fn build_app(state: AppState, static_dir: Option<PathBuf>) -> Router {
    let mut app = Router::new().route("/sync", get(sync_handler));

    if state.cloud.is_some() {
        app = app.merge(crate::auth::routes::router());
    }

    if let Some(dir) = static_dir {
        let spa = ServeFile::new(dir.join("200.html"));
        app = app.fallback_service(ServeDir::new(dir).fallback(spa));
    }

    app.with_state(state)
}
