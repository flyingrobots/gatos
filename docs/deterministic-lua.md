---
title: Deterministic Lua (stub)
---

# Deterministic Lua (stub)

GATOS will document a deterministic execution profile for Lua used in policies and (optionally) folds. This section is a placeholder pending finalized constraints. Target properties:

- Stable numeric semantics (no platform‑dependent float idiosyncrasies).
- Pure FFI boundaries; no ambient I/O; explicit capability invocation.
- Seeded RNG banned in deterministic mode; time/syscalls gated by policy.
- Fixed iteration order for tables where applicable; canonical serializers.
- Resource caps: step limits / fuel counters to ensure termination.

Once finalized, SPEC/TECH‑SPEC will reference this profile normatively.

