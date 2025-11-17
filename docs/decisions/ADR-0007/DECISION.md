---
Status: Draft
Date: 2025-11-09
ADR: ADR-0007
Authors: [flyingrobots]
Requires: [ADR-0001, ADR-0004, ADR-0005]
Related: [ADR-0008, ADR-0009]
Tags: [API, GraphQL, State]
Schemas:
  - schemas/v1/api/graphql_state_mapping.schema.json
---

# ADR-0007: GraphQL State API (Read-Only)

## Scope
Expose a **read-only GraphQL API** for querying GATOS state snapshots (“shape”) with precise, single-roundtrip selection.

## Rationale
State is hierarchical and interlinked; GraphQL matches the access pattern and avoids REST under/over-fetching.

## Decision
1. **Endpoint**: `POST /api/v1/graphql`.
2. **Versioning**: HTTP header `x-gatos-api: v1`. Introspection **MAY** be disabled in prod.
3. **State Targeting**: Every query **MUST** include one of:
   - `stateRef: "<commit-sha>"` (recommended), or
   - `refPath: "refs/gatos/state/public/<ns>/<channel>"` (server resolves to head).
4. **Object Identity**: `id` fields are stable content IDs: `<ns>:<path>:<digest>`.
5. **Pagination**: Relay connections for lists; cursors are opaque, signed.
6. **Pointer Handling**: Opaque pointers (ADR-0004) are exposed as typed nodes; server **MUST NOT** auto-resolve private blobs.
7. **AuthZ**: Queries are filtered by policy view; unauthorised paths are elided or pointerized per privacy policy.
8. **Caching**: `ETag` = `Shape-Root` of the resolved state; `Cache-Control: immutable` for historical `stateRef`.
9. **Errors**: Deterministic, typed error codes (e.g., `POLICY_DENIED`, `STATE_NOT_FOUND`).
10. **Rate Limits**: Default window limits; per-actor overrides via policy.

## Consequences
- Clients can build efficient UIs without bespoke endpoints.
- Server complexity moves into resolvers and policy filters.

## Open Questions
- Schema publishing: static SDL vs generated at build from spec?
- Field deprecation cadence.
