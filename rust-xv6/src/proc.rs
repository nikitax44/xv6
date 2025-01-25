use crate::bindings::{myproc, pid_t, proc_};
use core::ptr::NonNull;

#[must_use]
pub fn current_proc() -> Option<NonNull<proc_>> {
    // SAFETY: safe
    unsafe { NonNull::new(myproc()) }
}

#[must_use]
pub fn current_pid() -> Option<pid_t> {
    let proc = current_proc()?;
    // SAFETY: the reference does not escape the function
    let proc = unsafe { proc.as_ref() };
    Some(proc.pid)
}
