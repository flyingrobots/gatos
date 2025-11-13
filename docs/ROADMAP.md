<ANSWERS APPLIED: updated roadmap per feedback; see top-level ROADMAP.md for issue bundle.>

# 🐈‍⬛ GATOS ROADMAP
**Git As The Operating Surface — A Truth Machine for Distributed Systems & Science**

This roadmap outlines the path from **0 lines of code** to the **first reproducible scientific experiment** verified end-to-end with GATOS.

It follows a strict **proof-first** philosophy:
- **Proof-of-Fold (PoF)** — state is verifiably derived from history.
- **Proof-of-Execution (PoE)** — jobs are verifiably executed.
- **Proof-of-Experiment (PoX)** — experiments are verifiably reproducible.

---

## Guiding Principles

- **Proof-first Design** — Every claim is verifiable from first principles.
- **Deterministic by Construction** — Same history + same policy = same state, bit-for-bit.
- **Git as History, not Database** — Git stores events, checkpoints, and proofs; bulk data lives behind Opaque Pointers; heavy analytics via Explorer off-ramp.
- **Research Profile Defaults** — A conservative profile for scientific reproducibility (PoF required, policy FF-only, anchored audit refs).
- **At-Least-Once + Idempotency** — Delivery is at-least-once; consumers dedupe idempotently. No “exactly-once” fairy tales.

---

## Global Non-Goals (for the initial phases)

These are explicit non-goals until after the core truth machine is working:

- A fully featured **multi-peer networking layer** (start single-node).
- A **cluster scheduler** or full-blown job orchestration system.
- A replacement for **Kafka** or high-throughput brokers.
- A hosted “GATOS Cloud” product.
- Strong isolation / capability-based sandboxing beyond basic VM guarantees
  (initial focus is determinism and correctness, not perfect sandbox security).

---

## Milestones Overview

| Milestone | Goal |
|----------|------|
| **M0** | Repo, scaffolding, canonicalization, ADR process |
| **M1** | EchoLua fold engine + Proof-of-Fold (PoF) |
| **M2** | Push-gate, .rgs policy, DENY-audit, grants |
| **M3** | Commit-backed Message Bus (segmented, TTL, summaries) |
| **M4** | Job Plane + Proof-of-Execution (PoE) |
| **M5** | Opaque Pointers + privacy-preserving projection |
| **M6** | Explorer off-ramp + Explorer-Root verification |
| **M7** | Proof-of-Experiment (PoX) + reproduce/verify CLI |
| **M8** | Demos & examples (Bisect, ADR-as-policy, PoX) |
| **M9** | Conformance suite + `gatos doctor` |
| **M10** | Security & hardening |
| **M11** | Community & Launch (docs, blog, outreach) |
| **M12** | Wesley integration & schema tooling (optional Phase 2) |

---

## M0 — Repository Skeleton & Governance

... (content mirrors the feedback; see top-level ROADMAP.md for full version) ...

---

## **M0 — Repository Skeleton & Governance**
**1–2 weeks**

### Goals
- Establish clean project layout and contribution flow.
- Lock in canonical encodings and profiles.

### Deliverables
- Repo structure + Rust workspace + crate skeletons.
- ADR/RFC process (`/spec/adr`).
- Canonical encoding: **DAG-CBOR** for signed data.
- CLI shim: `git-gatos` (or alias).
- Profiles: `default` and `research`.
- Initial documentation scaffold:
  - `SPEC.md`, `TECH-SPEC.md`, `research-profile.md`
  - `opaque-pointers.md`, `exporter.md`, `proof-of-experiment.md`

### Done When
- `git gatos --help` prints.
- ADR to lock canonical encoding merged.
- CI workflows exist (lint, fmt, build).
- No code written, but architecture is set.

---

## **M1 — Deterministic Folds & Proof-of-Fold**
**3–5 weeks**

### Goals
- Turn Git history into deterministic state.
- Establish cryptographic proof-of-fold (PoF).

### Deliverables
- `gatos-core` fold engine (Lua or WASM reducer).
- EventEnvelope (DAG-CBOR) parser.
- StateRoot computation + PoF envelope.
- Checkpoint commits under `refs/gatos/state/**`.
- CLI:  
  - `git gatos state show`  
  - `git gatos fold verify <ref>`

### Done When
- Same ledger on different machines → identical state_root.
- PoF verifies across OSes.
- Platform determinism tests pass.

---

## **M2 — Push-Gate & Policy Plane**
**3–4 weeks**

### Goals
- Enforce governance at the boundary of history.

