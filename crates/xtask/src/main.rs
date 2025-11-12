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
    /// Markdown lint (subset of common rules). Use --fix to auto-fix.
    Md {
        /// Auto-fix a subset of rules (whitespace/blank lines/non-ASCII hyphens)
        #[arg(long)]
        fix: bool,
        /// Optional markdown files (default: all git-tracked *.md)
        #[arg(value_name = "FILE", required = false)]
        files: Vec<PathBuf>,
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
        Cmd::Md { fix, files } => md_lint(fix, files),
    }
}

fn pre_commit() -> Result<()> {
    // Call make directly (no shell), relies on CI/dev environment having make
    run("make", ["-s", "pre-commit"], None)
}

fn diagrams(all: bool, files: Option<Vec<PathBuf>>) -> Result<()> {
    let repo = repo_root()?;
    // Prefer script wrapper which selects Dockerized Node when available (Node-free path)
    let wrapper = repo.join("scripts/diagrams.sh");
    let shell = if which("bash").is_ok() { "bash" } else { "sh" };
    if all {
        // wrapper renders all diagrams when no args are given
        run(shell, [wrapper.as_os_str()], Some(&repo))?
    } else if let Some(files) = files {
        if files.is_empty() {
            bail!("No input provided. Pass --all to scan all tracked .md files, or list one or more files.");
        }
        let mut args: Vec<&OsStr> = Vec::with_capacity(files.len() + 1);
        args.push(wrapper.as_os_str());
        for f in &files { args.push(f.as_os_str()); }
        run(shell, args, Some(&repo))?
    } else {
        bail!("No input provided. Pass --all to scan all tracked .md files, or list one or more files.");
    }
    Ok(())
}

fn schemas() -> Result<()> {
    let repo = repo_root()?;
    let script = repo.join("scripts/validate_schemas.sh");
    let shell = if which("bash").is_ok() { "bash" } else if which("sh").is_ok() { "sh" } else { bail!("No suitable shell for {:?}", script) };
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

fn md_lint(fix: bool, files: Vec<PathBuf>) -> Result<()> {
    let repo = repo_root()?;
    // Collect files: use provided or git ls-files
    let md_files: Vec<PathBuf> = if files.is_empty() {
        let out = Command::new("git")
            .args(["ls-files", "--", "*.md"])
            .current_dir(&repo)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .context("git ls-files for *.md failed")?;
        let s = String::from_utf8_lossy(&out.stdout);
        s.lines().map(|l| repo.join(l.trim())).collect()
    } else {
        files
    };

    let mut total_issues = 0usize;
    for path in md_files {
        if !path.exists() { continue; }
        let orig = std::fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let (mut updated, issues) = lint_one(&orig);
        if issues > 0 {
            total_issues += issues;
            eprintln!("[md] {}: {} issue(s)", path.strip_prefix(&repo).unwrap_or(&path).display(), issues);
            if fix {
                // Apply safe fixes: trailing spaces (MD009), multiple blanks (MD012), blanks around headings (MD022), lists (MD032), non-ASCII hyphens
                // updated already contains fixes for these rules
                if updated != orig {
                    std::fs::write(&path, updated.as_bytes()).with_context(|| format!("write {}", path.display()))?;
                }
            }
        }
        // If not fixing, but we mutated updated for planning, ensure we don't write
        if !fix { updated.clear(); }
    }
    if total_issues > 0 && !fix {
        bail!("Markdown lint found {} issue(s). Run: cargo run -p xtask -- md --fix", total_issues);
    }
    Ok(())
}

fn lint_one(s: &str) -> (String, usize) {
    let mut issues = 0usize;
    let mut out: Vec<String> = Vec::new();
    let mut lines: Vec<&str> = s.split_inclusive('\n').collect();
    if lines.is_empty() { return (s.to_string(), 0); }

    // Track code fence state to avoid touching code blocks
    let mut in_fence = false;
    let fence_re = regex::Regex::new(r"^\s*(```|~~~)").unwrap();

    // First pass: normalize non-ASCII hyphen (U+2011) and trailing spaces (MD009)
    let mut norm: Vec<String> = Vec::with_capacity(lines.len());
    for l in lines.drain(..) {
        let mut ll = l.replace('\u{2011}', "-");
        if ll.contains('\u{2011}') { issues += 1; }
        if fence_re.is_match(&ll) { in_fence = !in_fence; }
        if !in_fence {
            // Remove single trailing space, keep double-space hard breaks
            if ll.ends_with(" \n") {
                if !ll.ends_with("  \n") {
                    ll = ll[..ll.len()-2].to_string(); ll.push('\n'); issues += 1;
                }
            } else if ll.ends_with(' ') { ll = ll.trim_end().to_string(); issues += 1; }
        }
        norm.push(ll);
    }

    // Second pass: enforce MD012 (no multiple blank lines), MD022 (blank around headings), MD032 (blank around lists)
    let heading_re = regex::Regex::new(r"^\s*#{1,6}\s+\S").unwrap();
    let list_re = regex::Regex::new(r"^\s*([-*+]\s+|\d+\.\s+)\S").unwrap();

    in_fence = false;
    let mut i = 0usize;
    while i < norm.len() {
        let mut line = norm[i].clone();
        if fence_re.is_match(&line) { in_fence = !in_fence; }

        let is_blank = line.trim().is_empty();
        // MD012: collapse multiple blank lines
        if is_blank {
            let prev_blank = out.last().map(|l| l.trim().is_empty()).unwrap_or(false);
            if prev_blank { issues += 1; /* skip adding this extra blank */ i+=1; continue; }
        }

        // Handle headings/lists only when not inside fences
        if !in_fence && heading_re.is_match(&line) {
            let prev_blank = out.last().map(|l| l.trim().is_empty()).unwrap_or(true);
            if !prev_blank { out.push("\n".to_string()); issues += 1; }
            out.push(line);
            // Ensure blank after heading
            let next = norm.get(i+1).cloned().unwrap_or_default();
            if !next.trim().is_empty() { out.push("\n".to_string()); issues += 1; }
            i += 1; continue;
        }

        if !in_fence && list_re.is_match(&line) {
            // Ensure blank line before list block
            let prev_blank = out.last().map(|l| l.trim().is_empty()).unwrap_or(true);
            if !prev_blank { out.push("\n".to_string()); issues += 1; }
            // Emit list block and ensure trailing blank after the block
            while i < norm.len() && list_re.is_match(norm[i].trim()) { out.push(norm[i].clone()); i+=1; }
            let next = norm.get(i).cloned().unwrap_or_default();
            if !next.trim().is_empty() { out.push("\n".to_string()); issues += 1; }
            continue;
        }

        out.push(line);
        i += 1;
    }

    (out.join(""), issues)
}
