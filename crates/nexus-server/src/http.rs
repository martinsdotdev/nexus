//! Axum application wiring: the `/sync` WebSocket route plus optional static
//! asset serving for the built SvelteKit UI (`/edit`, `/overlay`).

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};

use crate::runtime::WorkspaceRuntime;
use crate::ws::sync_handler;

/// Build the axum router. `static_dir`, when set, serves the built UI. Unmatched
/// paths fall back to the SPA shell (`200.html`) so clean URLs like `/edit` (not
/// just `/edit.html`) resolve and client-side routing renders the page.
pub fn build_app(runtime: Arc<WorkspaceRuntime>, static_dir: Option<PathBuf>) -> Router {
    let mut app = Router::new().route("/sync", get(sync_handler));
    if let Some(dir) = static_dir {
        let spa = ServeFile::new(dir.join("200.html"));
        app = app.fallback_service(ServeDir::new(dir).fallback(spa));
    }
    app.with_state(runtime)
}
