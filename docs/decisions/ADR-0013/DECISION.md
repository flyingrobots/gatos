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
1. **Unit Partitioning**
   - State namespaces declare unit boundaries in `.gatos/fold_units.yaml` (list of glob patterns). Default: `state/<ns>/<channel>/**` per channel.
   - Each unit’s cache key:
     ```text
     key = blake3(fold_code_hash || policy_root || event_ids_hash || upstream_unit_digests)
     ```
     - `fold_code_hash`: hash of the Echo fold code (compiled).
     - `event_ids_hash`: blake3 over the ordered list of events fed into the unit since last checkpoint.
     - `upstream_unit_digests`: sorted list of dependent unit digests (for DAG composition).

2. **Cache Store**
   - `gatos-kv` stores `key -> {digest, payload_pointer}`. Payload is a pointer to the unit’s serialized shape (ADR-0004 pointer envelope) to avoid duplicating blobs.
   - Cache entries expire when `policy_root` changes or when explicitly invalidated.

3. **Dependency Graph & Invalidation**
   - Fold authors declare dependencies between units (YAML adjacency). The engine builds a DAG and, upon new events, computes affected units via reverse dependencies.
   - When an upstream unit changes digest, all downstream units are marked dirty.
   - Cache evictions recorded under `refs/gatos/audit/fold-cache/<ulid>` for observability.

4. **Lazy Materialization**
   - Units outside the request (e.g., API query doesn’t touch a namespace) remain cold. The first access triggers `fold_unit` computation asynchronously; clients receive `loading=true` until ready.
   - CLI flag `--materialize all` forces full materialization for scenarios like exports.

5. **Concurrency & Scheduling**
   - Fold executor spawns up to `num_cpus` workers; DAG ensures topological order. Units with no unmet dependencies run in parallel.
   - Locks per unit prevent double computation; duplicate requests wait on the same future.

6. **Telemetry & Reporting**
   - Commit trailers include `Fold-Cache-Hit`, `Fold-Cache-Miss`, `Fold-Units`, `Fold-Duration`, `Fold-Parallelism`.
   - Metrics exported via Prometheus: `gatos_fold_unit_duration_ms`, `gatos_fold_cache_utilization`.

## Prewarming & Shared Cache Policy
1. **Background Prewarming**
   - Allowed only when `--prewarm` flag is explicitly set or `fold.prewarm=true` in `.gatos/fold_units.yaml`. Prewarm jobs enqueue idle-time recomputation for units touched in the last 24h and MUST honor global concurrency limits to avoid starving foreground folds.
   - Policy gate: if governance rules forbid speculative compute (e.g., sensitive namespaces), the fold daemon skips prewarming automatically and logs `fold.prewarm.skipped`.
   - Prewarm runs emit `Fold-Prewarm` trailers with the list of units warmed so auditors can correlate CPU usage.
2. **Shared Cache Stores**
   - Default path: `${GATOS_CACHE_ROOT:-.gatos/cache}/fold-units`. Multi-worktree setups point `GATOS_CACHE_ROOT` to a shared volume; locks are implemented via `flock` on `cache.lock` plus per-unit `.lck` files to prevent double writes.
   - Eviction policy: LRU capped at 50k units or 200 GB, whichever comes first. Operators may override via env vars `GATOS_CACHE_MAX_UNITS` / `GATOS_CACHE_MAX_BYTES`.
   - When multiple nodes share the cache (e.g., NFS), cache metadata includes the producing host ID; stale entries older than `policy_root` or with missing blobs are purged during startup sweep, and the purge is recorded under `refs/gatos/audit/fold-cache/<ulid>`.

```mermaid
graph TD
    E1[Event Stream] --> P1[Plan Affected Units]
    P1 --> U1[Unit Fold Cache]
    U1 -->|hit| C1[Join]\n(Reuse digest)
    U1 -->|miss| F1[Fold Unit]
    F1 --> C1
    C1 --> SR[Compute Shape-Root]
```

## Consequences
- Orders-of-magnitude faster folds; predictable latencies.
- Requires dependency modeling and careful cache invalidation.

## Open Questions
- Do we allow background prewarming or keep strictly on-demand for MVP?
- How do we cache across git worktrees (shared cache vs per-worktree)?
