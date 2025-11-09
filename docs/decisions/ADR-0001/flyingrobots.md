# ADR-0001

The new crate layout is a refinement that more strongly respects and implements the core ideas from the `SPEC.md` and `TECH-SPEC.md` documents.

Here’s a breakdown of how the new structure maps to the original goals:

## It Directly Implements the "Four Planes" from `SPEC.md`

**Ledger Plane**: This is now explicitly managed by the `gatos-ledger-*` family of crates (core, git, and the meta-crate).

**State Plane**: The logic for deterministic state computation and sessions is now in `gatos-echo`, with `gatos-kv` providing a clear home for the caching and materialized view aspect.

**Policy Plane**: This remains the clear responsibility of `gatos-policy`.

**Message Plane**: This is now the clear responsibility of `gatos-mind`.

## It Strengthens the `no_std` and Portability Goal

The most significant change is splitting `gatos-ledger` into `gatos-ledger-core` and `gatos-ledger-git`. It's a direct and robust implementation of the "`no_std` at the core, `std` at the edges" principle we just documented in `ADR-0001`.

> ***This makes the goal of running GATOS on microcontrollers or in WASM sandboxes not just a theoretical possibility but a core part of the
architecture.***

## It Refines the Original `TECH-SPEC.md` Layout

`gatos-core` has evolved into the more sophisticated `gatos-ledger-*` structure.

`gatos-bus` is now `gatos-mind`.

`gatos-session` is now `gatos-echo`, clarifying its implementation with the deterministic DPO engine.

`gatos-proof`'s responsibilities (the "proof math") are now correctly placed within the `no_std` `gatos-ledger-core`, as proofs are intrinsically tied to the ledger's data structures.

The new bindings/ directory provides a clear and organized home for the FFI and WASM bindings, which is a natural and necessary step for any core system intended to be used by other languages.

In conclusion, the new layout is a more mature, practical, and robust implementation plan that directly addresses the project's most ambitious goals, especially portability and determinism.
