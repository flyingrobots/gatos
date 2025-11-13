#!/usr/bin/env bash
set -euo pipefail

# Default concurrency: 6 if not provided by caller/CI
CONC="${MERMAID_MAX_PARALLEL:-6}"
# Intentionally unquoted to allow multiple -v segments. This is CI-controlled (see .github/workflows/ci.yml).
# If paths with spaces are ever required, refactor to build an array and append -v entries explicitly.
VOLS_STR="${MERMAID_DOCKER_VOLUMES:-}"
# Build a safe volumes array from MERMAID_DOCKER_VOLUMES. Accept either raw
# "host:container" lines or lines that start with "-v " / "--volume ". Split on newlines.
VOLS_ARR=()
if [ -n "$VOLS_STR" ]; then
  # Parse newline-separated volume specs. Each line is either "-v host:ctr" or "host:ctr".
  # We append the entire "-v host:ctr" token or prepend -v for bare specs.
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    case "$line" in
      -v\ *|--volume\ *) VOLS_ARR+=("$line");;
      *) VOLS_ARR+=("-v" "$line");;
    esac
  done <<< "$VOLS_STR"
fi
# Pins
# Load centralized pins if available
if [ -f "$(dirname "$0")/pins.sh" ]; then . "$(dirname "$0")/pins.sh"; fi
# Pin Node image for Docker runs (digest corresponds to node:20)
IMAGE_DEFAULT="${NODE_IMAGE_DIGEST:-node@sha256:47dacd49500971c0fbe602323b2d04f6df40a933b123889636fc1f76bf69f58a}"
IMAGE="${MERMAID_NODE_IMAGE:-$IMAGE_DEFAULT}"

# Backend selection: auto (default), docker, or node.
BACKEND="${MERMAID_BACKEND:-auto}"

is_in_container() {
  # Allow explicit override via DIAGRAMS_IN_CONTAINER=1|0|true|false
  case "${DIAGRAMS_IN_CONTAINER:-}" in
    1|true|TRUE|yes|YES) return 0;;
    0|false|FALSE|no|NO) return 1;;
  esac
  # Heuristics: special files, cgroup markers (incl. cgroup v2 0::/), and overlay/aufs mounts
  if [ -f "/.dockerenv" ] || [ -f "/run/.containerenv" ]; then return 0; fi
  if grep -qaE '(docker|containerd|kubepods|0::/)' /proc/1/cgroup 2>/dev/null; then return 0; fi
  if grep -qaE '(overlay|aufs)' /proc/self/mountinfo 2>/dev/null; then return 0; fi
  return 1
}

pick_backend() {
  # Strategy:
  # - Respect explicit MERMAID_BACKEND=docker|node.
  # - auto: inside containers prefer node (bundled runtime expected); do not auto-docker if node is missing.
  # - auto: on hosts prefer docker, then node.
  # - Return 'none' if neither is available.
  case "$BACKEND" in
    docker|node) echo "$BACKEND" ; return ;;
    auto)
      if is_in_container; then
        if command -v node >/dev/null 2>&1; then echo node; return; fi
        # No node inside container: do not attempt nested Docker by default
        echo none; return
      else
        if command -v docker >/dev/null 2>&1; then echo docker; return; fi
        if command -v node >/dev/null 2>&1; then echo node; return; fi
        echo none; return
      fi
      ;;
    *) echo none ; return ;;
  esac
}

backend=$(pick_backend)

case "$backend" in
  docker)
    # Note: auto-enumeration uses 'git ls-files -- "*.md"' (tracked .md only).
    # It will not include untracked files or other extensions (e.g., .markdown).
    # To expand coverage, add more patterns here or require explicit file args.
    # If args include --all, enumerate files on host (avoid git inside container); preserve --verify if provided.
    ALL_FLAG=0; VERIFY_FLAG=0
    for a in "$@"; do
      [ "$a" = "--all" ] && ALL_FLAG=1
      [ "$a" = "--verify" ] && VERIFY_FLAG=1
    done
    if [ "$ALL_FLAG" -eq 1 ]; then
      if ! command -v git >/dev/null 2>&1; then
        echo "git is required to enumerate Markdown files (for --all)" >&2; exit 1
      fi
      mapfile -d '' -t FILES < <(git ls-files -z -- '*.md')
      if [ ${#FILES[@]} -eq 0 ]; then
        echo "No tracked Markdown files found"; exit 0
      fi
      ARGS=()
      [ "$VERIFY_FLAG" -eq 1 ] && ARGS+=("--verify")
      ARGS+=("${FILES[@]}")
    else
      # Use provided args as-is
      ARGS=("$@")
    fi
    docker run --rm \
      -e MERMAID_MAX_PARALLEL="$CONC" \
      -v "$PWD:/work" -w /work "${VOLS_ARR[@]}" \
      "$IMAGE" \
      node scripts/mermaid/generate.mjs "${ARGS[@]}"
    ;;
  node)
    if ! command -v node >/dev/null 2>&1; then
      echo "Node.js is required for MERMAID_BACKEND=node (or when running inside a container without Docker)." >&2
      exit 1
    fi
    if [ "$#" -gt 0 ]; then
      ARGS=("$@")
    else
      ARGS=(--all)
    fi
    MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs "${ARGS[@]}"
    ;;
  none|*)
    echo "No suitable backend available for diagrams." >&2
    echo "Hints:" >&2
    echo " - Inside containers: set MERMAID_BACKEND=node if Node is available." >&2
    echo " - On hosts: install Docker or Node, or run on a machine with Docker." >&2
    echo " - Override detection via DIAGRAMS_IN_CONTAINER=1|0 to force/skip container heuristics." >&2
    exit 1
    ;;
esac
