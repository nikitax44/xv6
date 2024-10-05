use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};
use core::panic::PanicInfo;

const BUG_MSG: &CStr = c"rust: BUG in panic handler";
const EMPTY_MSG: &CStr = c"rust: empty panic message";
const RUST_PREFIX: &[u8] = b"rust: ";

extern "C" {
    fn panic(msg: *const c_char) -> !;
}

fn raw_panic(msg: &CStr) -> ! {
    unsafe { panic(msg.as_ptr()) }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    if let Some(msg) = info.message().as_str() {
        let mut out = Vec::with_capacity(RUST_PREFIX.len() + msg.len() + 1);
        out.extend_from_slice(RUST_PREFIX);
        out.extend_from_slice(msg.as_bytes());
        out.push(b'\0');
        if let Ok(cstr) = CString::from_vec_with_nul(out) {
            raw_panic(&cstr)
        } else {
            raw_panic(BUG_MSG)
        }
    } else {
        raw_panic(EMPTY_MSG)
    }
}
