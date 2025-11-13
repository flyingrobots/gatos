.PHONY: all clean test diagrams lint-md fix-md link-check schemas schema-compile schema-validate schema-negative pre-commit \
        xtask ci-diagrams ci-schemas ci-linkcheck help setup-dev

all: schemas lint-md link-check

clean:
	@rm -f docs/diagrams/generated/*.svg || true

test:
	@cargo test --workspace --locked

# Generate Mermaid diagrams for the entire repo (one-liner; script handles Node/Docker + defaults)
diagrams:
	@bash -c 'bash ./scripts/diagrams.sh'


# Markdown lint via xtask (no Node required)
lint-md:
	@bash -lc 'if command -v rumdl >/dev/null 2>&1; then rumdl check .; else cargo run -p xtask -- md; fi'

fix-md:
	@bash -lc 'if command -v rumdl >/dev/null 2>&1; then rumdl check . --fix; else cargo run -p xtask -- md --fix; fi'

link-check:
	@bash -lc 'if command -v lychee >/dev/null 2>&1; then \
	  lychee --no-progress --config .lychee.toml **/*.md; \
	elif command -v docker >/dev/null 2>&1; then \
	  docker run --rm -v "$$PWD:/work" -w /work ghcr.io/lycheeverse/lychee:latest --no-progress --config .lychee.toml **/*.md; \
	else echo "Need lychee or Docker" >&2; exit 1; fi'

schema-compile:
	@bash -lc 'set -euo pipefail; \
	 if ! command -v node >/dev/null 2>&1; then \
	   echo "Node.js required (or run in CI)" >&2; exit 1; fi; \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/job_manifest.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/proof_of_execution_envelope.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proposal.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/approval.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/grant.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/revocation.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proof_of_consensus_envelope.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json'

schema-validate:
	@bash -lc 'set -euo pipefail; \
	 if ! command -v node >/dev/null 2>&1; then \
	   echo "Node.js required (or run in CI)" >&2; exit 1; fi; \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/job_manifest.schema.json -d examples/v1/job/manifest_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/proof_of_execution_envelope.schema.json -d examples/v1/job/poe_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proposal.schema.json -d examples/v1/governance/proposal_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/approval.schema.json -d examples/v1/governance/approval_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/grant.schema.json -d examples/v1/governance/grant_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/revocation.schema.json -d examples/v1/governance/revocation_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proof_of_consensus_envelope.schema.json -d examples/v1/governance/poc_envelope_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y -p ajv-cli@5 -p ajv-formats@3 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json -d examples/v1/policy/governance_min.json'

schema-negative:
	@bash -lc 'set -euo pipefail; \
	 if ! command -v node >/dev/null 2>&1; then \
	   echo "Node.js required (or run in CI)" >&2; exit 1; fi; \
	 echo "{\"governance\":{\"x\":{\"ttl\":\"P\"}}}" > /tmp/bad1.json; \
	 echo "{\"governance\":{\"x\":{\"ttl\":\"PT\"}}}" > /tmp/bad2.json; \
	 if npx -y ajv-cli@5 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad1.json; then \
	   echo "Should have rejected ttl=P" >&2; exit 1; else echo "Rejected ttl=P as expected"; fi; \
	 if npx -y ajv-cli@5 ajv validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad2.json; then \
	   echo "Should have rejected ttl=PT" >&2; exit 1; else echo "Rejected ttl=PT as expected"; fi'

schemas: schema-compile schema-validate schema-negative

pre-commit:
	@bash -lc 'set -euo pipefail; \
	 echo "[make pre-commit] markdown lint (prefer rumdl, then xtask md --fix)…"; \
	 if [ -n "$$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then \
	   if command -v rumdl >/dev/null 2>&1; then rumdl check . --fix; \
	   else cargo run -p xtask -- md --fix; fi; \
	   git diff --cached --name-only -z --diff-filter=ACM -- "*.md" | xargs -0 git add --; \
	 fi; \
	 echo "[make pre-commit] JSON/YAML formatting (dprint)…"; \
	 if [ -n "$$(git diff --cached --name-only --diff-filter=ACM -- "*.json" "*.yml" "*.yaml")" ]; then \
	   if command -v dprint >/dev/null 2>&1; then \
	     dprint fmt; \
	   else \
	     # Interactive prompt if possible; otherwise emit a stern warning and continue
	     if [ -t 1 ]; then \
	       echo "Looks like you haven't installed the required development workflow tools yet." >&2; \
	       printf "Do you want to install them now? [Yes/no] " >&2; \
	       read -r REPLY; REPLY=$${REPLY:-Yes}; \
	       case "$${REPLY}" in \
	         y|Y|yes|Yes|YES) \
	           echo "[pre-commit] Installing tools and hooks…" >&2; \
	           bash ./scripts/setup-dev.sh || true; \
	           if command -v dprint >/dev/null 2>&1; then \
	             echo "[pre-commit] dprint installed; formatting staged JSON/YAML…" >&2; \
	             dprint fmt; \
	           else \
	             echo "[pre-commit][WARN] dprint still not available after install; continuing." >&2; \
	           fi \
	           ;; \
	         *) \
	           echo "OK… but you should REALLY consider installing them." >&2; \
	           echo "You can do so with:  make setup-dev\n\nContinuing… but grumbling." >&2; \
	           ;; \
	       esac; \
	     else \
	       echo "" >&2; \
	       echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" >&2; \
	       echo "[pre-commit][WARN][MISSING DPRINT] Skipping JSON/YAML formatting for staged files." >&2; \

	       echo "Install our hooks and tools with:  make setup-dev   (or: bash scripts/setup-dev.sh)" >&2; \
	       echo "CI will fail formatting if this is not fixed." >&2; \
	       echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" >&2; \
	     fi; \
	   fi; \
	   git diff --cached --name-only -z --diff-filter=ACM -- "*.json" "*.yml" "*.yaml" | xargs -0 git add --; \
	 fi; \
	 echo "[make pre-commit] Mermaid (staged MD only)…"; \
	 if [ -n "$$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then \
	   if command -v node >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 node scripts/mermaid/generate.mjs; \
	   elif command -v docker >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 -I{} docker run --rm -v "$$PWD:/work" -w /work node@sha256:47dacd49500971c0fbe602323b2d04f6df40a933b123889636fc1f76bf69f58a \
	           node scripts/mermaid/generate.mjs \"{}\"; \
	   else echo "Need Node.js or Docker" >&2; exit 1; fi; \
	   if [ -d docs/diagrams/generated ]; then git add -- docs/diagrams/generated; fi; \
	 fi; \
	 echo "[make pre-commit] Link check (staged MD)…"; \
	 if [ -n "$$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then \
	   if command -v lychee >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 lychee --no-progress --config .lychee.toml --; \
	   elif command -v docker >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 -I{} docker run --rm -v "$$PWD:/work" -w /work ghcr.io/lycheeverse/lychee:latest \
	           --no-progress --config .lychee.toml \"{}\"; \
	   else echo "lychee not found and Docker unavailable; skipping link check" >&2; fi; \
	 fi; \
	 echo "[make pre-commit] Done."'
## ---- xtask shims (Rust orchestrator) ----

# Generic xtask passthrough: use ARGS to forward to the xtask CLI.
# Example: `make xtask ARGS="diagrams --all"` or `make xtask ARGS="links"`
xtask:
	@cargo run -p xtask -- $(ARGS)

# CI-parity shims
# ci-diagrams: Generate all Mermaid diagrams via the shell wrapper.
ci-diagrams:
	@MERMAID_MAX_PARALLEL=${MERMAID_MAX_PARALLEL:-6} bash -c 'bash ./scripts/diagrams.sh --all'

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
	echo "Notes: diagrams are not handled by xtask; set MERMAID_MAX_PARALLEL for scripts/diagrams.sh only."; \
	echo "  make setup-dev                         — install repo-local hooks and tools (one-time)";
# One-step developer setup: install hooks and recommended CLI tools
setup-dev:
	@bash -lc 'bash ./scripts/setup-dev.sh'
