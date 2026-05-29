//! Axum application wiring: the `/sync` WebSocket route plus optional static
//! asset serving for the built SvelteKit UI (`/edit`, `/overlay`).

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;

use crate::runtime::WorkspaceRuntime;
use crate::ws::sync_handler;

/// Build the axum router. `static_dir`, when set, serves the built UI as the
/// fallback (so deep links resolve to the prerendered HTML).
pub fn build_app(runtime: Arc<WorkspaceRuntime>, static_dir: Option<PathBuf>) -> Router {
    let mut app = Router::new().route("/sync", get(sync_handler));
    if let Some(dir) = static_dir {
        app = app.fallback_service(tower_http::services::ServeDir::new(dir));
    }
    app.with_state(runtime)
}
