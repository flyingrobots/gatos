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
local_hooks_dir="$gitdir/hooks"
hooks=(pre-commit pre-push)

install() {
  # Configure repo-local hooks path only (never touches global config)
  git config --local core.hooksPath "$local_hooks_dir" || { echo "[hooks][ERROR] git config --local core.hooksPath '$local_hooks_dir' failed" >&2; exit 1; }
  mkdir -p "$local_hooks_dir" || { echo "[hooks][ERROR] Failed to create hooks directory: $local_hooks_dir" >&2; exit 1; }

  for name in "${hooks[@]}"; do
    src="$top/scripts/hooks/$name"
    dst="$local_hooks_dir/$name"
    if [[ ! -f "$src" || ! -r "$src" ]]; then
      echo "[hooks][WARN] Skipping missing hook: $src"
      continue
    fi
    cp -f "$src" "$dst" || { echo "[hooks][ERROR] Failed to copy hook to $dst" >&2; exit 1; }
    chmod 0755 "$dst" || { echo "[hooks][ERROR] Failed to chmod +x $dst" >&2; exit 1; }
    echo "[hooks] Installed repo-local $name hook -> $dst"
  done

  # Warn if a global core.hooksPath is present (can override repo-local hooks in some setups)
  if global_path="$(git config --global --get core.hooksPath 2>/dev/null)" && [[ -n "${global_path:-}" ]]; then
    echo "[hooks][WARN] A global core.hooksPath is set to: $global_path" >&2
    echo "[hooks][WARN] This script does NOT modify global config. This repo uses: $local_hooks_dir" >&2
    echo "[hooks][WARN] If global hooks interfere, consider: git config --global --unset core.hooksPath" >&2
  fi
}

install
