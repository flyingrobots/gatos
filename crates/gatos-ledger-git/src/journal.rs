//! Git-backed ledger journal append/read with CAS semantics.

use git2::Oid;
use git2::Repository;
use git2::Signature;

use crate::event::EventEnvelope;

/// Validate namespace parameter to prevent git reference injection.
///
/// Rules:
/// - Length: 1-64 characters
/// - Allowed: alphanumeric, hyphen, underscore
/// - Rejected: path traversal (`.`, `..`, `/`, `\`)
/// - Rejected: git special chars (`:`, `*`, `?`, `[`, `~`, `^`, `@`, `{`)
pub(crate) fn validate_namespace(ns: &str) -> Result<(), String> {
    const MAX_NS_LEN: usize = 64;

    if ns.is_empty() {
        return Err("namespace cannot be empty".into());
    }

    if ns.len() > MAX_NS_LEN {
        return Err(format!("namespace exceeds max length {}", MAX_NS_LEN));
    }

    // Reject path traversal sequences
    if ns.contains("..") || ns.contains('/') || ns.contains('\\') || ns.contains('.') {
        return Err(format!("invalid namespace '{}': path traversal not allowed", ns));
    }

    // Reject git special characters
    const GIT_SPECIAL_CHARS: &[char] = &[':', '*', '?', '[', '~', '^', '@', '{', '}', ']'];
    if ns.chars().any(|c| GIT_SPECIAL_CHARS.contains(&c)) {
        return Err(format!("invalid namespace '{}': contains git special characters", ns));
    }

    // Only allow alphanumeric, hyphen, underscore
    if !ns.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
        return Err(format!("invalid namespace '{}': only alphanumeric, hyphen, underscore allowed", ns));
    }

    Ok(())
}

/// Validate actor parameter to prevent git reference injection.
///
/// Rules:
/// - Length: 1-128 characters
/// - Allowed: alphanumeric, hyphen, underscore
/// - Rejected: path traversal (`.`, `..`, `/`, `\`)
/// - Rejected: git special chars (`:`, `*`, `?`, `[`, `~`, `^`, `@`, `{`)
pub(crate) fn validate_actor(actor: &str) -> Result<(), String> {
    const MAX_ACTOR_LEN: usize = 128;

    if actor.is_empty() {
        return Err("actor cannot be empty".into());
    }

    if actor.len() > MAX_ACTOR_LEN {
        return Err(format!("actor exceeds max length {}", MAX_ACTOR_LEN));
    }

    // Reject path traversal sequences
    if actor.contains("..") || actor.contains('/') || actor.contains('\\') || actor.contains('.') {
        return Err(format!("invalid actor '{}': path traversal not allowed", actor));
    }

    // Reject git special characters
    const GIT_SPECIAL_CHARS: &[char] = &[':', '*', '?', '[', '~', '^', '@', '{', '}', ']'];
    if actor.chars().any(|c| GIT_SPECIAL_CHARS.contains(&c)) {
        return Err(format!("invalid actor '{}': contains git special characters", actor));
    }

    // Only allow alphanumeric, hyphen, underscore
    if !actor.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
        return Err(format!("invalid actor '{}': only alphanumeric, hyphen, underscore allowed", actor));
    }

    Ok(())
}

/// Git-backed implementation of JournalStore using refs/gatos/journal/<ns>/<actor>.
pub struct GitJournalStore<'r> {
    repo: &'r Repository,
}

impl<'r> GitJournalStore<'r> {
    pub fn new(repo: &'r Repository) -> Self {
        Self { repo }
    }
}

impl gatos_ports::JournalStore for GitJournalStore<'_> {
    type Event = EventEnvelope;
    type Error = String;

    fn append(&mut self, ns: &str, actor: &str, event: Self::Event) -> Result<String, Self::Error> {
        append_event(self.repo, ns, actor, &event)
    }

    fn read_window(
        &self,
        ns: &str,
        actor: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        read_window(self.repo, ns, actor, start, end)
    }

    fn read_window_paginated(
        &self,
        ns: &str,
        actor: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
        limit: usize,
    ) -> Result<(Vec<Self::Event>, Option<String>), Self::Error> {
        read_window_paginated(self.repo, ns, actor, start, end, limit)
    }
}

