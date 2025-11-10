# GATOS — SPEC v0.3 (Draft)

> *The key to understanding GATOS is understanding that it's just Git.*

## Git As The Operating Surface

> You use Git for source control.  
> *I use Git for reality control.*  
> *We are not the same.*  
> **GATOS: Git Different.**

|  |  |
|--|--|
| **Status** | Draft (implementation underway) |
| **Scope** | Normative specification of data model, on-disk layout, protocols, and behavioral guarantees. |
| **Audience** | Implementers, auditors, integrators. |

---

## 0. Conventions

The keywords **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in `RFC 2119`.

```mermaid
graph TD
    A[RFC 2119] --> B{Keywords};
    B --> C[MUST];
    B --> D[SHOULD];
    B --> E[MAY];
```

**Git** refers to any conformant implementation supporting refs, commits, trees, blobs, notes, and atomic ref updates.

**Hash** defaults to BLAKE3 for content hashes and SHA‑256 for policy bundle digests unless otherwise stated.

---

## 1. System Model

A **GATOS node** is a Git repository with a disciplined layout of refs, notes, and artifacts. A **GATOS app** is a set of **schemas**, **policies**, and **folds** that operate on **append-only journals** to produce **deterministic state**.

**GATOS** defines five planes:

1) **Ledger plane** — append‑only journals (**events**).  
2) **State plane** — deterministic folds (**state roots**).  
3) **Policy/Trust plane** — enforceable rules, grants, and multi‑party consensus governance.  
4) **Message plane** — a commit‑backed pub/sub bus.
5) **Job plane** — Distributed, verifiable job execution.

```mermaid
graph TD
    subgraph "User / Client"
        CLI("gatosd (CLI)")
        SDK("Client SDK")
    end

    subgraph "GATOS System"
        Daemon("gatosd (Daemon)")

        subgraph "Policy Plane"
            Policy("gatos-policy");
        end

        subgraph "State Plane"
            Echo("gatos-echo");
            KV("gatos-kv");
        end

        subgraph "Message Plane"
            Mind("gatos-mind");
        end

        subgraph "Job Plane"
            Compute("gatos-compute");
        end

        subgraph "Ledger Plane"
            Ledger("gatos-ledger");
        end

        Daemon --> Policy;
        Daemon --> Echo;
        Daemon --> KV;
        Daemon --> Mind;
        Daemon --> Ledger;

        Echo --> Ledger;
        KV --> Ledger;
        Mind --> Ledger;
        Compute --> Mind;
        Compute --> Ledger;
    end

    CLI --> Daemon;
    SDK --> Daemon;

    style Policy fill:#f9f,stroke:#333,stroke-width:2px
    style Echo fill:#9cf,stroke:#333,stroke-width:2px
    style KV fill:#9cf,stroke:#333,stroke-width:2px
    style Mind fill:#9c9,stroke:#333,stroke-width:2px
    style Ledger fill:#c99,stroke:#333,stroke-width:2px
    style Compute fill:#f96,stroke:#333,stroke-width:2px
```

### Requirements

- Journals **MUST** be fast‑forward‑only.
- State refs **MUST** be derivable from journals and policies.
- Cache refs **MUST** be rebuildable and **MUST NOT** be authoritative.
- Epochs **MUST** form a cryptographically-linked chain.

---

## 2. On‑Disk Layout (Normative)

The following diagram illustrates the primary locations for GATOS artifacts within the Git repository (`.git`) and the working tree.

```mermaid
graph TD
    subgraph .git
        direction LR
        A(refs) --> A1(gatos)
        A1 --> B1(journal)
        A1 --> B2(state)
        A1 --> B3(mbus)
        A1 --> B4(jobs)
        A1 --> B5(sessions)
        A1 --> B6(audit)
        A1 --> B7(cache)
        A1 --> B8(epoch)
        A1 --> B9(private)
        C(notes) --> C1(gatos)
    end
    subgraph Workspace
        direction LR
        D(gatos) --> D1(policies)
        D --> D2(schema)
        D --> D3(folds)
        D --> D4(trust)
        D --> D5(objects)
    end
```

The normative layout is as follows:

