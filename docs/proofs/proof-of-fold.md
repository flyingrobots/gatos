---
title: Proof‑of‑Fold (PoF)
---

# Proof‑of‑Fold (PoF)

PoF binds a state checkpoint to a specific ledger window and fold/policy root.

See SPEC: §5.4.

## Contents (normative)

- `Ledger-Start` / `Ledger-End` — inclusive commit window
- `Policy-Root` — policy in effect
- `Fold-Id` — fold function/spec identifier
- `State-Root` — content hash of resulting checkpoint
- Signatures as required by policy

PoF may be embedded in trailers or attached as a sidecar manifest.

## CLI

```bash
# Verify a state checkpoint (recompute fold and compare root)
git gatos fold verify <state-ref>

# Verify a PoF envelope explicitly
git gatos verify proof --id <pof-id>
```

