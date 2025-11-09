# GATOS Schemas

This directory contains versioned JSON Schemas for GATOS envelopes and policy sections.

Versioning policy

- Schemas are published under `schemas/vN/…` where `N` is a major version.
- Minor/patch changes that are backward‑compatible will not change `vN`; breaking changes will introduce `v(N+1)`.
- Unversioned files at `schemas/<area>/…` are convenience copies of the latest major and may be removed in a future release. Consumers SHOULD pin to a specific `vN` path.

Canonical encodings

- BLAKE3 digests: `blake3:<lowercase hex32>`
- Ed25519 keys/signatures: `ed25519:<hex|base64>`

