---
title: git gatos export — Explorer Off‑Ramp
---

# git gatos export — Explorer Off‑Ramp

Exports derived views to Parquet/SQLite with verifiable Explorer‑Root checksums.

## Synopsis

```bash
# Export a state view
git gatos export parquet --state <refs/gatos/state/ns> --out exports/ns/

# Verify a prior export
git gatos export verify exports/ns/
```

## Behavior

- Derived state exports bind: `Explorer-Root = blake3(ledger_head || policy_root || state_root || extractor_version)`.
- Raw ledger exports omit `state_root`.
- `verify` recomputes Explorer‑Root from the repo and fails on mismatch.

See also: docs/exporter.md and SPEC §15.1.

