#[no_mangle]
pub extern "C" fn hello_ffi() -> *mut libc::c_char {
    let s = std::ffi::CString::new("Hello from gatos-ffi-bindings!").unwrap();
    s.into_raw()
}

// Remember to add a corresponding free function for the CString
#[no_mangle]
pub extern "C" fn gatos_ffi_free_string(s: *mut libc::c_char) {
    if s.is_null() { return }
    unsafe {
        let _ = std::ffi::CString::from_raw(s);
    }
}

// Need to add libc as a dependency for c_char
extern crate libc;
