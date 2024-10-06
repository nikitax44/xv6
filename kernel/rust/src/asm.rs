use core::arch::asm;

pub fn nop() {
    // SAFETY:
    // nop is safe
    unsafe {
        asm!("nop");
    }
}
