use crate::kalloc::pages::page::Page;
use crate::kalloc::pages::KMEM;
use crate::memlayout::PGSIZE;
use crate::util::lazy_cell::LazyCell;
use crate::util::spinlock::Spinlock;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

pub mod pages;
pub mod thin_box;

const LEVELS: usize = (128 * 1024 * 1024 / PGSIZE).ilog2() as usize;
type Heap = buddy_system_allocator::Heap<LEVELS>;
pub struct SharedHeap<F: FnOnce() -> Heap + Send>(Spinlock<LazyCell<Heap, F>>);

// would be unsound if was public
fn initial_heap() -> Heap {
    static INITIAL: Page = Page::initial();
    let mut heap = Heap::new();
    let addr = core::ptr::addr_of!(INITIAL) as usize;
    // SAFETY: we own the memory in this range
    unsafe {
        heap.init(addr, addr + PGSIZE);
    }
    heap
}

#[global_allocator]
pub static KALLOC: SharedHeap<fn() -> buddy_system_allocator::Heap<LEVELS>> =
    SharedHeap::new(initial_heap);

impl<F: FnOnce() -> Heap + Send> SharedHeap<F> {
    pub const fn new(init: F) -> Self {
        Self(Spinlock::new(LazyCell::new(init)))
    }

    fn in_context<T>(&self, op: impl FnOnce(&mut Heap) -> T) -> T {
        let mut lock = self.0.lock();
        op(lock.get_mut())
    }
}

/// # SAFETY:
/// we give the valid pointers
unsafe impl<F: FnOnce() -> Heap + Send> GlobalAlloc for SharedHeap<F> {
    /// # Safety
    /// safe
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.in_context(|heap| {
            if let Ok(ptr) = heap.alloc(layout) {
                return ptr.as_ptr();
            };
            let page = KMEM.lock().alloc("Buddy allocator").expect("KMEMError");
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
