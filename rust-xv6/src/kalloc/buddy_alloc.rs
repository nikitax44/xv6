use crate::kalloc::region::Region;
use crate::kalloc::Xv6Alloc;
use core::fmt::Debug;

const LEVELS: usize = (128 * 1024 * 1024 / 8u64).ilog2() as usize;
type Heap = buddy_system_allocator::LockedHeap<LEVELS>;

#[global_allocator]
pub(super) static KALLOC: Heap = Heap::new();

#[derive(Debug)]
pub struct BuddyHeapInfo {
    pub user: usize,
    pub allocated: usize,
    pub total_bytes: usize,
    pub free_bytes: usize,
}

impl From<BuddyHeapInfo> for super::MemoryInfo {
    fn from(value: BuddyHeapInfo) -> Self {
        Self {
            total_bytes: value.total_bytes,
            free_bytes: value.free_bytes,
        }
    }
}

impl Xv6Alloc for Heap {
    unsafe fn add_region(&self, region: Region) {
        // SAFETY: we have the ownership by precondition
        unsafe { self.lock().add_to_heap(region.start(), region.end()) };
    }

    fn get_info(&self) -> BuddyHeapInfo {
        let guard = self.lock();
        BuddyHeapInfo {
            user: guard.stats_alloc_user(),
            allocated: guard.stats_alloc_actual(),
            total_bytes: guard.stats_total_bytes(),
            free_bytes: guard.stats_total_bytes() - guard.stats_alloc_actual(),
        }
    }

    fn try_get_info(&self) -> Option<BuddyHeapInfo> {
        let guard = self.try_lock()?;
        Some(BuddyHeapInfo {
            user: guard.stats_alloc_user(),
            allocated: guard.stats_alloc_actual(),
            total_bytes: guard.stats_total_bytes(),
            free_bytes: guard.stats_total_bytes() - guard.stats_alloc_actual(),
        })
    }
}
