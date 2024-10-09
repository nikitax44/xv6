use crate::kalloc::pages::page::Page;
use crate::kalloc::pages::KMEM;
use core::panic::Location;
use core::ptr;

#[no_mangle]
unsafe extern "C" fn kinit() {
    // SAFETY:
    // precondition
    unsafe {
        super::init();
    }
}

#[no_mangle]
extern "C" fn kalloc() -> *mut Page {
    KMEM.lock()
        .alloc("ffi alloc")
        .map_or_else(ptr::null_mut, |page| page.leak().as_ptr())
}

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
