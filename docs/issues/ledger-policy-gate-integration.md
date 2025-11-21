# Ledger: Policy Gate Integration

- **Status:** TODO
- **Area:** gatosd / Policy / Ledger
- **Owner:** Triage
- **Context:** SPEC §6 requires policy gates to evaluate intents before ledger append and to log DENY decisions. The daemon lacks a gate-aware ledger service.

## Tasks
- Define `Intent` → `Decision` interface (Allow/Deny(reason)) and wire to policy engine (EchoLua compiled rules).
- On Allow: append event via ledger backend with policy_root bound in trailers.
- On Deny: write audit decision to `refs/gatos/audit/policy` with rule id + reason.
- Add capability checks for actor/caps in envelope.
- Integration tests: allow flow, deny flow writes audit, malformed envelope rejected pre-append.

## Definition of Done
- gatosd exposes a gate-checked append path; DENY writes audit entries; errors are spec-compliant.
