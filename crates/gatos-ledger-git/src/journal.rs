//! Git-backed ledger journal append/read with CAS semantics.

use git2::Oid;
use git2::Repository;
use git2::Signature;

use crate::event::EventEnvelope;

/// Append an event to refs/gatos/journal/<ns>/<actor> with CAS.
pub fn append_event(
    repo: &Repository,
    ns: &str,
    actor: &str,
    envelope: &EventEnvelope,
) -> Result<String, String> {
    append_event_with_expected(repo, ns, actor, envelope, None)
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

fn read_window_with_ids(
    repo: &Repository,
    ns: &str,
    actor: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<(Oid, EventEnvelope)>, String> {
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
    use tempfile::tempdir;

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
        let mut tb = repo.treebuilder(None).unwrap();
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
}
