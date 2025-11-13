---
title: Explorer Off‑Ramp & Explorer‑Root
---

# Explorer Off‑Ramp & Explorer‑Root

Exports let you analyze GATOS state outside the repo (e.g., Parquet/SQLite) while preserving verifiability.

## Explorer‑Root (Normative)

See SPEC §15.1. Exporters **MUST** compute `Explorer-Root`.

Derived state exports (from folds) include `fold_root`:

```
Explorer-Root = blake3(ledger_head || policy_root || fold_root || extractor_version)
```

Raw ledger exports (no folds) omit `fold_root`:

```
Explorer-Root = blake3(ledger_head || policy_root || extractor_version)
```

CLI:

```bash
# Export a view and write Explorer-Root alongside artifacts
git gatos export parquet --ns demo --out exports/demo/

# Verify an export matches the repo’s current ledger/policy
git gatos export verify exports/demo/
```

An export is valid if its computed Explorer‑Root matches recomputation from the repository at verification time. Any divergence (new commits, policy change, or extractor version change) must produce a mismatch.

## Recommended Layout

```
exports/<name>/
  ├─ data/*.parquet
  ├─ schema/*.json
  ├─ explorer-root.txt      # hex digest
  └─ meta.json              # { ledger_head, policy_root, extractor_version, generated_at }
```

## Failure Modes

- Mismatch: repo advanced or artifacts altered → `verify` fails.
- Partial exports: mark incomplete and refuse `verify`.
- Sampling: if present, must record sample strategy/seed; `verify` should refuse unless explicitly allowed.
