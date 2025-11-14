#!/usr/bin/env bash
set -euo pipefail

# Example pre-receive hook enforcing FF-only and PoF-required updates.
# Load config from gatos/config/stargate.json (matches docs/schemas/stargate.config.schema.json)

CONFIG_PATH="gatos/config/stargate.json"

if [ ! -f "$CONFIG_PATH" ]; then
  echo "[stargate] config not found: $CONFIG_PATH" >&2
  exit 1
fi

ff_only=( $(jq -r '.ff_only[]' "$CONFIG_PATH") )
require_pof=( $(jq -r '.require_pof[]' "$CONFIG_PATH") )
deny_ref=$(jq -r '.deny_audit_ref' "$CONFIG_PATH")

deny() {
  local ref=$1 reason=$2
  echo "[DENY] $ref — $reason" >&2
  # NOTE: in a real deployment, append an audit commit under $deny_ref here
  exit 1
}

is_glob_match() {
  local ref=$1; shift
  for pat in "$@"; do
    if [[ "$ref" == $pat ]]; then return 0; fi
  done
  return 1
}

is_ff() {
  local old=$1 new=$2
  if [ "$old" = "0000000000000000000000000000000000000000" ]; then
    # new ref, allow
    return 0
  fi
  git merge-base --is-ancestor "$old" "$new"
}

verify_pof() {
  local new=$1
  # Placeholder: check required trailers on the commit pointed to by $new
  git cat-file -p "$new" | grep -q "^State-Root: blake3:" && \
  git cat-file -p "$new" | grep -q "^Fold-Root: sha256:"
}

while read -r old new ref; do
  if is_glob_match "$ref" "${ff_only[@]}" && ! is_ff "$old" "$new"; then
    deny "$ref" "non-fast-forward update"
  fi
  if is_glob_match "$ref" "${require_pof[@]}" && ! verify_pof "$new"; then
    deny "$ref" "missing or invalid Proof-of-Fold trailers"
  fi
done

exit 0

