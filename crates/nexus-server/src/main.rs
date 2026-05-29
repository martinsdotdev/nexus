//! `nexus-server`: the trusted Nexus sync relay (walking-skeleton stage).
//!
//! Holds the canonical Loro replica, merges peer updates, validates/repairs,
//! persists a snapshot, and rebroadcasts to peers over the `/sync` WebSocket.
//! Deployable locally or hosted via the same binary (env-configured). See
//! ADR-0005 and the implementation plan.

mod cli;
mod http;
mod persistence;
mod protocol;
mod runtime;
mod ws;

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::persistence::FilePersistence;
use crate::runtime::WorkspaceRuntime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match Cli::parse().command {
        Command::Serve(args) => {
            let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
            std::fs::create_dir_all(&data_dir)?;

            let runtime = WorkspaceRuntime::new(FilePersistence::new(&data_dir))?;
            let app = http::build_app(runtime, args.static_dir);

            let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
            let listener = tokio::net::TcpListener::bind(addr).await?;
            tracing::info!(%addr, "nexus relay listening");
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus")
}
