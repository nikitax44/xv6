#[cfg_attr(feature = "talc", path = "talc.rs")]
#[cfg_attr(not(feature = "talc"), path = "buddy_alloc.rs")]
mod alloc_impl;

pub mod region;
pub mod thin_box;

use crate::kalloc::thin_box::ThinBox;
use crate::memlayout::PGSIZE;
use core::alloc::Layout;
use core::fmt::Debug;
use core::mem::MaybeUninit;
use core::panic::Location;
use core::ptr;
use region::Region;
use spin::Once;

#[repr(C, align(4096))]
#[must_use]
pub struct Page(MaybeUninit<[u8; PGSIZE]>);

#[derive(Debug)]
pub struct MemoryInfo {
    pub total_bytes: usize,
    pub free_bytes: usize,
}

pub trait Xv6Alloc {
    /// # Safety
    /// transfers memory ownership to allocator
    unsafe fn add_region(&self, region: Region);

    fn get_info(&self) -> impl 'static + Debug + Into<MemoryInfo>;

    fn try_get_info(&self) -> Option<impl 'static + Debug + Into<MemoryInfo>>;
}

#[must_use]
pub fn get_kalloc() -> &'static impl Xv6Alloc {
    &alloc_impl::KALLOC
}

pub(crate) fn init() {
    static INIT: [MaybeUninit<Page>; 32] = MaybeUninit::uninit_array();
    static STATE: Once = Once::new();
    let start = ptr::addr_of!(INIT) as usize;
    let region = Region::new(start, start + size_of_val(&INIT));
    // SAFETY: this is called only once
    STATE.call_once(|| unsafe { get_kalloc().add_region(region) });
}

#[rustc_std_internal_symbol]
#[track_caller]
unsafe fn __rg_oom(size: usize, align: usize) -> ! {
    // SAFETY: precondition
    let layout = unsafe { Layout::from_size_align_unchecked(size, align) };

    let free_mem = get_kalloc().try_get_info();

    panic!(
        "Alloc error: failed to allocate memory {:#x?} in {}\nfree memory: {:#x?}",
        layout,
        Location::caller(),
        free_mem
    );
}

crate::export_c_fn! {
    fn __kinit:kinit() {
        init();
    }

    fn __kalloc:kalloc() -> Option<ThinBox<Page>> {
        ThinBox::alloc_page().ok()
    }

    fn __kfree:kfree(ptr: Option<ThinBox<Page>>) {
        let page = ptr.expect("kfree(NULL)");
        page.free();
    }

    fn __free_pages:free_pages() -> usize {
        let info = get_kalloc().get_info().into();
        info.free_bytes/PGSIZE
    }
}
