use wasm_bindgen::prelude::*;

/// Returns a static greeting without heap allocation for native Rust callers.
#[must_use]
pub const fn hello_wasm() -> &'static str {
    "Hello from gatos-wasm-bindings!"
}

/// JS/WASM-friendly export that returns an owned `String`.
/// This simply wraps `hello_wasm()` so the JS boundary can copy the string.
#[wasm_bindgen]
#[must_use]
pub fn hello_wasm_js() -> String {
    hello_wasm().to_string()
}

/// Compute commit id (content id) from parts. Accepts optional 32-byte parent,
/// required 32-byte tree, and 64-byte signature. Returns lowercase hex string
/// on success.
#[wasm_bindgen]
/// # Errors
/// Returns `Err(JsValue)` when inputs have invalid lengths or serialization fails.
pub fn compute_commit_id_wasm(
    parent: Option<Vec<u8>>, // None means genesis
    tree: &[u8],
    signature: &[u8],
) -> Result<String, JsValue> {
    if signature.len() != 64 { return Err(JsValue::from_str("invalid signature size")); }
    let core = validate_and_build_core(parent, tree, String::new(), 0)?;
    gatos_ledger_core::compute_content_id(&core)
        .map(hex::encode)
        .map_err(|_| JsValue::from_str("serialize failure"))
}

/// v2: Compute content id from explicit core fields including message and timestamp.
/// Returns lowercase hex string on success.
#[wasm_bindgen]
/// # Errors
/// Returns `Err(JsValue)` when inputs have invalid lengths or serialization fails.
pub fn compute_content_id_wasm_v2(
    parent: Option<Vec<u8>>, // None means genesis
    tree: &[u8],
    message: &str,
    timestamp: u64,
) -> Result<String, JsValue> {
    let core = validate_and_build_core(parent, tree, message.to_string(), timestamp)?;
    gatos_ledger_core::compute_content_id(&core)
        .map(hex::encode)
        .map_err(|_| JsValue::from_str("serialize failure"))
}

fn validate_and_build_core(
    parent: Option<Vec<u8>>,
    tree: &[u8],
    message: String,
    timestamp: u64,
) -> Result<gatos_ledger_core::CommitCore, JsValue> {
    use gatos_ledger_core::Hash;
    if tree.len() != 32 {
        return Err(JsValue::from_str("invalid tree size"));
    }
    let mut tree_arr = [0u8; 32];
    tree_arr.copy_from_slice(tree);
    let parent_arr: Option<Hash> = match parent {
        Some(p) => {
            if p.len() != 32 { return Err(JsValue::from_str("invalid parent size")); }
            let mut a = [0u8; 32];
            a.copy_from_slice(&p);
            Some(a)
        }
        None => None,
    };
    Ok(gatos_ledger_core::CommitCore { parent: parent_arr, tree: tree_arr, message, timestamp })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_wasm_returns_static_str() {
        assert_eq!(hello_wasm(), "Hello from gatos-wasm-bindings!");
        // Also sanity check the JS wrapper compiles and returns the same string
        assert_eq!(hello_wasm_js(), "Hello from gatos-wasm-bindings!");
    }
}
