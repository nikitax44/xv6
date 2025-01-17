use crate::hw::console::CONSOLE;
use crate::hw::{halt_hart, shutdown};
use core::ffi::{c_char, CStr};
use core::fmt::{Arguments, Write};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

pub static PANICKED: AtomicBool = AtomicBool::new(false);

fn raw_panic(msg: Arguments) -> ! {
    riscv::interrupt::disable();

    let already_panicked = PANICKED.swap(true, Ordering::Acquire);
    if already_panicked {
        halt_hart();
    }

    let mut console = CONSOLE.lock();

    console.newline();
    console.write_fmt(msg).ok();
    console.newline();
    drop(console);
    shutdown()
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    raw_panic(format_args!("panic in RUST: {info}"))
}

/// # Safety
/// msg must point to valid null-terminated string
/// # Panics
/// always, that is the point of this function
/// the function itself shouldn't panic
#[no_mangle]
unsafe extern "C" fn panic(msg: *const c_char) -> ! {
    // SAFETY:
    // msg is valid CStr by precondition
    let cstr = unsafe { CStr::from_ptr::<'_>(msg) };

    let msg = cstr
        .to_str()
        .unwrap_or("panic message is not valid UTF-8. possibly garbage pointer");
    raw_panic(format_args!("panic from C FFI: {}", msg));
}
