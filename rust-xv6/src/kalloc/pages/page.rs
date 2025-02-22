use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::PGSIZE;
use core::cell::UnsafeCell;
use core::mem;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::panic::Location;

type Origin = &'static Location<'static>;

#[repr(C, align(4096))]
#[must_use]
/// `UnsafeCell` is used to indicate interior mutability
pub struct Page(UnsafeCell<[u8; PGSIZE]>);

impl Page {
    pub fn zeroed(&mut self) -> &mut [u8; PGSIZE] {
        self.0.get_mut().fill(0);
        self.0.get_mut()
    }
}

/// # SAFETY:
/// Page is accessible only through mutable reference
unsafe impl Sync for Page {}

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

    pub const fn set_origin(&mut self, origin: Origin, purpose: &'static str) {
        self.origin = origin;
        self.purpose = purpose;
    }

    /// # Panics
    /// never
    pub const fn into_box(mut self) -> ThinBox<Page> {
        let value = self.ptr.take().unwrap();
        mem::forget(self);
        value
    }

    pub const fn from_box(value: ThinBox<Page>, origin: Origin, purpose: &'static str) -> Self {
        Self {
            ptr: Some(value),
            origin,
            purpose,
        }
    }
}

impl Drop for PageHandle {
    fn drop(&mut self) {
        let _ = ManuallyDrop::new(self.ptr.take());
        panic!("memory leak: {:?}", core::hint::black_box(self));
    }
}
