use crate::kalloc::pages::page::PageHandle;
use crate::lazy_cell::LazyCell;
use crate::memlayout::PGSIZE;
use crate::spinlock::Spinlock;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

pub mod pages;
pub mod simple;
pub mod thin_box;

const LEVELS: usize = (128 * 1024 * 1024 / PGSIZE).ilog2() as usize;
type Heap = buddy_system_allocator::Heap<LEVELS>;
pub static KALLOC: Spinlock<LazyCell<Heap>> = Spinlock::new(LazyCell::new(|| {
    static INITIAL_BUFFER: [u8; PGSIZE] = [0; PGSIZE];
    let mut heap = Heap::new();
    let ptr = INITIAL_BUFFER.as_ptr() as usize;
    unsafe { heap.init(ptr, size_of_val(&INITIAL_BUFFER)) };
    heap
}));

impl Spinlock<LazyCell<Heap>> {
    fn in_context<T>(&self, op: impl FnOnce(&mut Heap) -> T) -> T {
        let mut lock = self.lock();
        op(lock.get_mut())
    }

    pub fn extend(&self, page: PageHandle) {
        let page = page.leak().as_ptr() as usize;
        // SAFETY: we have the ownership
        self.in_context(|heap| unsafe { heap.add_to_heap(page, page + PGSIZE) });
    }
}

/// # SAFETY:
/// we give the valid pointers
unsafe impl GlobalAlloc for Spinlock<LazyCell<Heap>> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.in_context(|heap| heap.alloc(layout).expect("failed to alloc").as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).expect("dealloc(null)");
        // heap.dealloc is actually unsound and thus should be unsafe
        self.in_context(|heap| heap.dealloc(ptr, layout));
    }
}
