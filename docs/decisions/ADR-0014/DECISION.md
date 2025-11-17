---
Status: Draft
Date: 2025-11-09
ADR: ADR-0014
Authors: [flyingrobots]
Requires: [ADR-0001, ADR-0005]
Related: [ADR-0002]
Tags: [Attestation, Proofs, State Engine]
Schemas:
  - schemas/v1/state/proof_of_fold_envelope.schema.json
---

# ADR-0014: Proof-Of-Fold (Attestation of State)

## Scope
Define a **cryptographic attestation** for state folds that proves which code and inputs produced a given `Shape-Root`.

## Rationale
Jobs already attest execution (ADR-0002 PoE). Folds need equivalent integrity guarantees.

## Decision
1. **Envelope** (canonical JSON):
   ```json
   {
     "fold_id": "blake3:<...>",
     "engine": { "program": "sha256:<...>", "version": "x.y.z", "platform": "..." },
     "policy_root": "sha256:<...>",
     "inputs": { "events": ["..."], "upstreams": ["blake3:...", "..."] },
     "output_shape_root": "blake3:<...>",
     "metrics": { "units": "N", "duration_ms": "M" },
     "ts": "<iso8601>"
   }
   ```
2. **Signature**: Engine signs `blake3(envelope)` with its key; trailers:
   - `Proof-Of-Fold: blake3:<digest>`
   - `Fold-Sig: ed25519:<sig>`
3. **Storage**: Persist envelope under `refs/gatos/audit/proofs/folds/<state-ref>`.
4. **Verification**: `gatos fold verify <state-ref>` checks engine key in trust graph, envelope hash, and output match.

## Consequences
- Auditable state derivations; reproducibility at the protocol layer.
- Requires key management for fold engines.

## Open Questions
- Do we include WASM module hash for portable fold engines in v1?
