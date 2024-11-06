use core::arch::asm;

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
