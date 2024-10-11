use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};
use core::fmt::{self, Write};
use core::panic::PanicInfo;

extern "C" {
    /// # Safety
    /// msg must point to valid C-string
    #[link_name = "_panic"]
    fn panic_impl(msg: *const c_char) -> !;
}

pub fn raw_panic(msg: &CStr) -> ! {
    // SAFETY:
    // msg is valid CStr
    unsafe { panic_impl(msg.as_ptr()) }
}

#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    let mut vec = Vec::new();
    let mut out = Bytes(&mut vec);
    writeln!(out, "RUST: {info}").ok();
    vec.push(b'\0');
    crate::println!("calling panic");
    CString::from_vec_with_nul(vec).map_or_else(
        |_| raw_panic(c"rust: NUL in panic message"),
        |msg| raw_panic(&msg),
    )
}

/// # Safety
/// msg must point to valid null-terminated string
/// # Panics
/// always, that is the point of this function
/// the function itself shouldn't panic
#[no_mangle]
unsafe extern "C" fn panic(msg: *const c_char) -> ! {
    // SAFETY:
    // see precondition
    let cstr = unsafe { CStr::from_ptr::<'_>(msg) };

    let mut vec = Vec::new();
    vec.extend_from_slice(b"C FFI:  ");
    vec.extend_from_slice(cstr.to_bytes_with_nul());
    let out = CStr::from_bytes_with_nul(&vec).unwrap();
    raw_panic(out)
}

struct Bytes<'s>(&'s mut Vec<u8>);

impl Write for Bytes<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
