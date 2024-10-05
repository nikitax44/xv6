#![no_std]

extern crate alloc;

mod kalloc {
    use alloc::alloc::{GlobalAlloc, Layout};
    use core::cell::UnsafeCell;
    use core::ptr::null_mut;
    use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

    const ARENA_SIZE: usize = 128 * 1024;
    const MAX_SUPPORTED_ALIGN: usize = 4096;
    #[repr(C, align(4096))] // 4096 == MAX_SUPPORTED_ALIGN
    struct SimpleAllocator {
        arena: UnsafeCell<[u8; ARENA_SIZE]>,
        remaining: AtomicUsize, // we allocate from the top, counting down
    }

    #[global_allocator]
    static ALLOCATOR: SimpleAllocator = SimpleAllocator {
        arena: UnsafeCell::new([0x07; ARENA_SIZE]),
        remaining: AtomicUsize::new(ARENA_SIZE),
    };

    unsafe impl Sync for SimpleAllocator {}

    unsafe impl GlobalAlloc for SimpleAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let size = layout.size();
            let align = layout.align();

            // `Layout` contract forbids making a `Layout` with align=0, or align not power of 2.
            // So we can safely use a mask to ensure alignment without worrying about UB.
            let align_mask_to_round_down = !(align - 1);

            if align > MAX_SUPPORTED_ALIGN {
                return null_mut();
            }

            let mut allocated = 0;
            if self
                .remaining
                .fetch_update(Relaxed, Relaxed, |mut remaining| {
                    if size > remaining {
                        return None;
                    }
                    remaining -= size;
                    remaining &= align_mask_to_round_down;
                    allocated = remaining;
                    Some(remaining)
                })
                .is_err()
            {
                return null_mut();
            };
            self.arena.get().cast::<u8>().add(allocated)
        }
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
    }
}

mod panic {
    use alloc::ffi::CString;
    use alloc::vec::Vec;
    use core::ffi::{c_char, CStr};
    use core::panic::PanicInfo;
    // const ALLOC_BROKEN_MSG: &CStr = c"rust: failed to allocate buffer for proper panic message";
    const EMPTY_MSG: &CStr = c"rust: empty message";
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
            raw_panic(&CString::from_vec_with_nul(out).unwrap())
        } else {
            raw_panic(EMPTY_MSG)
        }
    }
}

#[no_mangle]
pub extern "C" fn rustdiv(x: i32, y: i32) -> i32 {
    x / y
}

#[no_mangle]
pub extern "C" fn rustinit() {
    // println!("hello world from rust!");
}
