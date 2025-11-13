#!/usr/bin/env bash
set -euo pipefail
set -o errtrace

trap 's=$?; echo "[setup-dev][ERROR] command failed (exit $s): ${BASH_COMMAND:-<unknown>}" >&2' ERR

echo "[setup-dev] Developer environment bootstrap"
echo "[setup-dev] This will: install repo-local Git hooks and (optionally) CLI tools via cargo."

top="$(git rev-parse --show-toplevel 2>/dev/null || pwd -P)"
cd "$top"

# 1) Install repo-local hooks
if [[ -x scripts/setup-hooks.sh ]]; then
  echo "[setup-dev] Installing Git hooks…"
  scripts/setup-hooks.sh
else
  echo "[setup-dev][WARN] scripts/setup-hooks.sh is missing or not executable; skipping hook install" >&2
fi

# 2) Install CLI tools when cargo is available
if command -v cargo >/dev/null 2>&1; then
  echo "[setup-dev] Installing recommended CLI tools (via cargo)…"
  # Pin versions used in CI where applicable
  DPRINT_VERSION="0.50.2"
  LYCHEE_VERSION="0.21.0"

  if ! command -v dprint >/dev/null 2>&1; then
    echo "[setup-dev] cargo install dprint --locked --version ${DPRINT_VERSION}"
    cargo install dprint --locked --version "${DPRINT_VERSION}" || {
      echo "[setup-dev][WARN] Failed to install dprint; CI will still enforce formatting." >&2
    }
  else
    echo "[setup-dev] dprint already installed ($(dprint --version || echo unknown))"
  fi

  if ! command -v lychee >/dev/null 2>&1; then
    echo "[setup-dev] cargo install lychee --locked --version ${LYCHEE_VERSION}"
    cargo install lychee --locked --version "${LYCHEE_VERSION}" || {
      echo "[setup-dev][WARN] Failed to install lychee; link check will rely on Docker or skip locally." >&2
    }
  else
    echo "[setup-dev] lychee already installed ($(lychee --version || echo unknown))"
  fi

  # rumdl is optional; xtask md is the fallback. Try to install if published.
  if ! command -v rumdl >/dev/null 2>&1; then
    echo "[setup-dev] Optional: installing rumdl (if available)…"
    if cargo install rumdl --locked >/dev/null 2>&1; then
      echo "[setup-dev] rumdl installed"
    else
      echo "[setup-dev][INFO] rumdl not installed; xtask md will be used locally."
    fi
  else
    echo "[setup-dev] rumdl already installed"
  fi
else
  cat >&2 <<'MSG'
[setup-dev][WARN] Rust toolchain (cargo) not detected — skipping CLI installs.
Install Rust via rustup: https://rustup.rs
After installing, re-run:  bash ./scripts/setup-dev.sh
MSG
fi

echo "[setup-dev] Done. You can now use 'make help' for common tasks."

