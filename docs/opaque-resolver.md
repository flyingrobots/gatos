---
title: Opaque Pointer Resolver API
---

# Opaque Pointer Resolver API

Normative default: Bearer JWT authentication; requests and decisions are audited.

## Request

```http
GET /resolve/<algo>/<digest>
Authorization: Bearer <JWT>
Accept: application/octet-stream
```

JWT claims:

- `sub` — subject
- `aud` — audience (resolver/repo id)
- `exp` — expiry (short‑lived)
- Optional `scope` — dataset/namespace scope

## Response

```http
200 OK
Digest: blake3=<hex>
X-BLAKE3-Digest: <hex>
Content-Type: application/octet-stream

<bytes>
```

## Audit

On each resolve attempt, append an audit entry under `refs/gatos/audit/resolve/<ulid>` including:

```json
{ "ts": "<iso8601>", "alg": "blake3", "digest": "<hex>", "sub": "<sub>", "aud": "<aud>", "decision": "ALLOW|DENY" }
```

## Schemas

See JSON Schemas in `docs/schemas/opaque-resolver.request.schema.json` and `docs/schemas/opaque-resolver.response.schema.json`.

