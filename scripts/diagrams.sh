#!/usr/bin/env bash
set -euo pipefail

# Default concurrency: 6 if not provided by caller/CI
CONC="${MERMAID_MAX_PARALLEL:-6}"

if command -v docker >/dev/null 2>&1; then
  # Dockerized Node preferred; pass explicit concurrency as env; safely quote args
  if [ "$#" -gt 0 ]; then
    # Build a shell-escaped argument string
    ARGS_Q=""
    for a in "$@"; do
      ARGS_Q+=" $(printf '%q' "$a")"
    done
    CMD="npx -y @mermaid-js/mermaid-cli >/dev/null 2>&1; node scripts/mermaid/generate.mjs${ARGS_Q}"
  else
    CMD="npx -y @mermaid-js/mermaid-cli >/dev/null 2>&1; node scripts/mermaid/generate.mjs --all"
  fi
  docker run --rm \
    -e MERMAID_MAX_PARALLEL="$CONC" \
    -v "$PWD:/work" -w /work \
    node:20 bash -lc "$CMD"
elif command -v node >/dev/null 2>&1; then
  # Host Node path; pass explicit concurrency to mermaid via env
  if [ "$#" -gt 0 ]; then
    MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs "$@"
  else
    MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs --all
  fi
else
  echo "Need Node.js or Docker to render Mermaid diagrams" >&2
  exit 1
fi
