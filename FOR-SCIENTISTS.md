# For Scientists — Verify & Reproduce

Two commands validate results end-to-end.

```bash
# Verify a published experiment
git gatos verify <pox-id>

# Reproduce it in a clean room
git gatos reproduce <pox-id>
```

What’s checked:

- Proof-of-Experiment (PoX): inputs_root, program_id, policy_root, outputs_root.
- Proof-of-Execution (PoE) for jobs.
- Proof-of-Fold (PoF) for state checkpoints underpinning figures/tables.

Include the PoX ULID and repo commit (or DOI) in Methods. See docs/proofs/proof-of-experiment.md.

## Methods Appendix Template (copy-paste)

```
Repository: https://github.com/<org>/<repo> @ <commit>
Experiment: <title>

Proof-of-Experiment (PoX)
  id: <ULID>
  inputs_root: blake3:<hex>
  program_id: <container|wasm|code digest>
  policy_root: <commit-oid>
  policy_code_root: sha256:<hex>
  outputs_root: blake3:<hex>

Proof-of-Execution (PoE)
  ids: [ blake3:<hex>, ... ]

Proof-of-Fold (PoF)
  state_ref: refs/gatos/state/<ns>
  state_root: blake3:<hex>
  fold_root: sha256:<hex>

Explorer-Root (export verification)
  explorer_root: blake3:<hex>
  extractor_version: <semver>

Reproduce
  git clone <repo>
  git gatos verify <pox-id>
  git gatos reproduce <pox-id>
```
