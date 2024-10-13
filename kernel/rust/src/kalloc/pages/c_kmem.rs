use crate::kalloc::pages::page::{Page, PageHandle};
use crate::kalloc::pages::KMEMError;
use core::panic::Location;
use core::ptr::NonNull;

pub struct KMem;
pub struct KMemLock;
impl KMemLock {
    #[must_use]
    #[expect(clippy::unused_self, reason = "it is used to monomorphise the api")]
    pub const fn lock(&self) -> KMem {
        KMem
    }
}

pub static KMEM: KMemLock = KMemLock;
impl KMem {
    #[track_caller]
    /// # Errors
    /// out of memory
    pub fn alloc(&mut self, purpose: &'static str) -> Result<PageHandle, KMEMError> {
        extern "C" {
            pub fn kalloc() -> Option<NonNull<Page>>;
        }
        // SAFETY: safe
        unsafe { kalloc() }
            .ok_or_else(|| KMEMError::NoFreePages(self.free_pages()))
            // SAFETY: we now own the value. any bit pattern is valid
            .map(|mut ptr| unsafe { ptr.as_mut() })
            .map(|ptr| PageHandle::new(ptr, None))
            .map(|mut ph| {
                ph.mark_allocated(Location::caller(), purpose);
                ph
            })
    }

    pub fn free(&mut self, mut page: PageHandle) {
        extern "C" {
            pub fn kfree(ptr: NonNull<Page>);
        }

        page.mark_freed();
        // SAFETY: we own the value. it is of the valid type
        unsafe { kfree(page.into_box().leak()) }
    }

    #[must_use]
    pub fn free_pages(&self) -> usize {
        extern "C" {
            pub fn free_pages() -> usize;
        }
        // SAFETY: safe
        unsafe { free_pages() }
    }
}
