use crate::kalloc::pages::page::Page;
use crate::kalloc::pages::KMEM;
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
        .init(|| unsafe { super::init() })
        .expect("multiple initialisation");
}

#[no_mangle]
extern "C" fn kalloc() -> *mut Page {
    KMEM.lock()
        .alloc("ffi alloc")
        .map_or_else(ptr::null_mut, |page| page.into_box().leak().as_ptr())
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
    let mut page = super::PageHandle::new(rf, None);
    page.set_origin(Location::caller(), "ffi free");
    KMEM.lock().free(page);
}

#[no_mangle]
extern "C" fn free_pages() -> usize {
    KMEM.lock().free_pages()
}
