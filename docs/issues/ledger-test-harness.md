# Ledger: Integration Test Harness (Git Backend)

- **Status:** TODO
- **Area:** Testing / CI
- **Owner:** Triage
- **Context:** No automated coverage for ledger append/read/CAS behavior. Need containerized tests per repo policy (tests mutate refs).

## Tasks
- Build Docker-based integration tests that init a repo, append events concurrently, and verify CAS retry behavior.
- Add tests for read windowing, signature verification failures, and audit DENY logging.
- Wire into CI (make target) honoring `GATOS_TEST_IN_DOCKER=1`.
- Capture fixture envelopes and expected CIDs for determinism checks.

## Definition of Done
- `make test` (in Docker harness) exercises ledger end-to-end; failures block CI.
- Fixtures documented so future changes can be compared.
