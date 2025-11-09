# GATOS — TECH-SPEC v0.3

**Implementation Plan, Data Structures, and Algorithms**

> _This is how we GATOS._

---

## 1. Codebase Layout (Rust Workspace)

The GATOS workspace is organized into two main directories: `crates` for the core Rust components and `bindings` for foreign language integration.

```rust
gatos/
├── crates/
│   ├── gatos-ledger-core/ # no_std; graph + proof math
│   ├── gatos-ledger-git/  # std; git2 backend for the ledger
│   ├── gatos-ledger/      # feature-composed meta-crate for the ledger
│   ├── gatos-mind/        # Async message bus (pub/sub)
│   ├── gatos-echo/        # Deterministic state engine (folds, sessions)
│   ├── gatos-policy/      # Compiled rule interpreter
│   ├── gatos-kv/          # Git-backed state cache
│   └── gatosd/            # CLI + daemon entrypoint
└── bindings/
    ├── wasm/              # WebAssembly bindings via wasm-bindgen
    └── ffi/               # C ABI for integrations
```

### Reuse & refactor recommendations

Reuse **Echo** crates for fold determinism (`rmg-core` as the fold engine).

Adopt **`git-kv`** “Stargate” concepts for optional `push-gate` profile (separate project/service).

Integrate **Wesley** as a compiler target `--target gatos` to emit schemas, RLS, and fold specs.

---

## 2. Crate Architecture

GATOS follows a modular, "Ports and Adapters" (Hexagonal) architecture. The core logic is pure and portable (`no_std`), while I/O-dependent functionality is provided by specific "adapters."

### Crate Summary

| Crate | Purpose |
|:---|:---|
| [`gatos-ledger-core`](../../crates/gatos-ledger-core/README.md) | Defines the `no_std` core logic, data structures, and traits for the ledger. |
| [`gatos-ledger-git`](../../crates/gatos-ledger-git/README.md) | Provides a `std`-dependent storage backend using `libgit2`. |
| [`gatos-ledger`](../../crates/gatos-ledger/README.md) | A meta-crate that composes the ledger components via feature flags. |
| [`gatos-mind`](../../crates/gatos-mind/README.md) | Implements the asynchronous, commit-backed message bus (pub/sub). |
| [`gatos-echo`](../../crates/gatos-echo/README.md) | The deterministic state engine for processing events ("folds") and managing sessions. |
| [`gatos-policy`](../../crates/gatos-policy/README.md) | The deterministic policy engine for executing compiled rules. |
| [`gatos-kv`](../../crates/gatos-kv/README.md) | A Git-backed key-value state cache for materialized views. |
| [`gatosd`](../../crates/gatosd/README.md) | The main binary entrypoint for the CLI and the JSONL RPC daemon. |
| [`gatos-wasm-bindings`](../../bindings/wasm/README.md) | WebAssembly bindings for browser and Node.js environments. |
| [`gatos-ffi-bindings`](../../bindings/ffi/README.md) | A C-compatible FFI for integration with other languages. |
| `gatos-compute` | A worker/runner that discovers and executes jobs from the Job Plane. |

### Ledger Architecture: Ports and Adapters

The ledger is the primary example of this hexagonal design, as documented in [ADR-0001](../decisions/ADR-0001/DECISION.md).
-   **Core (Hexagon):** `gatos-ledger-core` is the `no_std` core. It defines the "port" for persistence via the `ObjectStore` trait.
-   **Adapters:** `gatos-ledger-git` is an adapter that implements `ObjectStore` using a standard Git repository. Other backends (e.g., in-memory, flat-file) can be added by creating new adapter crates.
-   **Composition:** The `gatos-ledger` meta-crate acts as the public entry point, using Cargo features to provide the consumer with the correct combination of core logic and storage backend.

---

## 3. Fold Engine (Echo integration)

### Use `rmg-core` as a library

#### Build a `FoldEngine` 

Consumes canonicalized events and produces a canonical JSON tree.

Compute `state_root = blake3(canonical_bytes)`.

#### Deterministic scheduler
  
- $O(n)$ radix ordering provided by Echo; 
- ensure domain-separated hashing.

---

## 4. Index & Cache

Roaring bitmap indexes per namespace under `refs/gatos/cache/<ns>-<index-id>`.

