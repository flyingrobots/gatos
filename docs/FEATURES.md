# GATOS — FEATURES (derived from SPEC v0.3)

Each feature includes user stories per relevant stakeholders (format requested), definition of ready, and test plan.

---

## F1 — Append-Only Journals (FF-only refs)

### F1-US-DEV

|   |   |
|--|--|
| **As a...** | App Developer |
| **I want..** | to append events atomically and read them back deterministically |
| **So that...** | I can build features on top of a stable event log without racey states |

#### Acceptance Criteria & DoR

- [ ] Atomic CAS update-ref used on every append
- [ ] Denormalized canonical JSON recorded
- [ ] Appends visible via `git log` and via JSONL stream

#### Test Plan

- [ ] Golden: append N events, verify ancestry and order
- [ ] Edge: concurrent append from 3 writers (two must retry)
- [ ] Failure: attempt non-FF update → rejected

---

### F1-US-SEC

|   |   |
|--|--|
| **As a...** | Security/Compliance |
| **I want..** | denies to produce audit entries with rule IDs |
| **So that...** | all rejections are explainable and traceable |

#### Acceptance Criteria

- [ ] Every deny → `audit.decision` with rule and reason
- [ ] Policy_root and trust_chain recorded

#### Test Plan

- [ ] Golden: missing cap → DENY with rule
- [ ] Edge: expired grant → DENY with `expired` reason
- [ ] Failure: tampered audit record → verification fails

---

## F2 — Deterministic Folds & State Roots

### F2-US-DEV

|   |   |
|--|--|
| **As a...** | App Developer |
| **I want..** | byte-identical state from the same inputs |
| **So that...** | bugs are reproducible across machines |

#### Acceptance Criteria

- [ ] Canonical serializer used; BLAKE3 hash stable across OS/arch
- [ ] Fold spec validated before run

#### Test Plan

- [ ] Golden vectors across linux/macos/windows
- [ ] Edge: empty input corpus
- [ ] Failure: non-canonical payload ordering → hash mismatch detected

---

## F3 — Policy Gate (Pure, Deterministic)

### F3-US-SEC

|   |   |
|--|--|
| **As a...** | Security/Compliance |
| **I want..** | a pure policy VM with deterministic results |
| **So that...** | replay/audit is possible offline |

#### Acceptance Criteria

- [ ] Policy VM forbids I/O, clock, RNG
- [ ] `policy_root` bound to every ALLOW

#### Test Plan

- [ ] Golden: same intent twice → same verdict bytes
- [ ] Edge: rule shadowing order deterministic
- [ ] Failure: impure policy attempt → build fails

---

## F4 — Capability Grants & Trust Graph

### F4-US-PENG

|   |   |
|--|--|
| **As a...** | Platform Engineer |
| **I want..** | N-of-M updates to trust data |
| **So that...** | no single maintainer can subvert policy |

#### Acceptance Criteria

- [ ] Quorum thresholds enforced
- [ ] Grant chains verify ancestry

#### Test Plan

- [ ] Golden: 2-of-3 signers → apply
- [ ] Edge: revoked signer → invalid
- [ ] Failure: split graph → reconcile requires governance merge

---

## F5 — Message Bus (QoS with Acks/Commits)

### F5-US-SRE

|   |   |
|--|--|
| **As a...** | SRE |
| **I want..** | exactly-once delivery for job dispatch |
| **So that...** | batch jobs don’t double-run under retries |

#### Acceptance Criteria

- [ ] `gmb.msg` + `gmb.ack` + `gmb.commit` protocol
- [ ] De-dup by (topic, ulid)

#### Test Plan

- [ ] Golden: dup publishes + consumer crash → single effect
- [ ] Edge: ack lag metrics emitted
- [ ] Failure: commitment without acks → reject

---

## F7 — Epochs & Compaction

### F7-US-PENG

|   |   |
|--|--|
| **As a...** | Platform Engineer |
| **I want..** | to bound clone size and keep continuity |
| **So that...** | new nodes sync fast without losing history |

#### Acceptance Criteria

- [ ] `gatos epoch new` creates anchor
- [ ] New clones fetch current epoch + anchors
- [ ] Verification across epochs

#### Test Plan

- [ ] Golden: verify continuity hashes
- [ ] Edge: orphaned refs detected by doctor
- [ ] Failure: missing anchor → verification fails

---

## F8 — Observability & Doctor

### F8-US-SRE

|   |   |
|--|--|
| **As a...** | SRE |
| **I want..** | health endpoints and a doctor tool |
| **So that...** | I can detect drift, cache staleness, and pack bloat |

#### Acceptance Criteria

- [ ] `/healthz`, `/readyz`, `/metrics` exposed
- [ ] `gatos doctor` covers refs, packs, epochs, caches

#### Test Plan

- [ ] Golden: metrics show non-zero counters post workload
- [ ] Edge: cache stale → doctor recommends rebuild
- [ ] Failure: FF-only violation → doctor flags critical
---

## F9 — Hybrid Privacy Model

See also: [ADR-0004](./decisions/ADR-0004/DECISION.md).

### F9-US-DEV

|   |   |
|--|--|
| **As a...** | App Developer |
| **I want..** | to store sensitive data (PII, secrets) in a private store |
| **So that...** | my public, verifiable state does not contain confidential information |

#### Acceptance Criteria

- [ ] Given a `policy.yaml` file with a rule to `pointerize` the path `sensitive.field`, when the state is folded, the resulting public state tree MUST replace the value of `sensitive.field` with a canonical Opaque Pointer.
- [ ] Given a `policy.yaml` file with a rule to `pointerize` a field, the public state MUST contain a canonical Opaque Pointer at the specified path, with its `digest`, `location`, and `capability` fields correctly populated.
- [ ] When the Client SDK attempts to resolve an Opaque Pointer using the specified `location` and `capability`, and the client possesses the necessary authorization, the SDK MUST successfully retrieve and decrypt the original private data.

### F9-US-SEC

|   |   |
|--|--|
| **As a...** | Security/Compliance |
| **I want..** | to audit the separation of public and private data |
| **So that...** | I can verify that sensitive data is properly isolated and access is controlled |

#### Acceptance Criteria

- [ ] Opaque Pointer resolution fails without a valid capability.
- [ ] Private blob digest matches the digest in the public pointer.
- [ ] Commit trailers (`Privacy-Redactions`, `Privacy-Pointers`) accurately report the number of redactions/pointers.

#### Test Plan

- [ ] Golden: project a unified state, resolve pointer, and verify content matches original.
- [ ] Edge: attempt to resolve a pointer with an invalid capability URI → DENY.
- [ ] Failure: tamper with a private blob → digest mismatch on resolution.
