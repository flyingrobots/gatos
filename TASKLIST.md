# Task Backlog

- [x] **ADR Coverage: Sessions & PoX**
  - *Summary*: Author ADRs that define the Sessions feature set and the Proof-of-Experiment workflow so roadmap items have specs.
  - *Problem Statement*: `docs/TASKS.md` references Sessions without any ADR, and PoX (M7) lacks a normative document; engineers cannot implement without an agreed contract.
  - *Acceptance Criteria*: ✅ ADR-0015 (Sessions) and ADR-0016 (PoX) exist with diagrams + consequences; roadmap/task references updated.
  - *Test Plan*: ✅ Markdown lint + `rg` show new ADR ids in README/ROADMAP.
  - *LLM Prompt*: “You are drafting an Architecture Decision Record. Produce an ADR that specifies the GATOS Sessions feature (start/undo/fork/merge with lattice/DPO joins) aligned with existing policy/state planes, including decision, diagrams, and consequences.”

- [ ] **Message Plane Implementation**
  - *Summary*: Build the segmented Git-backed pub/sub system defined in ADR-0005 so M3 can move from Proposed to Accepted.
  - *Problem Statement*: ADR-0005 is still Proposed; without actual crates and tests, Job Plane and downstream integrations are blocked.
  - *Acceptance Criteria*: (1) `gatos-message-plane` exposes publish/read/checkpoint APIs writing to `refs/gatos/messages/**` with rotation + TTL summaries; (2) CLI/RPC `messages.publish/read` implemented in `gatosd`; (3) Consumer checkpoints stored under `refs/gatos/consumers/**`; (4) Integration tests cover publishing, reading from ULIDs, rotation, pruning, and checkpoint persistence; (5) ADR-0005 status updated to Accepted with notes; (6) SPEC/TECH-SPEC reference the shipped implementation.
  - *Test Plan*: Rust unit tests for envelope canonicalization + ULID validation; git-based integration tests that publish messages, enforce CAS ordering, verify checkpoint refs, and simulate rotation/pruning; end-to-end tests hitting the JSONL `messages.read` RPC.
  - *LLM Prompt*: “Implement a Git-backed message bus per ADR-0005: create a Rust module that writes message topics under `refs/gatos/messages/<topic>/<date>/<segment-ulid>` with rotation at 100k messages or 192MB and checkpoints under `refs/gatos/consumers/...`. Include daemon RPCs, CLI, and tests for publish, subscribe, rotation, and pruning.”

- [ ] **Job Plane + PoE Integration**
  - *Summary*: Wire ADR-0002’s Job Plane into `gatosd`, enabling CAS claims, worker loops, and Proof-of-Execution commits.
  - *Problem Statement*: The Job Plane is specified but not implemented; no CLI/daemon support exists for enqueuing, claiming, or attesting jobs.
  - *Acceptance Criteria*: (1) CLI commands `git gatos jobs enqueue|ls|watch` exist; (2) Daemon exposes job topics/messages; (3) Workers create `refs/gatos/jobs/<job-id>/claims/<worker>` and result commits with PoE envelopes; (4) Tests prove only one worker can claim a job.
  - *Test Plan*: Integration test with two worker processes racing on the same job; verify PoE signature verification code path; ensure job topics emit events onto Message Plane.
  - *LLM Prompt*: “Extend the GATOS daemon to support ADR-0002: implement job enqueue, claim (CAS ref updates), and result commits with Proof-of-Execution envelopes plus CLI commands to drive the flow.”

- [ ] **Exporter CLI & Explorer-Root Verifier**
  - *Summary*: Finish M6 by building the `gatos export` command and Explorer-Root verification flow spelled out in ADR-0011/docs/exporter.md.
  - *Problem Statement*: Specs describe Parquet/SQLite exports and Explorer-Root, but no code produces or verifies artifacts, leaving analytics teams blocked.
  - *Acceptance Criteria*: (1) CLI supports `gatos export --format {sqlite,parquet}` plus `--since/--until`, column filters, and predicates; (2) Outputs include manifest + explorer-root digest; (3) `gatos export verify <path>` recomputes digests and fails on mismatches; (4) Tests cover pushdown filters and manifest hashing.
  - *Test Plan*: Golden tests exporting a fixture repo; fuzz tests on filter DSL; verification test that tampering data triggers failure.
  - *LLM Prompt*: “Implement the `gatos export` CLI per ADR-0011: emit SQLite/Parquet datasets with explorer-root metadata and add a verification subcommand that recomputes the digest.”

