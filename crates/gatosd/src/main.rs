#![allow(clippy::multiple_crate_versions)]
//! gatosd — GATOS daemon/CLI entrypoint
//!
//! Minimal scaffold: parses CLI flags, initializes logging, and runs
//! an async loop that waits for shutdown signals. The JSONL RPC server
//! will be implemented in a subsequent iteration.

mod message_plane;

use clap::Parser;
use message_plane::MessagePlaneService;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "gatosd", version, about = "GATOS daemon (JSONL RPC)")]
struct Args {
    /// Serve JSONL protocol over stdio instead of sockets
    #[arg(long)]
    stdio: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    let args = Args::parse();
    info!(?args, "starting gatosd");

    // TODO: wire up JSONL RPC server (stdio or TCP) per TECH-SPEC, including
    // `messages.read` handlers backed by crates/gatos-message-plane (ADR-0005).
    let mp = MessagePlaneService::new();
    info!(max_page_size = mp.max_page_size(), "message plane stub ready");
    // Placeholder: run until Ctrl-C
    if let Err(e) = tokio::signal::ctrl_c().await {
        error!(?e, "failed to install Ctrl-C handler");
        return Err(anyhow::anyhow!(e));
    }
    info!("shutdown");
    Ok(())
}

fn setup_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}
