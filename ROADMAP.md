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
- **Git as History, not Database** — Git stores Message Plane events, checkpoints, and proofs; bulk data lives behind Opaque Pointers; heavy analytics via Explorer off-ramp.
- **Research Profile Defaults** — A conservative profile for scientific reproducibility (PoF required, policy FF-only, anchored audit refs).
- **At-Least-Once + Idempotency** — Delivery is at-least-once; consumers dedupe idempotently. No “exactly-once” fairy tales.

---

## Global Non-Goals (for the initial phases)

These are explicit non-goals until after the core truth machine is working:

- A fully featured **multi-peer networking layer** (start single-node).
- A **cluster scheduler** or full-blown job orchestration system.
- A replacement for **Kafka** or high-throughput brokers (Message Plane stays Git-native, not a hosted queue).
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
| **M3** | Message Plane (Git-native append-only stream + queries) |
| **M4** | Job Plane + Proof-of-Execution (PoE) |
| **M5** | Opaque Pointers + privacy-preserving projection |
| **M6** | Explorer off-ramp + Explorer-Root verification |
| **M6.5** | GraphQL State API (read-only) |
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
- Local enforcement:
  - Ship `gatos watch` daemon enforcing read-only locks from `.gatos/policy.yaml` until grants land.
  - Managed Git hooks (`pre-commit`, `pre-push`, `post-merge`) installed via `gatos install-hooks` and logged under audit refs.
  - Lock UX: `gatos lock acquire/release` wired to ADR-0003 so artists get Perforce-style flows.

**Done when:**

- Rewriting policy history via rebase is impossible.
- Violating commits produce DENY entries with links back to the responsible ADR/policy.
- Policy rules can enforce e.g. “no API changes without 2-of-3 quorum”.
- Locked assets stay read-only locally until a Grant is available and hooks reject bypass attempts.

---

## M3 — Message Plane (ADR-0005)

**Goal:** Land the Git-native Message Plane so integrations can consume ordered events without parsing the entire ledger.

**Deliverables:**

- Refs & checkpoints:
  - `refs/gatos/messages/<topic>/head` per-topic parent chains.
  - `refs/gatos/consumers/<group>/<topic>` storing last processed `ulid` (+ optional commit) for each consumer group.
- Event envelope:
  - Canonical JSON payload with `ulid`, `ns`, `type`, `payload`, `refs`, and `content_id` (BLAKE3 of the canonical envelope).
  - Enforce `Event-Id` and `Content-Id` headers in Message Plane commit messages.
- APIs & tooling:
  - `gatos-message-plane messages.read(topic, since_ulid, limit)` returning canonical envelopes + commit ids, oldest → newest.
  - Consumer checkpoint helpers (list, advance, reset) plus tests for ULID monotonicity.
- Integration:
  - Automatically emit Message Plane events for ledger folds and governance transitions (e.g., `governance` topic).
  - Optional bridge mirroring Message Plane topics to external brokers (Kafka/NATS) without breaking Git-native ownership.

**Done when:**

- Consumers can resume from checkpoints and replay Message Plane topics deterministically on fresh clones.
- Governance transitions and ledger mirrors emit Message Plane events discoverable via `messages.read`.

---

## M4 — Job Plane & Proof-of-Execution (PoE)

**Goal:** Off-repo compute with verifiable provenance.

**Deliverables:**

- Job claims:
  - Exclusive CAS lock ref `refs/gatos/jobs/<job-id>/claim`.
- Worker:
  - Subscribe to the Message Plane `jobs` topic (`messages.read` helper).
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
  - Canonical JSON envelope with `kind: "opaque_pointer"`, `algo`, `digest`, and optional bucketed `size` (e.g., 1k/4k/16k/64k).
  - `location` URI for retrieval (e.g., `gatos-node://`, `https://`, `s3://`, `ipfs://`).
  - `capability` URI describing how to authorize/decrypt (e.g., `gatos-key://`, `kms://`, `age://`).
  - `digest` is the BLAKE3 hash of the raw plaintext blob; no ciphertext hash is tracked in Git.
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

## M6.5 — GraphQL State API

**Goal:** Provide a typed, cache-friendly read surface for state snapshots.

**Deliverables:**

- API service (crate or module) exposing `POST /api/v1/graphql` with the schema defined in `api/graphql/schema.graphql`.
- SDL publishing endpoint + CI check to keep schema + resolvers in sync.
- Resolver contract honoring `stateRef` / `refPath`, Relay pagination (`first/last`, opaque cursors, max 500), opaque pointer nodes, and deterministic ordering.
- Policy + privacy integration mirroring ADR-0003/0004 (return `POLICY_DENIED` errors; never auto-fetch private blobs).
- Rate-limiting (600 req / 60s default) and caching semantics (`shapeRoot`, `stateRefResolved`, `Cache-Control`/`ETag`).

**Done when:**

- Clients can issue GraphQL queries against historical or live state and receive deterministic results tied to a specific `stateRef`.
- SDL + schema live in-repo and the service passes conformance tests covering pagination, pointer handling, and error codes.
- Docs (README, SPEC, Guide) describe how to target states, interpret errors, and respect policy filters.

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
    - Message Plane head continuity & retention,
    - consumer checkpoint drift,
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
  - Message Plane consumer dedupe/resume logic,
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

M3 – Message Plane

- Implement `refs/gatos/messages/<topic>/head` parent chains
- Implement `refs/gatos/consumers/<group>/<topic>` checkpoints (ULID + commit)
- Define canonical Message Plane envelope + commit annotations (Event-Id/Content-Id)
- Add `gatos-message-plane messages.read` RPC + CLI helper
- Add consumer checkpoint management commands/tests (ULID monotonicity)
- Auto-emit ledger & governance events into appropriate Message Plane topics
- Add optional bridge to mirror Message Plane topics to external brokers

M4 – Job Plane + PoE

- Implement exclusive CAS lock ref for job claim
- Implement worker subscribe → claim → run → result
- Add PoE envelope
- Add CLI verbs for job lifecycle
- Add PoE verification CLI

M5 – Opaque Pointers + Privacy

- Implement pointer envelope (kind/algo/digest/size/location/capability)
- Implement private overlay store wired into capability URIs
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
