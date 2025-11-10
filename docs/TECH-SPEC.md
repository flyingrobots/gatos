# GATOS — TECH-SPEC v0.3

## Implementation Plan, Data Structures, and Algorithms

> *This is how we GATOS.*

---

## 1. Codebase Layout (Rust Workspace)

The GATOS workspace is organized into `crates` for core components and `bindings` for FFI.

```mermaid
graph TD
    subgraph gatos
        A(crates) --> A1(gatos-ledger-core)
        A --> A2(gatos-ledger-git)
        A --> A3(gatos-mind)
        A --> A4(gatos-echo)
        A --> A5(gatos-policy)
        A --> A6(gatos-kv)
        A --> A7(gatosd)
        A --> A8(gatos-compute)
        B(bindings) --> B1(wasm)
        B --> B2(ffi)
    end
```

### Reuse & refactor recommendations

- Reuse **Echo** crates for fold determinism (`rmg-core` as the fold engine).
- Adopt **`git-kv`** “Stargate” concepts for optional `push-gate` profile.
- Integrate **Wesley** as a compiler target to emit schemas and fold specs.

---

## 2. Crate Architecture

GATOS follows a "Ports and Adapters" architecture. The core logic is pure and portable (`no_std`), while I/O is handled by specific "adapters."

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

### Crate Summary

| Crate | Purpose |
|:---|:---|
| `gatos-ledger-core` | `no_std` core logic, data structures, and traits for the ledger. |
| `gatos-ledger-git` | `std`-dependent storage backend using `libgit2`. |
| `gatos-ledger` | Composes ledger components via feature flags. |
| `gatos-mind` | Asynchronous, commit-backed message bus (pub/sub). |
| `gatos-echo` | Deterministic state engine for processing events ("folds"). Privacy projection logic. |
| `gatos-policy` | Deterministic policy engine for executing compiled rules, managing Consensus Governance, and privacy rule evaluation. |
| `gatos-kv` | Git-backed key-value state cache. |
| `gatosd` | Main binary for the CLI, JSONL RPC daemon, and Opaque Pointer resolution endpoint. |
| `gatos-compute` | Worker that discovers and executes jobs from the Job Plane. |
| `gatos-wasm-bindings`| WASM bindings for browser and Node.js environments. |
| `gatos-ffi-bindings` | C-compatible FFI for integration with other languages. |

---

## 3. Fold Engine (Echo integration)

Note on policy rule naming

- Governance actions are referenced as `governance.<action>`; `action` tokens use dot‑notation (e.g., `publish.artifact`). Policy evaluation resolves these against the governance policy map defined in `schemas/v1/policy/governance_policy.schema.json`.

The Fold Engine consumes canonicalized events to produce a canonical state tree.

```mermaid
graph TD
    A[Canonical Events] --> B{FoldEngine};
    B -- uses --> C[rmg-core];
    B --> D[Canonical JSON Tree];
    D -- blake3 --> E[state_root];
```

---

## 4. Index & Cache

Rebuildable indexes are created by folding journal events into Roaring Bitmaps.

```mermaid
graph TD
    A[Journal Events] -- folded by --> B(Indexer);
    B -- produces --> C[Roaring Bitmap];
    C -- stored in --> D(refs/gatos/cache/);
```

---

## 5. Epochs & Compaction

Epochs manage history size by creating periodic anchors and enabling garbage collection.

```mermaid
sequenceDiagram
    participant User
    participant GATOS

    User->>GATOS: gatos epoch new <ns>
    GATOS->>GATOS: Create new anchor at refs/gatos/epoch/<ns>/<epoch-id>
    GATOS->>GATOS: Start Compactor
    GATOS->>GATOS: Walk reachability from state_root
    GATOS->>GATOS: Prune unreferenced blobs
```

---

## 6. Privacy Projection and Resolution

See also: [ADR‑0004](./decisions/ADR-0004/DECISION.md).

The implementation of the hybrid privacy model involves a coordinated effort between the state, policy, and daemon components.

### 6.1 Projection Implementation

The projection from a `UnifiedState` to a `PublicState` is handled by `gatos-echo` with rules supplied by `gatos-policy`.

```mermaid
sequenceDiagram
    participant gatos-echo
    participant gatos-policy
    participant gatos-ledger
    participant "PrivateStore (Interface)" as "Storage Backend"

    Echo->>Echo: 1. Fold event history to produce UnifiedState
    Echo->>Policy: 2. Request privacy rules for the current context
    Policy-->>Echo: 3. Return `select` and `action` rules
loop for each field path in the UnifiedState tree
        gatos-echo->>gatos-echo: 4. Match field path against rules
        alt rule matches (e.g., "pointerize")
            Echo->>Echo: 5. Generate Opaque Pointer envelope
            Echo->>PrivateStore: 6. Store original node value as private blob, keyed by its blake3 digest
            Echo->>Echo: 7. Replace node in state tree with pointer
        end
    end
    Echo->>Ledger: 8. Commit the final PublicState tree
```

