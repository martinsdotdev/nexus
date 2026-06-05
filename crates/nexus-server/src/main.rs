//! `nexus-server`: the trusted Nexus sync relay (walking-skeleton stage).
//!
//! Holds the canonical Loro replica, merges peer updates, validates/repairs,
//! persists a snapshot, and rebroadcasts to peers over the `/sync` WebSocket.
//! Deployable locally or hosted via the same binary (env-configured). See
//! ADR-0005 and the implementation plan.

mod auth;
mod cli;
mod http;
mod persistence;
mod protocol;
mod registry;
mod runtime;
mod workspaces;
mod ws;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::persistence::{DatabasePersistence, FilePersistence, WorkspacePersistence};
use crate::registry::WorkspaceRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match Cli::parse().command {
        Command::Serve(args) => {
            let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
            std::fs::create_dir_all(&data_dir)?;

            // Cloud mode (NEXUS_DATABASE_URL set): connect Postgres, apply migrations,
            // back auth with the session + email-code stores, and persist workspaces to
            // the database. Local file mode leaves `cloud` unset, persists to a single
            // file, and mounts no auth routes (ADR-0005/0009/0010).
            let (cloud, persistence): (Option<http::CloudAuth>, Arc<dyn WorkspacePersistence>) =
                match &args.database_url {
                    Some(url) => {
                        let pool = sqlx::PgPool::connect(url).await?;
                        sqlx::migrate!("./migrations").run(&pool).await?;
                        tracing::info!("cloud mode: Postgres connected, migrations applied");
                        let cloud = http::CloudAuth {
                            sessions: auth::session::SessionStore::new(pool.clone()),
                            emails: auth::email::EmailStore::new(pool.clone()),
                            sender: match &args.email_sink {
                                Some(path) => auth::email_sender::EmailSender::File(path.clone()),
                                None => auth::email_sender::EmailSender::Log,
                            },
                            workspaces: workspaces::store::WorkspaceStore::new(pool.clone()),
                        };
                        (Some(cloud), Arc::new(DatabasePersistence::new(pool)))
                    }
                    None => (None, Arc::new(FilePersistence::new(&data_dir))),
                };

            let registry = WorkspaceRegistry::new(persistence);
            let app = http::build_app(http::AppState { registry, cloud }, args.static_dir);

            let addr = socket_addr(&args.host, args.port)?;
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

/// Resolve the bind address from the host string and port. The host comes from
/// `--host` / `NEXUS_HOST` and must be an IP literal (e.g. `127.0.0.1` locally,
/// `0.0.0.0` when hosted); a bad value surfaces as a clean error, not a panic.
fn socket_addr(host: &str, port: u16) -> anyhow::Result<SocketAddr> {
    let ip: IpAddr = host
        .parse()
        .with_context(|| format!("invalid --host / NEXUS_HOST value: {host:?}"))?;
    Ok(SocketAddr::new(ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_loopback_and_all_interfaces() {
        assert_eq!(
            socket_addr("127.0.0.1", 7777).unwrap(),
            "127.0.0.1:7777".parse().unwrap()
        );
        assert_eq!(
            socket_addr("0.0.0.0", 8080).unwrap(),
            "0.0.0.0:8080".parse().unwrap()
        );
    }

    #[test]
    fn rejects_a_non_ip_host() {
        assert!(socket_addr("not-an-ip", 7777).is_err());
    }
}
