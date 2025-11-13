# GATOS WASM Bindings

This crate provides WebAssembly bindings for the GATOS core libraries, allowing them to be used in JavaScript environments (like a web browser or Node.js). It uses `wasm-bindgen` to generate the necessary JS-Rust interop code.

Exports

- `hello_wasm() -> &'static str` — zero-allocation helper for Rust callers.
- `hello_wasm_js() -> String` — `wasm-bindgen` export suitable for JS; wraps `hello_wasm()`.

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).
