---
Status: Accepted
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
2. **Versioning & Schema**: HTTP header `x-gatos-api: v1`. The server publishes the canonical SDL at `GET /api/v1/graphql/schema` (checked into the repo under `api/graphql/schema.graphql`). Introspection stays enabled in non-production environments; in prod it is disabled and the SDL endpoint is authoritative.
3. **State Targeting**: Every query **MUST** include one of:
   - `stateRef: "<commit-sha>"` (recommended), or
   - `refPath: "refs/gatos/state/public/<ns>/<channel>"` (server resolves to head).
4. **Object Identity**: `id` fields are stable content IDs: `<ns>:<path>:<digest>`.
5. **Pagination**: Relay connections for lists; cursors are opaque, HMAC-signed. `first`/`last` arguments are clamped to `[1, 500]` (default 100). Ordering is deterministic (lexicographic by path unless a field specifies a different order). Requests exceeding the limit return `USER_INPUT_ERROR` with message "PAGE_LIMIT_EXCEEDED".
6. **Pointer Handling**: Opaque pointers (ADR-0004) resolve to a dedicated `OpaquePointerNode { kind, algo, digest, location, capability }`. The API **MUST NOT** download private blobs automatically.
7. **AuthZ Behaviour**: Policy filters (ADR-0003/0004) apply per field. If the actor lacks read access:
   - When a pointerized projection exists, return the `OpaquePointerNode` and append a GraphQL error with `code: "POLICY_DENIED"`.
   - When no projection exists, return `null` and append the same error. Clients **MUST** inspect `errors[]` to detect truncation.
8. **Caching**: `ETag` = `Shape-Root` of the resolved state and the response body **MUST** include `shapeRoot` and `stateRefResolved` top-level fields. `Cache-Control: immutable` applies to historical `stateRef`; `refPath` responses are `Cache-Control: no-cache` so clients revalidate.
9. **Errors**: Deterministic JSON error extensions:
   - `POLICY_DENIED` (403)
   - `STATE_NOT_FOUND` (404)
   - `PAGE_LIMIT_EXCEEDED` (422)
   - `INVALID_STATE_REF` (400)
   - `INTERNAL_ERROR` (500, last resort)
   Each error entry includes `extensions.code` and `extensions.ref` (support ULID).
10. **Rate Limits**: Default 600 requests / 60s window per actor, enforced via shared limiter. Policy rules may override per namespace/project; responses include `X-RateLimit-Remaining` headers.

## Schema Evolution & Error Surfacing
1. **Deprecation Cadence**
   - *Announce (0–4 weeks)*: SDL publishes `@deprecated(reason: "removal in 4w")`, release notes summarize the change, and responses add `X-GATOS-Deprecations`. No behavior change beyond warnings.
   - *Dual-Serve (4–12 weeks)*: Legacy + successor fields resolve in parallel. Requests for the old field append `errors[]` entries with `extensions.code="FIELD_DEPRECATED"`; dashboards track usage daily so teams can confirm adoption.
   - *Removal (>12 weeks)*: Field disappears from SDL/introspection. Queries referencing it return `USER_INPUT_ERROR` with an `extensions.ref` ULID pointing to the removal notice. A 1-week emergency rollback window exists; after that, reintroducing the field requires a fresh ADR.
2. **Error Propagation**
   - *Policy Denied*: Resolver emits partial data with an error `{code:"POLICY_DENIED", path:[...], ref:<ulid>}` while still returning HTTP 200 and a `shapeRoot`. The auxiliary diagram highlights the `Policy` participant sending `deny/pointerize` before the response.
   - *Invalid Ref / Missing State*: When `stateRef` or `refPath` cannot be resolved, Resolver emits `{code:"STATE_NOT_FOUND"}` (404 when the top-level ref fails, 200 otherwise) or `{code:"INVALID_STATE_REF"}` (400). The auxiliary diagram’s second branch shows `State Store` returning a miss and Resolver surfacing the error bubble before writing the response.
   - *Diagram Note*: Add a companion Mermaid sequence titled “Error Paths” with two `alt` blocks—`Policy DENY` (shows pointerized/null field plus error) and `State MISS` (shows Resolver handling a missing ref). This diagram supplements the happy-path diagram above so operators see both flows.

```mermaid
sequenceDiagram
    participant Client
    participant API as GraphQL API
    participant Resolver
    participant Policy as Policy Filter
    participant State as State Store

    Client->>API: POST /api/v1/graphql (stateRef, query)
    API->>Resolver: resolve fields
    Resolver->>State: load shape nodes
    Resolver->>Policy: check access
    alt allowed
        Policy-->>Resolver: allow / pointerize
    else denied
        Policy-->>Resolver: deny (POLICY_DENIED)
    end
    Resolver-->>API: data + shapeRoot
    API-->>Client: JSON (data, errors, shapeRoot)
```

## Consequences
- Clients can build efficient UIs without bespoke endpoints.
- Server complexity moves into resolvers and policy filters.

## Open Questions
- Field deprecation cadence.
