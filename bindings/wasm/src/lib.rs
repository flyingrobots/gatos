use wasm_bindgen::prelude::*;

/// Returns a static greeting without heap allocation for native Rust callers.
pub fn hello_wasm() -> &'static str {
    "Hello from gatos-wasm-bindings!"
}

/// JS/WASM-friendly export that returns an owned `String`.
/// This simply wraps `hello_wasm()` so the JS boundary can copy the string.
#[wasm_bindgen]
pub fn hello_wasm_js() -> String {
    hello_wasm().to_string()
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
