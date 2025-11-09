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

/// Compute commit id from parts. Accepts optional 32-byte parent, required 32-byte tree,
/// and 64-byte signature. Returns lowercase hex string on success.
#[wasm_bindgen]
/// # Errors
/// Returns `Err(JsValue)` when inputs have invalid lengths or serialization fails.
pub fn compute_commit_id_wasm(
    parent: Option<Vec<u8>>, // None means genesis
    tree: &[u8],
    signature: &[u8],
) -> Result<String, JsValue> {
    use gatos_ledger_core::Hash;

    if tree.len() != 32 || signature.len() != 64 {
        return Err(JsValue::from_str("invalid input sizes"));
    }
    let mut tree_arr = [0u8; 32];
    tree_arr.copy_from_slice(tree);
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(signature);
    let parent_arr: Option<Hash> = match parent {
        Some(p) => {
            if p.len() != 32 {
                return Err(JsValue::from_str("invalid parent size"));
            }
            let mut a = [0u8; 32];
            a.copy_from_slice(&p);
            Some(a)
        }
        None => None,
    };

    let core = gatos_ledger_core::CommitCore { parent: parent_arr, tree: tree_arr };
    let core_id = gatos_ledger_core::compute_content_id(&core).map_err(|_| JsValue::from_str("serialize failure"))?;
    let commit = gatos_ledger_core::Commit { core_id, signature: sig_arr };
    gatos_ledger_core::compute_commit_id(&commit)
        .map(hex::encode)
        .map_err(|_| JsValue::from_str("serialize failure"))
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
