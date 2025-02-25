mod sysproc;

#[macro_export]
macro_rules! syscall {
    () => {};
    (fn $name:ident($($arg:ident : $tp:ident $([$sz:literal])?),* $(,)?) -> i64 $body:block) => {
        #[no_mangle]
        extern "C" fn $name () -> i64 {
            #[allow(unused_mut)]
            let mut argidx = 0;
            $(syscall!(@arg argidx $arg $tp$([$sz])?);)*
            let _ = argidx;

            $body
        }
    };

    (@arg $argidx:ident $arg:ident $tp:ident $([$sz:literal])?) => {
        let $arg = syscall!(@tp $argidx $tp$([$sz])?);
    };

    (@tp $argidx:ident u64) => {{
        // SAFETY: it is save
        let value = unsafe {
            $crate::bindings::argraw($argidx)
        };
        $argidx += 1;
        value
    }};

    (@tp $argidx:ident Proc) => {
        // SAFETY: it is safe
        unsafe { $crate::bindings::myproc() }
    };

    (@tp $argidx:ident CStr[$sz:literal]) => {{
        let mut buf = $crate::util::string::InlineCString {
            buffer: [0;$sz],
            size_with_null:0
        };
        // SAFETY: buffer was specified correctly
        let size = unsafe {
            $crate::bindings::argstr($argidx, buf.buffer.as_mut_ptr(), buf.buffer.len() as u64)
        };
        if (size == -1) {
            return -1;
        }
        assert!(size>=0);
        buf.size_with_null=size as usize + 1;

        $argidx += 1;
        buf
    }};
}

pub fn copyout<T>(outaddr: u64, value: &T) -> core::ffi::c_int {
    // SAFETY: safe
    let proc = unsafe { crate::bindings::myproc() };
    // SAFETY: `pagetable` is readonly
    let pagetable = unsafe { (*proc).pagetable };
    // SAFETY: `value` is valid for reads for `sizeof(value)` bytes and pagetable is valid
    unsafe {
        crate::bindings::copyout(
            pagetable,
            outaddr,
            &raw const value as *const _,
            size_of_val(value) as u64,
        )
    }
}