The `PrivateStore` is a pluggable trait, allowing for backends like a local filesystem, S3, or another GATOS node.

### 6.2 Resolution Implementation

The `gatosd` daemon exposes a secure endpoint for resolving Opaque Pointers.

-   **Endpoint**: `gatosd` will listen for authenticated requests, for example at `/gatos/private/blobs/{digest}`.
-   **Authentication**: The client SDK **MUST** send a `Authorization` header containing a JSON Web Signature (JWS) with a detached payload. The JWS payload **MUST** be the BLAKE3 hash of the request body. `gatosd` verifies the signature against the actor's public key.
-   **Authorization**: Upon receiving a valid request, `gatosd` queries `gatos-policy` to determine if the requesting actor has the capability to access the blob identified by `{digest}`.
-   **Response**: If authorized, `gatosd` fetches the (likely encrypted) blob from its configured `PrivateStore` and returns it to the client. The client is then responsible for decryption via the `capability` URI.

---

## 7. JSONL Protocol

Communication with `gatosd` occurs over a JSONL RPC protocol. Long‑running operations MUST quickly return an `{ "ack": true }` and stream progress lines keyed by id.

```mermaid
sequenceDiagram
    participant Client as Client (SDK/CLI)
    participant Daemon as gatosd

    Client->>Daemon: {"type":"append_event", "id":"01A", "ns":"...", "event":{...}}
    Daemon-->>Client: {"ok":true, "id":"01A", "commit_id":"..."}

    Client->>Daemon: {"type":"bus.subscribe", "id":"01C", "topic":"..."}
    Daemon-->>Client: {"ack":true, "id":"01C"}
    loop Subscription Stream
        Daemon-->>Client: {"type":"bus.message", "id":"01C", "topic":"...", "payload":{...}}
    end
```

---

## 8. Observability

`gatosd` exposes key performance metrics for monitoring.

```mermaid
graph TD
    subgraph "gatosd"
        A(Journal)
        B(Fold Engine)
        C(Message Bus)
    end
    subgraph "Metrics"
        M1(gatos_journal_append_latency_ms)
        M2(gatos_fold_latency_ms)
        M3(gatos_bus_ack_lag)
    end
    A --> M1
    B --> M2
    C --> M3
```

---

## 9. CI & Cross‑Platform Determinism

A CI matrix ensures determinism across platforms and runs specialized test suites.

```mermaid
graph TD
    A(CI Pipeline) --> B(Test Matrix);
    B --> B1(linux-amd64-glibc);
    B --> B2(macOS-arm64);
    B --> B3(Windows-amd64);
    A --> C(Test Suites);
    C --> C1(Golden Vectors);
    C --> C2(Torture Tests);
    C --> C3(Reconcile Harness);
    C --> C4(Projection Determinism);
```

- **Projection Determinism**: Verifies that applying the same privacy policy to the same `UnifiedState` on different platforms (Linux, macOS, Windows) produces a byte-for-byte identical `PublicState` and the same set of private blobs.

---

## 10. Security

Signature verification is a critical step in event processing.

```mermaid
sequenceDiagram
    participant Client
    participant GATOS
    participant Libsodium

    Client->>GATOS: Submit Signed Event
    GATOS->>GATOS: Canonicalize JSON
    GATOS->>Libsodium: ed25519_verify(signature, payload, pubkey)
    alt Signature is Valid
        Libsodium-->>GATOS: OK
        GATOS->>GATOS: Process Event
    else Signature is Invalid
        Libsodium-->>GATOS: Fail
        GATOS-->>Client: Reject Event
    end
```

Examples

```json
{"type":"append_event","id":"01A","ns":"finance","event":{}}
{"type":"bus.subscribe","id":"01C","topic":"gatos.jobs.pending"}
{"type":"fold_state","id":"01D","ns":"finance","channel":"table","spec":"folds/invoices.yaml"}
{"type":"governance.proposal.new","id":"02A","action":"publish.artifact","target":"gatos://assets/model.bin","quorum":"2-of-3@leads"}
{"type":"governance.approval.add","id":"02B","proposal":"<proposal-id-hash>"}
{"type":"governance.grant.verify","id":"02C","grant":"<grant-id-hash>"}
```

---

## 11.  Performance Guidance

Tuning batch size is a trade-off between latency and commit churn.

```mermaid
xychart-beta
    title "Batch Size Trade-off"
    x-axis "Batch Size"
    y-axis "Metric"
    line "Latency" [50, 40, 35, 32, 30]
    line "Commit Churn" [10, 20, 40, 80, 160]
```

---

## 12. Client SDKs

SDKs provide language-native access to the `gatosd` JSONL RPC endpoint.

