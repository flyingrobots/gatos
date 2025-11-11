# ROADMAP — xtask and CI tooling

Status: living document. Tracks phases, decisions, and next steps for repo tooling.

## Goals

- Single entrypoint for repo tasks (diagrams, schemas, links) with CI parity.
- Deterministic, reproducible pipelines (pin tools, cache heavy deps, verify outputs).
- Developer ergonomics: clear Makefile shims + `make help`.

## Landed

- `xtask` crate with `diagrams`, `schemas`, `links` subcommands.
- Mermaid: metadata embedding + verification (src/index/hash/cli), Chromium preinstall in CI.
- Schemas: AJV compile/validate/negative via `xtask schemas`.
- Linkcheck: lychee CLI with `LYCHEE_GITHUB_TOKEN`; binary cached in CI.
- Makefile shims + `help` target; docs(CONTRIBUTING) quickstart.
- Issue labeler + backfill workflows.

## Near-term

- Extract a reusable “puppeteer install” composite Action; add a “CI doctor” step (print versions, paths).
- Decide policy for diagrams: commit SVGs vs verify-only.
- Retire legacy scripts routed through Makefile; rely on `xtask` directly.

## Backlog

- Distribute `xtask` as a prebuilt binary (optional).
- Dev container for local parity.
