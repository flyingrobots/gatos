# Ledger: Event Read/Windowing for Folds & Jobs

- **Status:** Done
- **Area:** gatos-ledger-core / gatos-ledger-git
- **Owner:** Triage
- **Context:** Echo folds and Job Plane need to stream events between commit ranges. No API yet to read a window (start/end commits), filter by ns/actor, or return canonical payloads.

## Tasks
- Implement read API: given `start` (exclusive) and `end` (inclusive) commit ids or counts, return ordered events with metadata (policy_root, actor, ulid, Event-CID).
- Support tailing from head with pagination; surface `next_cursor` for resume.
- Validate envelopes on read (signature, ULID format) with option to skip heavy checks for trusted paths.
- Provide iterator abstraction for Echo/job consumers; include op to compute ledger window hash for PoF.
- Tests: window across multiple journal refs; unknown refs; signature failure path.

## Definition of Done
- Stable interface to stream events for folding and job claim logic.
- Resume tokens/cursors work across restarts; tests cover pagination.

## Progress Log
- 2025-11-21: Added start/end filtering support in git backend `read_window` with tests; pagination/cursors still TODO.
- 2025-11-22: Implemented `read_window_paginated` with cursor-based pagination; all tests passing.
