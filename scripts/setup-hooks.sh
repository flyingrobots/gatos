#!/usr/bin/env bash
set -euo pipefail
set -o errtrace

# Error diagnostics: print failing command, line and exit code
trap 'status=$?; cmd=${BASH_COMMAND:-"<unknown>"}; line=${BASH_LINENO[0]:-"?"}; echo "[hooks][ERROR] command failed (exit $status) at line $line: $cmd" >&2' ERR

top="$(git rev-parse --show-toplevel)"
# Resolve absolute .git directory path (handle older Git without --absolute-git-dir)
gitdir_raw="$(git rev-parse --git-dir)"
if abs_gitdir="$(git rev-parse --absolute-git-dir 2>/dev/null)"; then
  gitdir="$abs_gitdir"
else
  case "$gitdir_raw" in
    /*) gitdir="$gitdir_raw" ;;
    *) gitdir="$top/$gitdir_raw" ;;
  esac
  # Normalize to an absolute, symlink-resolved path without relying on realpath
  gitdir="$(cd "$gitdir" && pwd -P)"
fi
hook_src="$top/scripts/hooks/pre-commit"
local_hooks_dir="$gitdir/hooks"
hook_dst="$local_hooks_dir/pre-commit"

install() {
  # Force this repository to use its own hooks directory regardless of any global core.hooksPath
  if [[ ! -f "$hook_src" ]]; then
    echo "[hooks][ERROR] Hook source not found: $hook_src" >&2
    exit 1
  fi
  git config --local core.hooksPath "$local_hooks_dir" || { echo "[hooks][ERROR] git config --local core.hooksPath '$local_hooks_dir' failed" >&2; exit 1; }
  mkdir -p "$local_hooks_dir" || { echo "[hooks][ERROR] Failed to create hooks directory: $local_hooks_dir" >&2; exit 1; }
  cp -f "$hook_src" "$hook_dst" || { echo "[hooks][ERROR] Failed to copy hook to $hook_dst" >&2; exit 1; }
  chmod 0755 "$hook_dst" || { echo "[hooks][ERROR] Failed to chmod +x $hook_dst" >&2; exit 1; }
  echo "[hooks] Installed repo-local pre-commit hook -> $hook_dst"
  echo "[hooks] Configured core.hooksPath locally to: $local_hooks_dir (does not affect global settings)"
}

install
