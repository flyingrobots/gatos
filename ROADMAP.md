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

**Goal:** A clean project structure and decision process with no implementation yet.

**Deliverables:**
- Rust workspace layout:
  - `crates/gatos-core` — deterministic engine & types
  - `crates/gatosd` — daemon
  - `crates/git-gatos` — CLI shim (`git gatos ...`)
- ADR/RFC process (`/spec/adr`) + templates.
- Canonical encoding decision: **DAG-CBOR + CID** for signed artifacts.
- Profiles config file: `profile.default`, `profile.research`.
- Docs scaffolding:
  - `docs/SPEC.md`
  - `docs/TECH-SPEC.md`
  - `docs/research-profile.md`
  - `docs/opaque-pointers.md`
  - `docs/exporter.md`
  - `docs/proof-of-experiment.md`
- CI: format, lint, build, basic tests.

**Non-goal:** Any networking, multi-peer sync, or job scheduling. M0 is wiring the skeleton.

---

## M1 — EchoLua Fold Engine & Proof-of-Fold (PoF)

**Goal:** Deterministic folds from events → state, with verifiable proofs.

**Deliverables:**
- `gatos-core`:
  - EchoLua interpreter (deterministic subset).
  - `dpairs()` / sorted iteration, forbidden patterns, numeric model.
  - Fold runner: `fold(state, event) -> new_state`.
- EventEnvelope:
  - DAG-CBOR encoding.
  - Typed event structure.
- StateRoot computation:
  - canonical serialization of shape → hash.
- PoF envelope:
  - Proof metadata + signature over `(history_root, policy_root, state_root)`.
- Daemon:
  - Run fold over `refs/gatos/journal/*`.
  - Commit checkpoints to `refs/gatos/state/<name>`.
- CLI:
  - `git gatos state show`
  - `git gatos fold verify <state-ref>`

**Done when:**
- Same journal + same policy → identical `state_root` on two machines.
- PoF verification succeeds across platforms.

---

## M2 — Push-Gate & Policy Plane (.rgs + rgc)

**Goal:** Governance at the boundary of history; policies as executable law.

**Deliverables:**
- Push-gate (Stargate):
  - FF-only enforcement for `refs/gatos/policies/**`, `refs/gatos/state/**`, `refs/gatos/audit/**`.
  - PoF-required checks on state refs.
- Policy system:
  - `.rgs` authoring DSL (Rego/Datalog-inspired).
  - `.rgs -> .rgc` compiler (structured IR/bytecode).
  - Policy VM built on EchoLua runtime (or parallel deterministic VM).
  - DENY-audit: policy rejections logged to `refs/gatos/audit/policy/deny/<ulid>`.
- Governance:
  - Proposals → approvals → grants mapped to signed events.
  - Grants bound to `policy_root`.

**Done when:**
- Rewriting policy history via rebase is impossible.
- Violating commits produce DENY entries with links back to the responsible ADR/policy.
- Policy rules can enforce e.g. “no API changes without 2-of-3 quorum”.

---

## M3 — Message Bus (Commit-backed Pub/Sub)

**Goal:** A usable event bus that lives in Git without melting Git.

**Deliverables:**
- Namespaced mbus:
  - `refs/gatos/mbus/<topic>/<yyyy>/<mm>/<dd>/<ulid>`
- QoS:
  - At-least-once delivery.
  - Idempotency keys + content hashes.
  - Subscriber-side dedupe.
- Rotation and retention:
  - Segment rotation based on `max_messages_per_segment` OR `max_segment_bytes`.
  - TTL-based pruning for old segments.
  - Summary commits capturing:
    - Merkle root of message bodies.
    - count, min/max offsets.
- Observability:
  - Metrics: messages per segment, pack sizes, TTL age, rotation suggestions.

**Done when:**
- Duplicate messages do not cause duplicate effects if consumers obey idempotency.
- Git repos remain manageable under expected message load.

---

## M4 — Job Plane & Proof-of-Execution (PoE)

**Goal:** Off-repo compute with verifiable provenance.

**Deliverables:**
- Job claims:
  - Exclusive CAS lock ref `refs/gatos/jobs/<job-id>/claim`.
- Worker:
  - Subscribe to mbus.
  - Claim jobs.
  - Run configured program/container.
  - Commit results.
- PoE envelope:
  - `inputs_root`, `program_id` (container/WASM/Nix hash), `outputs_root`, status, signature.
- Audit:
  - PoE recorded under `refs/gatos/audit/jobs/<job-id>`.

**Done when:**
- Race between multiple workers → exactly one claim wins.
- PoE verification reproducibly ties inputs, program, and outputs together.

