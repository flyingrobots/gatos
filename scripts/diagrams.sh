#!/usr/bin/env bash
set -euo pipefail

# Default concurrency: 6 if not provided by caller/CI
CONC="${MERMAID_MAX_PARALLEL:-6}"
VOLS="${MERMAID_DOCKER_VOLUMES:-}"

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
    if [ "$#" -gt 0 ]; then
      docker run --rm \
        -e MERMAID_MAX_PARALLEL="$CONC" \
        -v "$PWD:/work" -w /work $VOLS \
        node:20 \
        node scripts/mermaid/generate.mjs "$@"
    else
      # Enumerate tracked Markdown files on the host to avoid requiring git inside the container
      if ! command -v git >/dev/null 2>&1; then
        echo "git is required to enumerate Markdown files (for --all)" >&2; exit 1
      fi
      # Read NUL-delimited list safely into an array
      mapfile -d '' -t FILES < <(git ls-files -z -- '*.md')
      if [ ${#FILES[@]} -eq 0 ]; then
        echo "No tracked Markdown files found"; exit 0
      fi
      docker run --rm \
        -e MERMAID_MAX_PARALLEL="$CONC" \
        -v "$PWD:/work" -w /work $VOLS \
        node:20 \
        node scripts/mermaid/generate.mjs "${FILES[@]}"
    fi
    ;;
  node)
    if ! command -v node >/dev/null 2>&1; then
      echo "Node.js is required for MERMAID_BACKEND=node (or when running inside a container without Docker)." >&2
      exit 1
    fi
    if [ "$#" -gt 0 ]; then
      MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs "$@"
    else
      MERMAID_MAX_PARALLEL="$CONC" node scripts/mermaid/generate.mjs --all
    fi
    ;;
  none|*)
    echo "Need Node.js or Docker to render Mermaid diagrams (set MERMAID_BACKEND=node to force Node in containerized CI)." >&2
    exit 1
    ;;
esac
