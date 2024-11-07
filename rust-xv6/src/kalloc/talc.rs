use crate::kalloc::region::Region;
use crate::kalloc::MemoryInfo;
use talc::{ErrOnOom, Span, Talc};

type Heap = talc::Talck<spin::Mutex<()>, ErrOnOom>;

#[global_allocator]
static KALLOC: Heap = Talc::new(ErrOnOom).lock();

/// # Safety
/// caller transfers the ownership over the memory in that region
pub unsafe fn add_region(region: Region) {
    let span = Span::from_base_size(region.start() as *mut u8, region.size());

    // SAFETY: we now own the memory
    unsafe {
        KALLOC.lock().claim(span).ok();
    }
}

pub fn get_info() -> MemoryInfo {
    let kalloc = KALLOC.lock();
    MemoryInfo {
        total_bytes: kalloc
            .get_counters()
            .total_claimed_bytes
            .try_into()
            .unwrap_or(usize::MAX),
        free_bytes: kalloc.get_counters().available_bytes,
    }
}
