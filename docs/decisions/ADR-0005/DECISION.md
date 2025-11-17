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

### Decision

1. **Shiplog namespaces**

refs/gatos/shiplog//head     # commit parent-chain per topic
refs/gatos/consumers//  # checkpoints (by ULID)

2. **Event envelope (normative)**  
Canonical JSON with a ULID and canonical `content_id`:

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
- Checkpoint value is the last processed `ulid` (and optionally commit).

5. **Queries**
- `gatos-mind` MUST support `shiplog.read(topic, since_ulid, limit)` returning canonical envelopes and commit ids.
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