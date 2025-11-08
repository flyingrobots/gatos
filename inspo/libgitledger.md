Got it. I will now read the specified documentation and source code to provide a detailed overview of the libgitledger project.

Based on a thorough review of the project's documentation and source code, here is a detailed overview of `libgitledger`.

### Project Overview and Goals

`libgitledger` is a portable, embeddable C library designed to create **Git-native, append-only ledgers**. The primary goal is to provide the cryptographic guarantees of a blockchain (immutability, non-repudiation, auditability) without introducing new infrastructure, by using a standard Git repository as the underlying database.

It is the reference implementation of the **Ledger-Kernel specification**, a formal model for building verifiable, deterministic state machines on top of Git's object model. The project fuses concepts from two prior systems: `shiplog` (providing the policy and trust model) and `git-mind` (providing the hexagonal architecture and high-performance indexing concepts).

The core problem it solves is the need for tamper-evident provenance for critical processes like software deployments, supply-chain attestations, configuration history, or schema registries, without the complexity and overhead of a full blockchain or reliance on a third-party SaaS vendor.

### Core Ideas and Principles

The project is built on a foundation of strong principles:

1.  **Git-Native:** The Git object store and its references (`refs`) are the database. There are no external databases or daemons. A ledger is a Git ref, and an entry is a Git commit.
2.  **Append-Only Immutability:** The ledger's history is immutable. This is enforced by ensuring that the Git reference representing the ledger can only be updated in a "fast-forward" manner, programmatically rejecting any history rewrites or rebases.
3.  **Policy as Code:** Each ledger's behavior is governed by a `policy.json` document stored within Git. This document defines rules such as which users are allowed to write entries, maximum payload sizes, and signature requirements.
4.  **Cryptographic Trust and Attestation:** A `trust.json` document defines a set of trusted maintainers and signers. Every entry in the ledger is cryptographically signed, providing non-repudiable proof of authorship and integrity. The system supports multi-signature thresholds (N-of-M) for managing changes to the trust document itself.
5.  **Deterministic Replay:** The state of any ledger can be perfectly and deterministically reconstructed by processing its entries in order from the beginning. This guarantees that anyone with read access can independently verify the ledger's integrity and final state.
6.  **Hexagonal Architecture (Ports and Adapters):** The core domain logic is written in pure, portable C and is completely decoupled from external dependencies. All interactions with the outside world (like `libgit2`, the filesystem, or logging) happen through abstract "ports," with concrete "adapters" providing the implementation. This makes the library highly testable, portable, and flexible.

### How It Works: Architecture and Data Model

`libgitledger` organizes its data within a dedicated Git namespace:

*   **Journal:** `refs/gitledger/journal/<ledger_name>` - The main ref for a ledger, where each commit is an entry.
*   **Policy & Trust:** `refs/gitledger/policy/<ledger_name>` and `refs/gitledger/trust/<ledger_name>` - Refs pointing to the current policy and trust documents.
*   **Cache:** `refs/gitledger/cache/<ledger_name>` - A ref for a rebuildable high-performance query index.
*   **Notes:** `refs/gitledger/notes/<ledger_name>` - A namespace for attaching arbitrary data (like logs or artifacts) to ledger entries using Git's `notes` mechanism.

The process of adding a new entry is atomic and safe:

1.  A payload is formatted into a commit message by a pluggable "encoder."
2.  The library checks the operation against the ledger's policy (e.g., is the author allowed?).
3.  It verifies the cryptographic signature against the trust document.
4.  A new Git commit is created, with its parent being the current head of the journal.
5.  The library attempts to update the journal ref using an atomic fast-forward operation. If another writer has updated the ledger in the meantime, this operation fails with a conflict error, which allows the client to retry safely.

### Core Features

The library is planned to provide a comprehensive C API, including:

*   **Ledger Lifecycle:** Opening and closing ledgers within a Git repository.
*   **Append and Read:** Securely appending new entries and reading existing ones.
*   **Policy and Trust Management:** Programmatically getting and setting the JSON documents that govern the ledger.
*   **High-Performance Querying:** An advanced query engine based on **Roaring Bitmaps** to allow for fast, boolean searches over terms extracted from ledger entries. This cache is entirely rebuildable from the journal.
*   **Integrity Verification:** A "deep verify" function to audit the entire history of a ledger, re-checking all parent links, policies, and signatures for every entry.
*   **Pluggable Components:** Users can provide their own functions for memory allocation, logging, and, most importantly, encoding payloads and indexing terms for queries.

### Intended Use and Applications

`libgitledger` is a foundational library intended for building applications that require a high degree of trust and verifiable history. The documentation points to several "Edge Systems" that can be built on top of it:

*   **Shiplog:** A tool for creating a tamper-evident log of software deployments and operational commands (`git shiplog run <command>`).
*   **Git-Mind:** A tool for creating version-controlled semantic knowledge graphs within Git.
*   **Wesley:** A "data layer compiler" that could use the ledger to immutably track database schema migrations.

In essence, it is designed for any scenario that benefits from an immutable log, such as supply-chain security, configuration management, and internal audit trails.

### Project Status and Roadmap

The project is in the early stages of development but is meticulously planned. The repository contains detailed planning documents, issue breakdowns, and a dependency graph for its milestone-driven roadmap.

*   **Current State:** The initial scaffolding, dual build systems (CMake and Meson), CI/CD pipelines, and core data types are in place.
*   **Roadmap:** The project plan is broken into clear milestones (M0-M7), progressing from core functionality (M2: Append/Read) to security (M3-M4: Policy/Signatures) and finally to advanced features and hardening (M6-M7: Query Index/Verification).
*   **Quality and Portability:** There is a strong focus on code quality, with strict coding standards, a containerized test environment, and even a successful proof-of-concept of building the library without the standard C library (`-nostdlib`), highlighting its portability.