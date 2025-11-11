use anyhow::{bail, Context, Result};
use clap::{ArgGroup, Parser, Subcommand};
use git2::Repository;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

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
        files: Option<Vec<PathBuf>>, // use Option so clap can tell presence vs empty
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
        Cmd::Diagrams { all, files } => diagrams(all, files),
        Cmd::Schemas => schemas(),
        Cmd::Links { files } => links(files),
    }
}

fn pre_commit() -> Result<()> {
    // Call make directly (no shell), relies on CI/dev environment having make
    run("make", ["-s", "pre-commit"], None)
}

fn diagrams(all: bool, files: Option<Vec<PathBuf>>) -> Result<()> {
    let repo = repo_root()?;
    let script = repo.join("scripts/mermaid/generate.mjs");
    if all {
        run(
            "node",
            [script.as_os_str(), OsStr::new("--all")],
            Some(&repo),
        )?
    } else if let Some(files) = files {
        if files.is_empty() {
            bail!("No input provided. Pass --all to scan all tracked .md files, or list one or more files.");
        }
        let mut args: Vec<&OsStr> = Vec::with_capacity(files.len() + 1);
        args.push(script.as_os_str());
        for f in &files {
            args.push(f.as_os_str());
        }
        run("node", args, Some(&repo))?
    } else {
        bail!("No input provided. Pass --all to scan all tracked .md files, or list one or more files.");
    }
    Ok(())
}

fn schemas() -> Result<()> {
    let repo = repo_root()?;
    let script = repo.join("scripts/validate_schemas.sh");
    // Execute via a shell explicitly for cross-platform compatibility
    let shell = if which("bash").is_ok() {
        "bash"
    } else if which("sh").is_ok() {
        "sh"
    } else {
        bail!(
            "No suitable shell found to execute {:?}. Install bash/sh or run in CI.",
            script
        );
    };
    run(shell, [script.as_os_str()], Some(&repo))?;
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
    if which("lychee").is_ok() {
        let mut args = vec!["--no-progress", "--config", ".lychee.toml"];
        for g in &arglist {
            args.push(g);
        }
        run("lychee", args, Some(&repo))?;
        Ok(())
    } else if which("docker").is_ok() {
        // Build docker args with borrowed strings to avoid unnecessary cloning
        let mount = format!("{}:/work", repo.display());
        let mut docker_args: Vec<&OsStr> = Vec::with_capacity(11 + arglist.len());
        docker_args.extend([
            OsStr::new("run"),
            OsStr::new("--rm"),
            OsStr::new("-v"),
            OsStr::new(mount.as_str()),
            OsStr::new("-w"),
            OsStr::new("/work"),
            OsStr::new("ghcr.io/lycheeverse/lychee:latest"),
            OsStr::new("--no-progress"),
            OsStr::new("--config"),
            OsStr::new(".lychee.toml"),
        ]);
        for g in &arglist {
            docker_args.push(OsStr::new(g.as_str()));
        }
        run("docker", docker_args, Some(&repo))?;
        Ok(())
    } else {
        bail!("Link check requires 'lychee' in PATH or Docker. Install lychee (https://github.com/lycheeverse/lychee) or install Docker to run the containerized check.")
    }
}

fn repo_root() -> Result<PathBuf> {
    // Prefer libgit2 discovery to handle worktrees, submodules, and .git files
    let cwd = std::env::current_dir()?;
    if let Ok(repo) = Repository::discover(&cwd) {
        if let Some(wd) = repo.workdir() {
            return Ok(wd.to_path_buf());
        }
        // Bare repo: use parent of the .git directory
        let git_dir = repo.path();
        if let Some(parent) = git_dir.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    // Fallback to manual traversal
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
    S: AsRef<OsStr>,
{
    let mut c = Command::new(cmd);
    c.args(args)
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
