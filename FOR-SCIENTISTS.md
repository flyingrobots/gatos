# For Scientists — Verify & Reproduce

Two commands validate results end‑to‑end.

```bash
# Verify a published experiment
git gatos verify <pox-id>

# Reproduce it in a clean room
git gatos reproduce <pox-id>
```

What’s checked:
- Proof‑of‑Experiment (PoX): inputs_root, program_id, policy_root, outputs_root.
- Proof‑of‑Execution (PoE) for jobs.
- Proof‑of‑Fold (PoF) for state checkpoints underpinning figures/tables.

Include the PoX ULID and repo commit (or DOI) in Methods. See docs/proofs/proof-of-experiment.md.

