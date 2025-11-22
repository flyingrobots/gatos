# Code Quality Checklist (2025-11-21)

## Findings To Fix (checklist)

- [ ] **Hexagonal boundary breach:** `gatos-message-plane` still depends directly on `git2`; per ADR-0017 it should sit on ledger-core ports with a git adapter.
- [ ] **Publisher signature magic strings:** `GitMessagePublisher` hardcodes author name/email (`gatos-message-plane` / `message-plane@gatos.local`) instead of injecting from config.
- [ ] **CLI/service SRP coupling:** `crates/gatosd/src/main.rs` mixes daemon bootstrap with CLI subcommands; consider separating a dedicated CLI module to keep the daemon entry focused.
- [ ] **MessagePlaneService dependencies not injected:** repo path and git-backed implementations are constructed internally; accept injected trait objects/paths to ease testing and alternative backends.
- [x] **Magic numbers (limits):** constants like `MAX_PAGE_SIZE = 512`, default CLI limit=128, rotation thresholds (100k msgs / 192MB) are duplicated; centralize in config/profile and reference from one source.
- [x] **Checkpoint ref strings duplicated:** ref prefixes (`refs/gatos/messages`, `refs/gatos/consumers`) appear in multiple modules; extract shared constants to avoid drift.
- [ ] **Tests rely on placeholder CIDs/envelopes:** ledger tests use fixed ULIDs and placeholder metadata; add fixtures derived from SPEC vectors and document them.

## Overall Impressions

- The code follows the tests-first pattern, but hexagonal layering is not yet enforced: message-plane still talks to git2 directly and constructs repos internally. Moving to ledger-core ports and DI will improve composability and testability.
- Responsibilities blur in `gatosd` (entrypoint + CLI + service wiring); a small refactor to separate CLI commands from daemon setup would tighten SRP.
- Configurable constants should be centralized (profile/config) rather than spread across crates to reduce “magic number” drift.
