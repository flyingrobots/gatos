#!/usr/bin/env bash
set -euo pipefail

# Default concurrency: 6 if not provided by caller/CI
CONC="${MERMAID_MAX_PARALLEL:-6}"

if command -v node >/dev/null 2>&1; then
  # Use xtask path (preferred); pass explicit concurrency
  MERMAID_MAX_PARALLEL="$CONC" cargo run -p xtask -- diagrams --all
elif command -v docker >/dev/null 2>&1; then
  # Dockerized Node fallback; pass explicit concurrency into container
  docker run --rm \
    -e MERMAID_MAX_PARALLEL="$CONC" \
    -v "$PWD:/work" -w /work \
    node:20 bash -lc 'npx -y @mermaid-js/mermaid-cli >/dev/null 2>&1; node scripts/mermaid/generate.mjs --all'
else
  echo "Need Node.js or Docker to render Mermaid diagrams" >&2
  exit 1
fi

