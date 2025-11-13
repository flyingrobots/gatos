# 🐈‍⬛ GATOS ROADMAP
**Git As The Operating Surface — A Truth Machine for Distributed Systems & Science**

This roadmap outlines the path from **0 lines of code** to the **first reproducible scientific experiment** verified end-to-end with GATOS.  
It follows a strict **proof-first** philosophy: every milestone introduces cryptographic verification (PoF, PoE, PoX) before features build upon it.

---

# Guiding Principles

- **Proof-first Design** — Every claim must be verifiable from first principles (PoF for state, PoE for jobs, signed governance).
- **Deterministic by Construction** — Same history + same policy = same state, bit-for-bit.
- **Git as History, Not as Database** — Bulk data behind Opaque Pointers; heavy analytics through Explorer off-ramp.
- **Research Profile Defaults** — Safe, conservative settings for scientific reproducibility.

---

# 🛠 Milestones Overview

| Milestone | Goal | Status |
|----------|------|--------|
| **M0** | Repo, scaffolding, canonicalization | ⬜ TODO |
| **M1** | Deterministic folds + Proof-of-Fold | ⬜ TODO |
| **M2** | Push-gate, policies, DENY-audit | ⬜ TODO |
| **M3** | Message Bus (segmented, TTL, summaries) | ⬜ TODO |
| **M4** | Job Plane + Proof-of-Execution | ⬜ TODO |
| **M5** | Opaque Pointers + Privacy Projection | ⬜ TODO |
| **M6** | Explorer off-ramp + Explorer-Root verify | ⬜ TODO |
| **M7** | Proof-of-Experiment (PoX) + reproduce | ⬜ TODO |
| **M8** | Demos & examples | ⬜ TODO |
| **M9** | Conformance suite + doctor | ⬜ TODO |

Estimated timeline for a solo senior engineer: **6–9 months**.

---

# 🧩 Detailed Milestones

(Contents mirror docs/ROADMAP.md and are kept in sync.)

---

# 📌 GITHUB ISSUE LIST (Milestones Board)

Paste these titles to create issues grouped by milestone.

M0 – Repo & Scaffolding

- Create Rust workspace structure (gatos-core, gatosd, git-gatos)
- Add ADR/RFC process and templates
- Choose canonical encoding: DAG-CBOR + CID
- Add initial docs: research-profile.md, proof-of-experiment.md
- Add CLI shim + smoke-test (git gatos --help)
- Add CI pipelines (fmt, lint, build)
- Add SECURITY, CONTRIBUTING, CODEOWNERS

M1 – Fold Engine + PoF

- Implement EventEnvelope (DAG-CBOR)
- Implement pure fold engine (gatos-core)
- Integrate Lua or WASM reducer
- Add state checkpoint format + PoF
- Implement state show
- Implement fold verify
- Add cross-platform determinism tests

M2 – Push-Gate & Policy

- Implement pre-receive FF-only enforcement
- Implement PoF-required validation for state refs
- Implement minimal policy VM (Lua/WASM)
- Add DENY audit logging
- Implement proposal/approval/grant flow
- Add policy verify

M3 – Message Bus

- Implement refs/gatos/mbus/* structure
- Add message publish + subscribe RPC
- Add at-least-once + idempotency support
- Implement segmented topics
- Implement TTL pruning
- Add summary commits with Merkle roots
- Add observability metrics

M4 – Job Plane + PoE

- Implement exclusive CAS lock ref for job claim
- Implement worker subscribe → claim → run → result
- Add PoE envelope
- Add CLI verbs for job lifecycle
- Add PoE verification CLI

M5 – Opaque Pointers + Privacy

- Implement pointer format (ciphertext_digest + bucketed size)
- Implement encrypted meta store
- Implement pointer resolver (JWT + Digest headers)
- Integrate privacy projection into fold pipeline
- Add projection determinism tests

M6 – Explorer Off-Ramp

- Implement Parquet/SQLite export
- Implement Explorer-Root hash
- Add CLI: export, export verify
- Add export mismatch tests

M7 – PoX + Reproduce

- Add PoX envelope + CID storage
- Implement gatos verify <pox-id>
- Implement gatos reproduce <pox-id>
- Add clean-room reproduction tests

M8 – Demos

- Create ADR-as-policy demo
- Create Bisect-for-State demo
- Create PoX demo
- Record GIFs and embed them in README

M9 – Conformance + Doctor

- Add conformance suite (QoS, exclusivity, projection)
- Add pointer privacy test suite
- Add Explorer-Root verification tests
- Add PoF enforcement tests
- Implement gatos doctor
- Ensure CI runs all conformance tests

