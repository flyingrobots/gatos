---
Status: Proposed
Date: 2025-11-09
ADR: ADR-0005
Authors: [flyingrobots]
Requires: [ADR-0001]
Related: [ADR-0002, ADR-0003]
Tags: [Shiplog, Event Stream, Consumers]
Schemas:
  - ../../../../schemas/v1/shiplog/event_envelope.schema.json
  - ../../../../schemas/v1/shiplog/consumer_checkpoint.schema.json
Supersedes: []
Superseded-By: []
---

## ADR-0005: Shiplog — A Parallel, Queryable Event Stream

### Scope

Introduce a **first-class, append-only event stream** ("shiplog") that runs in parallel with snapshot state folds. Provide queryability, consumer checkpoints, and causal ordering for integrations.

### Rationale

Problem: SPEC currently emphasizes state snapshots; many use-cases need a **stream** (integration, analytics, replay to external systems).  
Context: The origin convo proposed a dedicated, queryable append-only log.

### Alternatives Considered

*   **1. Using Git Notes:**
    *   **Description:** Attach event data to existing Git commits using `git notes`.
    *   **Reason for Rejection:** Git notes are not first-class citizens in the Git object model and are not replicated by default. This would make them difficult to query, replicate, and manage, and would not provide a clean, parallel stream of events.

*   **2. External Message Queue (e.g., Kafka, NATS):**
    *   **Description:** Use an external message queue as the primary event stream.
    *   **Reason for Rejection:** This would introduce a significant external dependency, increasing operational complexity and cost. It would also move a critical piece of the system's data model outside of the core Git repository, potentially compromising the project's goal of being self-contained and Git-native.

*   **3. No Shiplog (Consumers Parse Git History):**
    *   **Description:** Do not create a dedicated shiplog. Require consumers to parse the entire Git history of the main ledger to extract the events they need.
    *   **Reason for Rejection:** This would be highly inefficient and complex for consumers. It would require each consumer to implement its own logic for traversing the Git history, filtering commits, and managing its own state. A dedicated shiplog provides a much cleaner and more efficient integration surface.

### Decision

1. **Shiplog namespaces**

Each event in the shiplog corresponds to a single Git commit. The shiplog is organized into topics using the following ref structure:

refs/gatos/shiplog/<topic>/head     # commit parent-chain per topic
refs/gatos/consumers/<group>/<topic>  # checkpoints (by ULID)

2. **Event envelope (normative)**  
Canonical JSON with a ULID and canonical `content_id`. The `content_id` is the `blake3` hash of the canonical JSON of the event envelope itself.

{
“ulid”: “<26-char ULID>”,
“ns”: “”,              # e.g., “governance”
“type”: “<event.type>”,
“payload”: { … },               # canonical JSON
“refs”: { “state”: “blake3:…”, “proposal_id”: “blake3:…” }  # OPTIONAL cross-refs
}

Each shiplog commit message MUST include:

Event-Id: ulid:
Content-Id: blake3:

3. **Ordering**
- Per‑topic order is the Git parent chain order; ULIDs MUST be strictly monotonic per topic on a single node.
- Cross-topic causality is not guaranteed; consumers can join via `refs`.

4. **Consumers**
- Consumers store per‑topic checkpoints under `refs/gatos/consumers/<group>/<topic>`.
- Checkpoint value is the last processed `ulid` (and optionally commit). Storing the commit hash allows for faster lookups and can help resolve ordering if ULIDs are not strictly monotonic across distributed nodes.

5. **Queries**
- `gatos-mind` MUST support `shiplog.read(topic, since_ulid, limit)` returning canonical envelopes and commit ids, ordered by the Git parent chain (oldest to newest). If `since_ulid` is not found, the stream SHOULD start from the beginning of the topic.
- Bus bridge MAY mirror `shiplog` events onto message topics (configurable).

6. **Interaction with Ledger**
- Ledger events MAY be mirrored into shiplog automatically.
- Governance transitions (ADR‑0003) SHOULD emit shiplog events in the `governance` topic.

### Consequences

**Pros**: Clean integration surface; replay; analytics; stable consumer checkpoints.  
**Cons**: More refs to manage; duplication if mirroring ledger events.

### Security Considerations

- Don’t emit private overlay data (see ADR‑0004).  
- Consumers’ checkpoints are not authoritative; they’re advisory markers.

### Diagrams

```mermaid
graph TD
    subgraph Main Ledger
        L1[Commit 1] --> L2[Commit 2]
    end

    subgraph Shiplog (topic: governance)
        S1[Event A<br/>ulid: 01...A] --> S2[Event B<br/>ulid: 01...B]
    end

    subgraph Consumers
        C1[Consumer Group 1<br/>refs/gatos/consumers/group1/governance<br/>Value: 01...B]
        C2[Consumer Group 2<br/>refs/gatos/consumers/group2/governance<br/>Value: 01...A]
    end

    L2 -- "Mirrors event" --> S1
    S2 -- "Processed by" --> C1
    S1 -- "Processed by" --> C2
```