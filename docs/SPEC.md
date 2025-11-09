# GATOS — SPEC v0.3 (Draft)

```c
    __ ,                                                     
  ,-| ~        ,                                             
 ('||/__,  '  ||                                             
(( |||  | \\ =||=                                            
(( |||==| ||  ||                                             
 ( / |  , ||  ||                                             
  -____/  \\  \\,                                            
                                                             
                                                             
  ___                                                        
 -   -_,                                                     
(  ~/||                                                      
(  / ||   _-_,                                               
 \/==||  ||_.                                                
 /_ _||   ~ ||                                               
(  - \\, ,-_-                                                
                                                             
                                                             
 ___                                                         
-   ---___- ,,                                               
   (' ||    ||                                               
  ((  ||    ||/\\  _-_                                       
 ((   ||    || || || \\                                      
  (( //     || || ||/                                        
    -____-  \\ |/ \\,/                                       
              _/                                             
                                                             
    __                                                       
  ,-||-,                             ,                       
 ('|||  )                      _    ||   '         _         
(( |||--)) -_-_   _-_  ,._-_  < \, =||= \\ \\/\\  / \\       
(( |||--)) || \\ || \\  ||    /-||  ||  || || || || ||       
 ( / |  )  || || ||/    ||   (( ||  ||  || || || || ||       
  -____-   ||-'  \\,/   \\,   \/\\  \\, \\ \\ \\ \\_-|       
           |/                                     /  \       
           '                                     '----`      
                                                             
  -_-/                /\                                     
 (_ /                ||    _                                 
(_ --_  \\ \\ ,._-_ =||=  < \,  _-_  _-_                     
  --_ ) || ||  ||    ||   /-|| ||   || \\                    
 _/  )) || ||  ||    ||  (( || ||   ||/                      
(_-_-   \\/\\  \\,   \\,  \/\\ \\,/ \\,/                     
                                                             
                                                             
```

> _The key to understanding GATOS is understanding that it's just Git._

## Git As The Operating Surface

> You use Git for source control.  
> _I use Git for reality control._  
> _We are not the same._  
> **GATOS: Git Different.** 

|  |  |
|--|--|
| **Status** | Draft (implementation underway) |
| **Scope** | Normative specification of data model, on-disk layout, protocols, and behavioral guarantees. |  
| **Audience** | Implementers, auditors, integrators. |

GATOS is for anyone who's not afraid to try something new. It's for those who experiment and who ask _"What if?"_. GATOS is for innovators. Ground-breakers. It's for those that have the **GUTS** to try to see what happens.

BUT enough hype. Let's see what GATOS is really all about...

---

## 0. Conventions

**MUST/SHOULD/MAY** are to be interpreted as in `RFC 2119`.

**Git** refers to any conformant implementation supporting refs, commits, trees, blobs, notes, and atomic ref updates.

**Hash** defaults to BLAKE3 for content hashes and SHA‑256 for policy bundle digests unless otherwise stated.

---

## 1. System Model

A **GATOS node** is a Git repository with a disciplined layout of refs, notes, and artifacts.  

A **GATOS app** is a set of **schemas**, **policies**, and **folds** that operate on **append-only journals** to produce **deterministic state**.

**GATOS** defines four planes:

1) **Ledger plane** — append‑only journals (**events**).  
2) **State plane** — deterministic folds (**state roots**).  
3) **Policy/Trust plane** — enforceable rules and grants.  
4) **Message plane** — a commit‑backed pub/sub bus.

All planes serialize exclusively to standard Git objects.

### Requirements

#### Journals

- **MUST** be fast‑forward‑only. 
- History rewrite of `refs/gatos/journal/**` is invalid and **MUST** be rejected.

#### State refs 

- **MUST** point to commits with a `state_root` note and **MUST** be derivable from journals and policies.

#### Cache refs

- **MUST** be rebuildable and **MUST NOT** be authoritative.

#### Epochs 

- **MUST** form a cryptographicall- linked chain; 
- new clones **MAY** fetch only the current epoch plus epoch anchors.

---

## 2. On‑Disk Layout (Normative)

```
.git/
├── refs/
│   └── gatos/
│       ├── journal/
│       │   └── <namespace>/<actor>/              # append-only event streams (fast-forward only)
│       ├── state/
│       │   └── <namespace>/                      # checkpoints (materialized state)
│       ├── mbus/
│       │   └── <topic>/<shard>/                  # message topics
│       ├── mbus-ack/
│       │   └── <topic>/<consumer>/               # acknowledgements
│       ├── sessions/
│       │   └── <actor>/<ulid>/                   # ephemeral working branches
│       ├── audit/
│       │   ├── policy/                            # policy decision envelopes
│       │   └── proofs/<namespace>/                # ZK / commitment proof refs
│       ├── cache/
│       │   └── <index-id>/                        # rebuildable indexes (non-authoritative)
│       └── epoch/
│           └── <namespace>/<epoch-id>/            # epoch markers (bounded history)
│
├── notes/
│   └── gatos/
│       ├── policy/                                # decision notes on commits
│       └── artifacts/                             # stdout / stderr / log attachments
│
└── gatos/
    ├── policies/
    │   ├── <policy-name>.rgs                      # policy source (text)
    │   └── <policy-name>.rgc                      # compiled policy bundle (bytecode)
    │
    ├── schema/
    │   └── <namespace>.yaml                       # schema / namespace manifests
    │
    ├── folds/
    │   └── <fold-name>.yaml                       # fold specifications
    │
    ├── trust/
    │   ├── graph.json                             # trust DAG and thresholds
    │   └── grants/
    │       └── <grant-id>.json                    # signed capability grants
    │
    ├── objects/
    │   └── <algo>/<hash>                          # CAS blob store (opaque / large objects)
    │
    └── config/
        └── <profile>.yaml                         # profile / configuration
```

---

## 3. Identities, Actors, and Grants

### 3.1 Actors

Actors are strings of the form:

- `user:<name>` (human), 
- `agent:<name>` (automation), 
- `service:<name>` (daemon).

### 3.2 Capability Grants (Artifact)

```json
{
  "type": "grant",
  "ulid": "<ULID>",
  "issuer": "user:james",
  "subject": "agent:echo",
  "caps": ["journal:append","state:checkpoint","bus:publish"],
  "aud": ["repo://*/**"],        // optional audience constraints (globs)
  "exp": "2027-01-01T00:00:00Z", // ISO8601 expiration
  "sig": "ed25519:...",          // detached or inline signature of canonical JSON
  "prev": "sha256:<prev-grant-hash>" // for rotation chains (optional)
}
```

**Grants** **MUST** be committed under `gatos/trust/grants/`.

Verifiers **MUST** validate signature, issuer membership in trust graph, audience constraints, and expiry.

> [!IMPORTANT]
> Revocation is performed by committing a new grant with a revokes field referencing the original grant ULID.

---

## 4. Events (Ledger Plane)

### 4.1 Event Envelope (Canonical JSON)

```json
{
  "type": "intent.<domain>.<verb>",  // e.g., intent.exec.run
  "ulid": "<ULID>",
  "actor": "user|agent|service:…",
  "caps": ["cap1","cap2"],           // capabilities in use
  "labels": ["private","exportable"],// policy labels
  "payload": {...},                  // domain-specific content
  "attachments": [ { "kind":"blobptr", "algo":"blake3", "hash":"...", "size":123 } ],
  "policy_root": "sha256:...",       // compiled policy bundle hash used for this decision
  "trust_chain": "sha256:...",       // digest of grants/issuers
  "sig": "ed25519:..."               // signature over canonicalized envelope
}
```

### 4.2 Journal Semantics

Appending an event **MUST** create a new commit whose tree contains a blob of the envelope; the commit **MUST** be appended to `refs/gatos/journal/<ns>/<actor>`.

Ref updates **MUST** use atomic fast-forward with expected old OID (compare-and-swap).

Denied operations **MUST NOT** write to journal; instead, the policy gate **MUST** write an audit decision (see [§6](#)).

---

## 5. State (Deterministic Folds)

### 5.1 Fold Function

A **fold** is a pure function:

$state_root = F(events_stream, policy_root)$

#### Determinism 

For identical inputs, the byte sequence of `state_root` MUST be identical across nodes and architectures.

### 5.2 Fold Spec (YAML)

```yaml
version: 1
inputs:
  - "gatos/journal/finance/**"
reducers:
  - kind: map-join-lww        # example: last-writer-wins map
    key: "$.id"
  - kind: counter-gcounter    # example: grow-only counter
rewrites:
  - match: { type: "intent.exec", op: "approve_invoice" }
    apply: { set: { path: "$.status", value: "approved" } }
emit:
  - path: "state/finance/invoices.json"
hash:
  algo: blake3                 # canonical serialization rules fixed by engine
```

### 5.3 State Checkpoints

A checkpoint **MUST** be a commit on `refs/gatos/state/<ns>` whose tree contains emitted state artifacts and a `state_root` note containing the BLAKE3 of canonical serialization.

Checkpoints **SHOULD** include `inputs_root` (hash of input corpus) for verification and optional `proof_root` (see [§10](#)).

---

## 6. Policy & Decision Audit

### 6.1 Policy Bundle

Policy source files (`.rgs` or equivalent) **MUST** compile into a deterministic bytecode bundle (`.rgc`) with a content hash (`policy_root`).

Policies **MUST** be pure and MUST NOT depend on I/O, clocks, or randomness.

### 6.2 Gate Contract

$Decision = Gate.evaluate(intent, context) -> {Allow | Deny(reason)}$

On **ALLOW**, the gate **MUST** bind `policy_root` and `trust_chain` to the event.

On **DENY**, the gate **MUST** append an audit decision:

```json
{
  "type":"audit.decision",
  "ulid":"<ULID>",
  "actor":"user:james",
  "intent":"intent.exec.run",
  "resource":"gatos://finance",
  "result":"DENY",
  "rule":"policies/exec.rgs:12",
  "explain":"caps missing: exec:run",
  "policy_root":"sha256:...",
  "trust_chain":"sha256:...",
  "sig":"ed25519:..."
}
```

to `refs/gatos/audit/policy`.

---

## 7. Blob Pointers & Opaque Storage

### 7.1 Pointer Format

```json
{
  "kind":"blobptr",
  "algo":"blake3",
  "hash":"<hex>",
  "size":123456,
  "labels":["exportable"]
}
```

Pointers **MUST** refer to content-addressed bytes in `gatos/objects/<algo>/<hash>`.

For sensitive data, an opaque pointer **MAY** be used:

```json
{
  "kind":"opaque",
  "algo":"blake3",
  "hash":"<plaintext-hash>",
  "ciphertext_hash":"blake3:<hash>",
  "cipher_meta":{"kdf":"...", "alg":"age-v1"},
  "size":123456
}
```

> [!IMPORTANT] 
> No plaintext **MAY** be stored in Git for opaque objects.

---

## 8. Message Bus (Commit‑Backed Pub/Sub)

### 8.1 Message Envelope

```json
{
  "type":"gmb.msg",
  "ulid":"<ULID>",
  "topic":"echo/jobs",
  "from":"agent:sim",
  "reply_to":"echo/results",
  "qos":"at_least_once|exactly_once|at_most_once",
  "causal":{"parents":["<ULID>","<ULID>"]},
  "ttl":86400,
  "payload":{...},
  "attachments":[...],
  "sig":"ed25519:..."
}
```

Messages **MUST** be appended under `refs/gatos/mbus/<topic>/<shard>` where $shard = blake3(ulid) mod N$.

### 8.2 QoS

`at_most_once`: a single publish commit.

`at_least_once`: consumer **MUST** write an ack to `refs/gatos/mbus-ack/<topic>/<consumer>` referencing `msg.ulid`.

`exactly_once`: publisher **MUST** create a `gmb.commit` event after receiving a configured quorum of acks; 

Consumers **MUST** de‑duplicate by `(topic, ulid)`.

---

## 9. Sessions (Working Branches)

`gatos/sessions/<actor>/<ulid>` represents an ephemeral branch for interactive mutation.

`undo` **MUST** rebase session to the parent commit (private rebase).

`fork` **MUST** create a new session branch with same base.

`merge` **MUST** use deterministic lattice joins or DPO rules declared in the fold spec; 

Conflicts **MUST** be explicit.

---

## 10. Proofs (Commitments / ZK)

A proof envelope attests to deterministic execution:

```json
{
  "type":"proof.fairness",
  "ulid":"<ULID>",
  "inputs_root":"blake3:<hex>",
  "output_root":"blake3:<hex>",
  "policy_root":"sha256:<hex>",
  "proof":"zkp:plonk:<base64>|commitment:<base64>",
  "sig":"ed25519:<...>"
}
```

Proofs **MUST** be stored under `refs/gatos/audit/proofs/<ns>`.

Nodes **MAY** verify either commitment‑level proofs (baseline) or ZK proofs (advanced).

---

## 11. Offline Authority Protocol (OAP)

Authority envelopes **MUST** wrap privileged actions ***performed while offline*** and embed `{policy_root, trust_chain, (optional) epoch_anchor}`.

On reconnect, peers **MUST** exchange envelopes, validate signatures and ancestry of `policy_root`, and:

- prefer descendants in policy ancestry,
- if incomparable, append `governance.conflict` and require explicit policy merge.

---

## 12.  Profiles

### `local`

- All enforcement in‑process; 
- no remote hooks; 
- suitable for single‑user/offline.

### `push-gate`

- Writes go via an authoritative gateway enforcing policy; 
- mirrors to public remote; 
- provides RYW guarantees.

### `saas-hosted` 

- Enforcement via branch protection + required checks; 
- policy gate runs as CI.

Profiles **MUST** be discoverable via `gatos/config/profile.yaml`.

---

## 13.  Observability & Health

Implementations **SHOULD** expose:

- `/healthz`,
- `/readyz` (boolean),
- `/metrics` (Prometheus text) including:
  -  journal append latency, 
  -  fold latency, 
  -  bus ack lag, 
  -  cache rebuild counts.

CLI **MUST** include `gatos doctor` to diagnose ref invariants, epoch continuity, cache staleness, and pack health.

---

## 14.  Security Model

Default `deny`.

All privileged writes **MUST** carry verifiable capability grants.

Labels (`private`, `exportable`, `pii`) **MUST** gate mirroring and bus routing per policy.

- Opaque pointers **MUST** prevent plaintext leakage through Git; 
- only pointer metadata is replicated.

Keys/issuers rotation **MUST** be auditable (grant chains).

---

## 15.  Performance & GC

Large payloads **SHOULD** be chunked (e.g., 4–16 MiB) into `gatos/objects/`.

Implementations **SHOULD** provide epoch compaction:

- gatos epoch new `<ns>` to roll,
- verifiable anchors to preserve continuity,
- garbage collection for unreferenced blobs beyond retention windows (policy‑controlled).

---

## 16.  Compliance & Tests (Normative)

Implementations **MUST** pass a rigorous five-point certification inspection.

1. **Deterministic Fold**: identical `state_root` across platforms for a fixed corpus.
2. **Exactly‑Once Delivery**: duplicated publishes yield single consumer effect with `QoS=exactly_once`.
3. **Offline Reconcile**: divergent policies yield `governance.conflict`, not silent clobber.
4. **Deny Audit**: every `DENY` emits an audit decision with rule identifier.
5. **Blob Integrity**: pointer mismatch detection.

---

## 17. CLI (Reference)

```bash
git gatos init [repo]
git gatos session start|undo|fork <name>|merge <name>|snapshot
git gatos event add <ns> --json <file>
git gatos fold <ns> <channel> --spec folds/<x>.yaml --out <path>
git gatos bus publish <topic> --json <file> --qos <mode>
git gatos bus subscribe <topic> [--from HEAD-100]
git gatos policy check --intent <i> --resource <r>
git gatos trust grant <subject> <cap> [--exp]
git gatos epoch new <ns>
git gatos prove <state_root> | gatos verify <state_root>
git gatos doctor
```

---

## 18. Example Use Case: A Git-Native Work Queue

This section provides a practical example of how GATOS primitives can be used to build a sophisticated, multi-tenant, and auditable work queue, replacing a traditional system like a Redis-based queue.

### 18.1 Data Model & Ref Layout

The system is organized by a `tenant` namespace to provide multi-tenancy.

- **Journals**: `refs/gatos/journal/jobs/<tenant>/<producer>` for appending job creation and result events.
- **State**: `refs/gatos/state/jobs/<tenant>` for deterministic materialized views of the queue state (e.g., active jobs, DLQ).
- **Message Bus**: `refs/gatos/mbus/queue.<tenant>/<shard>` for job dispatching and `refs/gatos/mbus-ack/queue.<tenant>/<consumer>` for acknowledgements.
- **Audit**: `refs/gatos/audit/policy` for policy decisions and `refs/gatos/audit/proofs/jobs.<tenant>` for fold/execution proofs.

### 18.2 Event Schema

Two primary event types are used:

- `jobs.enqueue`: Represents a new job being added to the queue. Includes job ID, priority, payload pointer, and policy/signature details.
- `jobs.result`: Records the outcome of a job, including a `status` field ("ok" | "fail" | "retry"),
  `duration_ms`, `attempts`, and optional `error` details for failures; attachments may include logs.

Example envelopes (canonical JSON shape; values abbreviated for clarity):

```json
{
  "type": "jobs.enqueue",
  "ulid": "01HQ7Z4J7QWJ2CP2Z0Q9Y4K1P7",
  "actor": "agent:producer-1",
  "caps": ["journal:append", "bus:publish"],
  "labels": ["exportable"],
  "payload": {
    "job_id": "01HQ7Z4J7QWJ2CP2Z0Q9Y4K1P7",
    "tenant": "tenant-a",
    "priority": "high",
    "next_earliest_at": null,
    "payload_ptr": {"kind":"blobptr","algo":"blake3","hash":"ab…cd","size":12345}
  },
  "policy_root": "sha256:…",
  "trust_chain": "sha256:…",
  "sig": "ed25519:…"
}
```

```json
{
  "type": "jobs.result",
  "ulid": "01HQ7Z4J7QWJ2CP2Z0Q9Y4K1P7",
  "actor": "agent:worker-42",
  "caps": ["journal:append"],
  "labels": [],
  "payload": {
    "job_id": "01HQ7Z4J7QWJ2CP2Z0Q9Y4K1P7",
    "status": "ok", // or "fail" or "retry"
    "duration_ms": 5230,
    "attempts": 1
  },
  "attachments": [
    {"kind":"blobptr","algo":"blake3","hash":"de…ad","size":2048,"labels":["exportable"]}
  ],
  "policy_root": "sha256:…",
  "trust_chain": "sha256:…",
  "sig": "ed25519:…"
}
```

Additional bus/event envelopes used in delivery semantics:

```json
{
  "type": "gmb.ack",
  "ulid": "01HQ7ZACK0000000000000000",
  "actor": "agent:worker-42",
  "caps": ["bus:ack"],
  "labels": [],
  "payload": {
    "topic": "queue.acme",
    "msg_ulid": "01HQ7ZMSG0000000000000000",
    "shard": 12
  },
  "policy_root": "sha256:…",
  "trust_chain": "sha256:…",
  "sig": "ed25519:…"
}
```

```json
{
  "type": "gmb.commit",
  "ulid": "01HQ7ZCOMMIT00000000000000",
  "actor": "service:coordinator",
  "caps": ["bus:commit"],
  "labels": [],
  "payload": {
    "topic": "queue.acme",
    "msg_ulid": "01HQ7ZMSG0000000000000000",
    "acks": ["01HQ7ZACK0000000000000000"],
    "quorum": 1
  },
  "policy_root": "sha256:…",
  "trust_chain": "sha256:…",
  "sig": "ed25519:…"
}
```

```json
{
  "type": "jobs.release",
  "ulid": "01HQ7ZREL0000000000000000",
  "actor": "service:scheduler",
  "caps": ["journal:append", "bus:publish"],
  "labels": [],
  "payload": {
    "job_id": "01HQ7Z4J7QWJ2CP2Z0Q9Y4K1P7",
    "tenant": "tenant-a",
    "released_at": "2025-11-09T12:00:00Z"
  },
  "policy_root": "sha256:…",
  "trust_chain": "sha256:…",
  "sig": "ed25519:…"
}
```

### 18.3 State Folds

Deterministic folds compute the state of the work queue:

- **Queue View**: A Last-Writer-Wins (LWW) map of job metadata, keyed by `job.id`.
- **Dead-Letter-Queue (DLQ) View**: A filtered view of jobs where `status=fail` and `attempts` exceeds a defined maximum.
- **Counters**: Grow-only (G) or PN counters for per-tenant statistics (e.g., enqueued, running, failed).

### 18.4 Delivery Semantics (Exactly-Once)

Exactly-once delivery is achieved using the GATOS message bus:

1.  **Publish**: A producer appends a `jobs.enqueue` event to the journal and then publishes a `bus.message` to the message bus with `QoS=exactly_once`.
2.  **Consume**: A worker subscribes to the topic, de-duplicates messages, and processes the job. Upon completion, it appends a `jobs.result` event to the journal and writes a `gmb.ack` to the bus.
3.  **Commit**: A designated coordinator process (or the original publisher, if operating in
    coordinator mode) observes the required quorum of `gmb.ack` messages and publishes a
    `gmb.commit` message to finalize the transaction.

    Election and failover mechanisms are implementation‑specific (e.g., Raft, static
    leader‑per‑topic). Safety is normative: coordinators MUST guarantee that retries do not
    duplicate effects by EITHER maintaining durable state OR implementing idempotent commit
    semantics. Transactions MUST carry unique identifiers, and implementations MUST define
    timeouts and retry/abort rules for recovery.

### 18.5 Feature Implementation

- **Priority Queues**: Implemented using separate topics for high-priority and low-priority jobs (e.g., `queue.<tenant>.high` and `queue.<tenant>.low`).
- **Retries and Backoff**: A worker or producer can re-publish a job with a `next_earliest_at` field in the payload, which is handled by a scheduler agent.
- **Rate Limiting**: A fold computes a windowed count of jobs per tenant, which producers can consult before enqueueing new jobs.
- **Delayed Jobs**: A scheduler agent reads `jobs.enqueue` events with a `next_earliest_at` field and publishes a `jobs.release` event at the appropriate time.
- **RBAC**: Multi-tenancy and access control are handled by GATOS namespaces and capability grants, which can restrict access to specific topics and event types.
