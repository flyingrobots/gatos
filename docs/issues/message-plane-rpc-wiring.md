# Message Plane: JSONL RPC Wiring

- **Status:** TODO
- **Area:** gatosd / Message Plane
- **Owner:** Triage
- **Context:** `gatosd` still lacks a running JSONL server that exposes `messages.read` / `messages.publish`. CLI helpers exist, but the daemon runtime does not serve these methods over stdio/TCP per TECH-SPEC §8.4.

## Tasks
- Implement JSONL RPC dispatcher in `gatosd` for `messages.read` and `messages.publish` (with capability checks).
- Translate errors to spec codes (`topic_not_found`, `invalid_ulid`, `limit_out_of_range`, `head_conflict`).
- Thread repository path/config into the server bootstrap.
- Add integration tests that hit the RPC endpoint (using fixture repos) within the Docker harness.

## Definition of Done
- Daemon serves `messages.read` over JSONL with correct envelope and pagination (next_since).
- Publish path accepts canonical envelopes, performs CAS retries, and returns receipts.
- Tests pass in containerized CI.
