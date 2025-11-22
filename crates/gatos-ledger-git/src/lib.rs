//! Git-backed ledger backend (under active construction).
//!
//! Implements event canonicalization + signing (DAG-CBOR), and will grow
//! append/read primitives with CAS semantics per SPEC §4.

#![deny(unsafe_code)]

pub mod event;
pub mod journal;

/// Returns a static string explaining that the backend is still landing.
pub fn stub_notice() -> &'static str {
    "gatos-ledger-git backend is under reconstruction"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{sign_event, verify_event, EventEnvelope};
    use serde_json::json;

    fn require_docker() {
        assert_eq!(
            std::env::var("GATOS_TEST_IN_DOCKER").as_deref(),
            Ok("1"),
            "Tests must run inside the Docker harness (set GATOS_TEST_IN_DOCKER=1)"
        );
    }

    // Known test vector (dag-cbor + blake3-256)
    const EXPECTED_CID: &str = "bafyr4ifveoisniytx6etpqt7jnxjd6hbqul5utgfzvokn4rk3zdt5tgacu";

    fn sample_envelope() -> EventEnvelope {
        EventEnvelope {
            event_type: "event.append".to_string(),
            ulid: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            actor: "user:alice".to_string(),
            caps: vec!["cap.write".to_string()],
            payload: json!({"hello": "world"}),
            policy_root: "deadbeef".to_string(),
            sig_alg: Some("ed25519".to_string()),
            ts: Some("2025-11-21T00:00:00Z".to_string()),
        }
    }

    #[test]
    fn stub_notice_mentions_backend() {
        require_docker();
        assert!(stub_notice().contains("backend"));
    }

    #[test]
    fn canonical_bytes_are_stable() {
        require_docker();
        let env = sample_envelope();
        let bytes1 = env.canonical_bytes().expect("bytes");
        let bytes2 = env.canonical_bytes().expect("bytes");
        assert_eq!(bytes1, bytes2, "canonical bytes must be stable");
    }

    #[test]
    fn event_cid_matches_expected_placeholder() {
        require_docker();
        let env = sample_envelope();
        let cid = env.event_cid().expect("cid");
        assert_eq!(cid, EXPECTED_CID, "CID should match spec vector");
    }

    #[test]
    fn signing_and_verification_round_trip() {
        require_docker();
        let env = sample_envelope();
        let kp = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
        let sig = sign_event(&env, &kp).expect("sign");
        assert!(verify_event(&env, &kp.verifying_key(), &sig).unwrap());
    }
}
