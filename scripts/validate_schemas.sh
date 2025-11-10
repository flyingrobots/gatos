#!/usr/bin/env bash
set -euo pipefail

# Resolve AJV CLI (prefer local npx; fallback to dockerized node)
AJV_RUNNER=()
if command -v node >/dev/null 2>&1; then
  AJV_RUNNER=(npx -y ajv-cli@5)
elif command -v docker >/dev/null 2>&1; then
  AJV_RUNNER=(docker run --rm -v "$PWD:/work" -w /work node:20 npx -y ajv-cli@5)
else
  echo "Need Node.js or Docker to run AJV validation" >&2
  exit 1
fi

AJV_COMMON_REF="schemas/v1/common/ids.schema.json"
AJV_BASE_ARGS=(--spec=draft2020 --strict=true -c ajv-formats)

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
  "schemas/v1/privacy/opaque_pointer.schema.json"
)

for schema in "${SCHEMAS[@]}"; do
  echo "  - ajv compile: $schema"
  if [[ "$schema" == "$AJV_COMMON_REF" || "$schema" == "schemas/v1/policy/governance_policy.schema.json" ]]; then
    "${AJV_RUNNER[@]}" compile "${AJV_BASE_ARGS[@]}" -s "$schema"
  else
    "${AJV_RUNNER[@]}" compile "${AJV_BASE_ARGS[@]}" -s "$schema" -r "$AJV_COMMON_REF"
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
  ["schemas/v1/privacy/opaque_pointer.schema.json"]="examples/v1/privacy/opaque_pointer_min.json"
)

for schema in "${!EXAMPLES[@]}"; do
  data="${EXAMPLES[$schema]}"
  # Skip governance_policy here; it is validated separately without -r for consistency
  if [[ "$schema" == "schemas/v1/policy/governance_policy.schema.json" ]]; then
    continue
  fi
  echo "  - ajv validate: $data against $schema"
  "${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s "$schema" -d "$data" -r "$AJV_COMMON_REF"
done

echo "  - ajv validate: examples/v1/policy/governance_min.json against schemas/v1/policy/governance_policy.schema.json"
"${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s schemas/v1/policy/governance_policy.schema.json -d examples/v1/policy/governance_min.json
echo "  - ajv validate: examples/v1/policy/privacy_min.json against schemas/v1/policy/governance_policy.schema.json"
"${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s schemas/v1/policy/governance_policy.schema.json -d examples/v1/policy/privacy_min.json

echo "[schemas] Additional encoding tests (ed25519 base64url forms)…"
# Root schemas that reference defs using the canonical $id for proper resolution
printf '{"$schema":"https://json-schema.org/draft/2020-12/schema","$ref":"https://gatos.dev/schemas/v1/common/ids.schema.json#/$defs/ed25519Key"}' > /tmp/ed25519Key.schema.json
printf '{"$schema":"https://json-schema.org/draft/2020-12/schema","$ref":"https://gatos.dev/schemas/v1/common/ids.schema.json#/$defs/ed25519Sig"}' > /tmp/ed25519Sig.schema.json

# Generate canonical base64url encodings from actual byte lengths using Node
KEY_B64URL=$(node -e 'process.stdout.write(Buffer.alloc(32).toString("base64url"))')
SIG_B64URL=$(node -e 'process.stdout.write(Buffer.alloc(64).toString("base64url"))')

echo "  - positive: base64url unpadded key ($(echo -n "$KEY_B64URL" | wc -c) chars)"
printf '"ed25519:%s"' "$KEY_B64URL" > /tmp/key_b64url_unpadded.json
"${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s /tmp/ed25519Key.schema.json -d /tmp/key_b64url_unpadded.json -r "$AJV_COMMON_REF"

echo "  - positive: base64url unpadded sig ($(echo -n "$SIG_B64URL" | wc -c) chars)"
printf '"ed25519:%s"' "$SIG_B64URL" > /tmp/sig_b64url_unpadded.json
"${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s /tmp/ed25519Sig.schema.json -d /tmp/sig_b64url_unpadded.json -r "$AJV_COMMON_REF"

echo "  - negative: 44-char base64url key without '=' should be rejected"
KEY_BADLEN="${KEY_B64URL}A" # 43 -> 44 (no '=')
printf '"ed25519:%s"' "$KEY_BADLEN" > /tmp/key_b64url_badlen.json
if "${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s /tmp/ed25519Key.schema.json -d /tmp/key_b64url_badlen.json -r "$AJV_COMMON_REF"; then
  echo "[FAIL] Unexpected acceptance of bad key length (44 without '=')" >&2; exit 1
fi

echo "  - negative: 88-char base64url sig without '==' should be rejected"
SIG_BADLEN="${SIG_B64URL}AA" # 86 -> 88 (no '==')
printf '"ed25519:%s"' "$SIG_BADLEN" > /tmp/sig_b64url_badlen.json
if "${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s /tmp/ed25519Sig.schema.json -d /tmp/sig_b64url_badlen.json -r "$AJV_COMMON_REF"; then
  echo "[FAIL] Unexpected acceptance of bad sig length (88 without '==')" >&2; exit 1
fi

echo "[schemas] Negative tests (invalid ISO8601 durations)…"
echo '{"governance":{"x":{"ttl":"P"}}}' > /tmp/bad1.json
echo '{"governance":{"x":{"ttl":"PT"}}}' > /tmp/bad2.json
if "${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad1.json; then
  echo "[FAIL] Unexpected success: ttl=P should be rejected" >&2; exit 1
else
  echo "  - rejected ttl=P as expected"
fi
if "${AJV_RUNNER[@]}" validate "${AJV_BASE_ARGS[@]}" -s schemas/v1/policy/governance_policy.schema.json -d /tmp/bad2.json; then
  echo "[FAIL] Unexpected success: ttl=PT should be rejected" >&2; exit 1
else
  echo "  - rejected ttl=PT as expected"
fi

echo "[schemas] All schema checks passed."
