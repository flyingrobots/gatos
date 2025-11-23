use gatos_ports::{AppendContext, AuditSink, Clock, PolicyClient, PolicyOutcome};

use crate::event::EventEnvelope;
use crate::journal::append_event;

#[derive(Debug, thiserror::Error)]
pub enum PolicyGuardError {
    #[error("policy denied")]
    Denied,
    #[error("policy unavailable")]
    PolicyUnavailable,
    #[error("audit failed")]
    AuditFailed,
    #[error("append failed: {0}")]
    AppendFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::GitPolicyAudit;
    use crate::event::EventEnvelope;
    use gatos_ports::{Caller, PolicyDecision, PolicyError, PolicyOutcome};
    use git2::Repository;
    use serde_json::json;
    use tempfile::tempdir;

    fn require_docker() {
        assert_eq!(
            std::env::var("GATOS_TEST_IN_DOCKER").as_deref(),
            Ok("1"),
            "Tests must run inside the Docker harness (set GATOS_TEST_IN_DOCKER=1); use ./scripts/test.sh",
        );
    }

    struct FixedClock;
    impl Clock for FixedClock {
        fn now(&self) -> u64 {
            1_726_000_000
        }
    }

    struct AllowPolicy;
    impl PolicyClient for AllowPolicy {
        fn evaluate_append(&self, _ctx: &AppendContext) -> Result<PolicyDecision, PolicyError> {
            Ok(PolicyDecision {
                outcome: PolicyOutcome::Allow,
                policy_version: Some("v1".into()),
                reasons: vec!["ok".into()],
            })
        }
    }

    struct DenyPolicy;
    impl PolicyClient for DenyPolicy {
        fn evaluate_append(&self, _ctx: &AppendContext) -> Result<PolicyDecision, PolicyError> {
            Ok(PolicyDecision {
                outcome: PolicyOutcome::Deny,
                policy_version: Some("v1".into()),
                reasons: vec!["nope".into()],
            })
        }
    }

    fn env() -> EventEnvelope {
        EventEnvelope {
            event_type: "event.append".into(),
            ulid: "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
            actor: "user:alice".into(),
            caps: vec!["cap".into()],
            payload: json!({"x":1}),
            policy_root: "deadbeef".into(),
            sig_alg: None,
            ts: None,
        }
    }

    #[test]
    fn allow_path_appends_and_audits() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let clock = FixedClock;
        let policy = AllowPolicy;
        let audit = GitPolicyAudit::new(&repo, "ns", "actor").unwrap();
        let caller = Caller {
            subject: "alice".into(),
            groups: vec!["ops".into()],
        };

        let commit = append_with_policy(
            &repo,
            &clock,
            &policy,
            &audit,
            "ns",
            "actor",
            &env(),
            caller,
            vec![],
        )
        .expect("append allowed");

        // Verify journal head exists
        let head_ref = repo.refname_to_id("refs/gatos/journal/ns/actor").unwrap();
        assert_eq!(head_ref.to_string(), commit);

        // Verify audit ref exists and encodes allow
        let aid = repo
            .refname_to_id("refs/gatos/audit/policy/ns/actor")
            .unwrap();
        let commit = repo.find_commit(aid).unwrap();
        let tree = commit.tree().unwrap();
        let blob = repo
            .find_blob(tree.get_name("audit.json").unwrap().id())
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(blob.content()).unwrap();
        assert_eq!(v["decision"]["outcome"], "Allow");
        assert_eq!(v["timestamp"], 1_726_000_000u64);
    }

    #[test]
    fn deny_path_audits_and_blocks_append() {
        require_docker();
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let clock = FixedClock;
        let policy = DenyPolicy;
        let audit = GitPolicyAudit::new(&repo, "ns", "actor").unwrap();
        let caller = Caller {
            subject: "alice".into(),
            groups: vec!["ops".into()],
        };

        let res = append_with_policy(
            &repo,
            &clock,
            &policy,
            &audit,
            "ns",
            "actor",
            &env(),
            caller,
            vec![],
        );
        assert!(matches!(res, Err(PolicyGuardError::Denied)));

        // Audit ref should still be written
        let aid = repo
            .refname_to_id("refs/gatos/audit/policy/ns/actor")
            .unwrap();
        let commit = repo.find_commit(aid).unwrap();
        let tree = commit.tree().unwrap();
        let blob = repo
            .find_blob(tree.get_name("audit.json").unwrap().id())
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(blob.content()).unwrap();
        assert_eq!(v["decision"]["outcome"], "Deny");
        // Journal head must not exist
        assert!(repo.refname_to_id("refs/gatos/journal/ns/actor").is_err());
    }
}

/// Decorator that enforces policy before appending and logs the decision.
pub fn append_with_policy<C: Clock, P: PolicyClient, A: AuditSink>(
    repo: &git2::Repository,
    clock: &C,
    policy: &P,
    audit: &A,
    ns: &str,
    actor: &str,
    envelope: &EventEnvelope,
    caller: gatos_ports::Caller,
    metadata: Vec<u8>,
) -> Result<String, PolicyGuardError> {
    let content_id = envelope
        .event_cid()
        .map_err(|e| PolicyGuardError::AppendFailed(e.to_string()))?;
    let ctx = AppendContext {
        topic: ns.to_string(),
        ulid: envelope.ulid.clone(),
        content_id,
        caller,
        metadata,
    };

    let decision = policy.evaluate_append(&ctx).map_err(|e| match e {
        gatos_ports::PolicyError::Unavailable => PolicyGuardError::PolicyUnavailable,
        gatos_ports::PolicyError::InvalidRequest => PolicyGuardError::Denied,
        gatos_ports::PolicyError::Other(_) => PolicyGuardError::PolicyUnavailable,
    })?;

    let outcome = decision.outcome.clone();
    let audit_entry = gatos_ports::PolicyAuditEntry {
        decision,
        ctx: ctx.clone(),
        timestamp: clock.now(),
    };
    audit
        .record_policy_decision(&audit_entry)
        .map_err(|_| PolicyGuardError::AuditFailed)?;

    match outcome {
        PolicyOutcome::Allow => {
            append_event(repo, ns, actor, envelope).map_err(PolicyGuardError::AppendFailed)
        }
        PolicyOutcome::Deny => Err(PolicyGuardError::Denied),
    }
}
