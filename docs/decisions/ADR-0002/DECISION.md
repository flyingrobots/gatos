
## ADR-0002: Distributed Compute via a Job Plane

- **Status:** Accepted
- **Date:** 2025-11-08

### Scope

This ADR defines a system within GATOS for scheduling, executing, and recording the results of distributed, asynchronous jobs. This decision introduces the **Job Plane** and its associated Git namespaces and protocols.

### Rationale

**Problem:** GATOS can track and govern state, but cannot currently orchestrate or react to state changes with computation.

**Context:** To fulfill the vision of “Git as an Operating Surface”, computation must be a native citizen. Commits as “speech-acts” (the original metaphor) become literal when a commit can trigger a verifiable job.

### Decision

1. A new **Job Plane** **MUST** be introduced to the GATOS architecture.
2. The `refs/gatos/jobs/` namespace is reserved for this plane.
3. When a **Job** commit is created, a corresponding message **MUST** be published to a topic on the Message Plane (e.g., `gatos/jobs/pending`) for discovery by workers.
4. The job lifecycle **MUST** be represented entirely through Git objects:
   - **Job:** A commit whose tree contains a `job.yaml` manifest. The manifest **MUST** include `command`, `args`, and `timeout` fields, and **SHOULD** include `policy_root` and an `inputs` array for deterministic attestation.
   - **Claim:** A ref under `refs/gatos/jobs/<job-id>/claims/<worker-id>`. This ref **MUST** be created atomically (compare-and-swap) to prevent race conditions.
   - **Result:** A commit referencing the original job commit, containing output artifacts (as pointers) and a `Proof-Of-Execution`.
5. The **Proof-Of-Execution** **MUST** sign the job’s `content_id` and **MAY** include an attestation envelope with hashes of the runner binary and environment.
6. Each `Result` commit **MUST** include trailers for discoverability:
   - `Job-Id: <blake3:…>`
   - `Proof-Of-Execution: <blake3:…>`
   - `Worker-Id: <pubkey>`
   - `Attest-Program: <hash-of-runner-binary>` (optional)
   - `Attest-Sig: <signature>` (optional)

#### Canonical Job Identifier

The canonical job identifier is the job’s `content_id` (the BLAKE3 hash of the canonical serialization of the unsigned job core). All protocol elements that refer to a job MUST use this `job-id`.

- Claim refs MUST be named `refs/gatos/jobs/<job-id>/claims/<worker-id>`.
- Result trailers MUST use `Job-Id: <blake3:…>` corresponding to the same `job-id`.

ULIDs MAY be used as human-friendly aliases in messages (for deduplication, sorting, and UX). When present, the ULID MUST also be recorded in the job manifest. Workers MUST resolve ULIDs to the canonical `job-id` by reading the job commit and computing its `content_id`. ULIDs MUST NOT be used as ref keys for claims or results.

### Diagrams

#### Job Lifecycle

This diagram shows the standard lifecycle states for a job as it moves through the system.

```mermaid
stateDiagram-v2
    [*] --> pending
    pending --> claimed: Worker discovers & claims job
    claimed --> running: Worker begins execution
    running --> succeeded: Execution successful
    running --> failed: Execution fails
    succeeded --> [*]
    failed --> [*]
    claimed --> aborted: Canceled by user/policy
    pending --> aborted: Canceled by user/policy
```

#### Job Discovery and Execution Flow

This sequence shows how the different GATOS planes interact to schedule and execute a job.

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
    Worker->>GATOS (Ledger): 5. Atomically create Claim ref (by job-id)
    GATOS (Ledger)-->>Worker: 6. Claim successful
    Worker->>Worker: 7. Execute Job
    Worker->>GATOS (Ledger): 8. Create Result commit (with Job-Id trailer)
```

### Consequences

### Pros

- Makes GATOS an active system capable of executing work deterministically.
- Enables fully auditable automation workflows (“on state change, run test job”).
- Preserves Git’s distributed, offline semantics for job distribution and result collection.

### Cons

- Increases complexity; requires new runner/worker components to be built.
- Adds storage overhead for job logs and artifacts.

### Alternatives Considered

1. **External CI/CD Systems** — Rejected: breaks the self-contained, Git-native model.
2. **Webhooks** — Rejected: less reliable and less auditable than Git-tracked claims/results.

