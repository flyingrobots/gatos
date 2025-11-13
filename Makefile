.PHONY: all clean test diagrams lint-md fix-md link-check schemas schema-compile schema-validate schema-negative pre-commit \
        xtask ci-diagrams ci-schemas ci-linkcheck help setup-dev lint-all

# Pinned Node image used for ad-hoc Docker invocations (keep in sync with scripts)
NODE_IMAGE_DIGEST ?= $(shell bash -c '. ./scripts/pins.sh 2>/dev/null || true; printf "%s" "$$NODE_IMAGE_DIGEST" | sed -e "s/^$/node@sha256:47dacd49500971c0fbe602323b2d04f6df40a933b123889636fc1f76bf69f58a/"')

all: schemas lint-md link-check

clean:
	@rm -f docs/diagrams/generated/*.svg || true

test:
	@cargo test --workspace --locked

# Generate Mermaid diagrams for the entire repo (one-liner; script handles Node/Docker + defaults)
diagrams:
	@bash -eu -o pipefail ./scripts/diagrams.sh


# Markdown lint via xtask (no Node required)
lint-md:
	@bash -c 'if command -v rumdl >/dev/null 2>&1; then rumdl check .; else cargo run -p xtask -- md; fi'

fix-md:
	@bash -c 'if command -v rumdl >/dev/null 2>&1; then rumdl check . --fix; else cargo run -p xtask -- md --fix; fi'

link-check:
	@bash -lc 'if command -v lychee >/dev/null 2>&1; then \
	  lychee --no-progress --config .lychee.toml **/*.md; \
	elif command -v docker >/dev/null 2>&1; then \
	  docker run --rm -v "$$PWD:/work" -w /work ghcr.io/lycheeverse/lychee:latest --no-progress --config .lychee.toml **/*.md; \
	else echo "Need lychee or Docker" >&2; exit 1; fi'

schema-compile:
	@bash -eu -o pipefail ./scripts/validate_schemas.sh --compile-only

schema-validate:
	@bash -eu -o pipefail ./scripts/validate_schemas.sh --validate-only

schema-negative:
	@bash -eu -o pipefail ./scripts/validate_schemas.sh --negative-only

schemas: schema-compile schema-validate schema-negative

pre-commit:
	@bash -eu -o pipefail ./scripts/hooks/run-pre-commit.sh
## ---- xtask shims (Rust orchestrator) ----

# Generic xtask passthrough: use ARGS to forward to the xtask CLI.
# Example: `make xtask ARGS="diagrams --all"` or `make xtask ARGS="links"`
xtask:
	@cargo run -p xtask -- $(ARGS)

# CI-parity shims
# ci-diagrams: Generate all Mermaid diagrams via the shell wrapper.
ci-diagrams:
	@MERMAID_MAX_PARALLEL=${MERMAID_MAX_PARALLEL:-6} bash -eu -o pipefail ./scripts/diagrams.sh --all

# ci-schemas: Validate and compile all JSON Schemas and example payloads.
# No special env vars required; xtask handles Node/AJV invocation.
ci-schemas:
	@cargo run -p xtask -- schemas

# ci-linkcheck: Run Markdown link checks (uses local lychee if available, else Docker).
# Argument styles (by xtask subcommand):
#   - diagrams: uses the `--all` flag for full-repo scan.
#   - schemas: no `all` positional argument (runs the full suite by default).
#   - links: no `all` positional argument (optionally accepts file globs; default is **/*.md).
# If xtask subcommand signatures change, update this note and the shims below to keep rationale clear.
ci-linkcheck:
	@cargo run -p xtask -- links

# help: list available xtask-related targets for quick discovery
help:
	@echo "Tooling entrypoints:"; \
	echo "  make diagrams                          — render all diagrams via scripts/diagrams.sh (Docker/Node autodetect)"; \
	echo "  make ci-diagrams                       — same, explicit full-scan; honors MERMAID_MAX_PARALLEL (script handles concurrency)"; \
	echo "  make xtask ARGS=\"<subcommand> [opts]\"  — run Rust-only tasks (schemas, links, md)"; \
	echo "  make ci-schemas                        — validate and compile all schemas/examples (xtask)"; \
	echo "  make ci-linkcheck                      — run Markdown link checks (xtask)"; \
	echo "  make lint-all                          — run md + schemas + diagrams verify (CI-like)"; \
	echo "Notes: diagrams are not handled by xtask; set MERMAID_MAX_PARALLEL for scripts/diagrams.sh only."; \
	echo "  make setup-dev                         — install repo-local hooks and tools (one-time)";

# Aggregate CI-like checks
lint-all:
	@bash -c 'set -euo pipefail; \
	  echo "[lint-all] markdown (xtask md)…"; \
	  cargo run -p xtask -- md; \
	  echo "[lint-all] schemas (xtask schemas)…"; \
	  cargo run -p xtask -- schemas; \
	  echo "[lint-all] diagrams verify (wrapper)…"; \
	  MERMAID_MAX_PARALLEL=$${MERMAID_MAX_PARALLEL:-6} bash ./scripts/diagrams.sh --verify --all'
# One-step developer setup: install hooks and recommended CLI tools
setup-dev:
	@bash -eu -o pipefail ./scripts/setup-dev.sh
