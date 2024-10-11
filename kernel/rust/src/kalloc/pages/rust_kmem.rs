use super::page::{Page, PageHandle};
use crate::kalloc::pages::KMEMError;
use core::panic::Location;
use spin::Mutex;

/// # Invariant
/// `data` is linked list of page-aligned pointers
/// `free_pages` is length of that list
/// `KMem` has ownership over all pages in that list
pub struct KMem {
    data: Option<PageHandle>,
    free_pages: usize,
}

pub static KMEM: Mutex<KMem> = Mutex::new(KMem {
    data: None,
    free_pages: 0,
});

impl KMem {
    #[track_caller]
    pub fn free_range(&mut self, pages: impl IntoIterator<Item = &'static mut Page>) {
        for page in pages {
            self.free(PageHandle::new(page, Some(Location::caller())));
        }
    }

    #[track_caller]
    /// # Panics
    /// never
    pub fn free(&mut self, mut page: PageHandle) {
        page.mark_freed();
        let next = self.data.take();
        page.as_uninit::<Option<PageHandle>>().write(next);

        // for some reason it produces different assembly
        // self.data = Some(page);
        self.data.replace(page).ok_or(()).unwrap_err();

        self.free_pages += 1;
    }

    #[track_caller]
    /// # Errors
    /// out of memory
    pub fn alloc(&mut self, purpose: &'static str) -> Result<PageHandle, KMEMError> {
        let mut page = self
            .data
            .take()
            .ok_or(KMEMError::NoFreePages(self.free_pages))?;

        // SAFETY:
        // we wrote value of this type beforehand.
        self.data = unsafe {
            page.as_uninit::<Option<PageHandle>>()
                .assume_init_mut()
                .take()
        };
        self.free_pages -= 1;
        page.mark_allocated(Location::caller(), purpose);
        Ok(page)
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
