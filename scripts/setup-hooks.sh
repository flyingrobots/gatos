#!/usr/bin/env bash
set -euo pipefail

top="$(git rev-parse --show-toplevel)"
gitdir="$(git rev-parse --git-dir)"
hook_src="$top/scripts/hooks/pre-commit"
local_hooks_dir="$gitdir/hooks"
hook_dst="$local_hooks_dir/pre-commit"

install() {
  # Force this repository to use its own hooks directory regardless of any global core.hooksPath
  git config --local core.hooksPath "$local_hooks_dir"
  mkdir -p "$local_hooks_dir"
  cp "$hook_src" "$hook_dst"
  chmod +x "$hook_dst"
  echo "[hooks] Installed repo-local pre-commit hook -> $hook_dst"
  echo "[hooks] Configured core.hooksPath locally to: $local_hooks_dir (does not affect global settings)"
}

install
