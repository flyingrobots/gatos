# GATOS Ledger Git Backend

This crate provides a `std`-dependent storage backend for the GATOS ledger that uses `libgit2`. It implements the `ObjectStore` trait from `gatos-ledger-core`, acting as an "adapter" to connect the core ledger logic to a real Git repository on a filesystem.

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).
