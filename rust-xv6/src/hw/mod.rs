pub mod asm;
pub mod console;
pub mod disk;
pub(crate) mod hal;

mod __critical {
    #[no_mangle]
    unsafe extern "Rust" fn _critical_section_1_0_acquire() -> ::critical_section::RawRestoreState {
        // SAFETY: precondition
        unsafe { push_off() }
    }
    #[no_mangle]
    unsafe extern "Rust" fn _critical_section_1_0_release(
        _restore_state: ::critical_section::RawRestoreState,
    ) {
        // SAFETY: precondition
        unsafe { pop_off() }
    }

    extern "C" {
        fn push_off();
        fn pop_off();
    }
}
