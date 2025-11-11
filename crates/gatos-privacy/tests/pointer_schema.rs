use gatos_privacy::OpaquePointer;

fn read_example(rel: &str) -> String {
    let dir = env!("CARGO_MANIFEST_DIR");
    std::fs::read_to_string(format!("{}/../../examples/v1/{}", dir, rel)).unwrap()
}

#[test]
fn ciphertext_only_pointer_should_deserialize() {
    // This example omits plaintext digest by design (low-entropy class)
    let json = read_example("privacy/pointer_low_entropy_min.json");
    let ptr: Result<OpaquePointer, _> = serde_json::from_str(&json);
    assert!(ptr.is_ok(), "ciphertext-only opaque pointer must deserialize");
}

#[test]
fn both_digests_allowed_when_not_low_entropy() {
    let json = read_example("privacy/opaque_pointer_min.json");
    let ptr: OpaquePointer = serde_json::from_str(&json).unwrap();
    let has_digest = !ptr.digest.is_empty();
    assert!(has_digest);
    assert!(ptr.ciphertext_digest.is_some());
}

