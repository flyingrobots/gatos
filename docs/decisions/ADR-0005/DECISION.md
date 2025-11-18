---
Status: Proposed
Date: 2025-11-09
ADR: ADR-0005
Authors: [flyingrobots]
Requires: [ADR-0001]
Related: [ADR-0002, ADR-0003]
Tags: [Message Plane, Message Bus, Consumers]
Schemas:
  - ../../../../schemas/v1/message-plane/event_envelope.schema.json
  - ../../../../schemas/v1/message-plane/consumer_checkpoint.schema.json
Supersedes: []
Superseded-By: []
---

## ADR-0005: Message Plane — A Git-Native, Commit-Backed Message Bus

### Scope

Introduce a **first-class, append-only Message Plane** (commit-backed message bus) that runs in parallel with snapshot state folds. Provide queryability, consumer checkpoints, and causal ordering for integrations.

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

*   **3. No Message Plane (Consumers Parse Git History):**
    *   **Description:** Do not create a dedicated Message Plane. Require consumers to parse the entire Git history of the main ledger to extract the events they need.
    *   **Reason for Rejection:** This would be highly inefficient and complex for consumers. It would require each consumer to implement its own logic for traversing the Git history, filtering commits, and managing its own state. A dedicated Message Plane provides a much cleaner and more efficient integration surface.

### Decision

1. **Message Plane namespaces**

Each message corresponds to a single Git commit. Topics are organized using the following ref structure:

refs/gatos/messages/<topic>/head     # commit parent-chain per topic
refs/gatos/consumers/<group>/<topic>  # checkpoints (by ULID)

2. **Message envelope & commit layout (normative)**  
Each message commit MUST contain the envelope blob at `message/envelope.json` with no additional top-level files. Optional attachments MUST reside under `message/attachments/` and be referenced inside the envelope `refs` map.

- `message/envelope.json` MUST be Canonical JSON (UTF-8, sorted keys, no insignificant whitespace) conforming to [`schemas/v1/message-plane/event_envelope.schema.json`](../../../../schemas/v1/message-plane/event_envelope.schema.json).
- Attachments MUST NOT influence the canonical identifier; only the envelope bytes are hashed.
- Clients MAY include detached metadata (e.g., transport headers), but the canonical commit identifiers are derived solely from the envelope blob.

The `content_id` is the `blake3` hash of the canonical envelope bytes.

Example envelope payload:

{
"ulid": "<26-char ULID>",
"ns": "",              # e.g., "governance"
"type": "<event.type>",
"payload": { ... },               # canonical JSON
"refs": { "state": "blake3:...", "proposal_id": "blake3:..." }  # OPTIONAL cross-refs
}

Each Message Plane commit message MUST include:

Event-Id: ulid:
Content-Id: blake3:

3. **Ordering**
- Per‑topic order is the Git parent chain order; ULIDs MUST be strictly monotonic per topic on a single node.
- Cross-topic causality is not guaranteed; consumers can join via `refs`.

4. **Consumers**
- Consumers store per‑topic checkpoints under `refs/gatos/consumers/<group>/<topic>`.
- Checkpoint value is the last processed `ulid` (and optionally commit). Storing the commit hash allows for faster lookups and can help resolve ordering if ULIDs are not strictly monotonic across distributed nodes.

5. **Queries**
- `gatos-message-plane` MUST support `messages.read(topic, since_ulid, limit)` returning canonical envelopes and commit ids, ordered by the Git parent chain (oldest to newest). If `since_ulid` is not found, the stream SHOULD start from the beginning of the topic.
- Bus bridge MAY mirror Message Plane topics onto external brokers (configurable).

`messages.read` contract (normative):

- **Request:**
  - `topic` — string. Required. Matches `<topic>` portion of `refs/gatos/messages/<topic>/head`.
  - `since_ulid` — optional ULID string. When absent, start from the oldest message.
  - `limit` — integer 1–512 (inclusive). Servers MUST clamp >512 to 512.
- **Response:** JSON object with `messages: []` ordered oldest→newest. Each entry MUST include:
  - `ulid` (string) — envelope ULID.
  - `commit` (string) — Git OID of the message commit.
  - `content_id` (string) — `blake3:<hex>` digest of the envelope bytes.
  - `envelope_path` (string) — repository-relative path to `message/envelope.json` (default `message/envelope.json`).
  - `canonical_json` (string) — base64-encoded canonical JSON bytes for clients that cannot read from Git directly.
  - `checkpoint_hint` (object) — `{ "group": <consumer-group>, "topic": <topic> }` when the server auto-advances checkpoints; MAY be `null` otherwise.
- **Errors:**
  - `404 topic_not_found` when the requested ref does not exist.
  - `400 invalid_ulid` when `since_ulid` is malformed.
  - `409 range_exceeded` when `limit < 1`.

Servers SHOULD include the newest `ulid` in the response metadata (`next_since`) so clients can resume without re-reading.

6. **Interaction with Ledger**
- Ledger events MAY be mirrored into the Message Plane automatically.
- Governance transitions (ADR‑0003) SHOULD emit Message Plane events in the `governance` topic.

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

    subgraph Message Plane (topic: governance)
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
