# Chapter 7: Federation & Mind-Melds: Cross-Repo Operations

GATOS is designed as a distributed system where each Git repository is a self-contained, sovereign node. However, the true power of the architecture is realized when these independent nodes interact. This chapter explores how GATOS enables **federation** and deterministic state merging between repositories, even when they don't share a direct line of history.

## Federation: A Network of GATOS Nodes

A GATOS **federation** is a network of independent GATOS repositories that have agreed to share some portion of their state or policy. For example, a central "governance" repository could define policies that are consumed by dozens of "project" repositories.

This is achieved through the Message Plane (`gatos-mind`) and the State Plane's ability to read from multiple sources. A project repository can subscribe to the `gatos.policy.updated` topic on the governance repository. When a new policy is published, the project node can fetch it, validate it, and incorporate it into its own local policy engine.

## The Challenge: Merging Divergent Realities

A more complex problem arises when two repositories have independently evolved and need to merge their states. A standard `git merge` is insufficient because it operates on lines of text and has no understanding of the semantic meaning of the GATOS state model. A simple merge could lead to a corrupted or inconsistent state.

GATOS solves this with a concept called a **Mind-Meld**, a deterministic, conflict-free merge of two distinct **shapes** (state snapshots). It allows two separate operating surfaces to be folded together into a new, unified surface.

## The Mind-Meld: A Pushout in Practice

The Mind-Meld is the practical application of a mathematical concept from category theory called a **pushout**. You don't need to understand the deep theory to grasp the outcome.

Imagine two GATOS repositories, A and B, that have diverged. To merge them, you need a third piece of information: a **schema manifest (S)** that describes the correspondences between the two systems. It tells the meld engine how to map an object in repository A to its equivalent in repository B.

The meld operation is a pure function that takes these three inputs (Shape A, Shape B, and Schema S) and produces a single, new, deterministically-created shape, AB.

```mermaid
graph TD
  S[Schema Manifest]
  Sh_A[Shape A]
  Sh_B[Shape B]
  Sh_AB[Meld: Shape AB]

  S --> Sh_A
  S --> Sh_B
  Sh_A --> Sh_AB
  Sh_B --> Sh_AB

  classDef s fill:#f9f,stroke:#333,stroke-width:2px;
  classDef a fill:#9cf,stroke:#333,stroke-width:2px;
  classDef ab fill:#9c9,stroke:#333,stroke-width:2px;
  class S s;
  class Sh_A,Sh_B a;
  class Sh_AB ab;
```

Because the meld is a deterministic fold, any node that performs the operation with the same three inputs will arrive at the exact same resulting shape and, therefore, the same `state_root` hash.

### Proof-of-Meld

The output of a successful meld is not just the new shape, but also a **Proof-of-Meld**. This is a lightweight, attestable certificate that contains the hashes of the two input shapes and the schema manifest.

`Proof-of-Meld = BLAKE3(root(Shape A) || root(Shape B) || root(Schema S))`

This proof can be recorded in the Ledger Plane as a verifiable record that the merge occurred, linking the three previously independent histories into a new, unified timeline.

## Use Cases

This capability unlocks powerful workflows for decentralized collaboration:

*   **Cross-Repo Governance:** A central governance repo can define policies, and project repos can "meld" those policies into their own, ensuring consistent rules across an organization without a central server.
*   **Supply Chain Attestation:** A software artifact (like a Docker image) can have its own GATOS repository containing its SBOM and test results. A project that consumes this artifact can meld the artifact's state into its own, creating a verifiable, end-to-end chain of provenance.
*   **Distributed Knowledge Graphs:** Two researchers using `git-mind` can meld their knowledge graphs, combining their understanding into a new, richer graph that contains the union of their insights.

## Summary

Federation and Mind-Melds are what make GATOS a truly distributed operating surface. They provide a mathematically sound and verifiable way to compose and merge state across independent systems, moving beyond the limitations of a single repository. This enables a new class of decentralized applications where trust is established not by a central authority, but by shared, deterministic mathematics.

## Worked Mind‑Meld Example

Repo A (users):

```json
{ "user": { "id": 1, "email": "a@example.org" } }
```

Repo B (accounts):

```json
{ "account": { "id": 1, "primaryEmail": "a@example.org" } }
```

Schema Manifest (correspondences):

```json
{
  "map": [
    { "from": "user.id", "to": "account.id" },
    { "from": "user.email", "to": "account.primaryEmail" }
  ]
}
```

Resulting AB shape merges the two into a single entity with a consistent id/email. `Proof‑of‑Meld = blake3(root(Shape A) || root(Shape B) || root(Schema))`.

Schemas are versioned and signed under `refs/gatos/schemas/<ns>` and distributed like other state. See ADR‑0012 (Federation/Mounts) and ADR‑0011 (Exporter) for normative storage and export rules.
