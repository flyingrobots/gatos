# Message Plane: Docs & SDK Polish

- **Status:** TODO
- **Area:** Docs / SDKs / CLI UX
- **Owner:** Triage
- **Context:** Core behavior is implemented, but developer-facing docs and SDK scaffolding lag.

## Tasks
- Add CLI examples and usage blocks to gatosd help and docs/guide chapters.
- Add JSON schema (or example payload) for `messages.read` responses (canonical_json base64, next_since, etc.).
- Publish minimal client helpers (Rust/TS) or examples showing resume with checkpoints.
- Update CHANGELOG/ROADMAP to reflect shipped pieces and remaining gaps.

## Definition of Done
- Docs contain runnable examples; SDK helpers (or clear examples) exist for reading/publishing with checkpoints.
- Spec/TECH-SPEC references are consistent with current implementation.