```text
.git/
├── refs/
│   └── gatos/
│       ├── journal/
│       ├── state/
│       ├── private/
│       │   └── <actor-id>/
│       ├── mbus/
│       ├── mbus-ack/
│       ├── jobs/
│       │   └── <job-id>/
│       │       └── claims/<worker-id>
│       ├── proposals/
│       ├── approvals/
│       ├── grants/
│       └── revocations/
│       ├── sessions/
│       ├── audit/
│       ├── cache/
│       └── epoch/
└── gatos/
    ├── policies/
    ├── schema/
    ├── folds/
    ├── trust/
    ├── objects/
    └── config/
```

---

## 3. Identities, Actors, and Grants

### 3.1 Actors

Actors are strings of the form: `user:<name>`, `agent:<name>`, or `service:<name>`.

### 3.2 Capability Grants

Grants link an `issuer` Actor to a `subject` Actor, bestowing a set of capabilities (`caps`) that are valid until an expiration date (`exp`).

```mermaid
classDiagram
    class Grant {
        +String ulid
        +String issuer
        +String subject
        +String[] caps
        +Date exp
        +String sig
    }
    class Actor {
        <<enumeration>>
        user
        agent
        service
    }
    Grant "1" -- "1" Actor : issuer
    Grant "1" -- "1" Actor : subject
```

**Grants** **MUST** be committed under `gatos/trust/grants/`. Verifiers **MUST** validate the signature, issuer trust, audience, and expiry.

---

## 4. Events (Ledger Plane)

### 4.1 Event Envelope

All actions in GATOS are initiated via a signed Event.

```mermaid
classDiagram
    class EventEnvelope {
        +String type
        +String ulid
        +String actor
        +String[] caps
        +Object payload
        +String policy_root
        +String sig
    }
```

### 4.2 Journal Semantics

Appending an event **MUST** create a new commit on an append-only ref in `refs/gatos/journal/<ns>/<actor>`. Ref updates **MUST** use atomic compare-and-swap.

---

## 5. State (Deterministic Folds)

### 5.1 Fold Function

A **fold** is a pure function: $state_root = F(events_stream, policy_root)$.

```mermaid
graph TD
    A[Event Stream] --> B{Fold Function};
    C[Policy] --> B;
    B --> D[State Root];
```

For identical inputs, the byte sequence of `state_root` **MUST** be identical.

### 5.2 Fold Spec & Checkpoints

A Fold is defined by a `.yaml` spec. Its output, a **State Checkpoint**, is a commit on `refs/gatos/state/<ns>` whose tree contains the materialized state artifacts.

---

## 6. Policy & Decision Audit

### 6.1 Gate Contract

All events are evaluated by a Policy Gate before being accepted.
$Decision = Gate.evaluate(intent, context) -> {Allow | Deny(reason)}$

```mermaid
sequenceDiagram
    participant Client
    participant GATOS
    participant PolicyGate

    Client->>GATOS: Propose Action (Intent)
    GATOS->>PolicyGate: Evaluate(Intent, Context)
    alt Action is Allowed
        PolicyGate-->>GATOS: Decision: Allow
        GATOS->>GATOS: Bind policy_root to event
        GATOS-->>Client: Success
    else Action is Denied
        PolicyGate-->>GATOS: Decision: Deny(reason)
        GATOS->>GATOS: Write Audit Decision
        GATOS-->>Client: Failure(reason)
    end
```

On **DENY**, the gate **MUST** append an audit decision to `refs/gatos/audit/policy`.

---

## 7. Privacy and Opaque Pointers

See also: [ADR‑0004](./decisions/ADR-0004/DECISION.md).

GATOS supports a hybrid privacy model where state can be separated into a verifiable public projection and a confidential private overlay. This is achieved by applying a deterministic **Projection Functor** during the state fold process, which replaces sensitive or large data with **Opaque Pointers**.

### 7.1 Projection Model

The State Engine (`gatos-echo`) can be configured with privacy rules. When folding history, it first computes a `UnifiedState` containing all data. It then applies the privacy rules to produce a `PublicState` and a set of `PrivateBlobs`.

-   **`PublicState`**: Contains only public data and Opaque Pointers. This is committed to the public `refs/gatos/state/public/...` namespace and is globally verifiable.
-   **`PrivateBlobs`**: The raw data that was redacted or pointerized. This data is stored in a separate, private store (e.g., a local directory, a private object store) and is addressed by its content hash.

Any commit that is the result of a privacy projection **MUST** include trailers indicating the number of redactions and pointers created.

```text
Privacy-Redactions: 5
Privacy-Pointers: 2
```

### 7.2 Opaque Pointers