/// Append an event to refs/gatos/journal/<ns>/<actor> with CAS.
pub fn append_event(
    repo: &Repository,
    ns: &str,
    actor: &str,
    envelope: &EventEnvelope,
) -> Result<String, String> {
    validate_namespace(ns)?;
    validate_actor(actor)?;
    append_event_with_expected(repo, ns, actor, envelope, None)
}

/// Append with metrics tracking.
pub fn append_event_with_metrics<M: gatos_ports::Metrics>(
    repo: &Repository,
    metrics: &M,
    ns: &str,
    actor: &str,
    envelope: &EventEnvelope,
) -> Result<String, String> {
    validate_namespace(ns)?;
    validate_actor(actor)?;
    append_event_with_expected_and_metrics(repo, metrics, ns, actor, envelope, None)
}

fn append_event_with_expected_and_metrics<M: gatos_ports::Metrics>(
    repo: &Repository,
    metrics: &M,
    ns: &str,
    actor: &str,
    envelope: &EventEnvelope,
    expected_head: Option<Oid>,
) -> Result<String, String> {
    let sig = Signature::now("gatos-ledger", "ledger@gatos.local")
        .map_err(|e| e.message().to_string())?;
    let head_ref = format!("refs/gatos/journal/{}/{}", ns, actor);
    let mut attempts = 0;

    loop {
        attempts += 1;
        let current = if let Some(expected) = expected_head {
            repo.find_commit(expected).ok()
        } else {
            repo.find_reference(&head_ref)
                .ok()
                .and_then(|r| r.target())
                .and_then(|oid| repo.find_commit(oid).ok())
        };
        let parents: Vec<&git2::Commit> = current.iter().collect();
        let tree_oid = write_envelope_tree(repo, envelope)?;
        let message = format!(
            "{}\n\nEvent-CID: {}\n",
            envelope.event_type,
            envelope.event_cid().map_err(|e| e.to_string())?
        );
        let commit_oid = repo
            .commit(
                None,
                &sig,
                &sig,
                &message,
                &repo
                    .find_tree(tree_oid)
                    .map_err(|e| e.message().to_string())?,
                &parents,
            )
            .map_err(|e| e.message().to_string())?;

        let cas_result = if let Some(parent_commit) = current {
            repo.reference_matching(
                &head_ref,
                commit_oid,
                true,
                parent_commit.id(),
                "journal append",
            )
        } else {
            repo.reference(&head_ref, commit_oid, true, "journal append")
        };

        match cas_result {
            Ok(_) => {
                // Track successful append
                metrics.incr_counter(
                    "ledger_appends_total",
                    &[("ns", ns), ("actor", actor), ("result", "ok")],
                );
                return Ok(commit_oid.to_string());
            }
            Err(err)
                if err.class() == git2::ErrorClass::Reference
                    && err.code() == git2::ErrorCode::Locked =>
            {
                // Track conflict immediately
                metrics.incr_counter("ledger_cas_conflicts_total", &[]);
                if attempts >= 3 {
                    metrics.incr_counter(
                        "ledger_appends_total",
                        &[("ns", ns), ("actor", actor), ("result", "error")],
                    );
                    return Err("head_conflict".into());
                }
                continue;
            }
            Err(err)
                if err.class() == git2::ErrorClass::Reference
                    && err.code() == git2::ErrorCode::NotFound =>
            {
                // Track conflict immediately
                metrics.incr_counter("ledger_cas_conflicts_total", &[]);
                if attempts >= 3 {
                    metrics.incr_counter(
                        "ledger_appends_total",
                        &[("ns", ns), ("actor", actor), ("result", "error")],
                    );
                    return Err("head_conflict".into());
                }
                continue;
            }
            Err(err) => {
                // Check if this is a CAS conflict we didn't catch
                if err.class() == git2::ErrorClass::Reference {
                    metrics.incr_counter("ledger_cas_conflicts_total", &[]);
                }
                metrics.incr_counter(
                    "ledger_appends_total",
                    &[("ns", ns), ("actor", actor), ("result", "error")],
                );
                return Err(err.message().to_string());
            }
        }
    }
}

