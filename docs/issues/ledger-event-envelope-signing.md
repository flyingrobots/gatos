# Ledger: Event Envelope Canonicalization & Signing

 - **Status:** Done
- **Area:** gatos-ledger-core / crypto
- **Owner:** Triage
- **Context:** SPEC §4.1 requires DAG-CBOR canonicalization, `Event-CID`, and signature handling (ed25519 at minimum). No implementation exists to build/verify envelopes end-to-end.

## Tasks
- Implement `EventEnvelope` struct + DAG-CBOR canonicalizer per SPEC §4.1.
- Compute `Event-CID = cidv1(dag-cbor, blake3(canonical_bytes))` and expose helpers for trailers.
- Support signature creation/verification (ed25519; extensible for p256). Enforce omission of `sig` from canonical bytes.
- Add schema validation and ULID checks (idempotency key).
- Unit tests with test vectors (multi-algorithm, missing fields, canonical ordering).

## Definition of Done
- Library functions produce stable canonical bytes/`Event-CID` across platforms.
- Sign/verify passes reference vectors; invalid signatures rejected.
- Docs explain canonicalization and trailer usage.

## Progress Log
- 2025-11-21: Added failing tests in `gatos-ledger-git` for canonical bytes, CID, and sign/verify; stub implementations still TODO.
- 2025-11-21: Implemented DAG-CBOR canonicalization, CID (blake3-256), and ed25519 sign/verify; tests passing in Docker harness.
