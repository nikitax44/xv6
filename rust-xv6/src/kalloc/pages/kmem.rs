use crate::kalloc::pages::page::{Page, PageHandle};
use crate::kalloc::pages::KMEMError;
use crate::kalloc::KALLOC;
use crate::memlayout::PGSIZE;
use alloc::boxed::Box;
use core::panic::Location;

pub struct KMem;
pub struct KMemLock;
impl KMemLock {
    #[must_use]
    pub const fn lock(&self) -> KMem {
        let _ = self;
        KMem
    }
}

pub static KMEM: KMemLock = KMemLock;
impl KMem {
    #[track_caller]
    /// # Errors
    /// out of memory
    pub fn alloc(&mut self, purpose: &'static str) -> Result<PageHandle, KMEMError> {
        let caller = Location::caller();
        Box::<Page>::try_new_uninit()
            .map(Box::leak)
            .map(|page| PageHandle::new_uninit(page, caller, purpose))
            .map_err(|core::alloc::AllocError| KMEMError::NoInfo)
    }

    pub fn free(&mut self, page: PageHandle) {
        // SAFETY: we leaked the value for 'static
        let page = unsafe { Box::from_non_null(page.into_box().leak()) };
        drop(page);
    }

    #[must_use]
    pub fn free_pages(&self) -> usize {
        // heuristic
        let bytes = KALLOC.in_context(|heap| heap.stats_total_bytes() - heap.stats_alloc_actual());
        let pages = bytes / PGSIZE;
        pages * 3 / 4
    }
}
