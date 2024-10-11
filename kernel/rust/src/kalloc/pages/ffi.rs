use crate::kalloc::pages::page::{Page, PageHandle};
use crate::kalloc::pages::KMEM;
use crate::memlayout::{addrof_end_kernel, PHYSTOP};
use crate::println;
use crate::util::once::Once;
use core::panic::Location;
use core::ptr;

/// # Safety
/// this call transfers the memory in range `kernel_end..=PHYSTOP` with &'static mut
/// # Panics
/// must be called only once
#[no_mangle]
unsafe extern "C" fn kinit() {
    static STATUS: Once = Once::new();

    STATUS
        // SAFETY:
        // precondition
        .init(|| unsafe { init_kmem() })
        .expect("multiple initialisation");
}

/// # Safety
/// memory range from then end of kernel to the PHYSTOP must be owned by caller
/// this call transfers this ownership to the static KMEM
/// # Panics
/// must be called only once
/// `.end` symbol is not page-aligned
unsafe fn init_kmem() {
    static STATE: Once = Once::new();
    let mut kmem = KMEM.lock();

    let start = addrof_end_kernel() as *mut Page;
    assert!(start.is_aligned(), "kernel's .end is not page-aligned");
    let start = ptr::NonNull::new(start).unwrap();

    let inner = || {
        let pages = (0..)
            // SAFETY: it is one contiguous object
            .map(|i| unsafe { start.add(i) })
            .take_while(|&page| (page.as_ptr() as usize) < PHYSTOP)
            // SAFETY: we own the value. Page is valid for any bit pattern
            .map(|mut page| unsafe { page.as_mut() });
        kmem.free_range(pages);
    };

    STATE.init(inner).expect("multiple KMEM initialisations");

    println!("kalloc pool: {} pages", kmem.free_pages());
    println!("  start: {:?}", start);
    println!("  end:   {:?}", PHYSTOP as *const ());
    println!();
}

#[no_mangle]
extern "C" fn kalloc() -> *mut Page {
    KMEM.lock()
        .alloc("ffi alloc")
        .expect("failed to allocate for kalloc")
        .into_box()
        .leak()
        .as_ptr()
}

/// # Safety
/// ptr must point to page-aligned memory. it transfers the ownership over that page.
#[no_mangle]
unsafe extern "C" fn kfree(ptr: Option<ptr::NonNull<Page>>) {
    let mut ptr = ptr.expect("kfree(NULL)");
    assert!(ptr.is_aligned(), "kfree(unaligned)");

    // SAFETY:
    // precondition
    let rf = unsafe { ptr.as_mut() };
    let mut page = PageHandle::new(rf, None);
    page.set_origin(Location::caller(), "ffi free");
    KMEM.lock().free(page);
}

#[no_mangle]
extern "C" fn free_pages() -> usize {
    KMEM.lock().free_pages()
}
