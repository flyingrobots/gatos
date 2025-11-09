extern crate libc;

#[no_mangle]
pub extern "C" fn hello_ffi() -> *mut libc::c_char {
    std::ffi::CString::new("Hello from gatos-ffi-bindings!")
        .map_or(std::ptr::null_mut(), std::ffi::CString::into_raw)
}

// Remember to add a corresponding free function for the CString
/// # Safety
/// `s` must be a pointer previously returned by [`hello_ffi`], allocated by
/// Rust, and not yet freed. Passing any other pointer is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn gatos_ffi_free_string(s: *mut libc::c_char) {
    if s.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `s` originated from `hello_ffi`.
    let _ = std::ffi::CString::from_raw(s);
}
