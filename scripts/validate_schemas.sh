#!/usr/bin/env bash
set -euo pipefail

echo "[schemas] Using Dockerized AJV (no host Node required)…"
AJV_NODE_IMAGE_DEFAULT="node@sha256:47dacd49500971c0fbe602323b2d04f6df40a933b123889636fc1f76bf69f58a" # corresponds to node:20
AJV_NODE_IMAGE="${AJV_NODE_IMAGE:-$AJV_NODE_IMAGE_DEFAULT}"

run_ajv() {
  local subcmd="$1"; shift
  # Preinstall ajv-cli and ajv-formats in the npx sandbox to ensure plugin availability
  docker run --rm -v "$PWD:/work" -w /work "$AJV_NODE_IMAGE" \
    npx -y -p ajv-cli@5 -p ajv-formats@3 ajv "$subcmd" --spec=draft2020 --strict=true -c ajv-formats "$@"
}

AJV_COMMON_REF="schemas/v1/common/ids.schema.json"

echo "[schemas] Compiling JSON Schemas (v1)…"
SCHEMAS=(
  "schemas/v1/common/ids.schema.json"
  "schemas/v1/job/job_manifest.schema.json"
  "schemas/v1/job/proof_of_execution_envelope.schema.json"
  "schemas/v1/governance/proposal.schema.json"
  "schemas/v1/governance/approval.schema.json"
  "schemas/v1/governance/grant.schema.json"
  "schemas/v1/governance/revocation.schema.json"
  "schemas/v1/governance/proof_of_consensus_envelope.schema.json"
  "schemas/v1/policy/governance_policy.schema.json"
)

for schema in "${SCHEMAS[@]}"; do
  echo "  - ajv compile: $schema"
  if [[ "$schema" == "$AJV_COMMON_REF" || "$schema" == "schemas/v1/policy/governance_policy.schema.json" ]]; then
    run_ajv compile -s "$schema"
  else
    run_ajv compile -s "$schema" -r "$AJV_COMMON_REF"
  fi
done

echo "[schemas] Validating example documents (v1)…"
declare -A EXAMPLES=(
  ["schemas/v1/job/job_manifest.schema.json"]="examples/v1/job/manifest_min.json"
  ["schemas/v1/job/proof_of_execution_envelope.schema.json"]="examples/v1/job/poe_min.json"
  ["schemas/v1/governance/proposal.schema.json"]="examples/v1/governance/proposal_min.json"
  ["schemas/v1/governance/approval.schema.json"]="examples/v1/governance/approval_min.json"
  ["schemas/v1/governance/grant.schema.json"]="examples/v1/governance/grant_min.json"
  ["schemas/v1/governance/revocation.schema.json"]="examples/v1/governance/revocation_min.json"
  ["schemas/v1/governance/proof_of_consensus_envelope.schema.json"]="examples/v1/governance/poc_envelope_min.json"
)

for schema in "${!EXAMPLES[@]}"; do
  data="${EXAMPLES[$schema]}"
  # Skip governance_policy here; it is validated separately without -r for consistency
  if [[ "$schema" == "schemas/v1/policy/governance_policy.schema.json" ]]; then
    continue
  fi
  echo "  - ajv validate: $data against $schema"
  run_ajv validate -s "$schema" -d "$data" -r "$AJV_COMMON_REF"
done

echo "  - ajv validate: examples/v1/policy/governance_min.json against schemas/v1/policy/governance_policy.schema.json"
run_ajv validate -s schemas/v1/policy/governance_policy.schema.json -d examples/v1/policy/governance_min.json

echo "[schemas] Additional encoding tests (ed25519 base64url forms)…"
# Create temporary schemas within the repository workdir so the container can access them via the bind mount
TMPDIR_HOST="$(mktemp -d -p "$PWD" .ajvtmp.XXXXXX)"
TMPDIR_REL="$(basename "$TMPDIR_HOST")"
printf '{"$schema":"https://json-schema.org/draft/2020-12/schema","$ref":"https://gatos.dev/schemas/v1/common/ids.schema.json#/$defs/ed25519Key"}' > "$TMPDIR_HOST/ed25519Key.schema.json"
printf '{"$schema":"https://json-schema.org/draft/2020-12/schema","$ref":"https://gatos.dev/schemas/v1/common/ids.schema.json#/$defs/ed25519Sig"}' > "$TMPDIR_HOST/ed25519Sig.schema.json"

# Generate canonical base64url encodings from actual byte lengths using Node (in container)
KEY_B64URL=$(docker run --rm "$AJV_NODE_IMAGE" node -e "process.stdout.write(Buffer.alloc(32).toString('base64url'))")
SIG_B64URL=$(docker run --rm "$AJV_NODE_IMAGE" node -e "process.stdout.write(Buffer.alloc(64).toString('base64url'))")

echo "  - positive: base64url unpadded key ($(echo -n "$KEY_B64URL" | wc -c) chars)"
printf '"ed25519:%s"' "$KEY_B64URL" > "$TMPDIR_HOST/key_b64url_unpadded.json"
run_ajv validate -s "$TMPDIR_REL/ed25519Key.schema.json" -d "$TMPDIR_REL/key_b64url_unpadded.json" -r "$AJV_COMMON_REF"

echo "  - positive: base64url unpadded sig ($(echo -n "$SIG_B64URL" | wc -c) chars)"
printf '"ed25519:%s"' "$SIG_B64URL" > "$TMPDIR_HOST/sig_b64url_unpadded.json"
run_ajv validate -s "$TMPDIR_REL/ed25519Sig.schema.json" -d "$TMPDIR_REL/sig_b64url_unpadded.json" -r "$AJV_COMMON_REF"

echo "  - negative: 44-char base64url key without '=' should be rejected"
KEY_BADLEN="${KEY_B64URL}A" # 43 -> 44 (no '=')
printf '"ed25519:%s"' "$KEY_BADLEN" > "$TMPDIR_HOST/key_b64url_badlen.json"
if run_ajv validate -s "$TMPDIR_REL/ed25519Key.schema.json" -d "$TMPDIR_REL/key_b64url_badlen.json" -r "$AJV_COMMON_REF"; then
  echo "[FAIL] Unexpected acceptance of bad key length (44 without '=')" >&2; exit 1
fi

echo "  - negative: 88-char base64url sig without '==' should be rejected"
SIG_BADLEN="${SIG_B64URL}AA" # 86 -> 88 (no '==')
printf '"ed25519:%s"' "$SIG_BADLEN" > "$TMPDIR_HOST/sig_b64url_badlen.json"
if run_ajv validate -s "$TMPDIR_REL/ed25519Sig.schema.json" -d "$TMPDIR_REL/sig_b64url_badlen.json" -r "$AJV_COMMON_REF"; then
  echo "[FAIL] Unexpected acceptance of bad sig length (88 without '==')" >&2; exit 1
fi

echo "[schemas] Negative tests (invalid ISO8601 durations)…"
echo '{"governance":{"x":{"ttl":"P"}}}' > /tmp/bad1.json
echo '{"governance":{"x":{"ttl":"PT"}}}' > /tmp/bad2.json
if run_ajv validate -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad1.json; then
  echo "[FAIL] Unexpected success: ttl=P should be rejected" >&2; exit 1
else
  echo "  - rejected ttl=P as expected"
fi
if run_ajv validate -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad2.json; then
  echo "[FAIL] Unexpected success: ttl=PT should be rejected" >&2; exit 1
else
  echo "  - rejected ttl=PT as expected"
fi

echo "[schemas] All schema checks passed."
# Cleanup temporary files created in the repository workdir
rm -rf -- "$TMPDIR_HOST"
