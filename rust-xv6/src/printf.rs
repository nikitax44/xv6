use crate::hw::console::{Console, CONSOLE};
use core::ffi::{c_char, CStr, VaListImpl};
use core::fmt::Write;

#[no_mangle]
unsafe extern "C" fn printf(format: *const c_char, ap: ...) -> core::ffi::c_int {
    enum State {
        Normal,
        FormatError,
        Format { long: bool },
    }

    unsafe fn printf_impl(
        console: &mut Console,
        format: &CStr,
        mut ap: VaListImpl,
    ) -> core::fmt::Result {
        let mut state = State::Normal;
        for ch in format.to_bytes() {
            state = match (state, ch) {
                (State::Normal, b'%') => State::Format { long: false },
                (State::Format { long: false }, b'%') => {
                    console.putc(b'%');
                    State::Normal
                }

                (State::Format { long: _ }, b'p') => {
                    // SAFETY: precondition
                    let ptr: *const () = unsafe { ap.arg() };
                    write!(console, "{:?}", ptr)?;
                    State::Normal
                }

                (State::Format { long: false }, b'd') => {
                    // SAFETY: precondition
                    let ptr: i32 = unsafe { ap.arg() };
                    write!(console, "{}", ptr)?;
                    State::Normal
                }

                (State::Format { long: true }, b'd') => {
                    // SAFETY: precondition
                    let ptr: i64 = unsafe { ap.arg() };
                    write!(console, "{}", ptr)?;
                    State::Normal
                }

                (State::Format { long: false }, b'u') => {
                    // SAFETY: precondition
                    let ptr: u32 = unsafe { ap.arg() };
                    write!(console, "{}", ptr)?;
                    State::Normal
                }

                (State::Format { long: true }, b'u') => {
                    // SAFETY: precondition
                    let ptr: u64 = unsafe { ap.arg() };
                    write!(console, "{}", ptr)?;
                    State::Normal
                }

                (State::Format { long: false }, b'x') => {
                    // SAFETY: precondition
                    let ptr: u32 = unsafe { ap.arg() };
                    write!(console, "{:x}", ptr)?;
                    State::Normal
                }

                (State::Format { long: true }, b'x') => {
                    // SAFETY: precondition
                    let ptr: u64 = unsafe { ap.arg() };
                    write!(console, "{:x}", ptr)?;
                    State::Normal
                }

                (State::Format { long: false }, b's') => {
                    // SAFETY: precondition
                    let msg: *const c_char = unsafe { ap.arg() };
                    if msg.is_null() {
                        console.puts("(null)");
                    } else {
                        // SAFETY: not null and points to valid location by precondition
                        console.putcs(unsafe { CStr::from_ptr(msg) });
                    }
                    State::Normal
                }

                (State::Format { long: _ }, b'l') => State::Format { long: true },

                (State::Format { long }, &ch) => {
                    console.putc(b'%');
                    if long {
                        console.putc(b'l');
                    }
                    console.putc(ch);
                    State::FormatError
                }

                (State::Normal, &ch) => {
                    console.putc(ch);
                    State::Normal
                }

                (State::FormatError, &ch) => {
                    console.putc(ch);
                    State::FormatError
                }
            }
        }
        Ok(())
    }

    // SAFETY: precondition
    let fmt = unsafe { CStr::from_ptr(format) };
    let mut console = CONSOLE.lock();
    // SAFETY: precondition
    let res = unsafe { printf_impl(&mut console, fmt, ap) };
    match res {
        Ok(()) => 0,
        Err(core::fmt::Error) => 1,
    }
}
