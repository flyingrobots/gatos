use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(name = "xtask", version, about = "Repo task runner (cargo xtask)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run pre-commit pipeline (staged-only)
    PreCommit,
    /// Generate mermaid diagrams
    Diagrams {
        /// Process all git-tracked .md files
        #[arg(long)]
        all: bool,
        /// Specific markdown files
        #[arg(value_name = "FILE", required = false)]
        files: Vec<PathBuf>,
    },
    /// Validate JSON Schemas and examples (v1)
    Schemas {
        #[command(subcommand)]
        sub: SchemaSub,
    },
    /// Link checker for Markdown (lychee)
    Links {
        /// Optional file globs (default: **/*.md)
        #[arg(value_name = "GLOB", required = false)]
        files: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum SchemaSub {
    /// Compile + validate + negatives (full suite)
    All,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::PreCommit => pre_commit(),
        Cmd::Diagrams { all, files } => diagrams(all, &files),
        Cmd::Schemas { sub } => schemas(sub),
        Cmd::Links { files } => links(files),
    }
}

fn pre_commit() -> Result<()> {
    // Use existing Make target to keep parity; we can inline later
    run("bash", ["-lc", "make -s pre-commit"], None)
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
        bail!("pass --all or one or more files");
    }
    Ok(())
}

fn schemas(which: SchemaSub) -> Result<()> {
    let repo = repo_root()?;
    let script = repo.join("scripts/validate_schemas.sh");
    match which {
        SchemaSub::All => run(
            "bash",
            ["-lc", script.to_string_lossy().as_ref()],
            Some(&repo),
        )?,
    }
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
        return Ok(());
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
        return Ok(());
    } else {
        bail!("lychee or docker required for link check")
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
    bail!("could not locate repo root from {:?}", cwd)
}

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for p in std::env::split_paths(&path) {
        let cand = p.join(bin);
        if cand.is_file() {
            return Some(cand);
        }
        // Windows exe
        if cfg!(windows) {
            let ex = p.join(format!("{}.exe", bin));
            if ex.is_file() {
                return Some(ex);
            }
        }
    }
    None
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
