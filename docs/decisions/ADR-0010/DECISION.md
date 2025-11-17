---
Status: Draft
Date: 2025-11-09
ADR: ADR-0010
Authors: [flyingrobots]
Requires: [ADR-0002, ADR-0003, ADR-0007, ADR-0008]
Related: []
Tags: [Integration, GitHub App, CI/CD, Governance]
---

# ADR-0010: First-Class GitHub App Integration

## Scope
Define a **GitHub App** that enforces policy, brokers commands, and mirrors necessary context into the ledger.

## Rationale
Most teams live on GitHub; native enforcement and UX reduce friction.

## Decision
1. **Capabilities**:
   - Read PRs, reviews, comments; write status checks; limited content permissions.
   - Webhook ingestion → map to ledger events (`pr.opened`, `review.submitted`, etc.).
2. **Command Triggers**:
   - PR comment prefix `git mind ...` or `/gatos ...` → hits `POST /api/v1/commands`.
   - Responses posted as PR comments (summarised) + status checks.
3. **Policy Enforcement**:
   - `gatos-policy` exposes `merge_gate` check; the app **MUST** block merges until grants exist for gated actions (ADR-0003).
   - Status checks: `gatos/policy`, `gatos/fold`, `gatos/jobs`.
4. **Attestations**:
   - Job and fold proofs surfaced as artifacts/annotations with digest links.
5. **Security**:
   - Rotate app secrets, least-privilege scopes, per-repo installation.

```mermaid
sequenceDiagram
    participant Dev as GitHub Developer
    participant GH as GitHub
    participant App as GATOS App
    participant API as GATOS API
    participant Ledger

    Dev->>GH: Open/Update PR
    GH->>App: Webhook (pr.opened)
    App->>Ledger: record event
    Dev->>GH: /gatos lock acquire
    GH->>App: Command comment
    App->>API: POST /api/v1/commands {request_id}
    API-->>App: ack/job
    App-->>GH: PR comment + status check
    Ledger-->>API: merge gate satisfied
    App-->>GH: mark check success
```

## Consequences
- One consistent control point for GitHub-centric teams.
- Another moving part to maintain (secrets, webhooks, scale).

## Open Questions
- Optional mapping of GH approvals → governance approvals (off by default)?
