#!/usr/bin/env bash
set -euo pipefail

echo "[diagrams] Full regeneration for all Markdown files…"
if command -v node >/dev/null 2>&1; then
  node scripts/mermaid/generate.mjs
elif command -v docker >/dev/null 2>&1; then
  docker run --rm -v "$PWD:/work" -w /work node:20 bash -lc "npx -y @mermaid-js/mermaid-cli >/dev/null 2>&1; node scripts/mermaid/generate.mjs"
else
  echo "Neither node nor docker found; cannot generate diagrams" >&2
  exit 1
fi

echo "[diagrams] Done. Outputs in docs/diagrams/generated/"

