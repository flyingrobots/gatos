#!/usr/bin/env bash
set -euo pipefail

# If not already inside the Docker harness, re-exec via docker compose.
if [[ "${GATOS_TEST_IN_DOCKER:-}" != "1" ]]; then
  if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: Docker is required to run tests." >&2
    exit 1
  fi
  # Install required native deps inside the container before running tests.
  exec docker compose run --rm \
    -e GATOS_TEST_IN_DOCKER=1 \
    ci-tests bash -c "apt-get update && apt-get install -y pkg-config libssl-dev && cargo test --workspace --locked $*"
fi

# Inside Docker already: ensure deps are present, then run.
apt-get update >/dev/null 2>&1 && apt-get install -y pkg-config libssl-dev >/dev/null 2>&1
cargo test --workspace --locked "$@"
