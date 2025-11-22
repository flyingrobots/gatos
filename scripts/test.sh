#!/usr/bin/env bash
set -euo pipefail

# If not already inside the Docker harness, re-exec via docker compose.
if [[ "${GATOS_TEST_IN_DOCKER:-}" != "1" ]]; then
  if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: Docker is required to run tests." >&2
    exit 1
  fi
  exec docker compose run --rm \
    -e GATOS_TEST_IN_DOCKER=1 \
    ci-tests cargo test --workspace --locked "$@"
fi

cargo test --workspace --locked "$@"
