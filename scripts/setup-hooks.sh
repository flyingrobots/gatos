#!/usr/bin/env bash
set -euo pipefail

hook_src="$(git rev-parse --show-toplevel)/scripts/hooks/pre-commit"
hook_dst="$(git rev-parse --git-path hooks)/pre-commit"

install() {
  mkdir -p "$(dirname "$hook_dst")"
  cp "$hook_src" "$hook_dst"
  chmod +x "$hook_dst"
  echo "Installed pre-commit hook -> $hook_dst"
}

install

