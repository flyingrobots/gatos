1. WHAT IS THIS PROJECT?

       Echo is a next-generation game engine and simulation framework built on a novel computational model called Recursive Meta Graph (RMG). Rather than
       traditional object-oriented state machines like Unity or Unreal, Echo treats everything—code, data, assets, and even time itself—as a living graph
       that is transformed deterministically through rewrite rules.

       Vision statement from the README:
       "Echo is a recursive metagraph (RMG) simulation engine that executes and rewrites typed graphs deterministically across branching timelines and
       merges them through confluence."

       Echo enables:
       - Perfect determinism (same input = same output, always)
       - Branching timelines (fork reality, try changes, merge back like Git)
       - Confluence (independent edits converge to same canonical state)
       - Everything-as-graph (introspectable, hot-reloadable substrate)

       ---
       2. OVERALL ARCHITECTURE

       The Echo project is organized as a Rust workspace with multiple specialized crates:

       echo/
       ├── crates/
       │   ├── rmg-core/        (Core engine: 1765 LOC, Rust)
       │   ├── rmg-geom/        (Geometry primitives: AABB, transforms, broad-phase)
       │   ├── rmg-benches/     (Criterion microbenchmarks: snapshot_hash, scheduler_drain)
       │   ├── rmg-wasm/        (WebAssembly bindings for tools and web)
       │   ├── rmg-ffi/         (C ABI for Lua/host integration)
       │   └── rmg-cli/         (Command-line interface, demos launcher)
       ├── docs/                (Comprehensive specifications and diagrams)
       └── scripts/             (Build automation, benchmarking)

       Core Layers (per architecture-outline.md):

       1. ECS (Entity-Component-System): Entities with type-safe components, archetype-based storage
       2. Scheduler: Deterministic DAG ordering with phase-based execution
       3. Codex's Baby: Event bus for command buffering and deterministic event handling
       4. Timeline Tree: Branching/merging with Chronos (sequence), Kairos (possibility), Aion (significance)
       5. Ports & Adapters: Renderer, Input, Physics, Networking, Audio, Persistence
       6. Deterministic Math: Vec3, Mat4, Quat, PRNG with reproducible operations

       ---
       3. KEY ABSTRACTIONS & DESIGN PATTERNS

       A. Recursive Meta Graph (RMG) Core

       The engine operates on typed, directed graphs where:

       - Nodes = typed entities with optional payloads (component data)
       - Edges = typed relationships between nodes
       - Rules = deterministic transformations matching patterns and rewriting subgraphs

       Core identifiers (256-bit BLAKE3 hashes):
       pub type Hash = [u8; 32];
       pub struct NodeId(pub Hash);      // Entity identifiers
       pub struct TypeId(pub Hash);      // Type descriptors
       pub struct EdgeId(pub Hash);      // Directed edge identifiers

       All identifiers are domain-separated with prefixes (b"node:", b"type:", b"edge:") to prevent collisions.

       B. Deterministic Rewriting

       Each tick follows this transaction model:

       1. begin()           → TxId (new transaction)
       2. apply(tx, rule)   → enqueue pending rewrites
       3. commit(tx)        → execute rewrites in deterministic order, emit snapshot

       Rewrite rules contain:
       - Deterministic id (Hash)
       - Pattern descriptor
       - matcher(): returns true if rule matches
       - executor(): applies transformation to graph
       - compute_footprint(): computes read/write sets for independence
       - conflict_policy: Abort/Retry/Join

       C. O(n) Deterministic Scheduler

       Pending rewrites are ordered deterministically using stable radix sort (not comparison-based):

       Ordering: (scope_hash, rule_id, nonce) lexicographically
       Time complexity: O(n) with 20 passes of 16-bit radix digits

       This ensures identical initial state + rule set always produces identical execution order and final snapshot.

       D. Snapshot Hashing (Merkle Commits)

       Two hashes per commit:

       1. state_root: BLAKE3 of canonical graph encoding
         - Nodes in ascending NodeId order
         - Edges per node sorted by EdgeId
         - Fixed-size encoding (little-endian)
         - BFS reachability from root
       2. commit_id: BLAKE3 of commit header including:
         - state_root (32 bytes)
         - Parent hashes (for merge tracking)
         - plan_digest (canonical ready-set ordering)
         - decision_digest (Aion agency inputs)
         - rewrites_digest (applied rewrites)
         - policy_id (version pin)

       E. Footprints & Independence Checks (MWMR)

       For concurrent/parallel rewriting:

       struct Footprint {
           n_read, n_write: IdSet,      // Node reads/writes
           e_read, e_write: IdSet,      // Edge reads/writes
           b_in, b_out: PortSet,        // Boundary port reads/writes
           factor_mask: u64,            // Spatial/subsystem partitioning hint
       }

       Disjoint footprints = independent rewrites = safe parallel execution.

       ---
       4. COMPONENT ORGANIZATION & HOW THEY INTERACT

       rmg-core (Engine Core)

       Main modules:
       - engine_impl.rs: Engine struct, transaction lifecycle, rewrite application
       - scheduler.rs: DeterministicScheduler, O(n) radix drain, conflict detection
       - graph.rs: GraphStore, in-memory BTreeMap-based node/edge storage
       - snapshot.rs: Snapshot hashing, state_root and commit_id computation
       - rule.rs: RewriteRule, pattern matching, footprint computation
       - footprint.rs: Footprint, IdSet, PortSet for independence checks
       - ident.rs: ID construction, domain separation (NodeId, TypeId, EdgeId)
       - math/: Deterministic vector/matrix/quaternion/PRNG operations
       - demo/motion.rs: Example rewrite rule (position += velocity)
       - tx.rs: Transaction identifiers (TxId)
       - record.rs: Graph records (NodeRecord, EdgeRecord)
       - payload.rs: Canonical encoding for motion payload (24-byte position+velocity)

       Key traits/types:
       - MatchFn: fn(&GraphStore, &NodeId) -> bool
       - ExecuteFn: fn(&mut GraphStore, &NodeId)
       - FootprintFn: fn(&GraphStore, &NodeId) -> Footprint

       rmg-geom (Geometry)

       Geometry primitives for spatial queries:
       - AABB (axis-aligned bounding boxes)
       - Transform (3D transforms with determinism)
       - Temporal manifold and tick support
       - Broad-phase spatial indexing (AABBTree)

       rmg-benches (Benchmarks)

       Criterion-based microbenchmarks:
       - snapshot_hash.rs: Hashing performance vs. reachable node count (n=10, 100, 1000)
       - scheduler_drain.rs: Scheduler overhead for n rule applications
       - motion_throughput.rs: Motion rule execution throughput
       - Results visualized in interactive D3 dashboard (docs/benchmarks/)

       rmg-wasm (WebAssembly)

       Bindings via wasm-bindgen:
       - Exposes engine to JavaScript/TypeScript for tools and web
       - Optional console-panic feature for debugging

       rmg-ffi (Foreign Function Interface)

       C ABI bindings for:
       - Lua integration
       - Native host interop
       - Produces cdylib and staticlib artifacts

       rmg-cli (Command-Line)

       Future CLI launcher for:
       - Demos
       - Benchmarks
       - Inspector (planned)

       ---
       5. EXECUTION MODEL

       A. Single-Tick Loop

       loop {
           let tx = engine.begin();

           // Enqueue rewrites (application phase)
           for rule_name in rules_to_apply {
               engine.apply(tx, rule_name, &scope)?;
           }

           // Deterministically execute and snapshot
           let snapshot = engine.commit(tx)?;

           // Emit to tools, networking, etc.
           publish_snapshot(snapshot);
       }

       B. Transaction Lifecycle

       1. begin(): Allocates new TxId, marks as live
       2. apply():
         - Checks if rule matches via matcher()
         - Computes footprint via compute_footprint()
         - Enqueues PendingRewrite in scheduler
       3. commit():
         - Drains pending rewrites in O(n) deterministic order
         - Enforces MWMR via footprint independence checks
         - Executes reserved rewrites sequentially
         - Computes state_root and commit_id
         - Emits immutable Snapshot

       C. Branching & Replay

       - Fork: Capture snapshot hash, start new rewrite sequence (new Kairos branch)
       - Rollback: Load prior snapshot, replay commits deterministically
       - Merge: Three-way merge via Confluence rules (deterministic convergence)

       ---
       6. KEY DESIGN DECISIONS & PATTERNS

       Determinism as Core Principle

       - Every operation must produce identical results given identical input state
       - Domain-separated hashes prevent ID collisions
       - Little-endian encoding is explicit and stable
       - PRNG seeded per branch for reproducible randomness
       - Radix sort (not comparison-based) for deterministic ordering

       Snapshot Isolation, Not Logging

       - State is captured as snapshots (immutable graph hashes)
       - Append-only history is optional (not the primary model)
       - Replays work by loading snapshot + reapplying commits
       - Enables save/load, time-travel debugging, collaborative editing

       Hexagonal Architecture (Ports & Adapters)

       - Core engine never touches DOM, WebGL, timers directly
       - All I/O flows through ports (narrow interfaces)
       - Adapters implement ports (Pixi, WebGPU, SDL, etc.)
       - Easy to swap renderers, physics engines, input sources

       Dependency Injection Pattern

       - Bootstrap pipeline wires ports, services, systems before first tick
       - Supports editor-time hot reload without special cases
       - All subsystems request dependencies through DI container

       Property-Based Testing

       Tests extensively use proptest for:
       - Permutation commutativity (rewrite order independence)
       - Mathematical properties (floating-point round-trip)
       - Independence checks validation
       - Seed pinning for reproducibility

       ---
       7. NOTABLE IMPLEMENTATION DETAILS

       Graph Storage

       pub struct GraphStore {
           pub nodes: BTreeMap<NodeId, NodeRecord>,
           pub edges_from: BTreeMap<NodeId, Vec<EdgeRecord>>,
       }

       - BTreeMap ensures deterministic iteration order
       - Edges stored in insertion order (sorted explicitly for snapshots)
       - Minimal in-memory impl; production will swap in content-addressed store

       Motion Demo

       Canonical 24-byte payload encoding:
       bytes 0..12:   position [x, y, z] as 3 × f32 (little-endian)
       bytes 12..24:  velocity [vx, vy, vz] as 3 × f32 (little-endian)

       The motion_rule reads this payload, applies position += velocity, re-encodes, and updates the node.

       Deterministic Math Module

       All operations use f32 to match runtime float32 mode:
       - Vec3: 3D vectors with normalization, dot product, cross product
       - Mat4: 4x4 matrices for transforms
       - Quat: Quaternions with interpolation
       - Prng: Seeded RNG with deterministic sequences

       Conflict Resolution

       When footprint independence checks fail:
       - Abort: Rewrite is skipped
       - Retry: Rewrite is re-matched against updated state
       - Join: Custom rule-provided merge strategy

       ---
       8. RECENT WORK (Current Branch: echo/pr-12-snapshot-bench)

       The repository is actively working on benchmarking infrastructure:

       - ✅ Snapshot hashing benchmarks with D3 visualization
       - ✅ Scheduler drain throughput measurements
       - ✅ Criterion HTML reports with trend analysis
       - ✅ Makefile targets: make bench-report, make bench-bake
       - 🚧 Performance optimization (radix sort micro-tuning)
       - 📋 Determinism validation test suite

       ---
       9. PROJECT STATUS

       Current Phase: Phase 1 MVP (active development)

       Completed:
       - ✅ Formal proofs of confluence (tick-level determinism proven)
       - ✅ C implementation of independence checks and footprint calculus
       - ✅ 200+ iteration property tests validating commutativity
       - ✅ Rust core runtime with transaction model and scheduler
       - ✅ Comprehensive test suite (18+ test files)
       - ✅ Benchmark infrastructure with live dashboard

       In Progress:
       - 🚧 Performance optimization (subgraph matching, spatial indexing)
       - 🚧 Temporal mechanics (Aion integration)

       Not Started:
       - ❌ Lua scripting integration
       - ❌ Rendering backend (adapters planned)
       - ❌ Full physics engine integration
       - ❌ Inspector tooling

       ---
       10. KEY FILES TO UNDERSTAND THE CODEBASE

       Must-read documentation:
       - /README.md — Project vision and overview
       - /docs/architecture-outline.md — Full system design (storage, scheduler, ports)
       - /docs/rmg-runtime-architecture.md — RMG execution model
       - /docs/spec-rmg-core.md — RMG Core specification v2
       - /docs/spec-merkle-commit.md — Snapshot and commit hash spec
       - /docs/spec-scheduler.md — Deterministic scheduler design

       Core source files:
       - /crates/rmg-core/src/engine_impl.rs — Engine transactions and rewrite execution
       - /crates/rmg-core/src/scheduler.rs — O(n) radix drain scheduler
       - /crates/rmg-core/src/graph.rs — Graph storage
       - /crates/rmg-core/src/snapshot.rs — Snapshot hashing (state_root + commit_id)
       - /crates/rmg-core/src/footprint.rs — Independence checks
       - /crates/rmg-core/src/rule.rs — Rewrite rule definitions
       - /crates/rmg-core/src/demo/motion.rs — Example rewrite rule
       - /crates/rmg-core/src/math/ — Deterministic math primitives

       Tests:
       - /crates/rmg-core/tests/engine_motion_tests.rs — Basic engine workflows
       - /crates/rmg-core/tests/footprint_independence_tests.rs — MWMR validation
       - /crates/rmg-core/tests/permutation_commute_tests.rs — Determinism proofs
       - /crates/rmg-core/tests/math_validation.rs — Floating-point reproducibility

       Benchmarks:
       - /crates/rmg-benches/benches/snapshot_hash.rs — Hashing throughput
       - /crates/rmg-benches/benches/scheduler_drain.rs — Scheduler overhead

       ---
       11. DEVELOPMENT PRACTICES

       Code Quality:
       - Strict compiler flags: -D warnings, -D clippy::all, no unsafe, no panic
       - Automated formatting: cargo fmt --all
       - Linting: cargo clippy --all-targets
       - Testing: cargo test --workspace
       - Documentation: RUSTDOCFLAGS="-D warnings" cargo doc
       - Security: cargo audit --deny warnings, cargo deny check

       Git Workflow:
       - Feature branches: echo/<feature> convention
       - Keep main pristine with pre-commit hooks
       - Expressive commit messages (subject/body/trailer)
       - Determinism is sacred—never use non-deterministic APIs without wrapping

       Testing Discipline:
       - Tests validate behavior, not implementation details
       - Use real databases/storage when possible
       - Property-based testing for mathematical properties
       - Avoid mocking; prefer integration tests

       ---
       CONCLUSION

       Echo is an ambitious, mathematically-grounded game engine that replaces traditional OOP object hierarchies with a deterministic graph rewriting
       model. The current implementation (Phase 1) focuses on:

       1. Proving core RMG concepts with a working Rust engine
       2. Establishing determinism invariants through rigorous testing
       3. Building benchmarking infrastructure to guide optimization
       4. Creating a foundation for adapters (rendering, physics, networking)

       The architecture is clean, the code is well-documented, and the vision is compelling. The project is ideal for exploring:
       - Graph rewriting systems and DPO (Double Pushout) theory
       - Deterministic simulation and replay mechanics
       - Lock-free scheduling with O(n) complexity
       - Content-addressed storage and merkle trees
       - Temporal/branching simulations (Git-like timelines for games)