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

// Internal helpers to deduplicate parsing and encoding logic across FFI entrypoints.
use gatos_ledger_core::{compute_content_id, CommitCore, Hash};

/// # Safety
/// If `has` is true, `ptr` must be non-null and point to at least 32 readable bytes.
unsafe fn parse_hash_opt(has: bool, ptr: *const u8) -> Result<Option<Hash>, ()> {
    if has {
        Ok(Some(parse_hash(ptr)?))
    } else {
        Ok(None)
    }
}

/// # Safety
/// `ptr` must be non-null and point to at least 32 readable bytes.
unsafe fn parse_hash(ptr: *const u8) -> Result<Hash, ()> {
    if ptr.is_null() {
        return Err(());
    }
    let mut out = [0u8; 32];
    std::ptr::copy_nonoverlapping(ptr, out.as_mut_ptr(), 32);
    Ok(out)
}

fn compute_and_encode(core: &CommitCore) -> *mut libc::c_char {
    compute_content_id(core).map_or(std::ptr::null_mut(), |id| {
        let s = hex::encode(id);
        std::ffi::CString::new(s).map_or(std::ptr::null_mut(), std::ffi::CString::into_raw)
    })
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
    // Parse inputs
    let parent = match parse_hash_opt(has_parent, parent_ptr) {
        Ok(x) => x,
        Err(_) => return std::ptr::null_mut(),
    };
    // Signature is ignored for hashing but validated for compatibility
    if signature_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let tree = match parse_hash(tree_ptr) {
        Ok(x) => x,
        Err(_) => return std::ptr::null_mut(),
    };
    // Empty message + zero timestamp boundary
    let core = CommitCore {
        parent,
        tree,
        message: String::new(),
        timestamp: 0,
    };
    compute_and_encode(&core)
}

/// v2: Compute content id from core fields including message and timestamp.
/// Returns lowercase hex string (caller must free via `gatos_ffi_free_string`).
///
/// `parent_ptr`: pointer to 32-byte parent hash or NULL when `has_parent=false`.
/// `tree_ptr`: pointer to 32-byte tree hash.
/// `msg_ptr`/`msg_len`: UTF‑8 message bytes.
/// `timestamp`: seconds since Unix epoch (UTC).
///
/// # Safety
/// The caller must ensure that:
/// - When `has_parent` is true, `parent_ptr` points to at least 32 readable bytes.
/// - `tree_ptr` points to at least 32 readable bytes.
/// - `msg_ptr` is either NULL with `msg_len==0` or points to `msg_len` readable bytes of valid UTF‑8.
#[no_mangle]
pub unsafe extern "C" fn gatos_compute_content_id_hex_v2(
    has_parent: bool,
    parent_ptr: *const u8,
    tree_ptr: *const u8,
    msg_ptr: *const u8,
    msg_len: usize,
    timestamp: u64,
) -> *mut libc::c_char {
    let parent = match parse_hash_opt(has_parent, parent_ptr) {
        Ok(x) => x,
        Err(_) => return std::ptr::null_mut(),
    };
    let tree = match parse_hash(tree_ptr) {
        Ok(x) => x,
        Err(_) => return std::ptr::null_mut(),
    };

    let message = if msg_len == 0 {
        String::new()
    } else {
        if msg_ptr.is_null() {
            return std::ptr::null_mut();
        }
        let bytes = core::slice::from_raw_parts(msg_ptr, msg_len);
        match core::str::from_utf8(bytes) {
            Ok(s) => s.to_owned(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let core = CommitCore {
        parent,
        tree,
        message,
        timestamp,
    };
    compute_and_encode(&core)
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
