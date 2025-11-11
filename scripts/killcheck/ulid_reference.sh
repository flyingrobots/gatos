#!/usr/bin/env bash
set -euo pipefail
file="docs/decisions/ADR-0005/DECISION.md"
if ! rg -n "ULID Spec §4\.1 Monotonic Lexicographic Ordering" "$file" >/dev/null; then
  echo "Missing external reference to 'ULID Spec §4.1 Monotonic Lexicographic Ordering' in ADR-0005." >&2
  exit 1
fi
echo "ok: ULID external reference present"

