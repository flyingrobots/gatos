# gatos-ports

Cross-plane ports and interfaces for the GATOS system.

## Overview

This crate defines `no_std`-compatible traits that decouple the core GATOS planes (Policy, Ledger, Audit, Observability) using the **Ports & Adapters** pattern.

## Traits

### Policy Plane

- **`PolicyClient`**: Evaluates append requests and returns Allow/Deny decisions
- **`AuditSink`**: Records policy decisions to durable storage (e.g., Git refs)
- **`PolicyAuditEntry`**: Metadata for policy decisions (timestamp, decision, context)

### Ledger Plane

- **`JournalStore`**: Abstracts append-only event log operations
  - `append(ns, actor, event) -> commit_id`
  - `read_window(ns, actor, start, end) -> events`
  - `read_window_paginated(..., limit) -> (events, cursor)`
  - Associated types: `Event`, `Error`

### Observability

- **`Metrics`**: Facade for counters and histograms
- **`Clock`**: Returns POSIX timestamps (UTC)

## Architecture

### Dependency Flow

```
gatosd (binary)
  ├─> gatos-ledger (facade)
  │     └─> gatos-ledger-git (adapter)
  │           └─> gatos-ports (ports)
  └─> gatos-policy (adapter)
        └─> gatos-ports (ports)
```

### Why Ports & Adapters?

1. **Testability**: Inject mocks instead of real Git repos or policy engines
2. **Flexibility**: Swap Git backend for SQL, KV store, etc. without changing business logic
3. **Decoupling**: Policy, Ledger, and Audit planes can evolve independently
4. **no_std**: Core traits work in constrained environments (embedded, WASM)

## Example: Implementing JournalStore

```rust
use gatos_ports::JournalStore;

struct InMemoryJournal {
    events: Vec<MyEvent>,
}

impl JournalStore for InMemoryJournal {
    type Event = MyEvent;
    type Error = String;

    fn append(&mut self, ns: &str, actor: &str, event: Self::Event) -> Result<String, Self::Error> {
        self.events.push(event);
        Ok(format!("commit-{}", self.events.len()))
    }

    fn read_window(
        &self,
        ns: &str,
        actor: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        Ok(self.events.clone())
    }

    fn read_window_paginated(
        &self,
        ns: &str,
        actor: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
        limit: usize,
    ) -> Result<(Vec<Self::Event>, Option<String>), Self::Error> {
        let events = self.events.iter().take(limit).cloned().collect();
        Ok((events, None))
    }
}
```

## Implementations

### Git Backend (`gatos-ledger-git`)

- **`GitJournalStore`**: Implements `JournalStore` using `git2` refs
- **`GitPolicyAudit`**: Implements `AuditSink` writing to `refs/gatos/audit/policy`

### Test Doubles

See `#[cfg(test)]` blocks in this crate for minimal mock implementations.

## Future Ports

- `EventBus` for message plane pub/sub
- `ObjectStore` for content-addressed blobs (already in `gatos-ledger-core`)
- `MetricsExporter` for Prometheus/OTLP integration
