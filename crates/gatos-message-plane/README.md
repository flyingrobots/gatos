# GATOS Message Plane (Message Bus)

This crate implements the GATOS Message Plane (GMP), an asynchronous, commit-backed publish/subscribe
system. It handles topics, sharding, and different Quality of Service (QoS) guarantees for
distributed communication between GATOS components.

> ⚠️ Stability Note: The public API surface in this branch is an API sketch and subject to change.
> This document describes the planned architecture; implementation is in progress. For design
> details, see [TECH-SPEC.md](../../docs/TECH-SPEC.md).

Commit-backed means messages are persisted as Git commits to provide durability, auditability, and
at-least-once delivery with deterministic replay via ULID checkpoints. See the architecture notes in
[ADR-0005](../../docs/decisions/ADR-0005/DECISION.md) and protocol details in
[TECH-SPEC.md](../../docs/TECH-SPEC.md).

## Features

- Asynchronous messaging: non-blocking publish/subscribe operations.
- Commit-backed durability: persisted messages with canonical envelopes (`message/envelope.json`).

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
// use gatos_message_plane::{Publisher, Subscriber};
// #[tokio::main]
// async fn main() { /* publish/subscribe */ }
```

Examples are coming once the API lands.

## Integration

GMP is the Message Plane in the GATOS hexagonal architecture. It coordinates messaging across:

- `crates/gatos-ledger-core` and `crates/gatos-ledger-git`: ledger state events
- `crates/gatos-policy`: policy decision events
- `crates/gatos-kv`: materialized view updates
- `bindings/ffi` and `bindings/wasm`: cross-language event streaming

### Usage (API Sketch)

- Depend on `gatos-message-plane` in your crate.
- Use a `Publisher` to append canonical envelopes under `refs/gatos/messages/<topic>/head`; use a `Subscriber` (or the `messages.read` RPC) to stream them oldest→newest.
- Messages are persisted as Git commits and consumers store checkpoints in `refs/gatos/consumers/<group>/<topic>` so crashes can resume without duplication.

> Note: This section reflects the intended usage; concrete APIs will be added as implementation proceeds.

## Current API Skeleton

The crate currently exports lightweight traits and structs so downstream crates can start wiring integrations:

- `TopicRef` — identifies the repository + logical topic (`refs/gatos/messages/<topic>`).
- `MessageEnvelope` — holds the canonical JSON bytes (per `schemas/v1/message-plane/event_envelope.schema.json`) and can be built via `MessageEnvelope::from_json_str` to enforce canonicalization/validation.
- `GitMessagePublisher` — appends canonical envelopes, rotating segments hourly or when message/byte thresholds are exceeded. Commits store `message/envelope.json` plus `meta/meta.json` with segment metadata.
- `GitMessageSubscriber` — walks a topic head parent chain, returning canonical envelopes (ordered oldest→newest) while honoring `since_ulid` and `limit` per ADR-0005.
- `GitCheckpointStore` — persists consumer checkpoints under `refs/gatos/consumers/<group>/<topic>` (JSON blob containing `ulid` + optional `commit`) and can list/load checkpoints to coordinate pruning.
- `SegmentPruner` — finds/deletes aged segment refs only when all consumer checkpoints are at/after the segment’s newest ULID (TTL-safe pruning scaffold).

These types intentionally omit concrete transport plumbing; they document the expected shape so ADR work and downstream SDKs can evolve in parallel.

For protocol details, architecture rationale, and design patterns, see
[ADR-0001](../../docs/decisions/ADR-0001/DECISION.md) and
[TECH-SPEC.md](../../docs/TECH-SPEC.md).
