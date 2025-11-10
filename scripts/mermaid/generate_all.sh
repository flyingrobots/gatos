#!/usr/bin/env bash
set -euo pipefail

echo "[diagrams] Full regeneration for all Markdown files…"
if command -v node >/dev/null 2>&1; then
  MERMAID_MAX_PARALLEL="${MERMAID_MAX_PARALLEL:-}" node scripts/mermaid/generate.mjs --all
elif command -v docker >/dev/null 2>&1; then
  docker run --rm -e MERMAID_MAX_PARALLEL -v "$PWD:/work" -w /work node:20 bash -lc "npx -y @mermaid-js/mermaid-cli >/dev/null 2>&1; node scripts/mermaid/generate.mjs --all"
else
  echo "Neither node nor docker found; cannot generate diagrams" >&2
  exit 1
fi

echo "[diagrams] Done. Outputs in docs/diagrams/generated/"
