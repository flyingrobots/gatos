use anyhow::{bail, Context, Result};
use clap::{ArgGroup, Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(
    name = "xtask",
    version,
    about = "Repo task runner (cargo xtask)",
    help_template = "{name} {version}\n{about-with-newline}USAGE:\n  {usage}\n\nOPTIONS:\n{options}\n\nSUBCOMMANDS:\n{subcommands}\n\nENV:\n  MERMAID_MAX_PARALLEL     Concurrency for diagrams (default: min(cpu, 8))\n  MERMAID_CLI_VERSION      @mermaid-js/mermaid-cli pin (default: 10.9.0)\n  MERMAID_CMD_TIMEOUT_MS   Timeout for mmdc/npx (10s..15m, default: 120s)\n\nEXAMPLES:\n  cargo run -p xtask -- diagrams --all\n  cargo run -p xtask -- diagrams docs/TECH-SPEC.md\n  cargo run -p xtask -- schemas all\n  cargo run -p xtask -- links\n",
    after_help = "Tip: use 'make ci-*' shims or 'cargo run -p xtask -- <command>' for CI parity."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run pre-commit pipeline (staged-only)
    PreCommit,
    /// Generate mermaid diagrams
    #[command(group(
        ArgGroup::new("input")
            .args(["all", "files"]) // exactly one must be present
            .required(true)
            .multiple(false)
    ))]
    Diagrams {
        /// Process all git-tracked .md files
        #[arg(long)]
        all: bool,
        /// Specific markdown files
        #[arg(value_name = "FILE", required = false)]
        files: Vec<PathBuf>,
    },
    /// Validate JSON Schemas and examples (v1)
    Schemas,
    /// Link checker for Markdown (lychee)
    Links {
        /// Optional file globs (default: **/*.md)
        #[arg(value_name = "GLOB", required = false)]
        files: Vec<String>,
    },
}

// No subcommands for schemas; always run the full suite

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::PreCommit => pre_commit(),
        Cmd::Diagrams { all, files } => diagrams(all, &files),
        Cmd::Schemas => schemas(),
        Cmd::Links { files } => links(files),
    }
}

fn pre_commit() -> Result<()> {
    // Call make directly (no shell), relies on CI/dev environment having make
    run("make", ["-s", "pre-commit"], None)
}

fn diagrams(all: bool, files: &[PathBuf]) -> Result<()> {
    let repo = repo_root()?;
    let script = repo.join("scripts/mermaid/generate.mjs");
    if all {
        run(
            "node",
            [script.to_string_lossy().as_ref(), "--all"],
            Some(&repo),
        )?
    } else if !files.is_empty() {
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(script.to_string_lossy().to_string());
        for f in files {
            args.push(f.to_string_lossy().to_string());
        }
        run("node", &args, Some(&repo))?
    } else {
        bail!("No input provided. Pass --all to scan all tracked .md files, or list one or more files.");
    }
    Ok(())
}

fn schemas() -> Result<()> {
    let repo = repo_root()?;
    let script = repo.join("scripts/validate_schemas.sh");
    // Execute the script directly (it has a shebang and is executable)
    let script_str = script.to_string_lossy().to_string();
    run(&script_str, [] as [&str; 0], Some(&repo))?;
    Ok(())
}

fn links(files: Vec<String>) -> Result<()> {
    let repo = repo_root()?;
    let arglist = if files.is_empty() {
        vec!["**/*.md".to_string()]
    } else {
        files
    };
    // Prefer local lychee if present; otherwise Docker fallback
    if which::which("lychee").is_ok() {
        let mut args = vec!["--no-progress", "--config", ".lychee.toml"];
        for g in &arglist {
            args.push(g);
        }
        run("lychee", args, Some(&repo))?;
        Ok(())
    } else if which::which("docker").is_ok() {
        let mut docker_args: Vec<String> = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-v".to_string(),
            format!("{}:/work", repo.display()),
            "-w".to_string(),
            "/work".to_string(),
            "ghcr.io/lycheeverse/lychee:latest".to_string(),
            "--no-progress".to_string(),
            "--config".to_string(),
            ".lychee.toml".to_string(),
        ];
        for g in &arglist {
            docker_args.push(g.clone());
        }
        run("docker", docker_args, Some(&repo))?;
        Ok(())
    } else {
        bail!("Link check requires 'lychee' in PATH or Docker. Install lychee (https://github.com/lycheeverse/lychee) or install Docker to run the containerized check.")
    }
}

fn repo_root() -> Result<PathBuf> {
    // Assume xtask is run from repo; use current dir
    let cwd = std::env::current_dir()?;
    // Walk up to find .git dir/file
    let mut dir = cwd.as_path();
    for _ in 0..15 {
        if dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }
        if let Some(p) = dir.parent() {
            dir = p;
        } else {
            break;
        }
    }
    bail!(
        "Could not locate repository root from {:?}. Run xtask from within the repository or a child directory.",
        cwd
    )
}

fn run<I, S>(cmd: &str, args: I, cwd: Option<&Path>) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut c = Command::new(cmd);
    c.args(args.into_iter().map(|s| s.as_ref().to_string()))
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(dir) = cwd {
        c.current_dir(dir);
    }
    let status = c
        .status()
        .with_context(|| format!("failed to spawn {}", cmd))?;
    if !status.success() {
        bail!("{} exited with status {}", cmd, status);
    }
    Ok(())
}
