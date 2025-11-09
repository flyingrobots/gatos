extern crate libc;
#[cfg(test)]
extern crate std;

#[no_mangle]
pub extern "C" fn hello_ffi() -> *mut libc::c_char {
    std::ffi::CString::new("Hello from gatos-ffi-bindings!")
        .map_or(std::ptr::null_mut(), std::ffi::CString::into_raw)
}

// Remember to add a corresponding free function for the CString
/// # Safety
/// `s` must be a pointer previously returned by [`hello_ffi`] or one of the
/// allocation-returning functions below, allocated by Rust, and not yet freed.
#[no_mangle]
pub unsafe extern "C" fn gatos_ffi_free_string(s: *mut libc::c_char) {
    if s.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `s` originated from `hello_ffi`.
    let _ = std::ffi::CString::from_raw(s);
}

/// Convenience alias for freeing strings allocated by this FFI.
///
/// # Safety
/// See [`gatos_ffi_free_string`]. The same preconditions apply.
#[no_mangle]
pub unsafe extern "C" fn gatos_free(s: *mut libc::c_char) {
    gatos_ffi_free_string(s)
}

/// Compute the canonical commit identifier (content id) for a commit core
/// described by its parts and return it as a newly-allocated hex string
/// (caller must free via `gatos_ffi_free_string`). On failure, returns NULL.
///
/// `parent_ptr`: pointer to 32-byte parent hash or NULL when `has_parent=false`.
/// `tree_ptr`: pointer to 32-byte tree hash.
/// `signature_ptr`: pointer to 64-byte signature (ignored for id calculation; kept for API stability).
///
/// NOTE: As of ADR-0001, the canonical identifier is derived solely from the
/// unsigned core. The `signature_ptr` is ignored for hashing.
///
/// # Safety
/// The caller must ensure that:
/// - When `has_parent` is true, `parent_ptr` points to at least 32 readable bytes.
/// - `tree_ptr` points to at least 32 readable bytes.
/// - `signature_ptr` points to at least 64 readable bytes (ignored but validated).
/// - The provided pointers remain valid for the duration of this call and do not alias.
#[no_mangle]
pub unsafe extern "C" fn gatos_compute_commit_id_hex(
    has_parent: bool,
    parent_ptr: *const u8,
    tree_ptr: *const u8,
    signature_ptr: *const u8,
) -> *mut libc::c_char {
    use gatos_ledger_core::{compute_content_id, CommitCore, Hash};

    let parent: Option<Hash> = if has_parent {
        if parent_ptr.is_null() {
            return std::ptr::null_mut();
        }
        let mut buf = [0u8; 32];
        std::ptr::copy_nonoverlapping(parent_ptr, buf.as_mut_ptr(), 32);
        Some(buf)
    } else {
        None
    };

    if tree_ptr.is_null() || signature_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let mut tree = [0u8; 32];
    let mut _signature = [0u8; 64]; // Ignored per ADR-0001; retained for API compatibility
    std::ptr::copy_nonoverlapping(tree_ptr, tree.as_mut_ptr(), 32);
    std::ptr::copy_nonoverlapping(signature_ptr, _signature.as_mut_ptr(), 64);

    // For now, supply empty message and zero timestamp at the FFI boundary.
    let core = CommitCore {
        parent,
        tree,
        message: String::new(),
        timestamp: 0,
    };
    compute_content_id(&core).map_or(std::ptr::null_mut(), |id| {
        let s = hex::encode(id);
        std::ffi::CString::new(s).map_or(std::ptr::null_mut(), std::ffi::CString::into_raw)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_and_free_roundtrip() {
        // Calling `hello_ffi` is safe; freeing requires `unsafe` below.
        let p = hello_ffi();
        assert!(!p.is_null());
        unsafe { gatos_ffi_free_string(p) };
    }
}
