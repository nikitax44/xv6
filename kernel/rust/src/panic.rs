use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};
use core::fmt::{self, Write};
use core::panic::PanicInfo;

const BUG_MSG: &CStr = c"rust: NUL in panic message";

extern "C" {
    fn panic(msg: *const c_char) -> !;
}

fn raw_panic(msg: &CStr) -> ! {
    unsafe { panic(msg.as_ptr()) }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    let mut vec = Vec::new();
    let mut out = Bytes(&mut vec);
    writeln!(out, "RUST: {info}").ok();
    vec.push(b'\0');
    if let Ok(cstr) = CString::from_vec_with_nul(vec) {
        raw_panic(&cstr)
    } else {
        raw_panic(BUG_MSG)
    }
}

struct Bytes<'s>(&'s mut Vec<u8>);

impl fmt::Write for Bytes<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());

        Ok(())
    }
}
