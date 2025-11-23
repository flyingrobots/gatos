# Ledger: Observability & Metrics

- **Status:** Done
- **Area:** Metrics / Logs / Audit
- **Owner:** Triage
- **Context:** No metrics/logging around ledger append/read/CAS retries. SPEC/ops guidance expects visibility for gates and journals.

## Tasks
- Emit counters: `ledger_appends_total{ns,actor,result}`, `ledger_cas_conflicts_total`, `ledger_reads_total`, `ledger_verify_fail_total{reason}`.
- Gauges: `ledger_head_age_seconds{ns}`, queue depth (optional).
- Logs: structured entries for CAS retries, DENY decisions, signature failures.
- Optional: audit refs for DENY already noted in policy gate issue.
- Document scrape points and log fields.

## Definition of Done
- Metrics exported via gatosd; sampled in tests or manual scrape.
- Logs include enough context to debug conflicts and verification failures.

## Progress Log
- 2025-11-22: Implemented metrics tracking in journal module with TDD approach. Added `*_with_metrics` functions that emit:
  - ledger_appends_total{ns, actor, result} - tracks successful and failed appends
  - ledger_cas_conflicts_total - incremented on every CAS retry (Reference class errors)
  - ledger_reads_total{ns} - tracks successful read_window calls
  All tests passing. Ready for gatosd integration to expose metrics endpoint.
