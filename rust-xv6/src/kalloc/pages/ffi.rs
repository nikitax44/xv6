use crate::kalloc::pages::page::{Page, PageHandle};
use crate::kalloc::pages::KMEM;
use core::panic::Location;
use core::ptr;

#[no_mangle]
extern "C" fn kinit() {
    crate::kalloc::init();
}

#[no_mangle]
extern "C" fn kalloc() -> *mut Page {
    KMEM.lock()
        .alloc("ffi alloc")
        .map_or(ptr::null_mut(), |page| page.into_box().leak().as_ptr())
}

/// # Safety
/// ptr must point to page-aligned memory. it transfers the ownership over that page.
#[no_mangle]
unsafe extern "C" fn kfree(ptr: Option<ptr::NonNull<Page>>) {
    let mut ptr = ptr.expect("kfree(NULL)");
    assert!(ptr.is_aligned(), "kfree(unaligned)");

    // SAFETY: we now own the page
    let rf: &'static mut Page = unsafe { ptr.as_mut() };
    let page = PageHandle::new(rf, Location::caller(), "ffi free");
    KMEM.lock().free(page);
}

#[no_mangle]
extern "C" fn free_pages() -> usize {
    KMEM.lock().free_pages()
}
