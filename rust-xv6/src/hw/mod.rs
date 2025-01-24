use crate::memlayout::SYSCON;
use log::info;

pub mod asm;
pub mod console;
pub mod disk;
pub(crate) mod hal;

mod __critical {
    #[no_mangle]
    unsafe extern "Rust" fn _critical_section_1_0_acquire() -> ::critical_section::RawRestoreState {
        // SAFETY: precondition
        unsafe { crate::bindings::push_off() }
    }
    #[no_mangle]
    unsafe extern "Rust" fn _critical_section_1_0_release(
        _restore_state: ::critical_section::RawRestoreState,
    ) {
        // SAFETY: precondition
        unsafe { crate::bindings::pop_off() }
    }
}

const SYSCON_SHUTDOWN: u32 = 0x0000_5555;
const SYSCON_REBOOT: u32 = 0x0000_7777;

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
    info!("shutting down...");
    // SAFETY: SYSCON is mapped to that address
    unsafe {
        core::ptr::write_volatile(SYSCON as *mut u32, SYSCON_SHUTDOWN);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn reboot() -> ! {
    info!("rebooting...");
    // SAFETY: SYSCON is mapped to that address
    unsafe {
        core::ptr::write_volatile(SYSCON as *mut u32, SYSCON_REBOOT);
    }
    loop {
        core::hint::spin_loop();
    }
}
