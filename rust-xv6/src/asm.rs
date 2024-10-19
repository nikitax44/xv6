use core::arch::asm;
use core::ptr::NonNull;

#[must_use]
pub fn cpuid() -> u32 {
    let tp;
    // SAFETY: safe
    unsafe {
        asm!("mv {}, tp", out(reg) tp);
    }
    tp
}

#[must_use]
pub fn ticks() -> u64 {
    let ticks;
    // SAFETY: safe
    unsafe {
        asm!("csrr {}, time", out(reg) ticks);
    }
    ticks
}

#[must_use]
pub fn get_current_pt() -> Option<NonNull<crate::vm::pt_inner::IPagetable>> {
    let mut buf: usize;
    // SAFETY: safe
    unsafe {
        asm!("csrr {}, satp", out(reg) buf);
    }

    (buf >> 60 == 8)
        .then_some(())
        .and_then(|()| NonNull::new((buf << 12) as *mut crate::vm::pt_inner::IPagetable))
}