An Opaque Pointer is a canonical JSON object that acts as a verifiable, addressable link to a private blob. It replaces the sensitive data in the `PublicState`.

```mermaid
classDiagram
    class OpaquePointer {
        +string kind: "opaque_pointer"
        +string algo: "blake3"
        +string digest: "blake3:<hex>"
        +number size
        +string location
        +string capability
    }
```

-   `digest`: The **REQUIRED** `blake3` hash of the raw private data. This ensures the integrity of the private blob.
-   `location`: A **REQUIRED** URI indicating where the blob can be fetched (e.g., `gatos-node://ed25519:<pubkey>`, `s3://...`).
-   `capability`: A **REQUIRED** URI defining the auth/authz and decryption mechanism needed to access the blob (e.g., `gatos-key://...`, `kms://...`).

The pointer itself is canonicalized and its `content_id` can be computed for verification purposes.

### 7.3 Pointer Resolution

A client resolving an Opaque Pointer **MUST** perform the following steps:
1.  Fetch the private blob from the `location` URI, authenticating if required by the endpoint protocol.
2.  Acquire the necessary authorization and/or decryption keys by interacting with the `capability` URI's system.
3.  If the blob is encrypted, decrypt it.
4.  Verify that the `blake3` hash of the resulting plaintext exactly matches the `digest` in the pointer. The resolution **MUST** fail if the hashes do not match.

This process guarantees that even though the data is stored privately, its integrity is verifiable against the public ledger.

---

## 8. Message Bus (Commit‑Backed Pub/Sub)

The message bus provides a pub/sub system built on Git commits.

```mermaid
sequenceDiagram
    participant Publisher
    participant GATOS
    participant Consumer

    Publisher->>GATOS: Publish Message (QoS: exactly_once)
    GATOS-->>Consumer: Deliver Message
    Consumer->>Consumer: Process Message
    Consumer->>GATOS: Send Ack
    GATOS->>GATOS: Observe Ack Quorum
    GATOS->>GATOS: Create gmb.commit Event
```

Messages are appended to `refs/gatos/mbus/<topic>/<shard>`. `exactly_once` delivery semantics require consumers to write an `ack` to `refs/gatos/mbus-ack/`.

---

## 9. Sessions (Working Branches)

`gatos/sessions/<actor>/<ulid>` represents an ephemeral branch for interactive mutation.

```mermaid
graph TD
    A(main) --> B(session-1);
    B --> C(commit-a);
    C --> D(commit-b);
    subgraph "fork"
        B --> E(session-2);
    end
    subgraph "undo"
        D -- undo --> C;
    end
    subgraph "merge"
        D --> F(main);
        E --> F;
    end
```

---

## 10. Proofs (Commitments / ZK)

A proof envelope attests to the deterministic execution of a fold or job.

```mermaid
classDiagram
    class ProofEnvelope {
        +String type
        +String ulid
        +String inputs_root
        +String output_root
        +String policy_root
        +String proof
        +String sig
    }
```

Proofs **MUST** be stored under `refs/gatos/audit/proofs/<ns>`.

---

## 11. Offline Authority Protocol (OAP)

OAP governs how divergent changes from offline peers are reconciled upon reconnecting.

```mermaid
sequenceDiagram
    participant PeerA
    participant PeerB
    Note over PeerA, PeerB: Peers are offline and make divergent changes
    PeerA ->> PeerB: Reconnect & Exchange Envelopes
    PeerB ->> PeerB: Validate Signatures & Policy Ancestry
    alt Policies are comparable
        PeerB ->> PeerB: Prefer descendant policy
    else Policies are incomparable
        PeerB ->> PeerB: Append governance.conflict event
    end
```

---

## 12.  Profiles

Profiles define the enforcement and operational mode of a GATOS node.

```mermaid
graph TD
    A(GATOS) --> B(local);
    A --> C(push-gate);
    A --> D(saas-hosted);
```

Nodes **MUST** discover the active profile via `gatos/config/profile.yaml`.

---

## 13.  Observability & Health

Implementations **SHOULD** expose metrics and provide a health-check CLI command.

```mermaid
graph TD
    A[gatosd] -- Exposes --> B["/metrics"];
    C[Prometheus] -- Scrapes --> B;
    D[gatos doctor] -- Diagnoses --> E(Ref Invariants);
    D --> F(Epoch Continuity);
    D --> G(Cache Staleness);
```

