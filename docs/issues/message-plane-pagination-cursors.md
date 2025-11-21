# Message Plane: Pagination & Cursor Hardening

- **Status:** TODO
- **Area:** Message Plane / RPC UX
- **Owner:** Triage
- **Context:** Subscriber logic handles `since_ulid` and `limit`, but daemon/RPC still needs UX polish and edge-case handling.

## Tasks
- Ensure `messages.read` returns `next_since` in RPC responses and handles unknown `since_ulid` by starting from the oldest segment.
- Enforce limit clamping (1–512) at RPC boundary with clear errors.
- Add tests for empty topics, unknown cursors, and paging across segment boundaries.
- Consider multi-topic/shard fan-in (if/when topics shard) and document behavior.

## Definition of Done
- RPC responses include `next_since` consistently; unknown cursors are backfilled to oldest with no 500s.
- Paging tests pass; CLI/JSONL output matches spec fields.
