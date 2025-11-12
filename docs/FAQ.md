# FAQ

Yes. Holy shit indeed.

Let’s lock it in with clear answers to your questions, call out any missing pieces, and tighten a few bolts so the implementation path is zero-ambiguity.

---

Direct answers to your 5 previous questions:

## 1) Fold compilation — interpreted YAML or compiled?

**Decision**: Compile the fold spec to a deterministic bytecode (“RMG-IR”), then run it on the Echo engine.

**Source**: `folds/*.yaml` remains the human-authored DSL (stable, declarative).

**Compiler output**: `folds/*.rgf` (RMG Fold) canonical CBOR/MsgPack; `hash = fold_root` (e.g., sha256:…).

**Runtime**: Echo’s fold engine executes the compiled IR. No dynamic I/O. No wall-clock. Pure.

**Binding**: Every checkpoint and decision should record both policy_root and fold_root. If either changes, audits are explicit.

Example (new fields in spec):

```yaml
# folds/invoices.yaml (source)
version: 1
inputs: ["gatos/journal/finance/**"]
reducers:
  - kind: map-join-lww
    key: "$.id"
emit:
  - path: "state/finance/invoices.json"
```

Compiler produces:

```bash
folds/invoices.rgf     # compiled bytecode
folds/invoices.rgf.sha # contains fold_root = sha256:<hex>
```

Checkpoints carry:

```json
{
  "state_root":"blake3:…",
  "inputs_root":"blake3:…",
  "policy_root":"sha256:…",
  "fold_root":"sha256:…"     // NEW
}
```

Why compiled?

- Determinism + speed + auditability.
- YAML stays UX;
- RGF is law.

---

## 2) Echo integration — do DPO rules become merge strategy for sessions?

**Yes**. Session merges must be governed by the fold’s declared join lattice (for counters, sets, maps) and DPO rewrites (for typed graphs).

Per-fold merge contract:

- **Scalars**: define lww or min/max explicitly.
- **Sets**: OR-set or 2P-set (declared).
- **Counters**: G-counter/PN-counter.
- **Graphs**: DPO with canonical match order (Echo provides).
- **Conflict classes**: anything not covered by a lattice or DPO rule becomes a governance conflict (record an event: `governance.conflict` with precise paths and rule ids).

This makes gatos session merge a deterministic math operation, not a string diff.

---

## 3) Proof system — ZK choice?

### Phase plan

#### v1 (day-one)

- Commitment proofs (cheap, universal).
- We record `inputs_root`, `output_root`, `policy_root`, `fold_root` and a `signature → proof.fairness`.
- Verifier recomputes the public roots and signature.
  - No heavy crypto needed.

#### v2 (optional, pluggable)

- ZK proofs for selected folds.

### Trait

Prover/Verifier with backends (Plonky2/Halo2/Risc0/STARKy). We don’t hard-bind to a library.

Start with small circuits (map-join-lww, counters, set membership) to prove no forbidden branch was taken.

Support proof aggregation (multi-step chain → one proof) later.

### Spec tweak

Add `fold_root` to proof envelope and make proof field type-tagged:

```json
{ "proof": { "kind":"commitment", "bytes":"..." } }
```

---

## 4) Message bus scaling — changing shard count

Use a versioned shard map + dual-write migration.

- Store topic config at `refs/gatos/mbus-config/<topic>.json`:

```json
{
  "version": 3,
  "shards": 64,
  "hash": "blake3",
  "strategy": "consistent",
  "from_version": 2,
  "migrate": "dual-write"
}
```

Consistent hashing keeps most keys stable when shards changes.

Dual-write window:

- Publishers write to both old and new shard maps for a configurable epoch.
- Consumers subscribe to both maps; dedupe by (topic, ulid).

When ack lag on the old map is zero for N minutes, flip the active version and retire the old.

This gives smooth resharding with exactly-once semantics intact.

---

## 5) Policy language (`.rgs`) — what is it really?

Small, pure rule DSL inspired by Datalog/Rego, compiled to `.rgc` bytecode.

### Features

- Booleans,
- numeric/string ops,
- set membership,
- label checks,
- actor/trust lookups.
- No loops,
- no I/O,
- no time.
- Strictly total functions.
  
### Shape

```yaml
when: { intent: "write", path: "gatos/journal/finance/**" }
allow_if:
    - "caps includes 'journal:append'"
    - "trust[actor].level >= 2"
deny_if:
    - "labels contains 'private' and remote != 'internal'"
audit: true
```

### Compilation

`.rgs` → `.rgc` (CBOR), policy_root = SHA-256 of `.rgc`.

### Library

Standard predicates:

