# Message Plane: Publish Path Completion

- **Status:** TODO
- **Area:** gatosd / Message Plane
- **Owner:** Triage
- **Context:** `MessagePlaneService::publish` remains unimplemented. CLI/RPC cannot append messages end-to-end.

## Tasks
- Wrap `GitMessagePublisher` (or ledger-core port) inside `gatosd` service.
- Enforce envelope validation and topic sanitization at the daemon boundary.
- Handle CAS conflicts with retry/backoff; surface `HeadConflict` to clients with actionable error text.
- Add tests covering publish → read → checkpoint flow.

## Definition of Done
- `messages.publish` path works via CLI/RPC, writing commits and updating head/segment refs with CAS.
- Errors mapped to spec codes; retryable conflicts documented.
