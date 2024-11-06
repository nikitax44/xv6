use crate::kalloc::pages::page::Page;
use crate::kalloc::region::Region;
use crate::memlayout::PGSIZE;
use crate::println;
use core::alloc::{GlobalAlloc, Layout};
use core::mem::MaybeUninit;
use core::ptr;
use core::ptr::NonNull;
use spin::{Mutex, Once};

pub mod pages;
pub mod region;
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

    pub fn in_context<T>(&self, op: impl FnOnce(&mut Heap) -> T) -> T {
        let mut lock = self.0.lock();
        op(&mut lock)
    }
}

/// # Safety
/// caller transfers the ownership over the memory in that region
pub unsafe fn add_region(heap: &mut Heap, region: Region) {
    // SAFETY: we have the ownership by precondition
    unsafe { heap.add_to_heap(region.start(), region.end()) };
}

/// # SAFETY:
/// we give the valid pointers
unsafe impl GlobalAlloc for SharedHeap {
    /// # Safety
    /// no panics can happen
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        static BASE: [MaybeUninit<Page>; 32] = MaybeUninit::uninit_array();
        static INIT: Once = Once::new();

        self.in_context(|heap| {
            INIT.call_once(|| {
                let start = ptr::from_ref(&BASE) as usize;
                // SAFETY: we own this memory and give it only once
                unsafe {
                    heap.add_to_heap(start, start + size_of_val(&BASE));
                }
            });
            heap.alloc(layout)
                .map(NonNull::as_ptr)
                .ok()
                .unwrap_or_else(ptr::null_mut)
        })
    }

    /// # Safety
    /// `ptr` must be allocated with `alloc` previously
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let Some(ptr) = NonNull::new(ptr) else {
            println!("dealloc(null)");
            return;
        };
        // heap.dealloc is actually unsound and thus should be unsafe
        self.in_context(|heap| heap.dealloc(ptr, layout));
    }
}
