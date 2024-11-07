#[cfg(not(feature = "talc"))]
mod buddy_alloc;
#[cfg(not(feature = "talc"))]
pub use buddy_alloc::{add_region, get_info};
#[cfg(feature = "talc")]
mod talc;
#[cfg(feature = "talc")]
pub use talc::{add_region, get_info};

pub mod pages;
pub mod region;
pub mod thin_box;

use crate::kalloc::pages::page::Page;
use crate::kalloc::region::Region;
use core::mem::MaybeUninit;
use core::ptr;
use spin::Once;

pub struct MemoryInfo {
    total_bytes: usize,
    free_bytes: usize,
}

pub(crate) fn init() {
    static INIT: [MaybeUninit<Page>; 32] = MaybeUninit::uninit_array();
    static STATE: Once = Once::new();
    let start = ptr::addr_of!(INIT) as usize;
    // SAFETY: this is called only once
    STATE.call_once(|| unsafe { add_region(Region::new(start, start + size_of_val(&INIT))) });
}
