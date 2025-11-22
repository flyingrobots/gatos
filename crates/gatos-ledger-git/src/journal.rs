//! Git-backed ledger journal append/read (tests first, impl pending).

use git2::Oid;
use git2::Repository;
use git2::Signature;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::event::EventEnvelope;

/// Append an event to refs/gatos/journal/<ns>/<actor> with CAS.
pub fn append_event(
    repo: &Repository,
    ns: &str,
    actor: &str,
    envelope: &EventEnvelope,
) -> Result<String, String> {
    let sig = Signature::now("gatos-ledger", "ledger@gatos.local")
        .map_err(|e| e.message().to_string())?;
    let tree_oid = write_envelope_tree(repo, envelope)?;
    let head_ref = format!("refs/gatos/journal/{}/{}", ns, actor);
    let parent = repo
        .find_reference(&head_ref)
        .ok()
        .and_then(|r| r.target())
        .and_then(|oid| repo.find_commit(oid).ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();
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
    // CAS update (best-effort; not atomic yet)
    repo.reference(&head_ref, commit_oid, true, "journal append")
        .map_err(|e| e.message().to_string())?;
    Ok(commit_oid.to_string())
}

/// Read events between optional start/end commits (inclusive end).
pub fn read_window(
    repo: &Repository,
    ns: &str,
    actor: Option<&str>,
    _start: Option<&str>,
    _end: Option<&str>,
) -> Result<Vec<EventEnvelope>, String> {
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
        events.push(env);
        if commit.parent_count() == 0 {
            break;
        }
        commit = commit.parent(0).map_err(|e| e.message().to_string())?;
    }
    events.reverse();
    Ok(events)
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
}
