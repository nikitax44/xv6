use crate::vm::pagetable::Pagetable;

pub const PGSIZE: usize = 4096;
pub const PGSHIFT: usize = 12;

pub const SYSCON: usize = 0x10_0000;
pub const FW_CFG: usize = 0x1010_0000;
pub const UART0: usize = 0x1000_0000;
pub const VIRTIO0: usize = 0x1000_1000;
pub const PLIC: usize = 0x0c00_0000;
pub const TRAMPOLINE: usize = Pagetable::MAX_VA - PGSIZE;
pub const TRAPFRAME: usize = TRAMPOLINE - PGSIZE;
#[no_mangle]
pub static STACK_SIZE: usize = 32;
pub static KSTACK: fn(usize) -> usize = |p| TRAPFRAME - ((p) + 1) * (STACK_SIZE + 1) * PGSIZE;
pub const PGROUNDDOWN: fn(usize) -> usize = |addr| addr & !(PGSIZE - 1);

use core::ffi::c_void;

#[repr(transparent)]
pub struct Symbol {
    _placeholder: c_void,
}

#[macro_export]
macro_rules! extern_symbol {
    () => {};
    ($(#[$attr:meta])* $vis:vis symbol $N:ident <- $e:ident; $($t:tt)*) => {
        $(#[$attr])*
        #[must_use]
        $vis fn $N() -> usize {
            extern "C" {
                #[link_name = stringify!($e)]
                static SYM: $crate::memlayout::Symbol;
            }
            // SAFETY: we only read address at runtime
            let sym = unsafe { &SYM };
            ::core::ptr::from_ref(sym) as usize
        }
        $crate::extern_symbol!($($t)*);
    };
}

#[macro_export]
macro_rules! addrof_symbol {
    ($symbol: ident) => {{
        $crate::extern_symbol! {
            symbol __read <- $symbol;
        }
        __read()
    }};
}

extern_symbol! {
    pub symbol addrof_kernel <- _entry;
    pub symbol addrof_end_kernel <- end;
    pub symbol addrof_end_text <- etext;
}
