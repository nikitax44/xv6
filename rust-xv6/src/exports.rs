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

#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn syscall() {
    // SAFETY: precondition
    unsafe {
        crate::bindings::syscall_impl();
    }
}

#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_fork() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_fork() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_wait() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_wait() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_read() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_read() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_execve() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_execve() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_chdir() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_chdir() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_getpid() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_getpid() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_sleep() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_sleep() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_open() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_open() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_seek() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_seek() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_unlink() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_unlink() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_mkdir() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_mkdir() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_mmap() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_mmap() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_sysinfo() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_sysinfo() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_getdents() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_getdents() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_exit() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_exit() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_pipe() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_pipe() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_kill() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_kill() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_fstat() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_fstat() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_dup() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_dup() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_sbrk() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_sbrk() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_uptime() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_uptime() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_write() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_write() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_mknod() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_mknod() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_link() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_link() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_close() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_close() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_gettimeofday() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_gettimeofday() }
}
#[attr_wrapper::time_me(0)]
#[no_mangle]
unsafe extern "C" fn _sys_futimesat() -> u64 {
    // SAFETY: precondition
    unsafe { crate::bindings::sys_futimesat() }
}
