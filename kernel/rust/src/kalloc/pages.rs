const PHYSTOP: usize = 0x80000000 + 128 * 1024 * 1024;
const PGSIZE: usize = 4096;
use crate::println;
use crate::spinlock::Spinlock;
use core::ffi::c_char;
use core::ptr;

extern "C" {
    static end: c_char;
}

#[repr(align(4096))]
pub struct Page(pub [u8; PGSIZE]);

pub struct PageHandle {
    ptr: &'static mut Page,
}

/// # Invariant
/// `data` is linked list of page-aligned pointers
/// `free_pages` is length of that list
/// `KMem` has ownership over all pages in that list
pub struct KMem {
    data: Option<ptr::NonNull<Page>>,
    free_pages: usize,
}

/// # SAFETY:
/// pointer space is shared between all harts
unsafe impl Send for KMem {}

pub static KMEM: Spinlock<KMem> = Spinlock::new(KMem {
    data: None,
    free_pages: 0,
});

/// # Safety
/// must be called only once
/// memory range from then end of kernel to the PHYSTOP must be owned by caller
/// this call transfers this ownership to the static KMEM
/// # Panics
/// `end` or `PHYSTOP` aren't page-aligned
pub unsafe fn init() {
    let mut kmem = KMEM.lock();
    // SAFETY:
    // we're only using address and not actual value
    let start = unsafe { ptr::from_ref(&end) } as *mut ();
    assert_eq!(
        start as usize % PGSIZE,
        0,
        "kernel's .end is not page-aligned"
    );
    assert_eq!(PHYSTOP % PGSIZE, 0, "PHYSTOP is not page-aligned");
    // # SAFETY:
    // see preconditions
    unsafe { kmem.free_range(start, PHYSTOP as *mut ()) }
}

impl PageHandle {
    /// # Safety
    /// ptr must be page-aligned
    /// you must have the ownership over the page at ptr
    pub unsafe fn new(ptr: *mut Page) -> Self {
        Self {
            // SAFETY:
            // precondition
            ptr: unsafe { &mut *ptr },
        }
    }

    fn mark_freed(&mut self) {
        self.ptr.0.fill(0x19);
    }

    fn mark_allocated(&mut self) {
        self.ptr.0.fill(0x1d);
    }

    pub fn leak(self) -> *mut () {
        let ptr = ptr::from_ref(self.ptr);
        core::mem::forget(self);
        ptr as *mut ()
    }
}

impl Drop for PageHandle {
    fn drop(&mut self) {
        println!("memory leak: {:?}", ptr::from_ref(self.ptr));
    }
}

impl KMem {
    /// # Safety
    /// `start` and `end` must be aligned to PGSIZE
    /// you must have the ownership over this memory range
    /// # Panics
    /// `start` and `end` must be aligned to PGSIZE
    pub unsafe fn free_range(&mut self, start: *mut (), end_: *mut ()) {
        assert_eq!(start as usize % PGSIZE, 0, "start is not page-aligned");
        assert_eq!(end_ as usize % PGSIZE, 0, "end is not page-aligned");
        for p in (start as usize..end_ as usize).step_by(PGSIZE) {
            // SAFETY: we have ownership
            unsafe {
                self.free_page(p as *mut ());
            }
        }
    }

    /// # Safety
    /// you must have the ownership over page
    /// # Panics
    /// `page` is not page-aligned
    pub unsafe fn free_page(&mut self, page: *mut ()) {
        assert_eq!(page as usize % PGSIZE, 0, "ptr is not page-aligned");
        // SAFETY:
        // see preconditions
        self.free(unsafe { PageHandle::new(page as *mut Page) });
    }

    /// # Panics
    /// never
    pub fn free(&mut self, mut page: PageHandle) {
        page.mark_freed();
        let ptr = page.ptr as *mut Page;
        core::mem::forget(page);

        // SAFETY:
        // we have ownership over memory at ptr and ptr is page-aligned
        unsafe {
            ptr::write(ptr as *mut Option<ptr::NonNull<Page>>, self.data.take());
        }

        self.data = ptr::NonNull::new(ptr); // always returns Some
        assert!(self.data.is_some(), "wut?");

        self.free_pages += 1;
    }

    pub fn alloc(&mut self) -> Option<PageHandle> {
        if let Some(page) = self.data {
            // SAFETY:
            // we can read contents.
            self.data = ptr::NonNull::new(unsafe { ptr::read(page.as_ptr() as *mut *mut Page) });
            self.free_pages -= 1;
            // SAFETY:
            // we own the page
            let mut page = unsafe { PageHandle::new(page.as_ptr()) };
            page.mark_allocated();
            Some(page)
        } else {
            None
        }
    }

    #[must_use]
    pub fn free_pages(&self) -> usize {
        self.free_pages
    }
}

mod ffi {
    use super::KMEM;
    use crate::println;
    use core::ffi::c_void;
    use core::ptr;

    #[no_mangle]
    unsafe extern "C" fn kinit() {
        // SAFETY:
        // precondition
        unsafe {
            super::init();
        }
        println!("kalloc pool: {} pages", free_pages());
        // SAFETY:
        // ignore
        println!("  start: {:?}", unsafe { ptr::from_ref(&super::end) }
            as *mut ());
        println!("  end:   {:?}", super::PHYSTOP as *const ());
        kalloc();
    }

    #[no_mangle]
    extern "C" fn kalloc() -> *mut c_void {
        KMEM.lock()
            .alloc()
            .map(|page| page.leak().cast())
            .unwrap_or_else(ptr::null_mut)
    }

    #[no_mangle]
    unsafe extern "C" fn kfree(ptr: *mut c_void) {
        // SAFETY:
        // precondition
        unsafe {
            KMEM.lock().free_page(ptr as *mut ());
        }
    }

    #[no_mangle]
    extern "C" fn free_pages() -> usize {
        return KMEM.lock().free_pages();
    }
}
