use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::PGSIZE;
use core::mem::ManuallyDrop;
use core::panic::Location;

type Origin = &'static Location<'static>;

#[repr(C, align(4096))]
#[must_use]
#[derive(Copy, Clone)]
pub struct Page(pub [u8; PGSIZE]);

#[repr(C)]
union PageWrap {
    page: Page,
    handle: ManuallyDrop<Option<PageHandle>>,
}

impl Page {
    pub const fn initial() -> Self {
        Self([0x93; PGSIZE])
    }
}

#[must_use]
#[derive(Debug)]
pub struct PageHandle {
    in_use: bool,
    ptr: Option<ThinBox<PageWrap>>,
    origin: Origin,
    purpose: &'static str,
}

impl PageHandle {
    #[track_caller]
    pub fn new(ptr: &'static mut Page, origin: Option<Origin>) -> Self {
        Self {
            // SAFETY: transmuting Page to PageWrap is always sound
            ptr: Some(unsafe { ThinBox::from(ptr).cast() }),
            origin: origin.unwrap_or_else(Location::caller),
            purpose: "unknown",
            // indicates variant of ptr
            in_use: true,
        }
    }

    pub fn set_origin(&mut self, origin: Origin, purpose: &'static str) {
        self.origin = origin;
        self.purpose = purpose;
    }

    /// # Panics
    /// if self is not in use
    fn fill(&mut self, byte: u8) {
        assert!(self.in_use, "attempt to fill `!in_use` `PageWrap`");
        // SAFETY: tag is valid, any bit pattern is valid for Page
        unsafe { &mut self.ptr.as_mut().unwrap().page }.0.fill(byte);
    }

    /// # Panics
    /// if self is not in use
    pub fn mark_freed(&mut self, next: Option<Self>) {
        assert!(self.in_use, "attempt to free not allocated page");
        self.fill(0x19);
        self.in_use = false;
        self.ptr.as_mut().unwrap().handle = ManuallyDrop::new(next);
    }

    /// # Panics
    /// if self is already in use
    pub fn mark_allocated(&mut self, origin: Origin, purpose: &'static str) -> Option<Self> {
        assert!(!self.in_use, "attempt to get next_handle of in-use page");
        // SAFETY: we but the value beforehand as stated by !self.in_use
        let next = unsafe { &mut self.ptr.as_mut().unwrap().handle }.take();
        self.in_use = true;
        self.fill(0x1d);
        self.set_origin(origin, purpose);
        next
    }

    /// # Panics
    /// if self is not in use
    pub fn into_box(self) -> ThinBox<Page> {
        let mut this = ManuallyDrop::new(self);
        assert!(this.in_use, "attempt to leak the !in_use page");
        // SAFETY: cast from PageWrap to Page is always sound
        unsafe { this.ptr.take().unwrap().cast() }
    }

    pub fn zeroed(mut self) -> Self {
        self.fill(0);
        self
    }
}

impl Drop for PageHandle {
    fn drop(&mut self) {
        let _ = ManuallyDrop::new(self.ptr.take());
        panic!("memory leak: {:?}", core::hint::black_box(self));
    }
}
