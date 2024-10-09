use crate::memlayout::PGSIZE;
use crate::println;
use core::marker::PhantomPinned;
use core::mem::MaybeUninit;
use core::panic::Location;
use core::{mem, ptr};

type Origin = &'static Location<'static>;

#[repr(align(4096))]
#[must_use]
pub struct Page(pub MaybeUninit<[u8; PGSIZE]>, PhantomPinned);

impl Page {
    pub const fn initial() -> Self {
        Self(MaybeUninit::uninit(), PhantomPinned)
    }
}

#[must_use]
pub struct PageHandle {
    ptr: ptr::NonNull<Page>,
    origin: Origin,
    in_use: bool,
    purpose: &'static str,
}

/// # SAFETY:
/// safe to send because we own the Page
unsafe impl Send for PageHandle {}

impl PageHandle {
    #[track_caller]
    pub fn new(ptr: &'static mut Page, origin: Option<Origin>) -> Self {
        Self {
            ptr: ptr.into(),
            origin: origin.unwrap_or_else(Location::caller),
            purpose: "unknown",
            in_use: true,
        }
    }

    pub fn set_origin(&mut self, origin: Origin, purpose: &'static str) {
        self.origin = origin;
        self.purpose = purpose;
    }

    fn fill(&mut self, byte: u8) {
        // SAFETY: any bit pattern is valid
        unsafe { self.raw_page().0.assume_init_mut() }.fill(byte);
    }
    pub(crate) fn mark_freed(&mut self) {
        self.fill(0x19);
        self.in_use = false;
    }

    pub(crate) fn mark_allocated(&mut self, origin: Origin, purpose: &'static str) {
        self.fill(0x1d);
        self.set_origin(origin, purpose);
        self.in_use = true;
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
            "memory leak: {:?}, origin: {}, in use: {:?}, purpose: {}",
            self.ptr, self.origin, self.in_use, self.purpose
        );
    }
}
