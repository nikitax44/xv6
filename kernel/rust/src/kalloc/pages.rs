use crate::memlayout::{end_kernel, PGSIZE, PHYSTOP};
use crate::println;
use crate::spinlock::Spinlock;
use crate::util::once::Once;
use core::mem;
use core::panic::Location;
use core::ptr;

type Origin = &'static Location<'static>;

#[repr(align(4096))]
pub struct Page(pub mem::MaybeUninit<[u8; PGSIZE]>);

pub struct PageHandle {
    ptr: ptr::NonNull<Page>,
    origin: Origin,
    allocated: bool,
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
/// memory range from then end of kernel to the PHYSTOP must be owned by caller
/// this call transfers this ownership to the static KMEM
/// # Panics
/// must be called only once
/// `.end` symbol is not page-aligned
#[track_caller]
pub unsafe fn init() {
    static STATE: Once = Once::new();
    let mut kmem = KMEM.lock();

    // SAFETY:
    // we're only using address and not actual value
    let start = end_kernel() as *mut Page;

    let inner = || {
        assert!(start.is_aligned(), "kernel's .end is not page-aligned");
        let start = ptr::NonNull::new(start).unwrap();

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
}

impl PageHandle {
    #[must_use]
    #[track_caller]
    pub fn new(ptr: &'static mut Page, origin: Option<Origin>) -> Self {
        Self {
            ptr: ptr.into(),
            origin: origin.unwrap_or_else(Location::caller),
            allocated: false,
        }
    }

    pub fn set_origin(&mut self, origin: Origin) {
        self.origin = origin;
    }

    fn fill(&mut self, byte: u8) {
        // SAFETY: any bit pattern is valid
        unsafe { self.raw_page().0.assume_init_mut() }.fill(byte);
    }
    fn mark_freed(&mut self) {
        self.fill(0x19);
        self.allocated = false;
    }

    fn mark_allocated(&mut self, origin: Origin) {
        self.fill(0x1d);
        self.set_origin(origin);
        self.allocated = true;
    }

    #[must_use]
    pub fn leak(self) -> ptr::NonNull<Page> {
        mem::ManuallyDrop::new(self).ptr
    }

    pub fn raw_page(&mut self) -> &mut Page {
        // SAFETY:
        // we own the ptr contents and any bit pattern is valid for Page
        unsafe { self.ptr.as_mut() }
    }

    #[must_use]
    pub fn zeroed(mut self) -> Self {
        self.fill(0);
        self
    }

    /// # Panics
    /// if sizeof(T)>sizeof(Page)
    #[must_use]
    pub fn leak_uninit<T: 'static>(self) -> &'static mut mem::MaybeUninit<T> {
        assert!(
            size_of::<T>() <= size_of::<Page>(),
            "attempt to get uninit with size exceeding Page"
        );
        // SAFETY:
        // we own the ptr and MaybeUninit is valid for any bit pattern
        unsafe { self.leak().cast().as_mut() }
    }

    /// # Panics
    /// if sizeof(T)>sizeof(Page)
    #[must_use]
    pub fn as_uninit<T: 'static>(&mut self) -> &mut mem::MaybeUninit<T> {
        assert!(
            size_of::<T>() <= size_of::<Page>(),
            "attempt to get uninit with size exceeding Page"
        );
        // SAFETY:
        // we own the ptr and MaybeUninit is valid for any bit pattern
        unsafe { self.ptr.cast().as_mut() }
    }
}

impl Drop for PageHandle {
    fn drop(&mut self) {
        println!(
            "memory leak: {:?}, origin: {}, allocated: {:?}",
            self.ptr, self.origin, self.allocated
        );
    }
}

impl KMem {
    #[track_caller]
    pub fn free_range(&mut self, pages: impl IntoIterator<Item = &'static mut Page>) {
        for page in pages {
            self.free(PageHandle::new(page, Some(Location::caller())));
        }
    }

    #[track_caller]
    pub fn free(&mut self, mut page: PageHandle) {
        page.mark_freed();
        page.as_uninit::<Option<PageHandle>>()
            .write(self.data.take());
        self.data = Some(page);
        self.free_pages += 1;
    }

    #[track_caller]
    pub fn alloc(&mut self) -> Option<PageHandle> {
        if let Some(mut page) = self.data.take() {
            // SAFETY:
            // we wrote value of this type beforehand.
            self.data = unsafe { page.as_uninit::<Option<PageHandle>>().assume_init_mut() }.take();
            self.free_pages -= 1;
            page.mark_allocated(Location::caller());
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
    use super::{Page, KMEM};
    use core::ffi::c_void;
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
    extern "C" fn kalloc() -> *mut c_void {
        KMEM.lock()
            .alloc()
            .map(|page| page.leak().cast().as_ptr())
            .unwrap_or_else(ptr::null_mut)
    }

    #[no_mangle]
    unsafe extern "C" fn kfree(ptr: Option<ptr::NonNull<Page>>) {
        let mut ptr = ptr.expect("free(null)");
        assert!(ptr.is_aligned(), "free(unaligned)");
        // SAFETY:
        // precondition
        let rf = unsafe { ptr.as_mut() };
        KMEM.lock().free(super::PageHandle::new(rf, None));
    }

    #[no_mangle]
    extern "C" fn free_pages() -> usize {
        KMEM.lock().free_pages()
    }
}