---

## M5 — Opaque Pointers & Privacy-Preserving Projection

**Goal:** Publicly verifiable state with private data.

**Deliverables:**
- Public pointer schema:
  - Commitments and ciphertext digests.
  - Bucketed sizes (e.g., 1k/4k/16k/64k).
  - No plaintext digest for low-entropy classes.
- Resolver service:
  - Auth (Bearer JWT; optional HTTP signatures/mTLS).
  - Returns bytes + `Digest` headers.
  - Logs fetches to audit refs.
- Projection:
  - Folds never decrypt sensitive data.
  - Public “shape” contains pointers instead of raw values.
  - Projection is deterministic across platforms.

**Done when:**
- Public state cannot leak PII or sensitive details via pointer metadata.
- Pointer resolution is policy-controlled and auditable.

---

## M6 — Explorer Off-Ramp & Explorer-Root

**Goal:** Heavy analytics off-chain but still provable.

**Deliverables:**
- Export:
  - CLI: `git gatos export parquet|sqlite --state <ref>`.
  - Writes Parquet/SQLite plus metadata.
- Explorer-Root:
  - Checksum tying export back to `(ledger_head, policy_root, state_root, extractor_version)`.
- Verification:
  - CLI: `git gatos export verify <path>` checks Explorer-Root.

**Done when:**
- Exports verify on clean machines.
- Tampering with an export causes verification to fail.

---

## M7 — Proof-of-Experiment (PoX) & Reproduce/Verify

**Goal:** Make experiments machine-checkable.

**Deliverables:**
- PoX envelope:
  - Ties together `inputs_root`, `program_id`, `policy_root`, `policy_code_root`, `outputs_root`, PoF, and PoE.
  - Stored under `refs/gatos/audit/proofs/experiments/<ulid>`.
- CLI:
  - `git gatos verify <pox-id>`
  - `git gatos reproduce <pox-id>`
- Reproduction pipeline:
  - Fetch Opaque Pointers.
  - Re-run analysis in attested environment.
  - Compare outputs + PoF.

**Done when:**
- Reproduce yields bit-for-bit identical results in a “clean-room” setting.
- If not, verify explains exactly where/why it diverged.

---

## M8 — Demos & Examples

**Goal:** Show, don’t tell.

**Deliverables:**
- `examples/adr-as-policy/` — ADR → policy → DENY/ALLOW behavior.
- `examples/bisect-for-state/` — state regression + git gatos bisect.
- `examples/pox-research/` — synthetic experiment → PoX → reproduce.
- GIFs of:
  - ADR-as-policy,
  - Bisect-for-state,
  - PoX verification.

**Done when:**
- Each example runs with a single scripted command.
- GIFs are README-ready.

---

## M9 — Conformance Suite & `gatos doctor`

**Goal:** Turn correctness into automation.

**Deliverables:**
- Conformance tests:
  - QoS (at-least-once + dedupe).
  - Exclusive job claim.
  - Pointer privacy rules.
  - Projection determinism.
  - PoF enforcement on state pushes.
  - Explorer-Root export verification.
- `git gatos doctor`:
  - Checks for misconfigurations in:
    - profiles,
    - mbus rotation/TTL,
    - anchors,
    - PoF presence,
    - export consistency.

**Done when:**
- CI runs conformance suite on every change.
- `doctor` reliably flags misconfigurations.

---

## M10 — Security & Hardening

**Goal:** Move from “works” to “safe to trust.”

**Deliverables:**
- Threat models for all planes.
- Fuzzing harnesses for:
  - DAG-CBOR parsing,
  - EchoLua interpreter,
  - .rgs compiler,
  - mbus dedupe,
  - pointer resolver.
- External cryptography review:
  - PoF and PoE signing,
  - pointer encryption & AEAD usage,
  - hash choices and domain separation.
- Replay/forgery resilience testing.
- Hardened Research Profile defaults.

---

## M11 — Community & Launch

**Goal:** Turn GATOS into a living project.

**Deliverables:**
- Documentation site (mdBook or similar).
- “For Scientists” documentation section.
- Launch blog post: “GATOS: Git As The Operating Surface — A Reproducibility OS”.
- Early adopter outreach:
  - 2–3 design partner labs.
- Conference submissions, talks, and demos.

---

## M12 — Wesley Integration & Schema Tooling (Phase 2)

**Goal:** Make GATOS pleasant to program against.

**Deliverables:**
- `wesley build --target gatos`:
  - generates fold specs,
  - schemas,
  - RLS/policy scaffolding.
- Examples:
  - schema-first experiment spec flowing into GATOS.

---

*End of ROADMAP.*

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
