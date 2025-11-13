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
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    case "$line" in
      -v\ *|--volume\ *)
        flag="${line%% *}"; val="${line#* }"; VOLS_ARR+=("$flag" "$val");;
      *) VOLS_ARR+=("-v" "$line");;
    esac
  done <<< "$VOLS_STR"
fi
# Pin Node image for Docker runs (digest corresponds to node:20)
IMAGE_DEFAULT="node@sha256:47dacd49500971c0fbe602323b2d04f6df40a933b123889636fc1f76bf69f58a"
IMAGE="${MERMAID_NODE_IMAGE:-$IMAGE_DEFAULT}"

# Backend selection: auto (default), docker, or node.
BACKEND="${MERMAID_BACKEND:-auto}"

is_in_container() {
  # Heuristics: /.dockerenv or container-related cgroups
  if [ -f "/.dockerenv" ]; then return 0; fi
  if grep -qaE '(docker|containerd|kubepods)' /proc/1/cgroup 2>/dev/null; then return 0; fi
  return 1
}

pick_backend() {
  case "$BACKEND" in
    docker|node) echo "$BACKEND" ; return ;;
    auto)
      if is_in_container; then
        if command -v node >/dev/null 2>&1; then echo node; return; fi
        # no node in container: last resort try docker if available
        if command -v docker >/dev/null 2>&1; then echo docker; return; fi
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
    if [ "$#" -gt 0 ]; then ARGS=("$@"); else ARGS=(--all); fi
    MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs "${ARGS[@]}"
    ;;
  none|*)
    echo "Need Node.js or Docker to render Mermaid diagrams (set MERMAID_BACKEND=node to force Node in containerized CI)." >&2
    exit 1
    ;;
esac