- includes,
- contains,
- in_cidr,
- has_cap,
- before_epoch,
- etc.
  
### Explainability

- Compiler annotates rule ids;
- gate returns `Deny(rule_id, reason)` deterministically.

---

## Obvious gaps we should close (spec addenda)

### 1. **Bind fold version**

We added `fold_root` above — make it mandatory anywhere `policy_root` is mandatory.

### 2. **Canonical JSON rules**

- Call out key sorting,
- UTF-8 normalization,
- and number encoding explicitly (`u64`/`s128`/`fixed-point`) so hashes are cross-platform bit-exact.

### 3. Error code taxonomy (for JSONL)

- `POLICY_DENY`,
- `CAP_EXPIRED`,
- `FF_VIOLATION`,
- `ACK_TIMEOUT`,
- `DUP_COMMIT`,
- `EPOCH_BROKEN`,
- `PROOF_INVALID`.

### 4. Resource URIs

- Standardize `gatos://<repo>/<ns>/<path>` as the resource field everywhere.

### 5. Idempotency

- Require ulid stability + idempotency keys for exec/bus intents;
- Deny repeats unless allowed by QoS.
  
### 6. Key rotation

Grant chain fields (prev, revokes) and a rotation checklist in spec.

---

> [!faq] “Are we missing any big pieces?” (Short list)

**Docs for profiles** (`local` / `push-gate` / `SaaS`) with defaults (who enforces what, where).

**Doctor + Metrics**: implementation **MUST** ship `/healthz`, `/readyz`, `/metrics` and `gatos doctor`. (We wrote it in `TECH-SPEC`; surface it in `SPEC`’s normative section with metric names.)

**KV & Graph facades**: optional subcommands that expose familiar semantics on top of journals/folds (gatos kv, gatos graph query). Makes first-contact easier.

**Resilience helpers**: an out-of-the-box repair tool to re-stitch epochs, roll caches, and heal broken refs.

---

## Top 5 use-cases (and if we meet them)

### 1.Regulated config / feature flags

- Needs: signed appends, RLS, RYW, bounded history.
- We meet: journals + policy gate + epochs + (optional) push-gate.
- Add: canned policy templates + gatos doctor for GC/epoch sanity → ✅ production-grade.

### 2.Supply-chain / deploy attestation

- Needs: signed events, multi-sig policy changes, human/JSON logs, offline verify.
- We meet: Shiplog DNA + proof envelopes v1.
- Add: “evidence pack” command that bundles logs + proof → ✅ audit-ready.

### 3.Air-gapped ML registry

- Needs: huge blobs, provenance, selective export, encrypted storage.
- We meet: opaque pointers + CAS + epochs.
- Add: rekey tool + export policies → ✅.

### 4.LLM multi-agent orchestration

- Needs: pub/sub, exactly-once, backpressure, capability tokens.
- We meet: bus QoS + caps + acks/commit.
- Add: shard-map/versioning + subscription windows → ✅ at scale.

### 5.Cross-app data sharing (RLS-gated state)

