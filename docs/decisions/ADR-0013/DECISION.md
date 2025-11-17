---
Status: Draft
Date: 2025-11-09
ADR: ADR-0013
Authors: [flyingrobots]
Requires: [ADR-0001, ADR-0005]
Related: []
Tags: [Performance, State Engine, Caching]
---

# ADR-0013: Partial & Lazy Folds

## Scope
Specify **incremental** and **lazy** state folding to avoid recomputing the entire shape on small changes.

## Rationale
Large repos need sub-linear recomputation to stay responsive.

## Decision
1. **Fold Units**: Define shard boundaries (by namespace/path). Each unit has a cache key:
   ```text
   key = blake3(code_hash || policy_root || input_event_ids || upstream_unit_digests)
   ```
2. **Cache**: `gatos-kv` stores fold outputs per unit with `key -> digest`.
3. **Invalidation**: On new events, compute affected unit set via dependency graph; only recompute those.
4. **Lazy Materialization**: Units not requested by a client **MAY** remain cold; materialize on demand.
5. **Concurrency**: Units may fold in parallel if dependencies permit; global join computes `Shape-Root`.
6. **Telemetry**: Commit trailers: `Fold-Cache-Hit: <count>`, `Fold-Units: <n>`, `Fold-Duration: <ms>`.

## Consequences
- Orders-of-magnitude faster folds; predictable latencies.
- Requires dependency modeling and careful cache invalidation.

## Open Questions
- Do we allow background prewarming or keep strictly on-demand for MVP?
