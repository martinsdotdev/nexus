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
}
