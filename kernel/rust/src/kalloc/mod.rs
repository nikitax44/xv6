use crate::kalloc::pages::KMEM;
use crate::memlayout::PGSIZE;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use spin::Mutex;

pub mod pages;
pub mod thin_box;

const LEVELS: usize = (128 * 1024 * 1024 / PGSIZE).ilog2() as usize;
type Heap = buddy_system_allocator::Heap<LEVELS>;
pub struct SharedHeap(Mutex<Heap>);

#[global_allocator]
pub static KALLOC: SharedHeap = SharedHeap::new();

impl SharedHeap {
    #[must_use]
    #[allow(clippy::new_without_default, reason = "temporary solution??")]
    pub const fn new() -> Self {
        Self(Mutex::new(Heap::new()))
    }

    fn in_context<T>(&self, op: impl FnOnce(&mut Heap) -> T) -> T {
        let mut lock = self.0.lock();
        op(&mut lock)
    }
}

/// # SAFETY:
/// we give the valid pointers
unsafe impl GlobalAlloc for SharedHeap {
    /// # Safety
    /// safe
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.in_context(|heap| {
            if let Ok(ptr) = heap.alloc(layout) {
                return ptr.as_ptr();
            };
            let Ok(page) = KMEM.lock().alloc("Buddy allocator") else {
                return core::ptr::null_mut();
            };
            let page = page.into_box().leak().as_ptr() as usize;
            // SAFETY: we have the ownership
            unsafe { heap.add_to_heap(page, page + PGSIZE) };
            heap.alloc(layout)
                .expect("failed to allocate object larger than PGSIZE. TODO: use kvmmap")
                .as_ptr()
        })
    }

    /// # Safety
    /// `ptr` must be allocated with `alloc` previously
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).expect("dealloc(null)");
        // heap.dealloc is actually unsound and thus should be unsafe
        self.in_context(|heap| heap.dealloc(ptr, layout));
    }
}
