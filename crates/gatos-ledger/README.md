# GATOS Ledger

This is a meta-crate that composes the GATOS ledger components via feature flags. It acts as the single public-facing entry point for consumers, who can choose a storage backend (`git2-backend`) or use the core logic standalone (`core-only`).

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).
