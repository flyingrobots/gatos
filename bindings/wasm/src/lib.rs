use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello_wasm() -> String {
    "Hello from gatos-wasm-bindings!".to_string()
}