- [ ] **GraphQL Gateway Service**
  - *Summary*: Deliver M6.5 by creating the GraphQL API service described in ADR-0007 (SDL publish, Relay pagination, policy filtering).
  - *Problem Statement*: API contracts exist but no service handles GraphQL queries; consumers can’t query state snapshots without custom tooling.
  - *Acceptance Criteria*: (1) Gateway binary serves `POST /api/v1/graphql` + `GET /api/v1/graphql/schema`; (2) Resolver layer enforces `stateRef/refPath`, pagination caps, error codes; (3) Caching headers and rate limits match ADR-0007; (4) CI runs schema/regression tests.
  - *Test Plan*: GraphQL integration tests for policy-denied fields, pagination bounds, shapeRoot caching; load test verifying rate-limit headers.
  - *LLM Prompt*: “Build a Rust GraphQL gateway matching ADR-0007: expose POST /api/v1/graphql, enforce stateRef/refPath targeting, Relay pagination, OpaquePointerNode handling, caching headers, and rate limiting.”

- [ ] **Federation Stream Proxy ADR & Implementation**
  - *Summary*: Close ADR-0009’s open question by specifying and building the cross-node streaming proxy (fan-out next to `gatos mountd`).
  - *Problem Statement*: Federation currently lacks a story for streaming refs/topics across mounts, limiting multi-node deployments.
  - *Acceptance Criteria*: (1) New ADR details the stream proxy approach, credit windows, and auditing; (2) Implementation ships alongside `gatos mountd`, forwarding streams with deterministic seq IDs; (3) Tests cover replay, credit exhaustion, and failure telemetry.
  - *Test Plan*: Simulate upstream/downstream nodes with network hiccups; ensure audit refs record forwarded frames and that dedupe holds across hops.
  - *LLM Prompt*: “Author an ADR and implementation plan for a federation stream proxy that subscribes locally and replays frames downstream with deterministic sequence IDs and audit logging.”

- [ ] **Operations & Observability Guide Chapter**
  - *Summary*: Add a “Chapter 13: Operations & Observability” to the GATOS book covering SLOs, `/healthz`, watcher logs, and troubleshooting.
  - *Problem Statement*: Operators lack comprehensive guidance even though roadmap milestones (M8/M9) assume operational maturity.
  - *Acceptance Criteria*: (1) New chapter published with sections on daemons, health checks, metrics, and playbooks; (2) Cross-links to ADR-0006, ADR-0009, and exporter specs; (3) README/map-of-contents updated.
  - *Test Plan*: Run markdown lint; verify TOC autogen; spot-check links.
  - *LLM Prompt*: “Write Chapter 13 of the GATOS guide detailing Ops & Observability: cover gatosd health endpoints, metrics, watcher/audit logs, SLOs, and troubleshooting playbooks referencing relevant ADRs.”

- [ ] **Demo Suite (ADR-as-policy, Bisect State, PoX)**
  - *Summary*: Produce runnable demos + media assets once core planes ship, to fulfill M8.
  - *Problem Statement*: README promises demos, but none exist; marketing/onboarding lack concrete examples.
  - *Acceptance Criteria*: (1) Scripts or Make targets run each demo end-to-end; (2) Capture GIFs/screens for README; (3) Document steps in `docs/demos/*.md`.
  - *Test Plan*: CI job that runs demo scripts against a fixture repo; manual QA of media assets.
  - *LLM Prompt*: “Create demo scripts that showcase ADR-as-policy enforcement, state bisection, and PoX reproduction, including documentation and media assets for the README.”

- [ ] **`gatos doctor` Conformance Tooling**
  - *Summary*: Implement the conformance suite envisioned in M9 to automatically vet repos for policy/export/proof invariants.
  - *Problem Statement*: Without `gatos doctor`, operators cannot quickly validate installations, undermining trust.
  - *Acceptance Criteria*: (1) CLI command `gatos doctor` runs a battery of checks (policy FF-only refs, exporter manifests, proof coverage); (2) Reports actionable errors; (3) Tests cover healthy vs failing repos.
  - *Test Plan*: Integration tests against synthetic repos with intentional corruption; verify output codes and messages.
  - *LLM Prompt*: “Implement a `gatos doctor` CLI that validates repo invariants (policy FF-only branches, PoF/PoE coverage, exporter manifests) and reports actionable diagnostics.”

### Message Plane Implementation Checklist
- [ ] Flesh out `gatos-message-plane` core types (TopicRef, MessageEnvelope, MessageRecord, errors) with canonicalization + ULID validation tests.
- [ ] Implement Git publisher module: write `message/envelope.json`, add trailers, manage segment refs and `head` ref with CAS.
- [ ] Add rotation logic (threshold-based segment ULIDs) and summary commits for pruned segments.
- [ ] Implement checkpoint store writing `refs/gatos/consumers/<group>/<topic>` with ULID+commit payloads.
- [ ] Implement subscriber/reader returning canonical JSON + commit ids, honoring `since_ulid` and `limit`.
- [ ] Wire `messages.publish`/`messages.read` RPC handlers into `gatosd` JSONL protocol and add CLI commands.
- [ ] Write integration tests covering publish/read/rotation/checkpoint flows.
- [ ] Implement TTL/pruning task for old segments and checkpoint-aware pruning safety checks.
- [ ] Update docs (SPEC/TECH-SPEC/guide) and flip ADR-0005 to Accepted once tests pass.
