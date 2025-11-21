---
Status: Proposed
Date: 2025-11-21
ADR: ADR-0017
Authors: [assistant]
Requires: [ADR-0005]
Related: [ADR-0001, ADR-0002, ADR-0006]
Tags: [Message Plane, Ledger Core, Hexagonal]
Schemas: []
Supersedes: []
Superseded-By: []
---

# ADR-0017: Message Plane on Ledger-Core Ports (No Direct git2)

## Scope

Decouple the Message Plane publisher/subscriber from `git2` by expressing it purely in terms of `gatos-ledger-core` traits (object store + CAS-able refs + tree builder). A git-backed adapter lives in `gatos-ledger-git`, but the orchestration logic remains backend-agnostic.

## Decision

1. Define a minimal set of ports in `gatos-ledger-core` usable by the Message Plane:
   - `RefStore`: `resolve(ref) -> Option<CommitId>`, `compare_and_swap(ref, new, expected) -> Result<(), CasError>`.
   - `ObjectStore`: `put_object(bytes) -> Hash` (reuse existing `ObjectStore`).
   - `TreeBuilder` helper to assemble `message/` and `meta/` trees from blob ids.
   - Timestamp provider (POSIX seconds) passed in to enforce deterministic metadata.

2. Reimplement `GitMessagePublisher` logic as `LedgerMessagePublisher<B: LedgerBackend>` where `LedgerBackend` bundles the above ports. Rotation, segment metadata, commit message formatting, and CAS ordering stay identical to ADR-0005; only the storage calls go through the backend.

3. Provide a git adapter inside `gatos-ledger-git` that satisfies the ports using libgit2. Message Plane no longer imports `git2`; it links against `gatos-ledger-git` when the git backend is desired.

4. Keep `MessageSubscriber` and `CheckpointStore` on the same ports (read commit by id, load blobs, list refs with prefix). A backend that can enumerate topic refs can serve subscribers without git-specific code.

5. Preserve CAS safety: order remains commit -> CAS segment ref -> CAS topic head. Surface `HeadConflict` on CAS failure so callers can retry; never silently overwrite.

## Consequences

**Pros**
- Hexagonal boundary: Message Plane code depends only on abstract ports, enabling alternate backends (remote CAS, packed objects, partial-clone mirrors) without edits.
- Testability: swap in an in-memory backend to fuzz rotation/pruning without libgit2 overhead.
- Separation of concerns: git-specific optimizations and config (pack writing, commit-graph) stay in `gatos-ledger-git`.

**Cons**
- Slight indirection overhead; needs bench to ensure no regressions for hot paths.
- Additional work to expose ref listing and tree-building helpers in `gatos-ledger-git` cleanly.

## Alternatives Considered

1. **Keep direct `git2` usage inside Message Plane** (status quo): simplest short-term, but couples Message Plane to git API details, complicating backends and tests.
2. **Define a Message-Plane-specific git facade** (thin shim around libgit2): reduces surface slightly but still locks the Message Plane to git; duplicative of ledger-core ports.

## What If Not Adopted?

- Message Plane remains tied to libgit2, making it harder to target non-git stores or future ledger backends. Backend swap or in-memory testing would require heavy mocking, and git-specific concerns could leak into higher layers.

## Migration Notes

- Introduce ports in `gatos-ledger-core`, then add a `LedgerMessagePublisher`/`Subscriber` implementation using them.
- Implement git adapter in `gatos-ledger-git` and flip `gatos-message-plane` to depend on the adapter instead of `git2` directly.
- Keep current tests; add backend-agnostic fixtures to ensure parity across implementations.
