use crate::vm::pagetable::Pagetable;

pub const PGSIZE: usize = 4096;
pub const PGSHIFT: usize = 12;

pub const TEST0: usize = 0x10_0000;
pub const FW_CFG: usize = 0x1010_0000;
pub const UART0: usize = 0x1000_0000;
pub const VIRTIO0: usize = 0x1000_1000;
pub const PLIC: usize = 0x0c00_0000;
pub const TRAMPOLINE: usize = Pagetable::MAX_VA - PGSIZE;
pub const KSTACK: fn(usize) -> usize = |p| TRAMPOLINE - ((p) + 1) * 2 * PGSIZE;
pub const TRAPFRAME: usize = TRAMPOLINE - PGSIZE;

pub const RAMBASE: usize = 0x8000_0000;
pub const KERNBASE: usize = 0x8020_0000;
pub const PHYSTOP: usize = RAMBASE + 128 * 1024 * 1024;

use core::ffi::c_void;
use core::ptr;

#[repr(transparent)]
pub struct Symbol {
    _placeholder: c_void,
}

mod symbols {
    use crate::memlayout::Symbol;

    extern "C" {
        pub static end: Symbol;
        pub static etext: Symbol;
        pub static trampoline: Symbol;
    }
}

#[must_use]
pub fn end_kernel() -> usize {
    // SAFETY:
    // only address is accessed
    unsafe { ptr::from_ref(&symbols::end) as usize }
}

#[must_use]
pub fn end_text() -> usize {
    // SAFETY:
    // only address is accessed
    unsafe { ptr::from_ref(&symbols::etext) as usize }
}

#[must_use]
pub fn trampoline() -> usize {
    // SAFETY:
    // only address is accessed
    unsafe { ptr::from_ref(&symbols::trampoline) as usize }
}
