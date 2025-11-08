Okay, I will read the documentation and source files to give you an overview of the project. First, I'll locate all the relevant files.
Right, extensive documentation. I'll read it all. Now to the source code in `core/` and `include/`.
`core/src` for source, `core/include/` for headers. I'll verify.
Source structure confirmed. Listing headers in `core/include/`.
Headers are in `core/include/gitmind/`. I'll check the contents.
Okay, I understand the file structure: `docs/`, `core/include/gitmind/` and `include/` for headers, and `core/src/` for sources. I'll now read all files in these directories using `read_many_files`, starting with `docs`.
`read_many_files` needs globs, not just directories. I'll use `docs/**/*.md` for the markdown files and also fetch the non-markdown files separately. I'll also list the contents of `include/`.
Okay, I'll re-read the files using specific globs: all markdown and other files in `docs/`, all source in `core/src/`, all headers in `core/include/`, and the header in `include/`. Starting with `docs/` markdown.

Okay, I've reviewed the `docs/` markdown. Now to the source code. I'll read `core/src/`, `core/include/`, `include/`, and `apps/cli/` using `read_many_files`, starting with `core/src/`.
`read_many_files` is failing. I'll try a more specific glob, targeting all `.c` and `.h` files within the `core/` directory.

I've read the documentation and the source code. I will now synthesize this information to provide a detailed overview of the project.

Based on the documentation and source code, here is a detailed overview of the `git-mind` project:

### **Project Goal & Core Idea**

`git-mind` is a "Git-native semantic graph system." Its core idea is to treat the relationships and understanding *about* code, documentation, and other project artifacts as first-class citizens that can be versioned, branched, and merged just like the code itself. It aims to create a "persistent knowledge layer" for software development, capturing the "why" and "how" behind the code, not just the "what."

The project's philosophy is that your Git repository *is* the database. It avoids external servers and dependencies, making it a serverless, distributed, and forkable system for tracking the evolution of thought and understanding within a project.

### **How It Works: The Two-Layer Architecture**

`git-mind` uses a two-layer architecture to store and retrieve semantic information:

1.  **The Journal (Source of Truth):**
    *   **Storage:** Semantic relationships, called "edges," are stored as individual Git commits. These commits live on a dedicated, branch-specific ref, such as `refs/gitmind/edges/main`. This means each branch has its own independent history of semantic edges.
    *   **Edge Format:** Each edge is a small data structure containing the source and target of the relationship (identified by their Git blob OIDs), the type of relationship (e.g., `implements`, `tests`), a confidence score, a timestamp, and the original file paths for human readability.
    *   **Serialization:** This edge data is serialized into a compact binary format called **CBOR (Concise Binary Object Representation)** and then Base64-encoded to be stored safely in the Git commit message.
    *   **Immutability:** The journal is append-only. New edges create new commits, and "deletions" are handled by creating "tombstone" edges (edges with negative confidence), preserving a complete, auditable history.

2.  **The Cache (Performance Layer):**
    *   **Purpose:** To provide fast queries (<10ms) over a potentially large number of edges. The cache is optional and can be rebuilt from the journal at any time.
    *   **Technology:** It uses **Roaring Bitmaps**, a highly efficient compressed bitmap implementation, to create indexes.
    *   **Structure:** The cache is also stored in a branch-specific Git ref, like `refs/gitmind/cache/main`. It contains indexes that map Git object OIDs to a set of internal edge IDs. The cache is sharded by the prefix of the OID to keep lookups fast.
    *   **Lifecycle:** The cache is considered a disposable, local optimization. It is never pushed to a remote repository. It can become stale if the journal is updated, and the system includes logic to detect this and trigger a rebuild.

### **Core Features**

*   **Semantic Edges:** The fundamental unit of knowledge is the "edge," a directed link between two Git objects (e.g., `file_A.c --[implements]--> spec.md`).
*   **Names-as-Truth:** Relationship types (like `implements`) and "lanes" (like `verified` or `draft`) are stored as human-readable strings directly on the edge. Numeric IDs are derived only for performance in the cache.
*   **Attribution:** Every edge can be attributed to its creator, whether a human, a specific AI model (like Claude or GPT), or the system itself. This allows for filtering and building trust in different sources of information.
*   **AUGMENTS System:** This is a key automated feature. A `post-commit` Git hook automatically creates `AUGMENTS` edges to track the evolution of a file. When a file is modified, an edge is created from the old blob OID to the new blob OID, preserving the semantic history of the content.
*   **Time-Travel Queries:** Because the journal is a series of Git commits, you can check out any point in history and see the state of the knowledge graph at that time.
*   **Hexagonal Architecture (Ports and Adapters):** The C codebase is being migrated to a modern, modular architecture. This separates the core domain logic (the "what") from the infrastructure details like Git access, filesystem operations, and logging (the "how"). This makes the code more testable, maintainable, and portable.
*   **CLI and Library:** The project is designed as both a command-line tool (`git mind`) for users and a C library (`libgitmind`) that can be embedded in other applications.

### **What is it designed to be used as?**

`git-mind` is designed to be a foundational tool for **code intelligence** and **knowledge management** within a software development lifecycle. Its primary use cases appear to be:

1.  **A Developer's "Second Brain":** To explicitly capture the mental model of a codebase. A developer can link a function to the requirement it implements, a test to the code it covers, or a piece of documentation to the architecture it describes.
2.  **Human-AI Collaboration:** It provides a structured way for AI assistants to contribute to the understanding of a codebase. An AI can suggest relationships, and these suggestions can be stored in a separate "lane" (e.g., `suggested`) to be reviewed and "promoted" by a human. The attribution system ensures it's always clear who or what made a suggestion.
3.  **Code Archeology and Onboarding:** By traversing the graph and its history, new developers can quickly understand the architecture, dependencies, and rationale behind different parts of a system. The `AUGMENTS` system ensures this understanding isn't lost even as files are refactored.
4.  **A Serverless, Distributed Knowledge Base:** Because it's built on Git, the knowledge graph can be cloned, forked, branched, and merged. Teams can work on their own "branches of understanding" and then merge them, just like code.

In essence, `git-mind` aims to make the implicit knowledge held in developers' heads an explicit, version-controlled, and queryable part of the repository itself.
