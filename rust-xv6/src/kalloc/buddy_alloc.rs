use crate::kalloc::region::Region;
use crate::kalloc::MemoryInfo;
use crate::memlayout::PGSIZE;

const LEVELS: usize = (128 * 1024 * 1024 / PGSIZE).ilog2() as usize;
type Heap = buddy_system_allocator::LockedHeap<LEVELS>;

#[global_allocator]
static KALLOC: Heap = Heap::new();

/// # Safety
/// caller transfers the ownership over the memory in that region
pub unsafe fn add_region(region: Region) {
    // SAFETY: we have the ownership by precondition
    unsafe { KALLOC.lock().add_to_heap(region.start(), region.end()) };
}

pub fn get_info() -> MemoryInfo {
    let kalloc = KALLOC.lock();
    MemoryInfo {
        total_bytes: kalloc.stats_total_bytes(),
        free_bytes: kalloc.stats_total_bytes() - kalloc.stats_alloc_actual(),
    }
}
