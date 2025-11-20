# gatos-ledger-git (stub)

This crate temporarily satisfies the workspace dependency on the Git-backed ledger backend. The real implementation was removed and will be rebuilt to align with ADR-0001 and the Ledger-Kernel spec. For now, the crate simply exposes a placeholder API so other crates (e.g., Message Plane, gatosd) can compile and run tests.

See `LEDGER-COMPARE.md` for the roadmap toward the full implementation.
