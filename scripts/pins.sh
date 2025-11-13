#!/usr/bin/env bash
# Centralized pins for container images and tool versions.
# Scripts should source this file: `. "$(dirname "$0")/pins.sh"` (adjust path as needed).
# Makefile can read values via: $(shell bash -lc '. ./scripts/pins.sh; printf "%s" "$$NODE_IMAGE_DIGEST"')

# Node image digest (corresponds to node:20). Used for Dockerized Node in scripts and hooks.
export NODE_IMAGE_DIGEST="node@sha256:47dacd49500971c0fbe602323b2d04f6df40a933b123889636fc1f76bf69f58a"

# Mermaid CLI pin for reproducible diagram generation
# Mermaid CLI pin for reproducible diagram generation.
# Note: generate.mjs reads this file to avoid drift.
export MERMAID_CLI_VERSION="10.9.0"

# Transitional allowance for verify: if existing committed SVGs were generated
# with an older CLI, set this to that previous version so CI verify passes
# without forcing an immediate full re-render in the same PR.
# Example (future bump): MERMAID_CLI_PREV_ALLOW="10.9.0"
export MERMAID_CLI_PREV_ALLOW=""

# Puppeteer Chromium revision used in CI for consistent headless rendering
export CHROMIUM_REVISION="1108766"
