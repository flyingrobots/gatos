# GATOS — TECH-SPEC v0.3

**Implementation Plan, Data Structures, and Algorithms**

> _This is how we GATOS._

---

## 1. Codebase Layout (Rust workspace)

```rust
gatos/
crates/
gatos-core/        # Git-backed ledger + CAS + notes
gatos-policy/      # Gate + policy VM (RGS bytecode)
gatos-session/     # Session engine (undo/fork/merge)
gatos-bus/         # Message bus (QoS, shards, acks)
gatos-proof/       # Commitment/ZK proof interfaces
gatosd/            # JSONL RPC server + CLI
vendor/
libgitledger/      # (optional) C reference via FFI for parity tests
roaring-rs/        # Roaring bitmaps (cache)
```

### Reuse & refactor recommendations

Reuse **Echo** crates for fold determinism (`rmg-core` as the fold engine).

Optional: reuse **`libgitledger`** for journal append semantics or as a parity checker in tests.

Adopt **`git-kv`** “Stargate” concepts for optional `push-gate` profile (separate project/service).

Integrate **Wesley** as a compiler target `--target gatos` to emit schemas, RLS, and fold specs.

---

## 2. Core Crates

### 2.1 `gatos-core`

#### Responsibilities

- Git repository integration (`libgit2`)
- Ledger: append/read, atomic ref updates (CAS)
- CAS blob store (opaque/normal)
- Notes helpers (`policy_root`/`trust_chain` on commits)
- Epoch management

#### Key types

```rust
pub struct GitLedger { pub repo: Repository }
pub struct Event { /* canonical JSON */ }
pub struct BlobPtr { algo: Algo, hash: [u8;32], size: u64, kind: BlobKind }
pub enum BlobKind { Plain, Opaque{ ciphertext_hash: [u8;32], cipher_meta: CipherMeta } }
pub struct EpochId([u8;32]);

impl GitLedger {
  pub fn append(&mut self, ns: &str, actor: &str, ev: &Event, expect: Oid) -> Result<Oid>;
  pub fn iter(&self, ns_glob: &str) -> Result<EventIter>;
  pub fn checkpoint_state(&mut self, ns: &str, tree: &Tree, state_root: [u8;32]) -> Result<Oid>;
  pub fn note_policy(&mut self, commit: Oid, policy_root: [u8;32], trust_chain: [u8;32]) -> Result<()>;
}
```

#### Algorithms

##### Atomic ref update

Use `git2::Reference::set_target_checked(old_oid, new_oid, force=false)`.

##### Content-defined chunking (FastCDC) for CAS 

- library integration; 
- store manifests as blobs, 
- chunks as objects.

### 2.2 `gatos-policy`

#### Responsibilities

- Compile RGS → RGC (`policy_root = sha256(bytes)`)
- Deterministic interpreter (no I/O)
- Decision model with explainable results

#### API

```rust
pub struct PolicyGate { bundle: Arc<PolicyBundle>, trust: TrustGraph }
pub enum Decision { Allow, Deny { reason: String, rule_id: String } }

impl PolicyGate {
  pub fn evaluate(&self, intent: &Intent, ctx: &Context) -> Decision;
}
```

### 2.3 `gatos-session`

#### Responsibilities

- Ephemeral branch management under `refs/gatos/sessions/...`
- Undo/fork/merge using deterministic lattice-join + DPO adapters from Echo
- Workspace projection (optional)

### 2.4 `gatos-bus`

#### Responsibilities

- Topics, shards, QoS semantics
- Acks and commitments
- Backpressure windows; 
- dead-letter topics

#### Data structures

```rust
pub enum QoS { AtMostOnce, AtLeastOnce, ExactlyOnce }
pub struct Message { ulid: Ulid, topic: String, payload: Value, ... }
pub struct Ack { msg_ulid: Ulid, consumer: String, result: AckResult }
```

#### Algorithm (exactly-once)

1. Publisher writes `gmb.msg`.
2. Consumers de-dup by `(topic, ulid)` and write `gmb.ack`.
3. Publisher waits for quorum; writes `gmb.commit`.
4. Consumers discard duplicates `if (topic, ulid)` seen with a commit.

#### Storage

- `refs/gatos/mbus/<topic>/<shard>` for msgs and commits.
- `refs/gatos/mbus-ack/<topic>/<consumer>` for acknowledgements.

### 2.5 `gatos-proof`

#### Commitment proof generator 

Compute `(inputs_root, output_root, policy_root)` and sign.

ZK proof trait abstraction for pluggable backends.

#### API

```rust
pub trait Prover {
  fn commit(&self, inputs_root: [u8;32], output_root: [u8;32], policy_root: [u8;32]) -> Proof;
  fn verify(&self, proof: &Proof) -> bool;
}
```

### 2.6 `gatosd`

JSONL RPC server over stdin/stdout and optional TCP.

Command router for: 
- `append_event`, 
- `fold_state`,
- `policy.check`, 
- `bus.publish`, 
- `bus.subscribe`, 
- `session.*`, 
- `put_blob`, 
- `prove`, 
- `verify`, 
- `epoch new`, 
- `doctor`.

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

- Batch multiple events into a single tree and one commit where possible.
- Prefer manifest+chunking over Git‑LFS.
- Tune pack GC with size thresholds; 
- document recommended `gc.auto`, `fetch.writeCommitGraph`.
