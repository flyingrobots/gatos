# GATOS Ledger Core

This crate provides the `no_std`-compatible core logic for the GATOS ledger. It defines the pure, portable data structures and semantics for the commit graph, hashing, and proofs.

It defines the `ObjectStore` trait, which acts as a "port" for storage backends to implement.

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).
