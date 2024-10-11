use core::panic::Location;
use page::PageHandle;

// mod ffi;
// mod kmem;
pub mod page;

use crate::kalloc::pages::page::Page;
use core::ptr::NonNull;

pub struct KMem;
pub static KMEM: KMem = KMem;
impl KMem {
    #[must_use]
    pub const fn lock(&self) -> &Self {
        self
    }
    #[must_use]
    #[track_caller]
    pub fn alloc(&self, purpose: &'static str) -> Option<PageHandle> {
        extern "C" {
            pub fn kalloc() -> Option<NonNull<Page>>;
        }
        // SAFETY: safe
        unsafe { kalloc() }
            // SAFETY: we now own the value. any bit pattern is valid
            .map(|mut ptr| unsafe { ptr.as_mut() })
            .map(|ptr| PageHandle::new(ptr, None))
            .map(|mut ph| {
                ph.mark_allocated(Location::caller(), purpose);
                ph
            })
    }

    pub fn free(&self, mut page: PageHandle) {
        extern "C" {
            pub fn kfree(ptr: NonNull<Page>);
        }

        page.mark_freed();
        // SAFETY: we own the value. it is of the valid type
        unsafe { kfree(page.into_box().leak()) }
    }
}