- Indexers run as pure folds over journals; 
- outputs are rebuildable artifacts.

Provide `gatos cache rebuild <ns>` and lazy rebuild on staleness.

---

## 5. Epochs & Compaction

### Epoch anchor 

Commit stored at `refs/gatos/epoch/<ns>/<epoch-id>`.

`gatos epoch new <ns>` creates a new anchor (tag/ref) and prunes fetch depth for newcomers.

## Compactor

Walks reachability from current `state_root` and drops unreferenced blobs per policy retention.

---

## 6. Opaque Pointers

### CAS layout 

`gatos/objects/<algo>/<hash>` for plaintext or ciphertext.

- Pointer manifests committed to the repo; 
- ciphertext stored locally or on untrusted remote.
  
### Re-keying command 

`gatos blob rekey <ptr> --to <pubkey>` creates a new opaque pointer (new ciphertext_hash) and updates references atomically.

---

## 7. JSONL Protocol

- Every request/response is one JSON line.
- Long-running operations **MUST** return `{"ack":true}` quickly and stream progress lines keyed by id.

### Examples

```json
{"type":"append_event","id":"01A","ns":"finance","actor":"james","event":{...}}
{"type":"bus.subscribe","id":"01B","topic":"echo/jobs","from":"HEAD-100"}
{"type":"fold_state","id":"01C","ns":"finance","channel":"table","spec":"folds/invoices.yaml"}
```

### Errors

```json
{"ok":false,"id":"01A","error":{"code":"POLICY_DENY","rule":"exec.rgs:12","reason":"caps missing: exec:run"}}
```

---

## 8. Observability

Expose metrics via `gatosd /metrics`:

- `gatos_journal_append_latency_ms`
- `gatos_fold_latency_ms`
- `gatos_bus_ack_lag`
- `gatos_cache_rebuilds_total`

`gatos doctor` checks

- FF-only invariant, 
- epoch continuity, 
- cache staleness, 
- packfile size/ratio, 
- dangling objects threshold.

---

## 9. CI & Cross‑Platform Determinism

### Matrix

- linux-amd64 (glibc), 
- linux-amd64 (musl), 
- macOS‑arm64, 
- Windows‑amd64, 
- wasm32 (headless)

### Tests

- Golden vectors for folds,
- Bus exactly-once torture tests (dupe publishes, consumer crashes),
- Offline reconcile harness,
- CAS integrity tests (bit‑flip detection).

---

## 10. Security

### Signature verification

- `ed25519` via `libsodium`;
- SSH/GPG adapter optional.
- Canonical JSON (sorted keys, no whitespace variance) for signatures and hashing.
- Deny-by-default policy; 
- capability chains enforced before any write.
- Key rotation playbooks; 
- grant chain verification.

---

## 11.  Performance Guidance

- Batch multiple events into a single tree and one commit where possible (e.g., batching 64 enqueues to reduce ref churn).
- Prefer manifest+chunking over Git‑LFS.
- Tune pack GC with size thresholds; document recommended `gc.auto`, `fetch.writeCommitGraph`.
- For message bus topics, start with a reasonable number of shards (e.g., 64) and store the shard map in a configuration file (e.g., `gatos/mbus-config/<topic>.json`). Resharding can be achieved via a versioned shard map and a dual-write migration window.

> Tuning guidance: Batch size trades off latency vs. commit churn. Start with 64 and measure p99 latency and commits/sec; reduce batch size to lower latency, or increase it to reduce ref churn. Shard count trades off per-shard throughput vs. management overhead; start with 64 shards, measure messages/sec per shard and consumer lag, and scale up/down accordingly.

---

## 12. Client SDKs

To facilitate integration with existing applications, GATOS nodes can be accessed via client SDKs that communicate with the `gatosd` daemon over the JSONL RPC protocol. This provides a language-agnostic way to interact with a GATOS repository.

### Example: Go SDK (`gatos-go`)

A Go SDK would provide a simple API for interacting with GATOS, abstracting away the details of the JSONL protocol.

