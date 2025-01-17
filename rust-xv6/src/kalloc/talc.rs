use crate::kalloc::region::Region;
use crate::kalloc::{MemoryInfo, Xv6Alloc};
use crate::util::mutex::Mutex;
use crate::util::u64_to_usize;
use critical_section::RestoreState;
use talc::{ErrOnOom, Span, Talc};

type Heap = talc::Talck<Mutex<RestoreState>, ErrOnOom>;

#[global_allocator]
pub(super) static KALLOC: Heap = Talc::new(ErrOnOom).lock();

impl Xv6Alloc for Heap {
    unsafe fn add_region(&self, region: Region) {
        let span = Span::from_base_size(region.start() as *mut u8, region.size());

        // SAFETY: we now own the memory
        unsafe {
            self.lock().claim(span).ok();
        }
    }

    fn get_info(&self) -> MemoryInfo {
        let counters = *self.lock().get_counters();
        MemoryInfo {
            total_bytes: u64_to_usize(counters.total_claimed_bytes),
            free_bytes: counters.available_bytes,
        }
    }

    fn try_get_info(&self) -> Option<MemoryInfo> {
        let counters = *self.try_lock()?.get_counters();
        Some(MemoryInfo {
            total_bytes: u64_to_usize(counters.total_claimed_bytes),
            free_bytes: counters.available_bytes,
        })
    }
}