```mermaid
graph TD
    A(gatosd) -- JSONL RPC --> B(Go SDK);
    A -- JSONL RPC --> C(Python SDK);
    A -- JSONL RPC --> D(Rust SDK);
    A -- JSONL RPC --> E(Node.js SDK);
```

---

## 13. Migration Strategies

A phased migration ensures a safe transition to GATOS.

```mermaid
gantt
    title GATOS Migration Strategy
    dateFormat  YYYY-MM-DD
    section Phase A: Mirror
    Mirror Mode     :2025-01-01, 30d
    section Phase B: Shadow
    Shadow Consumers :2025-02-01, 30d
    section Phase C: Dual-Read
    Canary (10%)    :2025-03-01, 30d
    section Phase D: Cutover
    Full Cutover    :2025-04-01, 7d
```

---

## 14. Wire-Format Invariants

To ensure hash stability, GATOS uses a standard canonical encoding format.

```mermaid
classDiagram
    class BincodeConfig {
        <<Rust>>
        +standard()
    }
    class Hash {
        +[u8; 32]
    }
    BincodeConfig ..> Hash : Encodes
```

---

## 15. Compute Engine (Job Runner)

See also: [ADR‑0002](./decisions/ADR-0002/DECISION.md).

The `gatos-compute` crate provides the GATOS worker process.

```mermaid
sequenceDiagram
    participant Client
    participant GATOS (Ledger)
    participant Bus (Message Plane)
    participant Worker

    Client->>GATOS (Ledger): 1. Create Job Commit
    GATOS (Ledger)->>Bus (Message Plane): 2. Publish Job message
    Worker->>Bus (Message Plane): 3. Subscribe to job topic
    Bus (Message Plane)->>Worker: 4. Receive Job message
    Worker->>GATOS (Ledger): 5. Atomically create Claim ref
    GATOS (Ledger)-->>Worker: 6. Claim successful
    Worker->>Worker: 7. Execute Job
    Worker->>GATOS (Ledger): 8. Create Result commit
```

### Implementation Plan

1. **Subscription:** The worker will use `gatos-mind` to subscribe to job topics.
2. **Claiming:** The worker will use `gatos-ledger` to atomically claim a job via compare-and-swap on a Git ref.
3. **Execution:** The worker will execute the job's `command` in a sandboxed environment.
4. **Result & Proof:** The worker will create a `Result` commit containing output artifacts and a `Proof-Of-Execution`.
5. **Lifecycle Management:** The worker will handle timeouts, retries, and failures.

---

## 16. Governance Engine

See also: [ADR‑0003](./decisions/ADR-0003/DECISION.md).

### Engine Responsibilities

- Watchers: a service in `gatos-policy` watches `refs/gatos/proposals/**` and `refs/gatos/approvals/**`.
- Verification: for each new Approval, verify signature and eligibility using the trust graph.
- Quorum check: evaluate the policy rule (`governance.<action>`) to determine if quorum is satisfied.
- Grant creation: when quorum is met, create a Grant commit with a canonical Proof‑Of‑Consensus envelope and update `refs/gatos/grants/...`.
- Gate enforcement: the Policy Gate checks for a valid Grant before allowing any governed action.

### CLI Skeleton (This defines the normative CLI user interface; stub behavior acceptable initially)

- `gatos proposal new --action <id> --target <uri> --quorum <expr> [--ttl <dur>]`
- `gatos approve --proposal <blake3:…> [--expires-at <ts>]`
- `gatos grant verify --grant <blake3:…>`

### Group Resolution

Governance evaluator MUST resolve groups declared in policy (e.g., `group: leads`) against `gatos/trust/graph.json`.

### Revocation Propagation

Revocations MUST be surfaced to dependent systems (e.g., Job Plane). Implementations SHOULD emit `gatos.policy.grant.revoked` and deny actions gated by revoked grants.

### End‑to‑End Flow

```mermaid
sequenceDiagram
    participant Client
    participant Ledger as GATOS (Ledger)
    participant Policy as Policy Engine
    participant Bus as Message Bus
    participant Approver as Approver (via CLI)

    Client->>Ledger: 1. Create Proposal (Action, Target, Quorum)
    Ledger->>Policy: 2. Validate proposal
    Policy-->>Ledger: 3. Accepted
    Ledger->>Bus: 4. Publish proposal.created

    loop Approvals
        Approver->>Ledger: 5. Create Approval (Signer, Proposal-Id)
        Ledger->>Policy: 6. Verify signature + eligibility
        Policy-->>Ledger: 7. Approval valid
    end

    Ledger->>Policy: 8. Check quorum
    alt Quorum satisfied
        Ledger->>Ledger: 9. Create Grant (Proof-Of-Consensus)
        Ledger->>Bus: 10. Publish grant.created
    else Not yet satisfied
        Ledger-->>Client: Pending (partial)
    end
```
