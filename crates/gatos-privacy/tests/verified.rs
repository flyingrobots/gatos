use gatos_privacy::{OpaquePointer, VerifiedOpaquePointer};

fn read_example(rel: &str) -> String {
    let dir = env!("CARGO_MANIFEST_DIR");
    std::fs::read_to_string(format!("{}/../../examples/v1/{}", dir, rel)).unwrap()
}

#[test]
fn verified_accepts_ciphertext_only_low_entropy() {
    let json = read_example("privacy/pointer_low_entropy_min.json");
    let v: VerifiedOpaquePointer = serde_json::from_str(&json).expect("verified deserialize");
    assert!(v.ciphertext_digest.is_some());
    assert!(v.digest.is_none());
}

#[test]
fn verified_rejects_low_entropy_with_plain_digest() {
    let json = read_example("privacy/pointer_low_entropy_invalid.json");
    let v: Result<VerifiedOpaquePointer, _> = serde_json::from_str(&json);
    assert!(v.is_err(), "should reject invalid low-entropy pointer");
}

