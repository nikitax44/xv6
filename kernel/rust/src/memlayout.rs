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

use core::ffi::c_void;

#[repr(transparent)]
pub struct Symbol {
    _placeholder: c_void,
}

#[macro_export]
macro_rules! extern_symbol {
    () => {};
    ($(#[$attr:meta])* $vis:vis symbol $N:ident : $T:ty => $e:ident; $($t:tt)*) => {
        $(#[$attr])*
        #[must_use]
        $vis fn $N () -> $T {
            type Target = $T;
            extern "C" {
               static $e : $crate::memlayout::Symbol;
            }
            // SAFETY: we only read address at runtime
            (unsafe {::core::ptr::from_ref(& ($e))}) as Target
        }
        $crate::extern_symbol!($($t)*);
    };
}

#[macro_export]
macro_rules! addrof_symbol {
    ($symbol: ident) => {{
        $crate::extern_symbol! {
            symbol __read: usize => $symbol;
        }
        __read()
    }};
}

extern_symbol! {
    pub symbol addrof_kernel: usize => _entry;
    pub symbol addrof_end_kernel: usize => end;
    pub symbol addrof_end_text: usize => etext;
}
