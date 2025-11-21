#![allow(clippy::multiple_crate_versions)]
//! gatosd — GATOS daemon/CLI entrypoint
//!
//! Minimal scaffold: parses CLI flags, initializes logging, and runs
//! an async loop that waits for shutdown signals. The JSONL RPC server
//! will be implemented in a subsequent iteration.

mod message_plane;

use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use message_plane::MessagePlaneService;
use serde_json::json;
use tracing::{error, info};

use gatos_message_plane::{GitCheckpointStore, SegmentPruner, TopicRef};

#[derive(Parser, Debug)]
#[command(name = "gatosd", version, about = "GATOS daemon (JSONL RPC)")]
struct Args {
    /// Serve JSONL protocol over stdio instead of sockets
    #[arg(long)]
    stdio: bool,

    /// Optional one-shot command (e.g., messages.read). When omitted, runs the daemon.
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Read messages from a topic (JSON output)
    MessagesRead {
        /// Logical topic name (e.g., jobs/pending)
        #[arg(long)]
        topic: String,
        /// Optional since ULID cursor
        #[arg(long)]
        since: Option<String>,
        /// Max messages to return (1-512)
        #[arg(long, default_value_t = 128)]
        limit: usize,
        /// Repository path containing refs/gatos/messages
        #[arg(long, default_value = ".")]
        repo: String,
        /// Optional checkpoint group to persist after read
        #[arg(long)]
        checkpoint_group: Option<String>,
    },
    /// Prune expired message segments (TTL-aware)
    MessagesPrune {
        /// Logical topic name (e.g., jobs/pending)
        #[arg(long)]
        topic: String,
        /// Repository path containing refs/gatos/messages
        #[arg(long, default_value = ".")]
        repo: String,
        /// Retention window in days (default: 30)
        #[arg(long, default_value_t = 30)]
        retention_days: i64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    let args = Args::parse();
    info!(?args, "starting gatosd");

    match args.command {
        Some(Command::MessagesRead {
            topic,
            since,
            limit,
            repo,
            checkpoint_group,
        }) => run_messages_read(
            &topic,
            since.as_deref(),
            limit,
            &repo,
            checkpoint_group.as_deref(),
        )?,
        Some(Command::MessagesPrune {
            topic,
            repo,
            retention_days,
        }) => run_messages_prune(&topic, &repo, retention_days)?,
        None => {
            // TODO: wire up JSONL RPC server (stdio or TCP) per TECH-SPEC, including
            // `messages.read` handlers backed by crates/gatos-message-plane (ADR-0005).
            let mp = MessagePlaneService::open(".")?;
            info!(max_page_size = mp.max_page_size(), "message plane ready");
            if args.stdio {
                info!("stdio mode not yet implemented");
            }
            // Placeholder: run until Ctrl-C
            if let Err(e) = tokio::signal::ctrl_c().await {
                error!(?e, "failed to install Ctrl-C handler");
                return Err(anyhow::anyhow!(e));
            }
        }
    }
    info!("shutdown");
    Ok(())
}

fn run_messages_read(
    topic: &str,
    since: Option<&str>,
    limit: usize,
    repo: &str,
    checkpoint_group: Option<&str>,
) -> anyhow::Result<()> {
    let mp = MessagePlaneService::open(repo)?;
    let topic_ref = TopicRef::new(repo, topic);
    let response = mp.messages_read(&topic_ref, since, limit, checkpoint_group)?;
    let json = serde_json::to_string_pretty(&json!({
        "messages": response
            .messages
            .iter()
            .map(|m| json!({
                "ulid": m.ulid,
                "commit": m.commit,
                "content_id": m.content_id,
                "envelope_path": m.envelope_path,
                "canonical_json": m.canonical_json,
            }))
            .collect::<Vec<_>>(),
        "next_since": response.next_since,
    }))?;
    println!("{}", json);
    Ok(())
}

fn run_messages_prune(topic: &str, repo: &str, retention_days: i64) -> anyhow::Result<()> {
    let topic_ref = TopicRef::new(repo, topic);
    let checkpoints = GitCheckpointStore::open(repo)?.list_checkpoints(&topic_ref)?;
    let retention = Duration::days(retention_days);
    let deleted =
        SegmentPruner::open(repo)?.prune(&topic_ref, Utc::now(), retention, &checkpoints)?;
    println!(
        "prune complete: {} segment refs removed (retention={} days)",
        deleted.len(),
        retention_days
    );
    if !deleted.is_empty() {
        println!("deleted refs:");
        for r in deleted {
            println!("- {}", r);
        }
    }
    Ok(())
}

fn setup_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}
