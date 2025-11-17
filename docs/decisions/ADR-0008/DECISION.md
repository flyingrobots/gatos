---
Status: Draft
Date: 2025-11-09
ADR: ADR-0008
Authors: [flyingrobots]
Requires: [ADR-0001, ADR-0002, ADR-0003]
Related: [ADR-0007, ADR-0009]
Tags: [API, REST, Commands, Webhooks]
Schemas:
  - schemas/v1/api/command_envelope.schema.json
  - schemas/v1/api/webhook_delivery.schema.json
---

# ADR-0008: REST Commands & Webhooks

## Scope
Define a minimal **REST mutation surface** for commands and a **webhook** mechanism for outbound events.

## Rationale
Commands are side-effecting; REST is adequate and tool-friendly. Integrations need push-based notifications.

## Decision
1. **Command Endpoint**: `POST /api/v1/commands`
   - Body:
     ```json
     { "type": "<verb.noun>", "args": {...}, "expect_state": "<sha>", "request_id": "<ulid>" }
     ```
   - Semantics: Return quickly with `{ "ack": true, "job_id": "<ulid>" }` (async) or `{ "ok": true, ... }` (sync).
   - **Idempotency**: `request_id` **MUST** dedupe within a 24h window.
2. **Result Plumbing**: Long work **SHOULD** create a Job (ADR-0002) and stream progress on the Message Plane.
3. **Webhooks**:
   - Subscription admin endpoint: `POST /api/v1/webhooks`.
   - Events (normative names): `proposal.created`, `approval.created`, `grant.created`, `grant.revoked`, `job.created`, `job.claimed`, `job.succeeded`, `job.failed`.
   - Delivery: JSON body, `X-GATOS-Event`, `X-GATOS-Delivery`, **HMAC-SHA256** signature header.
   - Retries with exponential backoff; dead-letter queue optional.
4. **AuthN/Z**: OAuth2/JWT bearer; scopes per command prefix; webhook secrets per subscription.
5. **HTTP Codes**: `202` for async ack, `200` for sync success, `409` for `EXPECT_STATE_MISMATCH`.

## Consequences
- Clean separation of **mutations** (REST) vs **reads** (GraphQL).
- Webhooks unlock automation without polling.

## Open Questions
- Do we surface a lightweight sync mode with server-side time budget?
