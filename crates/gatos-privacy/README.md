# gatos-privacy

Opaque Pointer types and helpers for the GATOS hybrid privacy model (ADR-0004).

Key types
- `OpaquePointer`: JSON-facing struct that mirrors `schemas/v1/privacy/opaque_pointer.schema.json`.
  - `digest: Option<String>` — plaintext digest (may be omitted)
  - `ciphertext_digest: Option<String>` — ciphertext digest
  - `extensions.class = "low-entropy"` implies `ciphertext_digest` MUST be present and `digest` MUST be absent.

- `VerifiedOpaquePointer`: wrapper that enforces invariants during deserialization.
  - Use this at trust boundaries to guarantee the low-entropy rules.

Validation
- After deserializing `OpaquePointer`, call `pointer.validate()` to enforce:
  - At least one of `digest` or `ciphertext_digest` is present.
  - Low-entropy class requires `ciphertext_digest` and forbids `digest`.

Examples
```rust
use gatos_privacy::{OpaquePointer, VerifiedOpaquePointer};

// 1) Verified wrapper enforces invariants automatically
let v: VerifiedOpaquePointer = serde_json::from_str(json)?;

// 2) Manual validation on the plain struct
let p: OpaquePointer = serde_json::from_str(json)?;
p.validate()?;
```

Canonicalization
- When computing content IDs or digests, serialize JSON with RFC 8785 JCS (performed by higher layers).

