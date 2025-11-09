# GATOS Mind (Message Bus)

This crate implements the GATOS Message Bus (GMB), an asynchronous, commit-backed publish/subscribe
system. It handles topics, sharding, and different Quality of Service (QoS) guarantees for
distributed communication between GATOS components.

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

## Quick Start

```rust,no_run
// API sketch — final names may differ.
// use gatos_mind::{Publisher, Subscriber};
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let mut pubr = Publisher::connect("queue.acme").await?;
//     pubr.publish(b"hello").await?;
//
//     let mut sub = Subscriber::connect("queue.acme").await?;
//     if let Some(msg) = sub.next().await {
//         // process msg
//     }
//     Ok(())
// }
```

Examples coming soon; for now, this sketch illustrates the intended shape.

## Integration

GMB is the Message Plane in the GATOS hexagonal architecture. It coordinates messaging across:

- `crates/gatos-ledger-core` and `crates/gatos-ledger-git`: ledger state events
- `crates/gatos-policy`: policy decision events
- `crates/gatos-kv`: materialized view updates
- `bindings/ffi` and `bindings/wasm`: cross-language event streaming

### How it works (at a glance)

- Depend on `gatos-mind` in your crate.
- Use a `Publisher` to publish messages to a topic; use a `Subscriber` to consume.
- Messages are persisted as Git commits to provide auditability and coordinate exactly-once when combined with acknowledgements/commitments.

> Note: In this branch the public API is still a placeholder; the integration
> surface will expose `Publisher`/`Subscriber` types as the bus is implemented.

For protocol details, architecture rationale, and design patterns, see
[ADR-0001](../../docs/decisions/ADR-0001/DECISION.md) and
[TECH-SPEC.md](../../docs/TECH-SPEC.md).
