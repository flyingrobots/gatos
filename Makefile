.PHONY: all clean test diagrams lint-md fix-md link-check schemas schema-compile schema-validate schema-negative pre-commit \
        xtask ci-diagrams ci-schemas ci-linkcheck help

all: schemas lint-md link-check

clean:
	@rm -f docs/diagrams/generated/*.svg || true

test:
	@cargo test --workspace --locked

diagrams:
	@bash -lc 'scripts/mermaid/generate_all.sh'

lint-md:
	@bash -lc 'if command -v node >/dev/null 2>&1; then \
      npx -y markdownlint-cli "**/*.md" --config .markdownlint.json; \
	elif command -v docker >/dev/null 2>&1; then \
      docker run --rm -v "$$PWD:/work" -w /work node:20 bash -lc "npx -y markdownlint-cli \"**/*.md\" --config .markdownlint.json"; \
	else echo "Need Node.js or Docker" >&2; exit 1; fi'

fix-md:
	@bash -lc 'if command -v node >/dev/null 2>&1; then \
      npx -y markdownlint-cli "**/*.md" --fix --config .markdownlint.json; \
	elif command -v docker >/dev/null 2>&1; then \
      docker run --rm -v "$$PWD:/work" -w /work node:20 bash -lc "npx -y markdownlint-cli \"**/*.md\" --fix --config .markdownlint.json"; \
	else echo "Need Node.js or Docker" >&2; exit 1; fi'

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
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/job_manifest.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/proof_of_execution_envelope.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proposal.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/approval.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/grant.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/revocation.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proof_of_consensus_envelope.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/privacy/opaque_pointer.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/event_envelope.schema.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/consumer_checkpoint.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/deployment_trailer.schema.json && \
	npx -y ajv-cli@5 compile --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/anchor.schema.json'

schema-validate:
	@bash -lc 'set -euo pipefail; \
	 if ! command -v node >/dev/null 2>&1; then \
	   echo "Node.js required (or run in CI)" >&2; exit 1; fi; \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/job_manifest.schema.json -d examples/v1/job/manifest_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/job/proof_of_execution_envelope.schema.json -d examples/v1/job/poe_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proposal.schema.json -d examples/v1/governance/proposal_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/approval.schema.json -d examples/v1/governance/approval_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/grant.schema.json -d examples/v1/governance/grant_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/revocation.schema.json -d examples/v1/governance/revocation_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/governance/proof_of_consensus_envelope.schema.json -d examples/v1/governance/poc_envelope_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json -d examples/v1/policy/governance_min.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/privacy/opaque_pointer.schema.json -d examples/v1/privacy/opaque_pointer_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/event_envelope.schema.json -d examples/v1/shiplog/event_min.json -r schemas/v1/common/ids.schema.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/consumer_checkpoint.schema.json -d examples/v1/shiplog/checkpoint_min.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/deployment_trailer.schema.json -d examples/v1/shiplog/trailer_min.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/anchor.schema.json -d examples/v1/shiplog/anchor_min.json && \
	npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/privacy/opaque_pointer.schema.json -d examples/v1/privacy/pointer_low_entropy_min.json'

schema-negative:
	@bash -lc 'set -euo pipefail; \
	 if ! command -v node >/dev/null 2>&1; then \
	   echo "Node.js required (or run in CI)" >&2; exit 1; fi; \
	# Negative: checkpoint requires both fields
	! npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/consumer_checkpoint.schema.json -d examples/v1/shiplog/checkpoint_ulid_only_invalid.json; \
	! npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/shiplog/consumer_checkpoint.schema.json -d examples/v1/shiplog/checkpoint_commit_only_invalid.json; \
	# Negative: low-entropy pointer must not allow plaintext digest
	! npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/privacy/opaque_pointer.schema.json -d examples/v1/privacy/pointer_low_entropy_invalid.json'

.PHONY: kill-check
kill-check:
	@bash -lc 'scripts/killcheck/schema_headers.sh'
	@bash -lc 'scripts/killcheck/ulid_reference.sh'
	@bash -lc 'scripts/killcheck/error_casing.sh'

