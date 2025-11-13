#!/usr/bin/env bash
set -euo pipefail

# Verify and report status of version pins used in CI/tooling.
# This is a best-effort helper; it degrades gracefully if optional tools are missing.

HERE=$(cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=./pins.sh
. "$HERE/pins.sh"

echo "[pins] MERMAID_CLI_VERSION = ${MERMAID_CLI_VERSION:-}"
echo "[pins] MERMAID_CLI_PREV_ALLOW = ${MERMAID_CLI_PREV_ALLOW:-}"
echo "[pins] NODE_IMAGE_DIGEST = ${NODE_IMAGE_DIGEST:-}"
echo "[pins] CHROMIUM_REVISION = ${CHROMIUM_REVISION:-}"

echo
echo "[check] Latest @mermaid-js/mermaid-cli on npm:"
if command -v npm >/dev/null 2>&1; then
  npm view @mermaid-js/mermaid-cli version || true
else
  echo "npm not available; skip"
fi

echo
echo "[check] Docker node:20 digest vs pin:"
if command -v docker >/dev/null 2>&1; then
  DIGEST=$(docker manifest inspect node:20 --verbose 2>/dev/null | sed -n 's/\s*Digest: //p' | head -n1 || true)
  if [ -n "${DIGEST:-}" ]; then
    echo "resolved: node@${DIGEST}"
    if [ "node@${DIGEST}" = "${NODE_IMAGE_DIGEST}" ]; then
      echo "OK: pin matches node:20"
    else
      echo "WARN: pin mismatch (expected ${NODE_IMAGE_DIGEST})"
    fi
  else
    echo "Could not resolve digest (registry access or docker setup issue)"
  fi
else
  echo "docker not available; skip"
fi

echo
echo "[note] Puppeteer/Chromium: CI installs a specific Chromium via @puppeteer/browsers.\n       Keep CHROMIUM_REVISION in sync with the Puppeteer version bundled by mermaid-cli."

