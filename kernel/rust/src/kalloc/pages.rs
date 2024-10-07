const PHYSTOP: usize = 0x8000_0000 + 128 * 1024 * 1024;
const PGSIZE: usize = 4096;
use crate::println;
use crate::spinlock::Spinlock;
use core::ffi::c_char;
use core::mem;
use core::ptr;

extern "C" {
    static end: c_char;
}

#[repr(align(4096))]
pub struct Page(pub [u8; PGSIZE]);

pub struct PageHandle {
    ptr: ptr::NonNull<Page>,
}

/// # Invariant
/// `data` is linked list of page-aligned pointers
/// `free_pages` is length of that list
/// `KMem` has ownership over all pages in that list
pub struct KMem {
    data: Option<PageHandle>,
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
    let start = unsafe { ptr::from_ref(&end) }.cast::<Page>().cast_mut();
    assert!(start.is_aligned(), "kernel's .end is not page-aligned");
    assert_eq!(PHYSTOP % PGSIZE, 0, "PHYSTOP is not page-aligned");
    // # SAFETY:
    // see preconditions
    unsafe {
        kmem.free_range(
            ptr::NonNull::new(start).unwrap(),
            ptr::NonNull::new(PHYSTOP as *mut Page).unwrap(),
        );
    }
}

impl PageHandle {
    /// # Safety
    /// you must have the ownership over the page at ptr
    /// # Panics
    /// ptr must be page-aligned
    #[must_use]
    pub unsafe fn new(ptr: ptr::NonNull<Page>) -> Self {
        assert!(ptr.is_aligned(), "ptr is not page-aligned");
        Self { ptr }
    }

    fn mark_freed(&mut self) {
        self.raw_page().0.fill(0x19);
    }

    fn mark_allocated(&mut self) {
        self.raw_page().0.fill(0x1d);
    }

    #[must_use]
    pub fn leak(self) -> ptr::NonNull<Page> {
        mem::ManuallyDrop::new(self).ptr
    }

    pub fn raw_page(&mut self) -> &mut Page {
        // SAFETY:
        // we own the ptr contents
        unsafe { self.ptr.as_mut() }
    }

    #[must_use]
    pub fn zeroed(mut self) -> Self {
        self.raw_page().0.fill(0);
        self
    }

    /// # Panics
    /// if sizeof(T)>sizeof(Page)
    pub fn as_uninit<T>(&mut self) -> &mut mem::MaybeUninit<T> {
        assert!(
            mem::size_of::<T>() <= mem::size_of::<Page>(),
            "attempt to get uninit with size exceeding Page"
        );
        assert!(mem::align_of::<T>() <= mem::align_of::<Page>(), "wut?");
        // SAFETY:
        // we own the ptr and MaybeUninit is valid for any bit pattern as well as Page
        unsafe { self.ptr.cast().as_mut() }
    }
}

impl Drop for PageHandle {
    fn drop(&mut self) {
        println!("memory leak: {:?}", self.ptr);
    }
}

impl KMem {
    /// # Safety
    /// `start` and `end` must be aligned to PGSIZE
    /// you must have the ownership over this memory range
    /// # Panics
    /// `start` and `end` must be aligned to PGSIZE
    pub unsafe fn free_range(&mut self, start: ptr::NonNull<Page>, end_: ptr::NonNull<Page>) {
        assert!(start.is_aligned(), "start is not page-aligned");
        assert!(end_.is_aligned(), "end is not page-aligned");
        for p in (start.as_ptr() as usize..end_.as_ptr() as usize).step_by(PGSIZE) {
            let p = ptr::NonNull::new(p as *mut Page).unwrap();
            // SAFETY: we have ownership
            let page = unsafe { PageHandle::new(p) };
            self.free(page);
        }
    }

    pub fn free(&mut self, mut page: PageHandle) {
        page.mark_freed();
        page.as_uninit::<Option<PageHandle>>()
            .write(self.data.take());
        self.data = Some(page);
        self.free_pages += 1;
    }

    pub fn alloc(&mut self) -> Option<PageHandle> {
        if let Some(mut page) = self.data.take() {
            // SAFETY:
            // we wrote value of this type beforehand.
            self.data = unsafe { page.as_uninit::<Option<PageHandle>>().assume_init_mut() }.take();
            self.free_pages -= 1;
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
            .map(|page| page.leak().cast().as_ptr())
            .unwrap_or_else(ptr::null_mut)
    }

    #[no_mangle]
    unsafe extern "C" fn kfree(ptr: *mut c_void) {
        let ptr = ptr::NonNull::new(ptr.cast()).expect("kfree(null)");
        // SAFETY:
        // precondition
        let page = unsafe { super::PageHandle::new(ptr) };
        KMEM.lock().free(page);
    }

    #[no_mangle]
    extern "C" fn free_pages() -> usize {
        KMEM.lock().free_pages()
    }
}
