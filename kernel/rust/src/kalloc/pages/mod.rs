use crate::memlayout::{addrof_end_kernel, PHYSTOP};
use crate::println;
use crate::util::once::Once;
use crate::util::spinlock::Spinlock;
use core::panic::Location;
use core::ptr;
use page::{Page, PageHandle};

mod ffi;
pub mod page;

/// # Invariant
/// `data` is linked list of page-aligned pointers
/// `free_pages` is length of that list
/// `KMem` has ownership over all pages in that list
pub struct KMem {
    data: Option<PageHandle>,
    free_pages: usize,
}

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
    pub fn alloc(&mut self, purpose: &'static str) -> Option<PageHandle> {
        let mut page = self.data.take()?;

        // SAFETY:
        // we wrote value of this type beforehand.
        self.data = unsafe {
            page.as_uninit::<Option<PageHandle>>()
                .assume_init_mut()
                .take()
        };
        self.free_pages -= 1;
        page.mark_allocated(Location::caller(), purpose);
        Some(page)
    }

    #[must_use]
    pub const fn free_pages(&self) -> usize {
        self.free_pages
    }
}

impl Drop for KMem {
    fn drop(&mut self) {
        panic!("KMem was dropped");
    }
}
