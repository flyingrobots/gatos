# Chapter 5: Project Stargate: Local Enforcement & Magic Mirrors

A core challenge in any Git-based system is balancing the desire for centralized collaboration (like on GitHub) with the need for strict, low-latency enforcement of rules. If all writes must go to a central server like GitHub.com, you cannot enforce custom server-side logic. If you run your own Git server, you lose the rich UI and ecosystem of public platforms.

GATOS solves this with an architecture codenamed **Project Stargate**, a concept evolved from the `git-kv` project. It provides the best of both worlds: fast, local, and secure writes with the ability to use a public platform like GitHub as a **"Magic Mirror."**

## The Stargate Architecture

The **Stargate** is a transparent, local Git host that sits between a developer and the public remote (e.g., GitHub).

This is achieved with a simple but powerful Git configuration trick: the **`pushurl`**.

```bash
[remote "origin"]
  url     = git@github.com:org/repo.git         # Reads fetch from GitHub
  pushurl = ssh://git@stargate.local/org/repo.git  # Writes push to the local Stargate
```

To the developer, the workflow is unchanged. They still `git fetch origin` and `git push origin`. However, under the hood, the data flows differently for reads and writes.

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Stargate as Stargate (Local)
    participant Mirror as Magic Mirror (GitHub)

    Dev->>Mirror: git fetch origin (fast, scalable reads)
    Mirror-->>Dev: Returns latest state

    Dev->>Stargate: git push origin (writes are redirected)
    Stargate->>Stargate: Run pre-receive hooks (Policy, Validation)
    alt Push is Valid
        Stargate-->>Dev: Push Accepted (Success)
        Stargate->>Mirror: post-receive hook mirrors refs
    else Push is Invalid
        Stargate-->>Dev: Push Rejected (Failure)
    end
```

*   **Reads (`fetch`)** are fast and scalable, coming directly from GitHub's global CDN.
*   **Writes (`push`)** are redirected to the local Stargate server, which acts as the authoritative source of truth.

## Local-First Enforcement

By intercepting all writes, the Stargate can run powerful server-side **`pre-receive` hooks** to enforce the GATOS guarantees *before* a commit is accepted.

1.  **Policy Enforcement:** The hook can run the `gatos-policy` engine to ensure the actor has the right capabilities and that the proposed action meets all governance rules.
2.  **Attestation & Validation:** For performance, GATOS clients can attach **attestation trailers** to their commits. These trailers contain pre-computed hashes of the proposed changes. The Stargate's `pre-receive` hook can validate these trailers in **`O(1)` time**, verifying the integrity of a complex transaction without having to inspect every file.
3.  **Linear History:** The hook enforces that all journals are fast-forward only, preventing history rewrites and preserving the immutability of the ledger.

This local-first enforcement provides low-latency, high-security writes that would be impossible on a public SaaS platform.

## The Magic Mirror

After the Stargate accepts and processes a push, a **`post-receive` hook** triggers a mirroring process. The Stargate daemon pushes the newly accepted refs up to the public remote (GitHub).

This turns GitHub into a **Magic Mirror**: a read-only, eventually-consistent replica of the authoritative state held by the local Stargate.

This model provides significant benefits:
*   **Scalable Reads:** The global developer community can fetch data from GitHub's highly-available CDN without ever hitting your local server.
*   **Rich UI & Tooling:** You retain the full benefit of GitHub's ecosystem for code browsing, pull requests, issue tracking, and Actions.
*   **Read-After-Write Consistency:** While the mirror is eventually consistent, GATOS provides mechanisms for clients that need immediate consistency. A client can either read directly from the Stargate (`--read-from=stargate`) or use a `wait` command that polls until a specific commit is visible on the mirror.

## Summary

Project Stargate is the key to making GATOS a practical system for real-world teams. It provides a transparent, local-first architecture that combines the security and control of a self-hosted server with the scalability and rich ecosystem of a public platform. It allows developers to work with the familiar `git push origin` command while benefiting from a powerful, policy-driven, and auditable backend that enforces the GATOS guarantees at the source.

---

**Next**: [Chapter 6–The Message & Job Planes: Distributed Workflows](./CHAPTER-006.md)

**Prev**: [Chapter 4–The Policy Plane: Governance as Code](./CHAPTER-004.md)

---

**GATOS–_Git As The Operating Surface™_**  
James Ross / [Flying • Robots](https://github.com/flyingrobots) © 2025
