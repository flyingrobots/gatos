# GATOS Agent Playbook

Guidelines for contributors/agents working inside this repository.

## Development Workflow

- **Tests are the spec**: Write (and run) failing tests before implementing new behavior. Unit/integration tests define the contract.
- **Hexagonal architecture**: Expose behavior via traits/interfaces. Message Plane, Ledger, Job Plane, etc. should depend on ports, not concrete libs.
- **Frequent, small commits**: Each slice of work (e.g., adding a test, wiring a module) should be committed separately to keep history reviewable.
- **Forbidden git-fu**: Do **not** rebase, squash, amend, or force-push branches in this repo. Use linear commits only.

- **Timestamps**: Use POSIX epoch seconds for all internal metadata and commit-related timestamps to avoid timezone/locale ambiguity. No local timezones, ever.

## Technical Conventions

- Use POSIX timestamps (epoch seconds) for internal metadata to avoid timezone ambiguity.
- When writing to Git, go through the ledger abstractions (EventEnvelope, CAS helpers) wherever possible so commits share canonical trailers and audit behavior.
- Topic/segment names must be sanitized (alphanumeric + `-`/`_`, no `..`).

## Checklist Mindset

Before closing any task:

1. Tests capture the desired behavior.
2. Implementation follows hexagonal principles (interfaces over concrete types).
3. Commit history reflects incremental progress (no giant “everything” commits).
4. No disallowed git commands were used.

Feel free to extend this document as our processes evolve.

**IMPORTANT:** Do **NOT** run tests on the host machine. Tests touch git refs and could corrupt your working repo—use the Docker harness (which copies the repo and rewrites remotes) so you don't become 'that person' who clobbers origin.

- **Determinism is critical**: stick to canonical encodings, POSIX timestamps, and deterministic APIs (no NASA meter/inch mishaps).
