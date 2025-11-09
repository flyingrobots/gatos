extern crate libc;

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

/// Compute a BLAKE3-based commit id for a commit described by its parts and
/// return it as a newly-allocated hex string (caller must free via
/// `gatos_ffi_free_string`). On failure, returns NULL.
///
/// `parent_ptr`: pointer to 32-byte parent hash or NULL when `has_parent=false`.
/// `tree_ptr`: pointer to 32-byte tree hash.
/// `signature_ptr`: pointer to 64-byte signature.
/// # Safety
/// The caller must ensure that:
/// - When `has_parent` is true, `parent_ptr` points to at least 32 readable bytes.
/// - `tree_ptr` points to at least 32 readable bytes.
/// - `signature_ptr` points to at least 64 readable bytes.
/// - The provided pointers remain valid for the duration of this call and do not alias.
#[no_mangle]
pub unsafe extern "C" fn gatos_compute_commit_id_hex(
    has_parent: bool,
    parent_ptr: *const u8,
    tree_ptr: *const u8,
    signature_ptr: *const u8,
) -> *mut libc::c_char {
    use gatos_ledger_core::{compute_commit_id, Commit, Hash};

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
    let mut signature = [0u8; 64];
    std::ptr::copy_nonoverlapping(tree_ptr, tree.as_mut_ptr(), 32);
    std::ptr::copy_nonoverlapping(signature_ptr, signature.as_mut_ptr(), 64);

    let commit = Commit { parent, tree, signature };
    compute_commit_id(&commit).map_or(std::ptr::null_mut(), |id| {
        let s = hex::encode(id);
        std::ffi::CString::new(s)
            .map_or(std::ptr::null_mut(), std::ffi::CString::into_raw)
    })
}
