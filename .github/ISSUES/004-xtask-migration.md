Title: Migrate repo task orchestration to `cargo xtask` (and aliases)

Summary

Adopt the Rust “xtask” pattern to orchestrate common repo tasks (pre-commit flow, Mermaid generation, schema compile/validate, link-check, markdownlint fix) via `cargo xtask …`. Keep Makefile targets as thin pass-throughs. Update pre-commit hook and CI to call `cargo xtask` so developers get a single, portable entrypoint without requiring GNU make.

Motivation

- Cross-platform: removes dependency on `make` (esp. Windows environments).
- Consistency: devs, CI, and hooks all call the same code path.
- Testability: orchestration logic lives in Rust; easier to unit/integration test.
- UX: add helpful errors (detect missing `node`/`docker`, give hints), structured logging.

Proposed

- Create a new binary crate `crates/xtask` (private) using `clap` for subcommands.
- Subcommands:
  - `pre-commit` — staged-only: markdownlint fix, Prettier (json/yaml), Mermaid gen, link-check.
  - `diagrams gen` [--files …] — staged or full; `diagrams gen-all` full.
  - `schemas compile|validate|negative` — mirrors CI AJV jobs; draft2020; strict.
  - `lint md [--fix]` — runs markdownlint; mirrors CI.
  - `links check [--files …]` — lychee (staged/all).
- Tooling strategy:
  - Prefer `node`+`npx`; fallback to `docker` (node:20) when tools are missing.
  - Preserve current behavior and messages.
- Wiring:
  - Pre-commit hook: replace body with `cargo xtask pre-commit`.
  - CI: replace Makefile steps with `cargo xtask …` keeping identical flags and drift checks.
  - Keep Makefile targets as shims to `cargo xtask` (for muscle memory).

Acceptance Criteria

- `cargo xtask pre-commit` produces identical staged changes to current hook on a representative commit.
- CI passes with xtask wiring (schemas compile/validate + negative tests, markdownlint, diagrams drift, link-check).
- Makefile targets call into xtask successfully (`make diagrams`, `make lint-md`, `make fix-md`, `make link-check`).
- CONTRIBUTING updated to prefer `cargo xtask` usage, mention Makefile shims.

Tasks

- [ ] Scaffold `crates/xtask` with `clap` and a minimal `main.rs`.
- [ ] Implement subcommands: pre-commit, diagrams gen/gen-all, schemas compile/validate/negative, lint md (fix/verify), links check.
- [ ] Add helper to shell out with pretty errors and `docker` fallback.
- [ ] Update pre-commit hook to call `cargo xtask pre-commit`.
- [ ] Replace CI steps with `cargo xtask …` invocations (preserve `--spec=draft2020`, drift checks, caches).
- [ ] Update CONTRIBUTING (new section: Using `cargo xtask`).
- [ ] Optional: add `cargo` aliases in `.cargo/config.toml` (e.g., `pc = "xtask pre-commit"`).

Notes

- We’ll still depend on Node/Docker at execution time for external tools (markdownlint, AJV, Mermaid, lychee), but xtask centralizes orchestration in Rust.
- This is a no-behavior-change refactor; focus on parity first, then iterate.

References

- https://github.com/matklad/cargo-xtask (the pattern)
- Current CI workflow: `.github/workflows/ci.yml`
- Current hook and scripts: `scripts/hooks/pre-commit`, `scripts/mermaid/generate.mjs`