```go
// Publish a job
cli.PublishMsg(ctx, "queue.acme", Msg{
  Ulid: NewULID(),
  Headers: map[string]string{"priority":"high"},
  Payload: Job{ID: jobID, PayloadPtr: ptr},
  QoS: ExactlyOnce,
})

// Consume a job
sub := cli.Subscribe(ctx, "queue.acme", Shards{0,1,2})
for m := range sub.C {
  // Idempotent work
  res := run(m.Payload)
  cli.AppendEvent(ctx, "jobs/acme/worker-42", NewResult(res))
  cli.Ack(ctx, "queue.acme", m.ULID)
  cli.CommitIfQuorum(ctx, "queue.acme", m.ULID) // Can be run by publisher or a coordinator
}
```

Under the hood, the SDK would send and receive JSONL messages to and from the `gatosd` daemon for operations like `append_event`, `bus.publish`, `bus.subscribe`, `bus.ack`, and `bus.commit`.

---

## 13. Migration Strategies

Migrating an existing application to GATOS can be done in a phased approach to minimize risk and downtime. The following "mirror, shadow, dual-read, cutover" strategy is recommended.

### Phase A: Mirror Mode

- The existing system remains the source of truth.
- Application producers are modified to dual-write to both the existing system and to the GATOS journal and message bus.
- GATOS workers are not yet active.

### Phase B: Shadow Consumers

- Stand up GATOS workers that consume from the message bus but perform no-op operations.
- This allows for validation of the GATOS data flow, ordering, and performance without affecting the live system.

### Phase C: Dual-Read & Canary

- A small percentage of traffic (e.g., 1%) is routed to the GATOS workers to perform real work.
- The results are compared with the existing system to ensure parity.
- The percentage of traffic is gradually increased as confidence in the GATOS implementation grows.

### Phase D: Cutover

- Once parity is proven, producers are switched to write only to GATOS.
- The existing system can be kept in a read-only mode for a short period to allow for rollback if necessary, before being retired.

---

## 14. Wire-Format Invariants

To ensure interoperability and hash stability across implementations, GATOS adopts the following
wire‑format invariants:

- Canonical encoding: bincode v2 with `config::standard()` for all canonically serialized types.
- Endianness: Multi‑byte primitives are encoded by bincode; consumers MUST treat values as encoded
  bytes without reinterpretation.
- Fixed‑size arrays: Types like `Hash = [u8; 32]` are encoded verbatim as fixed‑length byte arrays.
- Floats: Avoid in content‑addressed data. If floats are required by policy, they MUST be treated as
  raw IEEE‑754 bytes and documented for the specific type.
- Versioning: Schema evolution MUST be explicit (e.g., enums or versioned wrappers). Reordering
  fields or changing enum variant orders is a breaking change that alters bytes.

These rules complement the module‑level documentation in `gatos‑ledger‑core` and the event envelope
schemas in `SPEC.md`.

---

## 15. Compute Engine (Job Runner)

The `gatos-compute` crate will provide the primary implementation of a GATOS worker process. This worker is responsible for discovering, claiming, and executing jobs defined in the Job Plane.

### Implementation Plan

1.  **Subscription:** The worker will use the `gatos-mind` crate to subscribe to one or more job topics on the Message Plane (e.g., `gatos/jobs/pending`).
2.  **Claiming:** Upon receiving a job message, the worker will use the `gatos-ledger` crate to attempt an atomic claim by creating a ref at `refs/gatos/jobs/<job-id>/claims/<worker-id>` where `job-id` is the job’s `content_id` (BLAKE3). The operation will use compare-and-swap semantics to ensure only one worker can claim a given job. Messages MAY carry a ULID for convenience; workers MUST resolve the ULID to the canonical `job-id` by reading the job commit.
3.  **Execution:** Once a job is claimed, the worker will execute the job's `command` as defined in its manifest. Execution will take place in a sandboxed environment (e.g., a container or a WASM runtime) to ensure isolation.
4.  **Result & Proof:** Upon completion, the worker will create a `Result` commit. This involves:
    *   Storing any output artifacts (e.g., logs, data) as blobs.
    *   Generating a `Proof-Of-Execution` by signing the job's `content_id` and an attestation envelope.
    *   Committing the result and proof to the ledger with the appropriate `Job-Id` and other trailers.
5.  **Lifecycle Management:** The worker will be responsible for updating job state and handling timeouts, retries (as dictated by policy), and failures.
