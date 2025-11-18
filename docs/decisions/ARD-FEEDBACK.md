# ADR Feedback Checklist

## ADR-0007 – Expand evolution story & error visuals

- [x] Resolved

> [!WARNING]- **Recommendation**
>
> Document a concrete GraphQL field deprecation schedule (e.g., warning periods, sunset dates) and add an error-handling sequence/flow diagram so operators can anticipate failure paths.
> 
> ### LLM Prompt
> 
> ```text
> You are a GraphQL API architect. Draft a “Schema Evolution & Error Surfacing” section for ADR-0007 that:
> 
> 1. Defines deprecation phases (announce, dual-serve, removal) with timelines in weeks.
> 2. Explains how errors propagate (policy denied vs. invalid ref) with an auxiliary diagram description.
> 
> Keep the tone consistent with existing ADRs.
> ```
>

---

## ADR-0008 – Flesh out auth scopes & webhook DLQ operations

- [x] Resolved

> [!WARNING]- **Recommendation**
>
>
> Add a table mapping OAuth scopes → command prefixes and specify observability/cleanup for the webhook dead-letter queue (visibility APIs, retention, alerting).
>
> ### LLM Prompt
>
> ```text
You are documenting a REST command surface. Produce a subsection for ADR-0008 covering:
>
> - A table mapping OAuth scopes to command/resource prefixes.
> - DLQ management: how operators list, replay, or purge failed webhooks, including retention guarantees.
>
>
> Output markdown suitable for an ADR.
> ```

---

## ADR-0009 – Clarify replay/backpressure across federated nodes

- [x] Resolved

> [!WARNING]- **Recommendation**
>
> Describe how sequence IDs and buffering work when streams traverse multiple nodes, and augment diagrams with replay/error/credit flows.
>
> ### LLM Prompt
> 
> ```text
Pretend you run a multi-node ref streaming service. Explain for ADR-0009:
>
> 1. How seq/credit propagation works when a client connects through a federation proxy.
> 2. Failure handling when replay windows expire mid-hop.
>
> Include guidance for an updated diagram (text description sufficient).
> ```

---

## ADR-0010 – Resolve PR approvals & multi-repo watcher strategy

- [x] Resolved

> [!WARNING]- **Recommendation**
>
>
> Decide whether GitHub approvals can satisfy governance approvals and document how installations spanning many repos share watcher queues/check workloads.
>
> ### LLM Prompt
> 
> ```text
> As the GitHub App owner, write a section for ADR-0010 that:
>
> - States the policy for mapping PR approvals to governance approvals (allowed? constraints?).
> - Details queue partitioning when one installation manages N repos (e.g., sharded workers, priority order).
>
> Provide actionable guidance.
> ```

---

## ADR-0011 – Add security/scale envelope for exports

- [x] Resolved

> [!WARNING]- **Recommendation**
> 
> Describe how sensitive data is redacted, expected dataset sizes / resource requirements, and include a sample manifest/table appendix to remove ambiguity.
> 
> ### LLM Prompt
> 
> ```text
> Channel your inner data engineer. Extend ADR-0011 with:
> 
> - A “Security & Resource Envelope” section (storage limits, IAM expectations, pointer redaction guarantees).
> - An example export manifest snippet plus one CREATE TABLE statement.
> 
> Keep everything deterministic.
> ```

---

## ADR-0012 – Decide on federation discovery/gossip

- [x] Resolved

> [!WARNING]- **Recommendation**
>
> Either commit to manual-only `.gatos/federation.yaml` or outline the gossip/discovery protocol (trust anchors, rate limits) so operators know what to expect.
>
> ### LLM Prompt
>
> ```text
> Act as the federation architect. Draft text for ADR-0012 that answers:
> 
> - Do we support automatic mount discovery? If yes, describe the gossip protocol, trust requirements, and operator controls. If no, justify manual-only.
> 
> Provide clear, testable language.
> ```

---

## ADR-0013 – Decide on prewarming & shared caches

- [x] Resolved

> [!WARNING]- **Recommendation**
>
> Specify whether background prewarming is supported and define how cache stores behave across multiple worktrees/nodes (shared path, eviction policy).
>
> ### LLM Prompt
>
> ```text
> From the perspective of the fold-engine owner, write guidance for ADR-0013 covering:
> 
> - Policy for background prewarming (allowed? triggers? safeguards?).
> - Strategy for sharing caches between worktrees/nodes, including locking and eviction rules.
> 
> Output concise markdown bullets.
>```
