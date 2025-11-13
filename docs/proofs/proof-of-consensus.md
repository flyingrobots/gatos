---
title: Proof‑of‑Consensus (PoC)
---

# Proof‑of‑Consensus (PoC)

PoC attests that a governance action satisfied its quorum rules under the effective policy.

See SPEC: §20.3.

## Envelope (normative)

- Canonical proposal envelope (by value or `Proposal-Id`)
- Sorted approvals (by `Signer`) (by value or `Approval-Id`)
- Policy rule id (`Policy-Rule`) + effective quorum parameters

Trailer on the Grant commit:

```text
Proof-Of-Consensus: blake3:<hex>
```

Storage: `refs/gatos/audit/proofs/governance/<proposal-id>`.

## CLI

```bash
# Verify PoC
git gatos verify proof --id <poc-id>
```

