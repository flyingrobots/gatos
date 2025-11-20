#!/usr/bin/env bash
set -euo pipefail

if [[ "${GATOS_TEST_IN_DOCKER:-}" != "1" ]]; then
  echo "ERROR: tests must run inside the Docker harness (set GATOS_TEST_IN_DOCKER=1)." >&2
  exit 1
fi

cargo test "$@"
