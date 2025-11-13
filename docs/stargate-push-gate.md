---
title: Stargate — Push‑Gate Enforcement
---

# Stargate — Push‑Gate Enforcement

The push‑gate enforces invariants at the repository boundary.

## Enforcement (Normative)

- Fast‑forward only: `refs/gatos/policies/**`, `refs/gatos/state/**`, `refs/gatos/audit/**`.
- PoF required: pushes to `refs/gatos/state/**` MUST include a verifiable Proof‑of‑Fold (SPEC §5.4).
- DENY decisions are logged under `refs/gatos/audit/policy/**`.

## Pre‑Receive Hook (Sketch)

```bash
# pseudo: pre-receive
while read old new ref; do
  if ff_only_ref "$ref" && ! is_fast_forward "$old" "$new"; then deny "$ref"; fi
  if is_state_ref "$ref" && ! verify_pof "$new"; then deny "$ref"; fi
done
```

Denied updates MUST be accompanied by an audit entry that includes `Policy-Rule`, reason, and context (actor, target, refs).

