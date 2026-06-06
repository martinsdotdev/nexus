//! Command-line surface. Each option also reads a `NEXUS_*` environment
//! variable (clap's `env`), giving 12-factor configuration without a separate
//! config crate, the same local-or-hosted seam ADR-0005 calls for.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "nexus", about = "Nexus trusted sync relay")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the sync relay.
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Host/IP to bind. Defaults to loopback for local-first use; set
    /// `0.0.0.0` (e.g. `NEXUS_HOST=0.0.0.0`) to accept external connections
    /// when hosted.
    #[arg(long, env = "NEXUS_HOST", default_value = "127.0.0.1")]
    pub host: String,

    /// Port to bind. Defaults to 7777.
    #[arg(long, env = "NEXUS_PORT", default_value_t = 7777)]
    pub port: u16,

    /// Directory for the workspace snapshot (defaults to the OS app-data dir).
    #[arg(long, env = "NEXUS_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// Directory of built UI assets to serve (the SvelteKit `build/` output).
    #[arg(long, env = "NEXUS_STATIC_DIR")]
    pub static_dir: Option<PathBuf>,

    /// Postgres connection string. When set, the relay runs in cloud mode
    /// (accounts, sessions, multi-tenant workspaces, ADR-0009/0010); when unset, it
    /// runs in local single-tenant file mode (the default, ADR-0005).
    #[arg(long, env = "NEXUS_DATABASE_URL")]
    pub database_url: Option<String>,

    /// Write each issued email code to this file (`<recipient>\t<code>` per line)
    /// instead of logging it. A simple local sink (cloud mode); the auth e2e points it
    /// at a temp file to read the code without a mailbox.
    #[arg(long, env = "NEXUS_EMAIL_SINK")]
    pub email_sink: Option<PathBuf>,

    /// Resend API key. When set (cloud mode), one-time login codes are emailed via Resend
    /// rather than logged. Keep it secret: provide it as `NEXUS_RESEND_API_KEY`, never in
    /// source or a committed config.
    #[arg(long, env = "NEXUS_RESEND_API_KEY")]
    pub resend_api_key: Option<String>,

    /// The verified Resend "from" address for login-code emails (e.g.
    /// `Nexus <noreply@nexus.umaru.dev>`). Required when `--resend-api-key` is set.
    #[arg(long, env = "NEXUS_RESEND_FROM")]
    pub resend_from: Option<String>,
}
