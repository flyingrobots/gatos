---
Status: Draft
Date: 2025-11-09
ADR: ADR-0009
Authors: [flyingrobots]
Requires: [ADR-0001, ADR-0005]
Related: [ADR-0007, ADR-0008]
Tags: [API, WebSocket, Streaming, Refs]
Schemas:
  - schemas/v1/api/stream_frame.schema.json
---

# ADR-0009: Real-Time Streams & Ref Subscriptions

## Scope
Provide **WebSocket** streams for ref updates and bus topics to enable reactive UIs and workers.

## Rationale
UIs and workers need near-real-time updates without wasteful polling.

## Decision
1. **Endpoint**: `GET /api/v1/stream` → WebSocket (JSON frames).
2. **Subscribe/Unsubscribe** frames:
   ```json
   {"op":"sub","refs":["refs/gatos/state/public/**","refs/mind/sessions/main"],"topics":["gatos.jobs.*"]}
   {"op":"unsub","refs":[...],"topics":[...]}
   ```
3. **Server → client frames**:
   ```json
   { "kind":"ref.update","ref":"refs/...","old":"<sha>","new":"<sha>","seq":123,"ts":"<iso8601>" }
   { "kind":"bus.event","topic":"gatos.jobs.pending","payload":{...},"seq":124,"ts":"<iso8601>" }
   ```
4. **Delivery**: At-least-once with monotonic `seq` per connection; clients MUST dedupe.
5. **Replay**: Optional `sinceSeq` on connect to catch recent history (bounded window).
6. **AuthZ**: Same policy filters as GraphQL; forbidden refs are not emitted.
7. **Heartbeat**: `{"kind":"ping"}` / `{"kind":"pong"}` every 30s.

## Consequences
- Reactive UX and workers with minimal glue.
- Requires sequence indexing on the server side.

## Open Questions
- Cross-node streaming for federation (see ADR-0012) — do we bridge or require local subscription?