---

## 14.  Security Model

The security model is deny-by-default, governed by capability grants evaluated by the policy engine.

```mermaid
graph TD
    subgraph "Access Control"
        A[Actor] -- Requests access to --> B[Resource];
        C[Capability Grant] -- Links --> A;
        C -- Links --> D{Policy};
        D -- Evaluates request for --> B;
    end
```

---

## 15.  Performance & GC

Epoch compaction is used to manage repository size over time.

```mermaid
graph TD
    A[Epoch N] --> B(Epoch N+1 Anchor);
    B -- Triggers --> C{Compaction};
    C -- Prunes --> D(Unreferenced Blobs in Epoch N);
```

---

## 16.  Compliance & Tests (Normative)

Implementations **MUST** pass a six-point certification inspection.

```mermaid
graph TD
    A(GATOS Implementation) --> B(Certification);
    B --> C(Deterministic Fold);
    B --> D(Exactly-Once Delivery);
    B --> E(Offline Reconcile);
    B --> F(Deny Audit);
    B --> G(Blob Integrity);
    B --> H(Consensus Integrity);
```

### 16.1 Consensus Integrity (Normative)

- An action gated by a `2-of-3` quorum policy MUST be denied with 1 approval and MUST be allowed with 2 approvals.
- A revoked grant MUST NOT be usable.

---

## 17. CLI (Reference)

The `git gatos` command provides the primary user interface.

```mermaid
graph TD
    A(git gatos) --> B(init);
    A --> C(session);
    A --> D(event);
    A --> E(fold);
    A --> F(bus);
    A --> G(policy);
    A --> H(trust);
    A --> I(epoch);
    A --> J(prove);
    A --> K(doctor);
```

---

## 18. Example Use Case: A Git-Native Work Queue

This diagram shows the data flow for enqueuing and processing a job.

```mermaid
sequenceDiagram
    participant Client
    participant Daemon as gatosd
    participant Ledger as gatos-ledger
    participant Bus as gatos-mind
    participant State as gatos-echo

    Client->>Daemon: 1. Enqueue Job (Event)
    Daemon->>Ledger: 2. Append `jobs.enqueue` event
    Ledger-->>Daemon: 3. Success
    Daemon->>Bus: 4. Publish `gmb.msg` to topic
    Bus-->>Daemon: 5. Success
    Daemon-->>Client: 6. Job Enqueued

    Note over Bus,State: Later, a worker consumes the job...

    participant Worker
    Worker->>Bus: 7. Subscribe to topic
    Bus->>Worker: 8. Delivers `gmb.msg`
    Worker->>Daemon: 9. Report Result (Event)
    Daemon->>Ledger: 10. Append `jobs.result` event
    Ledger-->>Daemon: 11. Success
    Daemon->>Bus: 12. Write `gmb.ack`
    Daemon-->>Worker: 13. Result Recorded

    Note over Ledger,State: A fold process runs...
    State->>Ledger: 14. Read events from journal
    State->>State: 15. Compute new state (e.g., update queue view)
    State->>Ledger: 16. Checkpoint new state
```

---

## 19. Job Plane (Compute)

See also: [ADR‑0002](./decisions/ADR-0002/DECISION.md).

The Job Plane provides a system for scheduling, executing, and recording the results of distributed, asynchronous jobs.

### 19.1 Job Lifecycle

This diagram illustrates how the state of a Job transitions based on events recorded in the GATOS ledger.

```mermaid
stateDiagram-v2
    [*] --> pending

    pending --> claimed: Worker claims job (CAS)
    claimed --> running: Worker begins execution
    running --> succeeded: `jobs.result` (ok)
    running --> failed: `jobs.result` (fail)

    succeeded --> [*]
    failed --> [*]

    pending --> aborted: Canceled by user/policy
    claimed --> aborted: Canceled by user/policy
    aborted --> [*]
```

The lifecycle is represented entirely through Git objects:

- **Job:** A commit whose tree contains a `job.yaml` manifest.
- **Claim:** An atomic ref under `refs/gatos/jobs/<job-id>/claims/<worker-id>`, where `<job-id>` is the canonical BLAKE3 `content_id` of the job manifest (see ADR‑0002 Canonical Job Identifier).
- **Result:** A commit referencing the job commit, containing a `Proof-Of-Execution`.

### 19.2 Job Discovery

When a **Job** commit is created, a message **MUST** be published to a topic on the Message Plane for discovery by workers.

