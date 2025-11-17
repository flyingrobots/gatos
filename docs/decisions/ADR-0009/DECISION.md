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
1. **Endpoint**: `GET /api/v1/stream` → WebSocket (JSON frames defined in `schemas/v1/api/stream_frame.schema.json`).
2. **Subscribe/Unsubscribe** frames:
   ```json
   {"op":"sub","refs":["refs/gatos/state/public/**"],"topics":["gatos.jobs.*"],"sinceSeq":1000}
   {"op":"unsub","refs":[...],"topics":[...]}
   ```
   - `sinceSeq` replays missed frames within the last 10 minutes (server clamps).
   - Subscriptions are additive and capped at 20 refs + 20 topics per connection.
3. **Server → client frames** (examples):
   ```json
   { "kind":"ref.update","ref":"refs/gatos/state/public/ui/main","old":"<sha>","new":"<sha>","seq":123,"ts":"<iso8601>" }
   { "kind":"bus.event","topic":"gatos.jobs.pending","payload":{...},"seq":124,"ts":"<iso8601>" }
   { "kind":"error","code":"POLICY_DENIED","message":"..." }
   ```
4. **Delivery**: At-least-once with monotonic `seq` per connection; clients MUST dedupe.
5. **Replay**: `sinceSeq` replays buffered frames up to `STREAM_REPLAY_LIMIT` (default 1,000 frames or 10 minutes, whichever comes first). Requests beyond the window respond with `error` frame `code="REPLAY_EXPIRED"` and start streaming live.
6. **AuthZ**: Same policy filters as GraphQL. Forbidden refs are silently dropped and an `error` frame with `code="POLICY_DENIED"` is emitted once per ref/topic to inform the client.
7. **Heartbeat & Backpressure**: Server sends `ping` every 30s; clients MUST reply within 10s or the connection is closed. Frames include a `credit` field when the server asks clients to apply backpressure (default window 1,000 outstanding frames).

8. **Errors & Close Codes**: Protocol errors result in immediate close with WebSocket code `1008`. The final frame MAY include `{kind:"error", code:"INVALID_SUB", message:"..."}`.

```mermaid
sequenceDiagram
    participant Client
    participant Stream as /api/v1/stream
    participant Refs as Ref Watcher
    participant Bus as Message Plane

    Client->>Stream: GET + WebSocket upgrade
    Stream-->>Client: {kind:"ack"}
    Client->>Stream: {op:"sub", refs:[...], topics:[...]}
    Stream->>Refs: register ref filters
    Stream->>Bus: register topic filters
    Refs-->>Stream: ref.update events
    Bus-->>Stream: bus.event frames
    Stream-->>Client: frames with seq IDs
    Stream-->>Client: {kind:"ping"}
    Client-->>Stream: {kind:"pong"}
```

## Consequences
- Reactive UX and workers with minimal glue.
- Requires sequence indexing on the server side.

## Open Questions
- Cross-node streaming for federation (see ADR-0012) — do we bridge or require local subscription?
