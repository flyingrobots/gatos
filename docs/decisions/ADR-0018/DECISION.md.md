# ADR-0018: Adoption of the **Nine Lives** Resilience & Control Framework in GATOS
_Status: Approved_
_Authors: James Ross_
_Date: 2025-11-24_

---
## 1. Context
GATOS currently implements:
- An **append-only, content-addressed event log** (GitLedger)    
- A **deterministic compute layer** (Wesley/Echo)
- A **policy enforcement layer**
- A **declarative domain model**
- A **CLI + API** for data, events, and projections

However, GATOS lacks:
- A **resilience layer** for async operations  
- A **runtime control plane** for live reconfiguration
- A **telemetry plane** for structured visibility
- A **meta-policy engine** for autonomous decision-making
- A **feedback loop** that observes system behavior and acts upon it
- A unified abstraction for cross-cutting concerns like retry, timeout, jitter, circuit breaking, bulkhead, or failure domain isolation

Failure scenarios where this appears today:
- CAS conflicts → retry storms
- Backpressure → unbounded concurrency
- Network calls to external policy evaluators → unbounded latency
- Replication → unpredictable distributed failures
- Materialized view rebuilds → no rate limiting
- Policy evaluations → no standardized timeout or fallback behavior
- Zero ability to adapt to failures dynamically or intelligently

**Observation:** GATOS has a deterministic state plane _but no resilience/control plane_. It has storage, compute, and semantics, but no brainstem.

This ADR introduces **Nine Lives**, an algebraic resilience + control framework built on Tower Service semantics, as the official resilience + adaptive control subsystem for GATOS.

---
## 2. Decision
GATOS will adopt the **Nine Lives v2** framework as its official:
1. **Resilience Layer** (retry, timeout, bulkhead, backoff, jitter, circuit breaker)
2. **Telemetry Plane** (PolicyEvent, TelemetrySink, event bus)
3. **Control Plane** (`Adaptive<T>`, CommandHandler, ControlPlaneRouter)
4. **Autonomous Meta-Policy Plane (Sentinel)** (dynamic policy reconfiguration + shadow evaluation)

This forms the **Sentinel Plane**, the missing top-level supervisory layer for GATOS.

---
## 3. Architecture Overview
GATOS will now operate across **three interconnected planes**:

```text
┌────────────────────────────────────────────────────────┐
│                 Sentinel Plane (Nine Lives)            │
│           Observe → Decide → Act → Reconfigure         │
│   - Retry, Timeout, Bulkhead, CircuitBreaker, Backoff  │
│   - PolicyEvents, TelemetrySink, Observer              │
│   - Adaptive<T>, ControlPlaneRouter, CommandHandlers   │
│   - Meta-Policy Engine (Rhai) + Shadow Evaluation      │
└────────────────────────────────────────────────────────┘
                              ↓
┌────────────────────────────────────────────────────────┐
│               GATOS State & Policy Plane               │
│        (GitLedger, GATOS Domain Model, Policies)       │
│              - append-only truth                       │
│              - versioned configs                       │
│              - deterministic policy evaluation         │
└────────────────────────────────────────────────────────┘
                              ↓
┌────────────────────────────────────────────────────────┐
│              Deterministic Compute Plane               │
│                    (Wesley / Echo)                     │
│               - Reproducible execution                 │
│               - Provenance-complete AST/IR             │
│               - Deterministic simulation               │
└────────────────────────────────────────────────────────┘
```

**Nine Lives sits on top of all async operations**, just like a hyper-boosted, distributed-aware Promise runtime:
- Wrap any operation in resilience
- Emit telemetry
- Feed telemetry back into Sentinel
- Sentinel adapts the policies live
- GitLedger persists policy state
- GATOS re-evaluates domain logic
- Deterministic compute replays effects

This closes the loop.

---
## 4. Why Nine Lives? (Rationale)

### 4.1 Uniform Abstraction
Everything becomes a `tower::Service`:
- journal append
- journal read
- policy evaluation
- replication
- view rebuild
- external HTTP calls
- telemetry emission
- control plane commands

This uniformity creates recursion and composability.
### 4.2 Algebraic Composition
Nine Lives enables:
- Sequential composition (A + B)    
- Fallback (A | B)
- Parallel race (A & B)

