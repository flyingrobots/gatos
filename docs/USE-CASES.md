# GATOS — USE CASES

> *Have **you** tried GATOS yet?*

This document illustrates practical scenarios where GATOS provides unique value.

---

## 1) Programmable Git (Policy-Enforced Repos)

| | |
|---|---|
|**Goal** | Treat Git as a programmable ledger with rule‑checked writes. |
| **How** | Journals under `refs/gatos/journal/**`, policy gate enforces who/what/where, audits on deny. |
| **Why GATOS** | No server required (local), or push‑gate profile for RYW and centralized enforcement. |

---

## 2) Distributed State Machines (Deterministic)

| | |
|---|---|
|**Goal** | Model business processes as append‑only events → deterministic state. |
| **How** | Echo folds compute `state_root`; checkpoints under `refs/gatos/state/**`. |
| **Why GATOS** | Any node can replay to the same byte‑identical result; offline‑first. |

---

## 3) Distributed General Computer (Agents on a Bus)

| | |
|---|---|
|**Goal** | Multi‑agent orchestration with exactly‑once semantics and audit. |
| **How** | Git message bus (`refs/gatos/mbus/**`) with acks/commitments; capabilities gate topics. |
| **Why GATOS** | Works without Kafka; merges cleanly; persists forever. |

---

## 4) Supply‑Chain & Deploy Attestation

| | |
|---|---|
|**Goal** | Immutable, signed, verifiable deploy records. |
| **How** | Ship every critical action as an event; store stdout/stderr as notes; multi‑sig trust for policy changes. |
| **Why GATOS** | Incident response + compliance with zero vendor lock. |

---

## 5) Air‑Gapped ML Registry

| | |
|---|---|
|**Goal** | Version large models/datasets with provenance and selective export. |
| **How** | Opaque pointers for ciphertext artifacts; policies for export; epochs bound repo growth. |
| **Why GATOS** | Portable archives; verifiable lineage; offline friendly. |

---

## 6) Cross‑App Data Sharing (RLS‑gated)

| | |
|---|---|
|**Goal** | App A reads materialized state from App B without custom APIs. |
| **How** | App B publishes state under `refs/gatos/state/<ns>`; App A fetches and enforces RLS via shared policy bundles. |
| **Why GATOS** | Zero glue code; shared Merkle truth. |

---

## 7) Knowledge Graph for Code & Ops

| | |
|---|---|
|**Goal** | Persist “why” relationships alongside “what” code changes. |
| **How** | Edges as journal events; roaring‑bitmap caches for fast queries. |
| **Why GATOS** | Time‑travelable semantics baked into Git. |

---

## 8) Regulated Feature Flags & Config

| | |
|---|---|
|**Goal** | Signed toggles with audit and rollbacks. |
| **How** | KV‑style events + index refs; push‑gate for enforcement. |
| **Why GATOS** | Auditable configuration without a new database. |

---

## 9) Verifiable, Compliant PII Management

| | |
|---|---|
|**Goal** | Manage customer data (PII) in a way that is both auditable and privacy-preserving. |
| **How** | A privacy policy projects the unified state into a public state with PII replaced by Opaque Pointers. The private data lives in an actor-anchored, encrypted blob store. |
| **Why GATOS** | Provides a verifiable public audit trail ("a user's data was accessed") without ever exposing the private data ("the user's address is...") to the public ledger. Access is gated by cryptographic capabilities. |
