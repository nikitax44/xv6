use crate::kalloc::pages::page::PageHandle;
use crate::kalloc::pages::KMEM;

crate::export_c_fn! {
    fn __kinit:kinit() {
        crate::kalloc::init();
    }

    fn __kalloc:kalloc() -> Option<PageHandle> {
        KMEM.lock()
            .alloc("ffi alloc")
            .ok()
    }

    /// # Safety
    /// ptr must point to page-aligned memory. it transfers the ownership over that page.
    fn __kfree:kfree(ptr: Option<PageHandle>) {
        let mut page = ptr.expect("kfree(NULL)");

        page.set_origin(core::panic::Location::caller(), "ffi kfree");
        KMEM.lock().free(page);
    }

}

#[no_mangle]
extern "C" fn free_pages() -> usize {
    KMEM.lock().free_pages()
}
