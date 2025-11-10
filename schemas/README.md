# GATOS Schemas

This directory contains versioned JSON Schemas for GATOS envelopes and policy sections.

Versioning policy

- Schemas are published under `schemas/vN/…` where `N` is a major version.
- Minor/patch changes that are backward‑compatible will not change `vN`; breaking changes will introduce `v(N+1)`.
- Unversioned files at `schemas/<area>/…` are convenience copies of the latest major and may be removed in a future release. Consumers SHOULD pin to a specific `vN` path.

Canonical encodings

- BLAKE3 digests: `blake3:<64-char lowercase hex>` (32 bytes; lowercase hex only; no padding)
- Ed25519 keys/signatures: `ed25519:<lowercase-hex|base64|base64url>`
  - Public key (32 bytes):
    - Hex: 64 lowercase hex chars
    - Base64 (RFC 2045): 44 chars with one '=' padding (ends with '=')
    - Base64url (RFC 4648 §5): 43 chars unpadded, or 44 with one '=' padding
  - Signature (64 bytes):
    - Hex: 128 lowercase hex chars
    - Base64 (RFC 2045): 88 chars with '==' padding (ends with '==')
    - Base64url (RFC 4648 §5): 86 chars unpadded, or 88 with '==' padding
- Actors (identities): `user:<name>`, `agent:<name>`, or `service:<name>`

Time values

- Integer `ttl` in governance policy is specified in seconds.
- String `ttl` and `timeout` values use ISO 8601 duration syntax (e.g., `PT30S`, `PT5M`, `P1DT2H`).