### 19.3 Proof-Of-Execution

The **Proof‑Of‑Execution (PoE)** MUST sign the job’s canonical `content_id` (BLAKE3 of the canonical unsigned job core). Trailers MUST use canonical, prefixed encodings as follows:

- `Job-Id: blake3:<hex>` — canonical job identifier (content_id)
- `Proof-Of-Execution: blake3:<hex>` — digest of the PoE envelope
- `Worker-Id: ed25519:<pubkey>` — worker public key identifier
- `Attest-Program: blake3:<hex>` — hash of runner binary or WASM module (RECOMMENDED)
- `Attest-Sig: ed25519:<sig>` — signature over the attestation envelope (OPTIONAL)

Example (trailers):

```text
Job-Id: blake3:<hex>
Worker-Id: ed25519:<pubkey>
Proof-Of-Execution: blake3:<hex>
Attest-Program: blake3:<hex>
Attest-Sig: ed25519:<sig>
```

See ADR‑0002 for the normative PoE requirements and ADR‑0001 for the definition of `content_id` and canonical serialization.

---

## 20. Consensus Governance (Normative)

See also: [ADR‑0003](./decisions/ADR-0003/DECISION.md).

Governs gated actions via proposals, approvals, and grants. Governance artifacts are Git commits under dedicated refs (see on‑disk layout). All trailers MUST use canonical, prefixed encodings (`blake3:<hex>`, `ed25519:<pubkey>`).

### 20.1 Workflow

Proposal → Approvals (N‑of‑M) → Grant. Quorum groups (e.g., `@leads`) MUST be defined in the trust graph (`gatos/trust/graph.json`).

### 20.2 Commit Structures (Trailers)

- Proposal (at `refs/gatos/proposals/…`):

  ```text
  Action: <string>
  Target: <uri>
  Proposal-Id: blake3:<hex>
  Required-Quorum: <expr>
  Expire-At: <ISO8601>
  Policy-Rule: <policy id>
  Created-By: <actor>
  ```

  (Note: `gatos://` is the canonical URI scheme for addressing resources managed within the GATOS operating surface.)
- Approval (at `refs/gatos/approvals/…`):

  ```text
  Proposal-Id: blake3:<hex>
  Approval-Id: blake3:<hex>
  Signer: ed25519:<pubkey>
  Expires-At: <ISO8601>   # OPTIONAL
  ```

- Grant (at `refs/gatos/grants/…`):

  ```text
  Proposal-Id: blake3:<hex>
  Grant-Id: blake3:<hex>
  Proof-Of-Consensus: blake3:<hex>
  ```

### 20.3 Proof‑Of‑Consensus (PoC)

`Proof-Of-Consensus` is the BLAKE3 of a canonical JSON envelope containing:

- The canonical proposal envelope (by value or `Proposal-Id`).
- A sorted list (by `Signer`) of all valid approvals used to reach quorum (by value or `Approval-Id`).
- The governance rule id (`Policy-Rule`) and effective quorum parameters.

PoC envelope SHOULD be stored canonically under `refs/gatos/audit/proofs/governance/<proposal-id>`; the Grant’s `Proof-Of-Consensus` trailer MUST equal `blake3(envelope_bytes)`.

### 20.4 Lifecycle States

| State    | Meaning                         |
|:---------|:--------------------------------|
| proposal | Awaiting votes                  |
| partial  | Some approvals collected        |
| granted  | Quorum reached; action allowed  |
| expired  | Proposal timed out              |
| revoked  | Grant withdrawn or superseded   |

```mermaid
stateDiagram-v2
    [*] --> proposal
    proposal --> partial: approval received
    partial --> partial: additional approvals
    proposal --> expired: ttl elapsed
    partial --> expired: ttl elapsed
    partial --> granted: quorum satisfied
    granted --> revoked: revocation committed
    expired --> [*]
    revoked --> [*]
```

### 20.5 Revocation

A grant MAY be revoked by creating a `revocation` commit under `refs/gatos/revocations/` with trailers:

```text
Grant-Id: blake3:<hex>
Revocation-Id: blake3:<hex>
Reason: <free-text>
Revoked-By: <actor>
```

### 20.6 Bus Topics (recommended)

`gatos.governance.proposal.created`, `gatos.governance.approval.created`, `gatos.governance.grant.created`, `gatos.governance.grant.revoked`.
