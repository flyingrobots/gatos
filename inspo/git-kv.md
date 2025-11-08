Based on the documentation you've provided, here is a detailed overview of the `git-kv` (Project Stargate) project.

### Project Goals & Core Idea

**`git-kv` is an auditable, versioned Key-Value store that uses Git *as* the database.**

The core idea is to leverage the power of Git's immutable, versioned history to create a "ledger-grade" database for storing configuration, policies, and application state. It is explicitly **not** a replacement for high-throughput systems like Redis but is designed for workflows where a complete and verifiable audit trail is more important than microsecond latency.

The project's tagline encapsulates this philosophy: **"State as code. Commits are transactions. Git is the database. GitHub is the showroom."**

### How It Works: The "Stargate" Architecture

The central innovation of `git-kv` is the **Stargate**, a transparent gateway that intercepts write operations (`git push`) while allowing read operations (`git fetch`) to go directly to a standard Git host like GitHub. This provides the best of both worlds: the speed and availability of a CDN-like service for reads, and the strong guarantees of a centralized, controlled system for writes.

This is achieved with a simple Git configuration on the developer's machine:

*   **`remote.origin.url`** (for reads) points to GitHub.
*   **`remote.origin.pushurl`** (for writes) points to a self-hosted **Stargate** service.

When a developer pushes, the request goes to the Stargate, which runs a series of checks and validations in a `pre-receive` hook before accepting the commit. Once accepted, a `post-receive` hook on the Stargate mirrors the changes back to GitHub, making them visible to everyone else.

### Core Features

1.  **Ledger-Grade History:** Every change is a signed, immutable Git commit. The history is linear, with no merge commits or force-pushes allowed, creating a verifiable audit trail.

2.  **Atomic Multi-Key Transactions:** `git-kv` supports setting and deleting thousands of keys in a single, atomic transaction using the `git kv mset` command. The Stargate validates these complex transactions in constant time (`O(1)`) using **client attestation**, where the client includes cryptographic proof of the transaction's validity in the commit message itself.

3.  **Fast Prefix Listing:** To avoid scanning the entire repository for keys, `git-kv` maintains a separate `refs/kv-index/...` ref. This acts as a dedicated index, allowing for fast key lookups by prefix (e.g., `git kv list --prefix user:`).

4.  **Native Large File Support (Chunking):** Values larger than a configured threshold (e.g., 1MB) are automatically broken into smaller pieces using **Content-Defined Chunking (FastCDC)**. These chunks are stored as regular Git objects, and a "manifest" file is created in their place. This provides efficient, Git-native deduplication without relying on Git-LFS.

5.  **Bounded Clone Size (Epochs):** To prevent repositories from growing infinitely large, `git-kv` uses a concept of **Epochs**. An administrator can create a new epoch (`git kv epoch new`), which essentially snapshots the database. New clones only need to fetch the current epoch, keeping them small and fast, while history across epochs remains traversable.

6.  **Policy Enforcement:** A `.kv/policy.yaml` file in the repository allows administrators to define rules that are enforced by the Stargate. These policies can control:
    *   Who is allowed to write (based on SSH/GPG keys).
    *   What key prefixes are allowed.
    *   Maximum value sizes.
    *   Whether Git LFS is forbidden.

7.  **High Availability (HA) & Resilience:** The Stargate is designed to be a critical service. It supports an active-passive HA model with leader election, health check endpoints (`/healthz`, `/readyz`), and detailed metrics for monitoring. It also includes tools for recovering from "split-brain" scenarios where the GitHub mirror might diverge from the Stargate.

8.  **Read-After-Write (RYW) Guarantees:** Because the GitHub mirror is eventually consistent, `git-kv` provides two mechanisms for ensuring you can read a value immediately after writing it:
    *   `git kv wait --oid <hash>`: A command that pauses a script until a specific commit is visible on the GitHub mirror.
    *   `git kv get --read-from=stargate`: A flag that tells the `get` command to read directly from the authoritative Stargate instead of the potentially stale mirror.

### What is it designed to be used as?

`git-kv` is designed to be a foundational piece of infrastructure for a variety of use cases where auditable, versioned state management is critical. The documentation highlights several examples:

*   **Audit-grade Feature Flags & Policy Toggles:** Atomically flip multiple feature flags with a signed commit history.
*   **Regulated Configuration & Compliance Evidence:** Store compliance artifacts and configuration for regulated environments where every change must be tracked.
*   **ML Model Registry:** Store model binaries and manifests using the native chunking, providing reproducibility by commit hash.
*   **GitOps State for Air-Gapped Environments:** Use the Stargate as the source of truth in a secure zone, with a public mirror for visibility.
*   **Secrets Rotation Records:** Store encrypted secrets and maintain a verifiable ledger of when and by whom they were rotated.
*   **Data Pipeline Checkpoints:** Atomically write manifests at each stage of a data pipeline, allowing downstream consumers to wait for updates.

In summary, **`git-kv` is a sophisticated, Git-native database designed for storing critical, slow-moving data that requires a high degree of trust, auditability, and operational control.** It cleverly combines the developer-friendly ergonomics of Git and GitHub with the strong integrity guarantees of a centralized, policy-enforcing gateway.