fn append_event_with_expected(
    repo: &Repository,
    ns: &str,
    actor: &str,
    envelope: &EventEnvelope,
    expected_head: Option<Oid>,
) -> Result<String, String> {
    let sig = Signature::now("gatos-ledger", "ledger@gatos.local")
        .map_err(|e| e.message().to_string())?;
    let head_ref = format!("refs/gatos/journal/{}/{}", ns, actor);
    let mut attempts = 0;
    loop {
        attempts += 1;
        let current = if let Some(expected) = expected_head {
            repo.find_commit(expected).ok()
        } else {
            repo.find_reference(&head_ref)
                .ok()
                .and_then(|r| r.target())
                .and_then(|oid| repo.find_commit(oid).ok())
        };
        let parents: Vec<&git2::Commit> = current.iter().collect();
        let tree_oid = write_envelope_tree(repo, envelope)?;
        let message = format!(
            "{}\n\nEvent-CID: {}\n",
            envelope.event_type,
            envelope.event_cid().map_err(|e| e.to_string())?
        );
        let commit_oid = repo
            .commit(
                None,
                &sig,
                &sig,
                &message,
                &repo
                    .find_tree(tree_oid)
                    .map_err(|e| e.message().to_string())?,
                &parents,
            )
            .map_err(|e| e.message().to_string())?;

        let cas_result = if let Some(parent_commit) = current {
            repo.reference_matching(
                &head_ref,
                commit_oid,
                true,
                parent_commit.id(),
                "journal append",
            )
        } else {
            repo.reference(&head_ref, commit_oid, true, "journal append")
        };

        match cas_result {
            Ok(_) => return Ok(commit_oid.to_string()),
            Err(err)
                if err.class() == git2::ErrorClass::Reference
                    && err.code() == git2::ErrorCode::Locked =>
            {
                if attempts >= 3 {
                    return Err("head_conflict".into());
                }
                continue;
            }
            Err(err)
                if err.class() == git2::ErrorClass::Reference
                    && err.code() == git2::ErrorCode::NotFound =>
            {
                if attempts >= 3 {
                    return Err("head_conflict".into());
                }
                continue;
            }
            Err(err) => return Err(err.message().to_string()),
        }
    }
}

