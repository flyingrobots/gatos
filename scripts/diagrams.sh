#!/usr/bin/env bash
set -euo pipefail

# Default concurrency: 6 if not provided by caller/CI
CONC="${MERMAID_MAX_PARALLEL:-6}"

if command -v docker >/dev/null 2>&1; then
  # Dockerized Node fallback; pass explicit concurrency into container
  if [ "$#" -gt 0 ]; then ARGS="$*"; else ARGS="--all"; fi
  docker run --rm \
    -e MERMAID_MAX_PARALLEL="$CONC" \
    -v "$PWD:/work" -w /work \
    node:20 bash -lc 'npx -y @mermaid-js/mermaid-cli >/dev/null 2>&1; MERMAID_MAX_PARALLEL="'$CONC'" node scripts/mermaid/generate.mjs '$ARGS''
elif command -v node >/dev/null 2>&1; then
  # Host Node path; pass explicit concurrency to mermaid via env
  if [ "$#" -gt 0 ]; then MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs "$@"; else MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs --all; fi
else
  echo "Need Node.js or Docker to render Mermaid diagrams" >&2
  exit 1
fi
