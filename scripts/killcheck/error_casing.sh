#!/usr/bin/env bash
set -euo pipefail

# Disallow common lowercase/underscore variants of error codes anywhere in docs
bad=$(rg -n "append_rejected|not_fast_forward|temporalorder|siginvalid|policydenied|notfound" docs || true)
if [[ -n "$bad" ]]; then
  echo "Found non-canonical error code casing or names:" >&2
  echo "$bad" >&2
  exit 1
fi
echo "ok: error code casing canonical"