schema-negative:
	@bash -lc 'set -euo pipefail; \
	 if ! command -v node >/dev/null 2>&1; then \
	   echo "Node.js required (or run in CI)" >&2; exit 1; fi; \
	 echo "{\"governance\":{\"x\":{\"ttl\":\"P\"}}}" > /tmp/bad1.json; \
	 echo "{\"governance\":{\"x\":{\"ttl\":\"PT\"}}}" > /tmp/bad2.json; \
	 if npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad1.json; then \
	   echo "Should have rejected ttl=P" >&2; exit 1; else echo "Rejected ttl=P as expected"; fi; \
	 if npx -y ajv-cli@5 validate --spec=draft2020 --strict=true -c ajv-formats -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad2.json; then \
	   echo "Should have rejected ttl=PT" >&2; exit 1; else echo "Rejected ttl=PT as expected"; fi'

schemas: schema-compile schema-validate schema-negative

pre-commit:
	@bash -lc 'set -euo pipefail; \
	 echo "[make pre-commit] markdownlint fix…"; \
	 if [ -n "$$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then \
	   if command -v node >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 npx -y markdownlint-cli --fix --config .markdownlint.json --; \
	   elif command -v docker >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 -I{} docker run --rm -v "$$PWD:/work" -w /work node:20 bash -lc \
	           "npx -y markdownlint-cli --fix --config .markdownlint.json -- \"{}\""; \
	   else echo "Need Node.js or Docker" >&2; exit 1; fi; \
	   git diff --cached --name-only -z --diff-filter=ACM -- "*.md" | xargs -0 git add --; \
	 fi; \
	 echo "[make pre-commit] Prettier JSON/YAML…"; \
	 if [ -n "$$(git diff --cached --name-only --diff-filter=ACM -- "*.json" "*.yml" "*.yaml")" ]; then \
	   if command -v node >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.json" "*.yml" "*.yaml" \
	       | xargs -0 npx -y prettier -w --; \
	   elif command -v docker >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.json" "*.yml" "*.yaml" \
	       | xargs -0 -I{} docker run --rm -v "$$PWD:/work" -w /work node:20 bash -lc \
	           "npx -y prettier -w -- \"{}\""; \
	   else echo "Need Node.js or Docker" >&2; exit 1; fi; \
	   git diff --cached --name-only -z --diff-filter=ACM -- "*.json" "*.yml" "*.yaml" | xargs -0 git add --; \
	 fi; \
	 echo "[make pre-commit] Mermaid (staged MD only)…"; \
	 if [ -n "$$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then \
	   if command -v node >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 node scripts/mermaid/generate.mjs; \
	   elif command -v docker >/dev/null 2>&1; then \
	     git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
	       | xargs -0 -I{} docker run --rm -v "$$PWD:/work" -w /work node:20 bash -lc \
	           "npx -y @mermaid-js/mermaid-cli >/dev/null 2>&1; node scripts/mermaid/generate.mjs \"{}\""; \
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
# ci-diagrams: Generate all Mermaid diagrams. This target sets MERMAID_MAX_PARALLEL
# for faster local/CI runs; other targets do not use parallelism env vars.
ci-diagrams:
	@MERMAID_MAX_PARALLEL=${MERMAID_MAX_PARALLEL:-6} cargo run -p xtask -- diagrams --all

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
	@echo "xtask shims:"; \
	echo "  make xtask ARGS=\"<subcommand> [opts]\"  — passthrough to xtask (e.g., diagrams --all, links, schemas)"; \
	echo "  make ci-diagrams                       — generate all Mermaid diagrams (MERMAID_MAX_PARALLEL honored)"; \
	echo "  make ci-schemas                        — validate and compile all schemas/examples"; \
	echo "  make ci-linkcheck                      — run Markdown link checks"; \
	echo "Env: MERMAID_MAX_PARALLEL overrides diagram concurrency (default 6 here)";