These are foundational distributed computing patterns.
### 4.3 Telemetry Plane
GATOS lacked a structured observability defense line.
Nine Lives introduces:
- PolicyEvent    
- TelemetrySink
- multicasting and fallback sinks  
- an Observer component that builds a SystemState

This powers the Sentinel.
### 4.4 Control Plane
GATOS had no dynamic configuration. Nine Lives introduces:
- `Adaptive<T>` (“live-updatable” policy knobs)
- CommandHandler (a Service)    
- ControlPlaneRouter (dynamic dispatch)
- Audit/Authorization layers
- GitLedger-backed versioning
### 4.5 Sentinel: Autonomous Governance
This is the game-changer:
- Meta-policy evaluation
- Live tuning
- Shadow evaluation (simulate before enact)
- Automated healing
- Automated stabilization
- Progressive rollout logic  

This turns GATOS into a **self-healing distributed system**.

---

## 5. Implications

### 5.1 GitLedger becomes the persistent configuration backend
Dynamic policies get versioned as commits.
### 5.2 Deterministic compute (Echo/Wesley) remains the execution substrate
Nothing changes — it simply gains guard rails and supervisory intelligence.
### 5.3 The Nine Lives event bus becomes core GATOS infrastructure
It becomes the GATOS nervous system.
### 5.4 Service boundaries disappear
Everything async becomes a `Service`, layered with Nine Lives policies.

---
## 6. Alternatives Considered
### 6.1 `Tower`-only
**Rejected**: `Tower` provides Layers but no:
- event model
- control plane
- sentinel
- adaptive policies
- meta-policy engine
- distributed invariants

`Tower` is necessary but insufficient.
### 6.2 Ad-hoc retry/timeout libs (e.g., tokio-retry)
**Rejected**: not composable across all async contexts.
### 6.3 Deterministic Lua for meta-policy
**Rejected**: harder sandbox, less Rust-native, nondeterministic behavior.
### 6.4 WASM scripting
**Rejected**: heavier runtime, less ergonomic for policy DSL.

---
## 7. Risks

- **Too powerful**: improper meta-policy rules could destabilize system
- Scripting engine introduces operational risk
- Telemetry volume must be bounded
- Sentinel must be sandboxed and monitored

Mitigation strategies include:
- Shadow evaluation    
- Rate limiting
- Audit logs
- Meta-policy tests
- Fallback to safe/default policies

---

## 8. Rollout Plan
### Phase 1
- [ ] Introduce Nine Lives as a dependency; 
- [ ] wrap journal append + read windows.
### Phase 2
- [ ] Introduce Telemetry Streams + Observer.
### Phase 3
- [ ] Introduce Control Plane + Adaptive.  
### Phase 4
- [ ] Introduce Meta-Policy Engine + shadow eval.
### Phase 5
- [ ] Roll out Sentinel across internal GATOS services.

---
## 9. Status & Next Steps
- Nine Lives primitives: COMPLETE
- ADR: THIS DOCUMENT
- Next: Generate implementation roadmap (from this ADR)
- Next: Write ADR for GitLedger control-plane persistence
- Next: Implement the Sentinel skeleton

---
## 10. Appendix: Mapping Nine Lives Layers to GATOS

| **Nine Lives Layer**  | **GATOS Equivalent**               | **Notes**                           |
| --------------------- | ---------------------------------- | ----------------------------------- |
| `RetryLayer`          | CAS conflict retry / network retry | Domain-specific retry policies      |
| `TimeoutLayer`        | Request timeout                    | Protects from hung state evaluators |
| `BulkheadLayer`       | Concurrency limit                  | Protects journal from runaway loads |
| `CircuitBreakerLayer` | Node/service health                | Auto-opens on repeated failures     |
| `TelemetrySink`       | GATOS Events                       | Feeds Observer                      |
| `Observer`            | GATOS metrics/state lens           | Allows meta-policy evaluation       |
| `ControlPlaneRouter`  | GATOS administrative commands      | GitLedger-backed configuration      |
| `MetaPolicyEngine`    | GATOS autoregulation               | Self-healing behaviors              |
