# Code Review Fixes: Phase 1 & 2 (BLOCKING)

**Status:** Generated 2025-11-22
**Branch:** feature/git-ledger
**Must complete before merge to main**

---

## Phase 1: Security & DoS Fixes (CRITICAL)

### 1.1 Input Validation for Git Reference Injection

- [x] **Add `validate_namespace()` function** in `crates/gatos-ledger-git/src/journal.rs`
  - Reject path traversal (`..`, `/`, `\`)
  - Reject git special chars (`:`, `*`, `?`, `[`, `~`, `^`, `@`, `{`)
  - Max length: 64 chars
  - Allow only alphanumeric, `-`, `_`
  - File: `journal.rs`
  - Lines affected: 81, 189, 294, 297

- [x] **Add `validate_actor()` function** in `crates/gatos-ledger-git/src/journal.rs`
  - Same rules as namespace
  - Max length: 128 chars
  - File: `journal.rs`
  - Lines affected: 81, 189, 294, 297

- [x] **Apply validation at entry points**
  - `append_event()`: validate `ns` and `actor` before line 81
  - `append_event_with_expected()`: validate before line 189
  - `read_window_with_ids()`: validate before lines 294, 297
  - File: `journal.rs`

- [x] **Apply validation in audit module**
  - `GitPolicyAudit::new()`: validate `ns` and `actor`
  - File: `audit.rs`, line 14-20

- [x] **Write failing tests for validation**
  - Test path traversal rejection (`../../../heads/main`)
  - Test special char rejection (`ns:evil`, `actor~1`)
  - Test empty/too-long rejection
  - File: `journal.rs` test module

- [x] **Verify tests pass after implementation**

### 1.2 ULID Validation (Commit Message Injection)

- [ ] **Add `validate_ulid()` function** in `crates/gatos-ledger-git/src/event.rs`
  - Length must be exactly 26 chars
  - Must be uppercase Crockford base32 (`0-9A-HJKMNP-TV-Z`)
  - File: `event.rs`

- [ ] **Add `EventEnvelope::validate()` method**
  - Call `validate_ulid()` on `self.ulid`
  - File: `event.rs`, after line 50

- [ ] **Call validation in append functions**
  - `append_event_with_expected()`: before line 189
  - `append_event_with_expected_and_metrics()`: before line 71
  - File: `journal.rs`

- [ ] **Write failing test for ULID injection**
  - Test newline injection: `"01ARZ3\nMalicious: evil"`
  - Test invalid chars: `"01ARZ3NDEKTSV4RRFFQ69G5F@V"`
  - Test wrong length: `"01ARZ3"`
  - File: `event.rs` test module

- [ ] **Verify tests pass**

### 1.3 Event Type Validation

- [ ] **Add `validate_event_type()` function** in `crates/gatos-ledger-git/src/event.rs`
  - Allow alphanumeric, `.`, `-`, `_` only
  - Max length: 64 chars
  - Reject newlines/control chars explicitly
  - File: `event.rs`

- [ ] **Call from `EventEnvelope::validate()`**
  - File: `event.rs`

- [ ] **Write failing test for event type injection**
  - Test newline: `"event.append\nSigned-off-by: evil"`
  - Test control chars
  - File: `event.rs` test module

- [ ] **Verify tests pass**

### 1.4 Resource Limits (DoS Prevention)

- [ ] **Add constants to `journal.rs`**
  ```rust
  const MAX_HISTORY_WALK: usize = 10_000;
  const MAX_EVENTS: usize = 10_000;
  const MAX_PAYLOAD_BYTES: usize = 1024 * 1024; // 1MB
  ```
  - File: `journal.rs`, near top

- [ ] **Add counter to `read_window_with_ids()` loop**
  - Track `walked` count
  - Error if `walked > MAX_HISTORY_WALK`
  - File: `journal.rs`, lines 314-336

- [ ] **Enforce MAX_EVENTS in `read_window()` trait impl**
  - Lines 28-36 in `journal.rs`
  - Return error if result exceeds limit
  - Suggest pagination in error message

- [ ] **Add payload size validation**
  - New method in `event.rs`: `validate_size()`
  - Check `serde_json::to_vec(&self.payload).len() <= MAX_PAYLOAD_BYTES`
  - Call from `EventEnvelope::validate()`

- [ ] **Write tests for limits**
  - Test large history rejection
  - Test large event set rejection
  - Test large payload rejection
  - Files: `journal.rs`, `event.rs` test modules

- [ ] **Verify all limit tests pass**

### 1.5 CAS Retry Backoff

- [ ] **Add backoff constants**
  ```rust
  const MAX_CAS_RETRIES: usize = 5;
  const BASE_BACKOFF_MS: u64 = 10;
  ```
  - File: `journal.rs`

- [ ] **Implement exponential backoff in `append_event_with_expected()`**
  - Replace hardcoded `3` with `MAX_CAS_RETRIES`
  - Add `std::thread::sleep(Duration::from_millis(BASE_BACKOFF_MS * 2^attempts))`
  - Add jitter: `rand::random::<u64>() % backoff`
  - File: `journal.rs`, lines 239-248

- [ ] **Implement backoff in `append_event_with_expected_and_metrics()`**
  - Same as above
  - File: `journal.rs`, lines 141-156

- [ ] **Add `rand` dependency to Cargo.toml** (if not present)
  - File: `crates/gatos-ledger-git/Cargo.toml`

- [ ] **Write test for retry behavior**
  - Mock contention scenario
  - Verify exponential backoff occurs
  - File: `journal.rs` test module

- [ ] **Verify test passes**

---

## Phase 2: API Cleanup (Before Merge)

### 2.1 Replace String-Based Errors with Typed Enum

- [ ] **Define `JournalError` enum in `gatos-ports/src/lib.rs`**
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum JournalError {
      Io(String),
      Conflict,
      NotFound(String),
      InvalidRange,
      Serialization(String),
      Validation(String),
  }
  ```
  - File: `crates/gatos-ports/src/lib.rs`, after line 103

- [ ] **Update `JournalStore` trait to use `JournalError`**
  - Change associated type documentation
  - File: `crates/gatos-ports/src/lib.rs`, line 113

- [ ] **Update `GitJournalStore` impl**
  - Change `type Error = JournalError;`
  - File: `crates/gatos-ledger-git/src/journal.rs`, line 22

- [ ] **Update all function signatures in `journal.rs`**
  - `append_event()`: `Result<String, JournalError>`
  - `append_event_with_metrics()`: `Result<String, JournalError>`
  - `append_event_with_expected()`: `Result<String, JournalError>`
  - `append_event_with_expected_and_metrics()`: `Result<String, JournalError>`
  - `read_window()`: `Result<Vec<EventEnvelope>, JournalError>`
  - `read_window_with_metrics()`: `Result<Vec<EventEnvelope>, JournalError>`
  - `read_window_with_ids()`: `Result<Vec<(Oid, EventEnvelope)>, JournalError>`
  - `read_window_paginated()`: `Result<(Vec<EventEnvelope>, Option<String>), JournalError>`
  - `write_envelope_tree()`: `Result<Oid, JournalError>`
  - Files: Multiple locations in `journal.rs`

- [ ] **Update all error returns to use enum variants**
  - `"head_conflict"` → `JournalError::Conflict`
  - `"no journal refs found"` → `JournalError::NotFound(...)`
  - `"start commit not found"` → `JournalError::NotFound(...)`
  - Git errors → `JournalError::Io(...)`
  - Serde errors → `JournalError::Serialization(...)`
  - Validation errors → `JournalError::Validation(...)`
  - File: `journal.rs`

- [ ] **Update test assertions**
  - Match on enum variants instead of strings
  - File: `journal.rs` test module

- [ ] **Verify all tests pass**

### 2.2 Extract Magic Constants

- [ ] **Define ledger signature constants**
  ```rust
  const LEDGER_AUTHOR: &str = "gatos-ledger";
  const LEDGER_EMAIL: &str = "ledger@gatos.local";
  ```
  - File: `journal.rs`, near top

- [ ] **Use constants in `append_event_with_expected()`**
  - Replace hardcoded strings at line 187
  - File: `journal.rs`

- [ ] **Use constants in `append_event_with_expected_and_metrics()`**
  - Replace hardcoded strings at line 79
  - File: `journal.rs`

- [ ] **Use constants in `audit.rs`**
  - Replace hardcoded strings at line 30
  - File: `audit.rs`

- [ ] **Extract DAG-CBOR codec constant**
  ```rust
  /// IPLD DAG-CBOR codec (multicodec registry)
  const DAG_CBOR_CODEC: u64 = 0x71;
  ```
  - File: `event.rs`, make existing constant public or add comment

- [ ] **Verify constants are used consistently**

### 2.3 Consolidate Duplicate Append Functions

- [ ] **Create internal helper: `append_event_internal()`**
  - Takes optional `metrics: Option<&M>`
  - Contains all append logic once
  - File: `journal.rs`

- [ ] **Refactor `append_event_with_expected()`**
  - Call `append_event_internal(repo, ns, actor, envelope, expected_head, None)`
  - File: `journal.rs`

- [ ] **Refactor `append_event_with_expected_and_metrics()`**
  - Call `append_event_internal(repo, ns, actor, envelope, expected_head, Some(metrics))`
  - File: `journal.rs`

- [ ] **Verify all existing tests still pass**
  - No behavior change, just refactor
  - File: `journal.rs` test module

---

## Acceptance Criteria

**Phase 1 Complete When:**
- [ ] All input validation in place (ns, actor, ulid, event_type)
- [ ] All resource limits enforced (history walk, events, payload)
- [ ] Exponential backoff implemented for CAS retries
- [ ] All new validation tests passing
- [ ] All existing tests still passing
- [ ] `./scripts/test.sh` passes with 0 failures

**Phase 2 Complete When:**
- [ ] `JournalError` enum replaces all `String` errors
- [ ] All magic constants extracted
- [ ] Duplicate append code consolidated
- [ ] All tests updated and passing
- [ ] No compilation warnings
- [ ] `./scripts/test.sh` passes with 0 failures

---

## Testing Checklist

After all fixes:

- [ ] Run full test suite: `./scripts/test.sh`
- [ ] Verify no new warnings: `cargo clippy --all-targets`
- [ ] Check formatting: `cargo fmt --check`
- [ ] Security review validation tests pass
- [ ] Performance regression tests pass (if applicable)
- [ ] Documentation builds: `cargo doc --no-deps`

---

## Commit Strategy

1. **Commit after Phase 1.1-1.3**: "feat: add input validation for ns, actor, ulid, event_type"
2. **Commit after Phase 1.4**: "feat: add resource limits to prevent DoS"
3. **Commit after Phase 1.5**: "feat: add exponential backoff to CAS retries"
4. **Commit after Phase 2.1**: "refactor: replace String errors with JournalError enum"
5. **Commit after Phase 2.2**: "refactor: extract magic constants"
6. **Commit after Phase 2.3**: "refactor: consolidate duplicate append functions"

---

**Total Tasks:** 61 checkboxes
**Estimated Time:** 4-6 hours with pair programming
**Priority:** BLOCKING - do not merge without completion
