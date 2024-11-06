use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::PGSIZE;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::panic::Location;

type Origin = &'static Location<'static>;

#[repr(C, align(4096))]
#[must_use]
pub struct Page([u8; PGSIZE]);

impl Page {
    pub fn zeroed(&mut self) -> &mut [u8; PGSIZE] {
        self.0.fill(0);
        &mut self.0
    }
}

#[must_use]
#[derive(Debug)]
pub struct PageHandle {
    ptr: Option<ThinBox<Page>>,
    origin: Origin,
    purpose: &'static str,
}

impl PageHandle {
    #[track_caller]
    pub fn new(ptr: &'static mut Page, origin: Origin, purpose: &'static str) -> Self {
        Self {
            ptr: Some(ThinBox::from(ptr)),
            origin,
            purpose,
        }
    }

    #[track_caller]
    pub fn new_uninit(
        ptr: &'static mut MaybeUninit<Page>,
        origin: Origin,
        purpose: &'static str,
    ) -> Self {
        // SAFETY: Page is write-only until written to
        Self::new(unsafe { ptr.assume_init_mut() }, origin, purpose)
    }

    pub fn set_origin(&mut self, origin: Origin, purpose: &'static str) {
        self.origin = origin;
        self.purpose = purpose;
    }

    /// # Panics
    /// never
    pub fn into_box(self) -> ThinBox<Page> {
        let mut this = ManuallyDrop::new(self);
        this.ptr.take().unwrap()
    }
}

impl Drop for PageHandle {
    fn drop(&mut self) {
        let _ = ManuallyDrop::new(self.ptr.take());
        panic!("memory leak: {:?}", core::hint::black_box(self));
    }
}