/// Read events between optional start/end commits (inclusive end).
pub fn read_window(
    repo: &Repository,
    ns: &str,
    actor: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<EventEnvelope>, String> {
    let with_ids = read_window_with_ids(repo, ns, actor, start, end)?;
    Ok(with_ids.into_iter().map(|(_, env)| env).collect())
}

/// Read window with metrics tracking.
pub fn read_window_with_metrics<M: gatos_ports::Metrics>(
    repo: &Repository,
    metrics: &M,
    ns: &str,
    actor: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<EventEnvelope>, String> {
    let result = read_window(repo, ns, actor, start, end);
    if result.is_ok() {
        metrics.incr_counter("ledger_reads_total", &[("ns", ns)]);
    }
    result
}

fn read_window_with_ids(
    repo: &Repository,
    ns: &str,
    actor: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<(Oid, EventEnvelope)>, String> {
    validate_namespace(ns)?;
    if let Some(a) = actor {
        validate_actor(a)?;
    }

    let target_ref = match actor {
        Some(a) => format!("refs/gatos/journal/{}/{}", ns, a),
        None => {
            // pick any actor under the namespace (first ref)
            let prefix = format!("refs/gatos/journal/{}/", ns);
            let mut iter = repo
                .references()
                .map_err(|e| e.message().to_string())?
                .flatten()
                .filter_map(|r| r.name().map(|n| n.to_string()))
                .filter(|name| name.starts_with(&prefix));
            iter.next()
                .ok_or_else(|| "no journal refs found".to_string())?
        }
    };
    let head = repo
        .refname_to_id(&target_ref)
        .map_err(|e| e.message().to_string())?;
    let mut commit = repo
        .find_commit(head)
        .map_err(|e| e.message().to_string())?;
    let mut events = Vec::new();
    loop {
        let tree = commit.tree().map_err(|e| e.message().to_string())?;
        let msg_tree = tree
            .get_name("message")
            .ok_or_else(|| "missing message".to_string())?;
        let msg_tree = repo
            .find_tree(msg_tree.id())
            .map_err(|e| e.message().to_string())?;
        let blob = msg_tree
            .get_name("envelope.json")
            .ok_or_else(|| "missing envelope.json".to_string())?;
        let blob = repo
            .find_blob(blob.id())
            .map_err(|e| e.message().to_string())?;
        let env: EventEnvelope =
            serde_json::from_slice(blob.content()).map_err(|e| e.to_string())?;
        events.push((commit.id(), env));
        if commit.parent_count() == 0 {
            break;
        }
        commit = commit.parent(0).map_err(|e| e.message().to_string())?;
    }
    events.reverse();

    let mut filtered = Vec::new();
    let mut seen_start = start.is_none();
    for (cid, env) in events {
        if !seen_start {
            if let Some(s) = start {
                if cid.to_string() == s {
                    seen_start = true;
                    continue; // exclusive start
                }
            }
        } else {
            filtered.push((cid, env));
            if let Some(e) = end {
                if cid.to_string() == e {
                    break; // inclusive end
                }
            }
        }
    }
    if !seen_start && start.is_some() {
        return Err("start commit not found".into());
    }
    Ok(filtered)
}

/// Read with limit and return next cursor (commit id) for pagination.
pub fn read_window_paginated(
    repo: &Repository,
    ns: &str,
    actor: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
    limit: usize,
) -> Result<(Vec<EventEnvelope>, Option<String>), String> {
    let mut events = read_window_with_ids(repo, ns, actor, start, end)?;
    if events.len() > limit {
        let cursor = events[limit - 1].0.to_string();
        events.truncate(limit);
        let page = events.into_iter().map(|(_, env)| env).collect();
        return Ok((page, Some(cursor)));
    }
    Ok((events.into_iter().map(|(_, env)| env).collect(), None))
}

fn write_envelope_tree(repo: &Repository, env: &EventEnvelope) -> Result<Oid, String> {
    let bytes = serde_json::to_vec(env).map_err(|e| e.to_string())?;
    let blob = repo.blob(&bytes).map_err(|e| e.message().to_string())?;
    let mut msg = repo
        .treebuilder(None)
        .map_err(|e| e.message().to_string())?;
    msg.insert("envelope.json", blob, 0o100644)
        .map_err(|e| e.message().to_string())?;
    let msg_oid = msg.write().map_err(|e| e.message().to_string())?;
    let mut root = repo
        .treebuilder(None)
        .map_err(|e| e.message().to_string())?;
    root.insert("message", msg_oid, 0o040000)
        .map_err(|e| e.message().to_string())?;
    let tree_oid = root.write().map_err(|e| e.message().to_string())?;
    Ok(tree_oid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventEnvelope;
    use git2::Repository;
    use serde_json::json;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn require_docker() {
        assert_eq!(
            std::env::var("GATOS_TEST_IN_DOCKER").as_deref(),
            Ok("1"),
            "Tests must run inside the Docker harness (set GATOS_TEST_IN_DOCKER=1); use ./scripts/test.sh",
        );
    }

    #[derive(Default, Clone)]
    struct TestMetrics {
        counters: RefCell<HashMap<(String, Vec<(String, String)>), u64>>,
    }

    impl gatos_ports::Metrics for TestMetrics {
        fn incr_counter(&self, name: &'static str, labels: &[(&'static str, &str)]) {
            let key = (
                name.to_string(),
                labels
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            );
            *self.counters.borrow_mut().entry(key).or_insert(0) += 1;
        }

        fn observe_seconds(&self, _name: &'static str, _value: f64, _labels: &[(&'static str, &str)]) {
            // no-op for counter tests
        }
    }

    impl TestMetrics {
        fn get_counter(&self, name: &str, labels: &[(&str, &str)]) -> u64 {
            let key = (
                name.to_string(),
                labels
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            );
            self.counters.borrow().get(&key).copied().unwrap_or(0)
        }
    }

    fn envelope(ulid: &str) -> EventEnvelope {
        EventEnvelope {
            event_type: "event.append".into(),
            ulid: ulid.into(),
            actor: "user:alice".into(),
            caps: vec!["cap.write".into()],
            payload: json!({"hello":"world"}),
            policy_root: "deadbeef".into(),
            sig_alg: Some("ed25519".into()),
            ts: Some("2025-11-21T00:00:00Z".into()),
        }
    }

    #[test]
    fn append_sets_journal_head_and_trailers() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let cid = append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .expect("append");
        assert!(!cid.is_empty(), "receipt should return cid");
        let head = repo
            .find_reference("refs/gatos/journal/default/alice")
            .expect("head ref");
        assert!(head.target().is_some());
    }

    #[test]
    fn append_is_linear_with_cas() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let a = append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .expect("append a");
        let b = append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBA"),
        )
        .expect("append b");
        assert_ne!(a, b);
        // head should be the second
        let head_oid = repo
            .refname_to_id("refs/gatos/journal/default/alice")
            .expect("head");
        let head_commit = repo.find_commit(head_oid).unwrap();
        assert_eq!(head_commit.parent_count(), 1);
    }

    #[test]
    fn read_window_returns_ordered_events() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBA"),
        )
        .unwrap();
        let events = read_window(&repo, "default", None, None, None).expect("read");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(events[1].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FBA");
    }

    #[test]
    fn read_window_honors_start_and_end() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBA"),
        )
        .unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBB"),
        )
        .unwrap();

        let head = repo
            .refname_to_id("refs/gatos/journal/default/alice")
            .unwrap();
        let head_commit = repo.find_commit(head).unwrap();
        let mid_commit = head_commit.parent(0).unwrap();
        let first_commit = mid_commit.parent(0).unwrap();

        // start at first (exclusive) should return mid+head
        let events = read_window(
            &repo,
            "default",
            Some("alice"),
            Some(&first_commit.id().to_string()),
            None,
        )
        .expect("read window");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FBA");
        assert_eq!(events[1].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FBB");

        // end at mid (inclusive) should return first+mid
        let events = read_window(
            &repo,
            "default",
            Some("alice"),
            None,
            Some(&mid_commit.id().to_string()),
        )
        .expect("read window");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(events[1].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FBA");
    }

    #[test]
    fn pagination_returns_next_cursor() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBA"),
        )
        .unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBB"),
        )
        .unwrap();

        let head_oid = repo
            .refname_to_id("refs/gatos/journal/default/alice")
            .unwrap();
        let mid_oid = repo.find_commit(head_oid).unwrap().parent(0).unwrap().id();

        let (page1, cursor) =
            read_window_paginated(&repo, "default", Some("alice"), None, None, 2).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(cursor, Some(mid_oid.to_string()));

        let (page2, cursor2) =
            read_window_paginated(&repo, "default", Some("alice"), cursor.as_deref(), None, 2)
                .unwrap();
        assert_eq!(page2.len(), 1);
        assert_eq!(page2[0].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FBB");
        assert_eq!(cursor2, None);
    }

    #[test]
    fn read_window_unknown_start_returns_error() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .unwrap();

        let result = read_window(&repo, "default", Some("alice"), Some("deadbeef"), None);
        assert!(result.is_err());
    }

    #[test]
    fn append_conflict_returns_error_after_retries() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        // First append to create head
        append_event(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .unwrap();

        // Advance head with an out-of-band commit to trigger CAS conflict
        let stale_head = repo
            .refname_to_id("refs/gatos/journal/default/alice")
            .unwrap();
        let new_head = make_dummy_commit(&repo, Some(stale_head));
        repo.reference("refs/gatos/journal/default/alice", new_head, true, "race")
            .unwrap();

        let result = append_event_with_expected(
            &repo,
            "default",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBA"),
            Some(stale_head),
        );
        assert!(result.is_err());
    }

    fn make_dummy_commit(repo: &Repository, parent: Option<Oid>) -> Oid {
        let sig = Signature::now("gatos-ledger", "ledger@gatos.local").unwrap();
        let tb = repo.treebuilder(None).unwrap();
        let tree_oid = tb.write().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let parents = if let Some(p) = parent {
            vec![repo.find_commit(p).unwrap()]
        } else {
            Vec::new()
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(None, &sig, &sig, "dummy", &tree, &parent_refs)
            .unwrap()
    }

    #[test]
    fn append_increments_success_metric() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let metrics = TestMetrics::default();

        append_event_with_metrics(
            &repo,
            &metrics,
            "ns1",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .expect("append");

        assert_eq!(
            metrics.get_counter("ledger_appends_total", &[("ns", "ns1"), ("actor", "alice"), ("result", "ok")]),
            1
        );
    }

    #[test]
    fn append_cas_conflict_increments_conflict_metric() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let metrics = TestMetrics::default();

        // Create initial commit
        append_event_with_metrics(
            &repo,
            &metrics,
            "ns1",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .unwrap();

        // Simulate CAS conflict by appending with stale expected head
        let stale_head = repo.refname_to_id("refs/gatos/journal/ns1/alice").unwrap();
        let new_head = make_dummy_commit(&repo, Some(stale_head));
        repo.reference("refs/gatos/journal/ns1/alice", new_head, true, "race")
            .unwrap();

        let result = append_event_with_expected_and_metrics(
            &repo,
            &metrics,
            "ns1",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FBA"),
            Some(stale_head),
        );

        // Should fail due to CAS conflict
        assert!(result.is_err(), "Expected CAS conflict error, got: {:?}", result);
        assert!(metrics.get_counter("ledger_cas_conflicts_total", &[]) >= 1);
    }

    #[test]
    fn read_window_increments_read_metric() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let metrics = TestMetrics::default();

        append_event(&repo, "ns1", "alice", &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV")).unwrap();

        read_window_with_metrics(&repo, &metrics, "ns1", Some("alice"), None, None).expect("read");

        assert_eq!(
            metrics.get_counter("ledger_reads_total", &[("ns", "ns1")]),
            1
        );
    }

    #[test]
    fn git_journal_store_implements_trait() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let mut store = GitJournalStore::new(&repo);

        // Test append via trait
        let commit_id = gatos_ports::JournalStore::append(
            &mut store,
            "ns1",
            "alice",
            envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        )
        .expect("append");
        assert!(!commit_id.is_empty());

        // Test read via trait
        let events = gatos_ports::JournalStore::read_window(&store, "ns1", Some("alice"), None, None)
            .expect("read");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ulid, "01ARZ3NDEKTSV4RRFFQ69G5FAV");
    }

    // Security: Input validation tests
    #[test]
    fn validate_namespace_rejects_path_traversal() {
        require_docker();
        assert!(validate_namespace("../../../heads").is_err());
        assert!(validate_namespace("ns/../audit").is_err());
        assert!(validate_namespace("./local").is_err());
        assert!(validate_namespace("ns/subdir").is_err());
        assert!(validate_namespace(r"ns\windows").is_err());
    }

    #[test]
    fn validate_namespace_rejects_git_special_chars() {
        require_docker();
        assert!(validate_namespace("ns:evil").is_err());
        assert!(validate_namespace("ns*glob").is_err());
        assert!(validate_namespace("ns?query").is_err());
        assert!(validate_namespace("ns[bracket").is_err());
        assert!(validate_namespace("ns~1").is_err());
        assert!(validate_namespace("ns^caret").is_err());
        assert!(validate_namespace("ns@at").is_err());
        assert!(validate_namespace("ns{brace").is_err());
    }

    #[test]
    fn validate_namespace_rejects_empty_and_too_long() {
        require_docker();
        assert!(validate_namespace("").is_err());
        assert!(validate_namespace(&"a".repeat(65)).is_err());
    }

    #[test]
    fn validate_namespace_accepts_valid_names() {
        require_docker();
        assert!(validate_namespace("ns1").is_ok());
        assert!(validate_namespace("my-namespace").is_ok());
        assert!(validate_namespace("my_namespace").is_ok());
        assert!(validate_namespace("MyNamespace123").is_ok());
        assert!(validate_namespace(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn validate_actor_rejects_invalid_input() {
        require_docker();
        assert!(validate_actor("../../../admin").is_err());
        assert!(validate_actor("actor~1").is_err());
        assert!(validate_actor("").is_err());
        assert!(validate_actor(&"a".repeat(129)).is_err());
    }

    #[test]
    fn validate_actor_accepts_valid_names() {
        require_docker();
        assert!(validate_actor("alice").is_ok());
        assert!(validate_actor("user-123").is_ok());
        assert!(validate_actor("my_actor").is_ok());
        assert!(validate_actor(&"a".repeat(128)).is_ok());
    }

    #[test]
    fn append_with_invalid_namespace_fails() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        let result = append_event(
            &repo,
            "../../../heads/main",
            "alice",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("namespace"));
    }

    #[test]
    fn append_with_invalid_actor_fails() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        let result = append_event(
            &repo,
            "ns1",
            "actor~1",
            &envelope("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("actor"));
    }
}
