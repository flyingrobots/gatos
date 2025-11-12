# Contributing

## Developer Tooling

This repo includes optional tooling to keep docs tidy and diagrams fresh.

### Markdown Linting

- CI runs markdownlint (CLI2) via Node 20.
- Local (preferred): no setup required. The pre-commit hook uses `npx` or a Docker fallback so you don’t need Node installed globally.
- Manual runs:

```bash
# Check
npx -y markdownlint-cli2
# Auto-fix
npx -y markdownlint-cli2-fix
```

### Mermaid Diagrams

- Preferred entrypoint (auto-selects Docker or Node):

```bash
bash ./scripts/diagrams.sh            # scan all tracked Markdown files
bash ./scripts/diagrams.sh --all      # explicit full scan
bash ./scripts/diagrams.sh docs/TECH-SPEC.md  # specific file(s)
```

Outputs are written to `docs/diagrams/generated/`.

Modes

- Pre-commit: generates SVGs only for staged Markdown (fast).
- CI: regenerates all diagrams and fails if there’s drift (reproducibility).

Notes

- Concurrency: set `MERMAID_MAX_PARALLEL` (default 6 in CI). Example:

```bash
export MERMAID_MAX_PARALLEL=6
```

- You can also run the Node entrypoint directly if you have Node installed:

```bash
node scripts/mermaid/generate.mjs --all
```

- CI inside containers: set `MERMAID_BACKEND=node` to avoid nested Docker. Ensure Node 20 is present and, for deterministic runs, pin Chromium and pass `PUPPETEER_EXECUTABLE_PATH` (see `.github/workflows/ci.yml`).

- Faster Docker runs in CI: mount caches into the container by exporting `MERMAID_DOCKER_VOLUMES`, for example in GitHub Actions:

```yaml
- name: Generate Mermaid diagrams (full repo) via script
  env:
    MERMAID_DOCKER_VOLUMES: "-v $HOME/.npm:/root/.npm -v $HOME/.cache/puppeteer:/root/.cache/puppeteer"
  run: MERMAID_MAX_PARALLEL=6 bash ./scripts/diagrams.sh --all
```

### Git Hooks

Install the pre-commit hook (runs markdownlint fix + mermaid generation for staged files and stages results; uses Node if available, otherwise Docker with a Node 20 image):

```bash
scripts/setup-hooks.sh
```

If the hook fails, fix the reported issues and retry the commit.

## xtask quickstart (CI parity)

This repo uses a small Rust utility (`cargo xtask`) to run common tasks in a cross-platform, reproducible way.

Prerequisites

- Rust toolchain (install via `rustup`; includes `cargo`)
- Docker (preferred for Mermaid and AJV); Node.js + npm optional locally
- git (for normal development flows)
- Optional: a GitHub Personal Access Token for link checks (set `LYCHEE_GITHUB_TOKEN`); in CI, `GITHUB_TOKEN` is provided automatically

Common commands

- Build/tests: `cargo test --workspace --locked`
- Schemas (AJV compile/validate/negative via Docker): `cargo run -p xtask -- schemas`
- Link check (lychee): `cargo run -p xtask -- links`
  - To avoid GitHub rate limiting locally, export `LYCHEE_GITHUB_TOKEN` (you can also use `export LYCHEE_GITHUB_TOKEN=$GITHUB_TOKEN` in CI).

Diagrams are intentionally outside xtask. Use `make diagrams` or `bash ./scripts/diagrams.sh`.

Tip: `make help` lists handy shims (`ci-diagrams`, `ci-schemas`, `ci-linkcheck`) that mirror CI. For ad-hoc invocations, use `make xtask ARGS="<subcommand> [opts]"` for Rust-based flows.
