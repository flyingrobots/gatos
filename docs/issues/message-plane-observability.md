# Message Plane: Observability Hooks

- **Status:** TODO
- **Area:** Metrics / Logs / Audit
- **Owner:** Triage
- **Context:** ADR-0005 addendum lists required metrics/logs/audit refs, but no code emits them.

## Tasks
- Emit Prometheus-style counters/gauges (`gmp_publish_total`, `gmp_segment_rotations_total{reason}`, `gmp_checkpoint_writes_total`, `gmp_prune_segments_total`, `gmp_prune_skipped_total`, `gmp_head_age_seconds`, `gmp_min_checkpoint_ulid`).
- Add structured logs for rotation/prune actions with segment prefixes, ULID ranges, counts, bytes, and gating checkpoint ULID.
- Optional: write prune summaries to `refs/gatos/audit/message-plane/<ulid>` when configured.
- Document scraping and log formats in ops guide.

## Definition of Done
- Metrics exported from gatosd (or a sidecar), validated in tests.
- Logs present at info level for rotations/prunes; errors on CAS conflicts.
- Audit summary path either implemented or explicitly deferred with config guard.