### Deliverables
- Pre-receive gate:
  - Reject non-FF pushes to `policies/**`, `state/**`, `audit/**`.
  - Reject state pushes lacking PoF.
- Policy VM (Lua/WASM).
- DENY-audit under `refs/gatos/audit/policy/**`.
- Governance MVP:
  - proposals → approvals → grants (N-of-M).

### Done When
- Rebasing policy refs is impossible.
- Violating commits produce DENY events.
- Policy ADR-as-code works end-to-end.

---

## **M3 — Message Bus (Commit-backed Pub/Sub)**
**3–5 weeks**

### Goals
- Robust commit-backed pub/sub without turning Git into Kafka.

### Deliverables
- Namespaced mbus:  
  `refs/gatos/mbus/<topic>/<yyyy>/<mm>/<dd>/<ulid>`
- QoS: **at-least-once + idempotency + ack/dedupe**.
- Rotation thresholds:  
  - max 100k messages **or** 192MB per segment
- TTL: default 30 days.
- Summary commits: Merkle root, counts, offsets.
- Metrics: segment size, rotation hints.

### Done When
- Subscribers dedupe correctly.
- Segments rotate & prune.
- Repo does NOT balloon.

---

## **M4 — Job Plane + Proof-of-Execution (PoE)**
**4–6 weeks**

### Goals
- Off-chain compute with verifiable proofs.

### Deliverables
- Exclusive CAS lock ref: `refs/gatos/jobs/<id>/claim`.
- Worker loop:
  - subscribe → claim → run → commit result.
- PoE envelope: inputs_root, program_id, outputs_root.
- CLI helpers for job lifecycle.

### Done When
- Two workers → exactly one claim.
- PoE verifies replayably.
- Jobs run deterministically with tracked provenance.

---

## **M5 — Opaque Pointers & Privacy Projection**
**4–6 weeks**

### Goals
- Public verifiability + private data.

### Deliverables
- Public pointer schema:
  - **ciphertext_digest** REQUIRED
  - NO plaintext digest for low-entropy data
  - size bucketed (1k/4k/16k/64k)
- Resolver: JWT + Digests; logs under audit.
- Projection engine performs pointerization deterministically.

### Done When
- Public state contains no leakable fields.
- Resolver always returns correct bytes & digest.
- Projection determinism holds across OSes.

---

## **M6 — Explorer Off-Ramp + Explorer-Root**
**3–4 weeks**

### Goals
- Verifiable large-scale analytics outside Git.

### Deliverables
- Export to Parquet/SQLite.
- Explorer-Root checksum.
- CLI: `export`, `export verify`.

### Done When
- Exports verify on clean machines.
- Mutations → `verify` fails.

---

## **M7 — Proof-of-Experiment (PoX) & Reproduce/Verify**
**4–6 weeks**

### Goals
- Machine-checkable science.

### Deliverables
- PoX envelope stored under  
  `refs/gatos/audit/proofs/experiments/<ulid>`
- CLI:
  - `gatos verify <pox-id>`
  - `gatos reproduce <pox-id>`
- Clean-room reproduction: pointer fetch + PoE + PoF.

### Done When
- Reproduction yields bit-for-bit identical results.
- Drift causes explicit, inspectable failure.

---

## **M8 — Demos & Examples**
**1–2 weeks**

### Deliverables
- ADR-as-policy demo
- Bisect-for-state demo
- PoX reproducibility demo
- GIFs embedded in README

### Done When
- All three run with a single command.
- Suitable for readme, talks, Twitter/YouTube.

---

## **M9 — Conformance & `gatos doctor`**
**3–4 weeks**

### Goals
- Turn correctness into automation.

### Deliverables
- Conformance suite:
  - QoS  
  - exclusive claim race  
  - pointer privacy  
  - projection determinism  
  - PoF verification  
  - Explorer-Root  
- CLI: `gatos doctor`

### Done When
- CI runs full suite green.
- `doctor` correctly warns for misconfigurations.

---

# 🎯 “For Science” Early Access Checklist

**GATOS is science-ready when:**
- Profile: `research` is default for new repos  
- PoF required  
- Policy FF-only  
- Opaque Pointer privacy safe  
- Bus segmented & rotated  
- PoX demo runs clean-room  
- Export → verify works  
- Bisect-for-state works  

---

# 🎉 Endgame

When **M7** lands (PoX + reproduce), GATOS becomes the **first reproducibility OS**.  
When **M8/M9** land, it becomes a **trustworthy distributed compute fabric.**

*Now you Git it.*
