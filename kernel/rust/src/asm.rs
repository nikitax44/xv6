use core::arch::asm;

pub fn nop() {
    unsafe {
        asm!("nop");
    }
}
