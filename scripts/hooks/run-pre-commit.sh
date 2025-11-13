#!/usr/bin/env bash
set -euo pipefail

# One place to maintain pre-commit logic; called by Makefile and/or hook.

top="$(git rev-parse --show-toplevel)"
cd "$top"

# Load pins (image digests, versions)
if [ -f scripts/pins.sh ]; then . scripts/pins.sh; fi
NODE_IMG="${NODE_IMAGE_DIGEST:-node:20}"

echo "[pre-commit] markdown lint (prefer rumdl, then xtask md --fix)…"
if [ -n "$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then
  if command -v rumdl >/dev/null 2>&1; then
    rumdl check . --fix
  else
    cargo run -p xtask -- md --fix
  fi
  git diff --cached --name-only -z --diff-filter=ACM -- "*.md" | xargs -0 git add --
fi

echo "[pre-commit] JSON/YAML formatting (dprint)…"
if [ -n "$(git diff --cached --name-only --diff-filter=ACM -- "*.json" "*.yml" "*.yaml")" ]; then
  if command -v dprint >/dev/null 2>&1; then
    dprint fmt
  else
    # Only prompt in non-CI TTYs; otherwise warn and continue
    if [ -t 1 ] && [ -z "${CI:-}" ]; then
      echo "Looks like you haven't installed the required development workflow tools yet." >&2
      printf "Do you want to install them now? [Yes/no] " >&2
      read -r REPLY; REPLY=${REPLY:-Yes}
      case "$REPLY" in
        y|Y|yes|Yes|YES)
          echo "[pre-commit] Installing tools and hooks…" >&2
          bash ./scripts/setup-dev.sh || true
          if command -v dprint >/dev/null 2>&1; then
            echo "[pre-commit] dprint installed; formatting JSON/YAML…" >&2
            dprint fmt
          else
            echo "[pre-commit][WARN] dprint still not available; continuing." >&2
          fi
          ;;
        *)
          echo "OK… but you should REALLY consider installing them." >&2
          echo "You can do so with:  make setup-dev\n\nContinuing… but grumbling." >&2
          ;;
      esac
    else
      echo "[pre-commit][WARN] dprint not found; skipping JSON/YAML formatting (CI enforces)." >&2
    fi
  fi
  git diff --cached --name-only -z --diff-filter=ACM -- "*.json" "*.yml" "*.yaml" | xargs -0 git add --
fi

echo "[pre-commit] Mermaid (staged MD only)…"
if [ -n "$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then
  if command -v node >/dev/null 2>&1; then
    git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
      | xargs -0 node scripts/mermaid/generate.mjs
  elif command -v docker >/dev/null 2>&1; then
    git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
      | xargs -0 docker run --rm -v "$PWD:/work" -w /work "$NODE_IMG" \
          node scripts/mermaid/generate.mjs
  else
    echo "[pre-commit][ERROR] Need Node.js or Docker for Mermaid generation." >&2
    exit 1
  fi
  if [ -d docs/diagrams/generated ]; then git add -- docs/diagrams/generated; fi
fi

echo "[pre-commit] Link check (staged MD)…"
if [ -n "$(git diff --cached --name-only --diff-filter=ACM -- "*.md")" ]; then
  if command -v lychee >/dev/null 2>&1; then
    git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
      | xargs -0 lychee --no-progress --config .lychee.toml --
  elif command -v docker >/dev/null 2>&1; then
    git diff --cached --name-only -z --diff-filter=ACM -- "*.md" \
      | xargs -0 -I{} docker run --rm -v "$PWD:/work" -w /work ghcr.io/lycheeverse/lychee:latest \
          --no-progress --config .lychee.toml "{}"
  else
    echo "[pre-commit][WARN] lychee not found and Docker unavailable; skipping link check" >&2
  fi
fi

echo "[pre-commit] Done."

