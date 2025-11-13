---
title: git gatos policyc — Compile .rgs Policy to RGC/ELC
---

# git gatos policyc — Compile .rgs Policy to RGC/ELC

Compiles the declarative policy DSL (`.rgs`) into a deterministic, executable form (`.rgc`/ELC) for the policy engine.

## Synopsis

```bash
git gatos policyc <src.rgs> -o <out.rgc>
```

## Behavior

- Parses .rgs, generates canonical IR/ELC.
- Prints `policy_code_root = sha256:<hex>` on success.
- Compatible with the same EchoLua runtime determinism guarantees as folds.

## Notes

- Policy bundles stored in‑repo; proofs MUST record `Policy-Code-Root`.

