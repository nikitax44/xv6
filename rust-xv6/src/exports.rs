use zerocopy::IntoBytes;

#[no_mangle]
unsafe extern "C" fn size_of_last_char(
    start: *const core::ffi::c_char,
    end: *const core::ffi::c_char,
) -> usize {
    // SAFETY: precondition
    let bytes = unsafe { core::slice::from_ptr_range(start..end) };
    let Ok(str) = core::str::from_utf8(bytes.as_bytes()) else {
        return 1;
    };
    str.len() - str.floor_char_boundary(str.len() - 1)
}

extern "C" {
    fn syscall_impl();
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn syscall() {
    // SAFETY: precondition
    unsafe {
        syscall_impl();
    }
}
