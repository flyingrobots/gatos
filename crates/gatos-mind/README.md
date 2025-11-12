# GATOS Mind (Message Bus)

This crate implements the GATOS Message Bus (GMB), an asynchronous, commit-backed publish/subscribe
system. It handles topics, sharding, and different Quality of Service (QoS) guarantees for
distributed communication between GATOS components.

> ⚠️ Stability Note: The public API surface in this branch is an API sketch and subject to change.
> This document describes the planned architecture; implementation is in progress. For design
> details, see [TECH-SPEC.md](../../docs/TECH-SPEC.md).

Commit-backed means messages are persisted as Git commits to provide durability, auditability, and
exactly-once semantics when combined with acknowledgements/commitments. See the architecture notes
in [ADR-0001](../../docs/decisions/ADR-0001/DECISION.md) and protocol details in
[TECH-SPEC.md](../../docs/TECH-SPEC.md).

## Features

- Asynchronous messaging: non-blocking publish/subscribe operations.
- Commit-backed durability: persisted messages with auditability and exactly-once when combined

  with acks/commitments.

- Topic-based routing: logical message organization and filtering.
- Sharding: horizontal scalability via topic partitioning.
- QoS guarantees: at-most-once, at-least-once, exactly-once.

## Feature Flags

- `std` (default): standard library support
  - Enabled (default): Full Message Bus functionality including async publishers/subscribers and topic sharding.
  - Disabled (`no_std`): Core message types and trait definitions only; no async runtime, publishers, or subscribers. Use this for embedded environments or constrained WASM profiles.

## Planned API Shape

> ⚠️ The following is an aspirational sketch to convey intent; names and behavior will change.
> For the evolving design and protocol, see [TECH-SPEC.md](../../docs/TECH-SPEC.md).

```text
// use gatos_mind::{Publisher, Subscriber};
// #[tokio::main]
// async fn main() { /* publish/subscribe */ }
```

Examples are coming once the API lands.

## Integration

GMB is the Message Plane in the GATOS hexagonal architecture. It coordinates messaging across:

- `crates/gatos-ledger-core` and `crates/gatos-ledger-git`: ledger state events
- `crates/gatos-policy`: policy decision events
- `crates/gatos-kv`: materialized view updates
- `bindings/ffi` and `bindings/wasm`: cross-language event streaming

### Usage (API Sketch)

- Depend on `gatos-mind` in your crate.
- Use a `Publisher` to publish messages to a topic; use a `Subscriber` to consume.
- Messages are persisted as Git commits to provide auditability and coordinate exactly-once when combined with acknowledgements/commitments.

> Note: This section reflects the intended usage; concrete APIs will be added as implementation proceeds.

For protocol details, architecture rationale, and design patterns, see
[ADR-0001](../../docs/decisions/ADR-0001/DECISION.md) and
[TECH-SPEC.md](../../docs/TECH-SPEC.md).
