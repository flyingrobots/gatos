#![cfg_attr(not(test), no_std)]

//! Cross-plane ports/interfaces for GATOS (policy, audit, observability).
//!
//! Intent: keep planes decoupled via small, no_std-friendly traits. All
//! timestamps are POSIX seconds.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Clock returning POSIX seconds since Unix epoch (UTC).
pub trait Clock {
    fn now(&self) -> u64;
}

/// Outcome of a policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyOutcome {
    Allow,
    Deny,
}

/// Policy decision metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    pub outcome: PolicyOutcome,
    /// Optional policy version/hash that produced this decision.
    pub policy_version: Option<String>,
    /// Human-readable reasons (for deny) or notes (for allow).
    pub reasons: Vec<String>,
}

/// Context for evaluating an append into the ledger/message journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendContext {
    /// Logical topic/stream name.
    pub topic: String,
    /// ULID of the event being appended.
    pub ulid: String,
    /// Content identifier (CID/hash) of the canonical event payload.
    pub content_id: String,
    /// Caller identity (opaque to the ledger plane).
    pub caller: Caller,
    /// Arbitrary metadata the policy plane may inspect (serialized JSON, CBOR, etc.).
    pub metadata: Vec<u8>,
}

/// Minimal caller identity for policy decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    pub subject: String,
    pub groups: Vec<String>,
}

/// Errors returned by a policy client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    Unavailable,
    InvalidRequest,
    Other(String),
}

/// Port for consulting the Policy Plane.
pub trait PolicyClient {
    fn evaluate_append(&self, ctx: &AppendContext) -> Result<PolicyDecision, PolicyError>;
}

/// Audit entry written for every policy decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyAuditEntry {
    pub decision: PolicyDecision,
    pub ctx: AppendContext,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    Io,
    Conflict,
    Other(String),
}

/// Sink for durable audit logging (e.g., Git refs).
pub trait AuditSink {
    fn record_policy_decision(&self, entry: &PolicyAuditEntry) -> Result<(), AuditError>;
}

/// Minimal metrics facade for counters and histograms.
pub trait Metrics {
    fn incr_counter(&self, name: &'static str, labels: &[(&'static str, &str)]);
    fn observe_seconds(&self, name: &'static str, value: f64, labels: &[(&'static str, &str)]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn require_docker() {
        assert_eq!(
            core::option::Option::Some("1"),
            std::env::var("GATOS_TEST_IN_DOCKER").as_deref().ok(),
            "Tests must run inside the Docker harness (set GATOS_TEST_IN_DOCKER=1); use ./scripts/test.sh",
        );
    }

    struct AllowAllPolicy;
    impl PolicyClient for AllowAllPolicy {
        fn evaluate_append(&self, ctx: &AppendContext) -> Result<PolicyDecision, PolicyError> {
            Ok(PolicyDecision {
                outcome: PolicyOutcome::Allow,
                policy_version: Some("v1".into()),
                reasons: alloc::vec![format!("topic={}", ctx.topic)],
            })
        }
    }

    #[test]
    fn allow_all_policy_returns_allow() {
        require_docker();
        let client = AllowAllPolicy;
        let ctx = AppendContext {
            topic: "jobs/pending".into(),
            ulid: "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
            content_id: "cid123".into(),
            caller: Caller {
                subject: "alice".into(),
                groups: alloc::vec!["ops".into()],
            },
            metadata: alloc::vec![],
        };
        let decision = client.evaluate_append(&ctx).unwrap();
        assert_eq!(decision.outcome, PolicyOutcome::Allow);
    }
}
