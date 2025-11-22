# Ledger: Git Backend Append + CAS Journals

- **Status:** In Progress
- **Area:** gatos-ledger-git
- **Owner:** Triage
- **Context:** `gatos-ledger-git` is a stub. Need a git2-backed backend that appends event commits under `refs/gatos/journal/<ns>/<actor>` with atomic compare-and-swap per SPEC §4.2.

## Tasks
- Implement append API: build commit tree with envelope blob + optional attachments; write trailers (`Event-CID`, `Sig-Alg`, `Sig`, `Policy-Root`, timestamp).
- CAS update journal refs via `reference_matching`; handle `HeadConflict` with retry/backoff policy (configurable).
- Enforce no merges on journal refs; forbid non-fast-forward updates.
- Expose `resolve_head`, `append`, `list_refs` functions needed by consumers (Echo, policy gate, job plane).
- Add repo init helper to create namespace roots if missing.

## Definition of Done
- Integration test: append N events concurrently → single linear history with expected head; retries logged.
- Commit layout matches SPEC; references updated atomically.
- Error types mapped for callers (conflict vs IO).

## Progress Log
- 2025-11-21: Added git2-backed append/read_window with envelope tree layout and tests; CAS still best-effort (reference_matching not wired yet).
