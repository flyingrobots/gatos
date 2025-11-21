# Message Plane: Ledger-Core Port (ADR-0017)

- **Status:** TODO
- **Area:** Architecture / gatos-ledger-core
- **Owner:** Triage
- **Context:** ADR-0017 proposes moving Message Plane off direct `git2` to ledger-core ports with a git adapter. Implementation not started.

## Tasks
- Add `RefStore` + `TreeBuilder` ports to `gatos-ledger-core` suitable for Message Plane.
- Implement git-backed adapter in `gatos-ledger-git` exposing CAS ref ops and tree/blob helpers.
- Refactor Message Plane publisher/subscriber/checkpoint/pruner to target the ports, removing direct `git2` dependency.
- Keep existing tests green; add backend-agnostic fixtures.

## Definition of Done
- Message Plane builds against ledger-core ports; `git2` is used only inside the adapter crate.
- Tests pass for both git adapter and (if added) in-memory test backend.
