Based on the documentation and source code, here is a detailed overview of the Shiplog project.

### **Project Goal and Core Idea**

Shiplog's primary goal is to create a **secure, auditable, and Git-native ledger for deployment and operational events**. The core idea is to treat deployment logging not as an external process but as an intrinsic part of the codebase's history, living alongside the code but separate from the source branches.

It solves the "2 AM incident" problem: when something breaks, the "who, what, where, when, why, and how" of a deployment are often scattered across ephemeral logs, CI/CD dashboards, and chat messages. Shiplog centralizes this information into a **cryptographically signed, append-only log** stored directly within the project's Git repository.

It is designed to be a **deployment primitive**, a fundamental building block that integrates with existing workflows rather than replacing them. You keep your deployment scripts and tools, but you wrap them with `git shiplog run` to create an immutable record of their execution.

### **How It Works: The Git-Native Approach**

Shiplog leverages Git's own data model to store its information, avoiding any need for external databases, services, or SaaS products. All data is stored in Git's `refs` namespace, which keeps it separate from your source code branches (like `main`).

Here's the key ref structure:

*   **Journals (`refs/_shiplog/journal/<env>`)**: This is the append-only log of all deployment events for a specific environment (e.g., `prod`, `staging`). Each entry is a Git commit containing structured metadata about an event.
*   **Policy (`refs/_shiplog/policy/current`)**: This ref points to a commit containing the `policy.json` file. This file defines the rules of who can do what, such as which users are allowed to write to the log and signature requirements.
*   **Trust (`refs/_shiplog/trust/root`)**: This points to the root of trust for cryptographic verification. It contains the list of trusted signers (GPG or SSH public keys) and the multi-signature policy (e.g., requiring a quorum of maintainers).
*   **Notes (`refs/_shiplog/notes/logs`)**: Large outputs, like the `stdout` and `stderr` from a command, are stored as Git "notes," which are attachments to the journal commits. This keeps the journal entries themselves lightweight.

This "Git all the way down" approach means that your deployment history is versioned, backed up, and replicated just like your code.

### **Core Features**

1.  **Command Wrapping (`git shiplog run`)**: The flagship feature. You can wrap any command (e.g., `kubectl apply`, `terraform plan`, a shell script) with `git shiplog run`. Shiplog captures its `stdout`, `stderr`, exit code, duration, and execution context (user, timestamp, reason) and records it as a signed entry in the journal.

2.  **Policy as Code**: Shiplog enforces rules defined in a `policy.json` file. This includes:
    *   **Allow Lists**: Defining which users (by email) are permitted to write entries for a given environment.
    *   **Signature Requirements**: Enforcing that entries must be cryptographically signed.
    *   **Per-Environment Policies**: Different environments can have different rules (e.g., `prod` is stricter than `dev`).

3.  **Multi-Signature Quorum (Trust Modes)**: For high-stakes environments, Shiplog supports requiring multiple maintainers to approve changes to the trust policy itself. It offers two modes:
    *   **`chain` mode**: Requires multiple maintainers to co-sign a Git commit to the trust ref. This is a very Git-native workflow.
    *   **`attestation` mode**: Maintainers use SSH keys to sign a canonical representation of the trust data (`ssh-keygen -Y verify`). This is more flexible for automation and CI/CD systems.

4.  **Human and Machine-Readable Output**: The CLI provides clean, human-readable tables (`git shiplog ls`) and detailed views (`git shiplog show`). At the same time, every command supports structured output (`--json`, `--json-compact`, `--jsonl`) for easy integration with scripts, dashboards, and monitoring tools.

5.  **SaaS vs. Self-Hosted Enforcement**: Shiplog understands the difference between Git hosting environments.
    *   **Self-Hosted (e.g., GitHub Enterprise, GitLab)**: You can install a `pre-receive` hook on the server to enforce policy directly.
    *   **SaaS (e.g., GitHub.com)**: Since you can't run custom server hooks, Shiplog guides you to use the platform's native features (like GitHub's Branch Protection Rules and Required Status Checks) to achieve the same level of security.

6.  **Interactive Configuration (`git shiplog config --interactive`)**: A setup wizard that asks questions about your hosting environment and security needs, then generates a recommended policy and configuration plan.

### **What It Is Designed To Be Used As**

Shiplog is designed to be a **system of record for production changes**. It's not a CI/CD runner or a deployment tool itself, but rather a universal audit layer that sits alongside them.

Its primary use cases are:
*   **Auditing and Compliance**: Creating an unimpeachable, signed history of all production events to meet security and compliance requirements.
*   **Incident Response**: Quickly determining what changed, who changed it, and why, by consulting a single, trusted source of truth.
*   **DevOps Automation**: Providing a stable, scriptable interface for recording and querying deployment metadata, enabling more advanced automation and GitOps workflows.
*   **Change Management**: Enforcing formal approval and sign-off for critical operations through its policy and multi-signature features.

In essence, Shiplog brings the discipline and cryptographic certainty of Git to the world of operations, creating a durable, decentralized, and developer-friendly deployment ledger.