- Needs: stable materialized views, policy-consistent reads across repos.
- We meet: refs/gatos/state/** as public contract + shared policy bundles.
- Add: Wesley target that emits both fold + RLS bundle → ✅ smooth.

---

## Compare & contrast (adjacent tech)

### Kafka / EventStoreDB — event logs at scale

- not offline-first;
- no built-in deterministic fold proofs;
- no Git audit trail.
  
#### GATOS

- lower TPS,
- higher assurance,
- portable,
- auditable,
- Git-native.

### Dolt / LakeFS  — data versioning with Git-like semantics

#### GATOS

- uses Git itself as database + runtime + policy plane;
- no heavyweight DB server.

### DVC / Pachyderm — ML artifact + pipeline versioning

#### GATOS

- adds policy,
- proofs,
- and deterministic compute on the same DAG;
- no external control plane.

## Nix/Guix — reproducible builds

**GATOS**:

- generalizes reproducibility to runtime state, messages, and governance.

## Blockchains — global consensus

- huge cost.

**GATOS**:

- local determinism + cryptographic attestations,
- no global chain,
- no miners.

---

## New features this meditation suggests

1. `fold_root` binding across checkpoints and proofs (spec update).
2. Versioned shard maps and dual-write migration for the bus.
3. Proof envelopes v1 as a hard requirement for “privileged” folds; ZK optional.
4. Wesley→GATOS target to generate `*.rgf` and `.rgc` bundles, plus schema manifests.
5. KV and Graph facades as on-ramps: `gatos kv`, `gatos graph`.
6. Policy templates: `gatos policy init --template minimal|regulated|airgapped`.
7. Doctor is mandatory (not “nice-to-have”).

---

## Concrete next steps

- **Spec delta**:
  - add `fold_root` everywhere `policy_root` appears;
  - define canonical JSON rules;
  - add error codes.
- **Compiler**:
  - build `rgf` bytecode generator (YAML → Echo IR → CBOR).
- **Gate**:
  - finalize `.rgs` grammar + deterministic interpreter;
  - emit rule ids in Deny.
- **Bus**:
  - `mbus-config/<topic>.json` with versioned shard maps + dual-write migration.
- **Proofs**:
  - implement commitment proofs today;
  - leave ZK behind a trait.
- **CLI**:
  - `gatos doctor`,
  - `gatos epoch new`,
  - `gatos prove/verify`,
  - `gatos bus {publish,subscribe}`.

Ship those, and this stops being “just” a gorgeous spec and becomes an operating surface people can run.

---

> [!faq] Could the deterministic byte code just be Lua running on an echo-Lua VM w/echo's deterministic floats etc?

**Short answer**: Yes — with constraints.
You can absolutely make the “deterministic bytecode” be Lua on an Echo-Lua VM if you run a restricted, deterministic profile and you do not rely on stock Lua bytecode for portability. Treat Lua as the authoring language, compile it to a portable, canonical IR, and execute it on a VM that enforces Echo’s determinism rules.

Below is the concrete plan.

---

### ✅ What works (and what does not)

Use Lua as the source, not the on-disk bytecode

- Do: Parse Lua source → canonical AST → Echo Lua IR (call it ELC), serialize to CBOR/MsgPack, and hash that (`fold_root`).
- Don’t: Ship stock Lua precompiled chunks (luac). They’re not portable across Lua versions/architectures (endianness, ints/floats, build flags). We need a portable, stable compiled form.

### Deterministic runtime profile (“EchoLua”)

You’ll run the IR on a small VM with a deterministic standard library and a pure execution model:

| Area | Default Lua | EchoLua profile (deterministic) |
| :--- | :--- | :--- |
| **Time/OS** | os.clock, os.time, io.* | Removed (forbidden) |
| **Random** | math.random (MT) | Replaced with deterministic PRNG (e.g., PCG/Xoroshiro) seeded from {inputs_root, policy_root, fold_root} |
| **Floats** | IEEE-754 via host FPU | Software float or fixed-point with defined rounding + NaN canonicalization (or integer/fixed-point only) |
| **Iteration order** | pairs() order unspecified | pairs shadowed by dpairs() that sorts keys deterministically; ipairs allowed for arrays |
| **Tables & hashing** | Hash seed randomized per process | Fixed hash seed inside VM; but you still must not rely on hash order |
| **Coroutines** | Arbitrary yields | Disallowed (folds are pure; no scheduler) |
| **Metamethods (__gc)** | Finalizer order GC-dependent | Disallowed; GC visible effects forbidden |
| **FFI/dynamic load** | possible via add-ons | Forbidden |
| **Math (exp/log/sin)** | Host-lib accuracy varies | Deterministic math lib (CR-libm-style or pure integer/decimal path) |

**TL;DR**: fold execution must be pure, total, side-effect free. Same inputs ⇒ same bytes.

---

## 🎯 Architecture

### 1) Compilation pipeline

```text
Lua source  ──parse/normalize──► AST ──lower──► ELC (Echo Lua IR)
                                         │
                                         └─► CBOR bytes  (hash = fold_root)
```

- **Normalize**: remove syntactic sugar, canonicalize constant folding, resolve upvalues.
- **Lowering**: emit a small, explicitly typed IR (ops like `map_join`, `reduce`, `emit_json`, `cmp_sort`, etc.) + a minimal VM op set (LOADK, GET, SET, CALL, RET, …).
- **Hash**: `fold_root = sha256(ELC_bytes)`. Record alongside `policy_root` anywhere `state_root` is recorded.

### 2) Execution

The EchoLua VM interprets ELC with:

- A pure deterministic math layer (either fixed-point or software floats; pick one and lock rounding mode).
- Canonical JSON emission (UTF-8 normalized, sorted keys, fixed number encoding).
- Deterministic PRNG only if you explicitly allow it in a fold (most folds shouldn’t use it).

### 3) Standard library (deterministic subset)

- `table`: `dkeys`, `dvalues`, `dsort`, `dpairs(t)` (sorted iteration).
- `json`: `encode_canonical`, `decode_strict`.
- `math`: `add`/`sub`/`mul`/`div` (deterministic), optional `exp`/`log`/`sin`/`cos` via a fixed, correctly rounded library. If you can, prefer fixed-point/integers in folds for simplicity and speed.
- `set`, `counter`: OR-Set/2P-Set primitives; G/PN counters; deterministic lattice joins.
- No debug, package, io, os.

---

## 🧪 Determinism hazards & how we neutralize them

### 1.Floating point drift

- Use software float (e.g., SoftFloat-style) or fixed-point (e.g., Q32.32) for all math in folds.
- Canonicalize NaNs and rounding to ties-to-even.
- If you need transcendental functions, ship a deterministic math lib (CR-libm-like) and pin versions.

### 2.Table iteration order

- Forbid raw pairs; replace with dpairs that sorts keys by canonical comparator.
- Lint/compile error on pairs/metamethod __pairs.

### 3.Randomness/time

- Remove math.random, os.time. Provide rng() that returns a stream seeded from {inputs_root, policy_root, fold_root} and document it as discouraged.

### 4.GC/finalizers

- Disallow __gc metatables; VM forbids finalizers during fold execution.

### 5.Bytecode portability

- Never ship stock Lua bytecode. Only ship ELC (your portable IR) with a stable encoder.

### 6.String hashing / locales

- VM sets fixed hash seed internally; string compare uses pure bytewise lexicographic (UTF-8), locale-independent.

---

### Where Lua shines here

- **Developer experience**: great; tiny language, loved by game engines, easy to sandbox.
- **Embedding**: trivial in Rust/C; small binary; fast interpretive performance for control flow.
- **Safety**: easy to freeze the global env and hand a tiny stdlib.

---

## Spec deltas to support EchoLua

### Add these to `SPEC/TECH-SPEC`

1.Fold compilation outputs

- fold_root (SHA-256 of ELC bytes) MUST be recorded anywhere policy_root is recorded (events, checkpoints, proofs).

2.Canonical JSON rules

- Keys sorted lexicographically; UTF-8 NFC; numeric encoding fixed (decimals or integers only); no trailing zeros; set a single representation for -0.

3.Deterministic VM profile

- Define the forbidden modules (io, os, debug, package) and replaced functions (pairs→dpairs, math.random→rng), plus the float/fixed-point policy.

4.Linter rules (build-time)

- Hard fail on: pairs, coroutines, metamethods __gc/__pairs, any import outside allowed stdlib, and any non-canonical numeric literal.

5.Proof envelopes

- Include fold_root in the proof.fairness envelope alongside {inputs_root, output_root, policy_root}.

---

## Minimal example

### folds/invoices.lua (authoring)

```lua
-- Pure fold: Last-writer-wins by invoice id, deterministic order.
function fold(events)
  local by_id = {}
  for _, e in dpairs(events) do              -- dpairs sorts by ULID
    if e.type == "intent.exec.insert_invoice" then
      local id = e.payload.id
      by_id[id] = { id=id, amount=e.payload.amount, status="draft", ts=e.ulid }
    elseif e.type == "intent.exec.approve_invoice" then
      local id = e.payload.id
      if by_id[id] then by_id[id].status = "approved"; by_id[id].ts = e.ulid end
    end
  end
  return emit_json_canonical("state/finance/invoices.json", by_id)
end
```

### Build

```bash
gatos foldc folds/invoices.lua -o folds/invoices.rgf
# outputs fold_root = sha256:...
```

### Checkpoint (conceptual)

```json
{
  "state_root":"blake3:…",
  "inputs_root":"blake3:…",
  "policy_root":"sha256:…",
  "fold_root":"sha256:…"
}
```

---

## Performance notes

- Interpreted Lua is plenty for control + small reductions;
- heavy math should be in Echo’s native reducers (map-joins, counters, OR-sets) callable from Lua as intrinsics.
- Keep the VM single-threaded per fold;
- parallelize across partitions upstream (multiple sessions/namespaces) to preserve determinism without scheduler complexity.

---

## Test matrix (lock this down)

- Golden vectors across: Linux (glibc & musl), macOS ARM, Windows, WASM.
- Folds using integers, fixed-point, (optional) software floats with transcendental functions.
- Lints catching pairs, time/random, coroutines.
- Poison tests: NaN propagation, signed zero, hash collisions, iteration order.

---

## Bottom line

Yes: make the deterministic “bytecode” Lua-authored, Echo-compiled, and VM-enforced.
No: don’t trust raw Lua bytecode or the stock standard library.

Do it as EchoLua:

- Lua for ergonomics,
- Echo for determinism,
- ELC for portable compiled form,
- A tiny, pure stdlib,
- And a linter that turns foot-guns into build errors.

We get a friendly developer experience and the bit-for-bit guarantees GATOS demands.
