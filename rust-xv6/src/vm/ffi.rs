use crate::errno::ErrNo;
use crate::kalloc::pages::KMEM;
use crate::memlayout::{KSTACK, PGSIZE, STACK_SIZE};
use crate::vm::kernel_map::make_kernel_map;
use crate::vm::mode::Mode;
use crate::vm::pagetable::Pagetable;
use crate::vm::pt_inner::IPagetable;
use crate::vm::pte::PtEntry;
use crate::vm::PTError;
use spin::rwlock::RwLock;

static KERNEL_PAGETABLE: RwLock<Option<Pagetable>> = RwLock::new(None);

#[allow(clippy::large_stack_frames, reason = "it is fine")]
#[attr_wrapper::time_me(0)]
#[no_mangle]
extern "C" fn map_stack(pos: usize) -> ErrNo {
    // SAFETY: precondition
    match unsafe { KernelStack(pos).map() } {
        Ok(()) => ErrNo::SUCCESS,
        Err(PTError::AllocFail(_)) => ErrNo::ENOMEM,
        Err(PTError::Remap) => panic!("page remap"),
        Err(err) => panic!("map_stack: {err}"),
    }
}

#[allow(clippy::large_stack_frames, reason = "it is fine")]
#[attr_wrapper::time_me(0)]
#[no_mangle]
extern "C" fn unmap_stack(pos: usize) {
    // SAFETY: precondition
    unsafe { KernelStack(pos).unmap() }.expect("failed to unmap page");
}

#[derive(Copy, Clone)]
struct KernelStack(pub usize);

impl KernelStack {
    fn bottom(self) -> usize {
        KSTACK(self.0)
    }
    fn pages(self) -> impl Iterator<Item = usize> {
        let bot = self.bottom();
        (0..STACK_SIZE).map(move |i| bot + i * PGSIZE)
    }

    unsafe fn map(self) -> Result<(), PTError> {
        for ptr in self.pages() {
            let page = KMEM.lock().alloc("proc stack")?;
            KERNEL_PAGETABLE
                .write()
                .as_mut()
                .expect("map_page on None")
                .map_page(ptr, page.into_box().leak().as_ptr() as usize, Mode::_RW_)?;
        }
        Ok(())
    }

    unsafe fn unmap(self) -> Result<(), PTError> {
        for ptr in self.pages() {
            // SAFETY: precondition
            unsafe {
                KERNEL_PAGETABLE
                    .write()
                    .as_mut()
                    .expect("unmap_page on None")
                    .unmap_page_and_free(ptr)?;
            }
        }
        Ok(())
    }
}

/// # Safety
/// no one owns memory outside of kernel and bios regions,
/// or it is declared in dtb's reserved regions
#[no_mangle]
unsafe extern "C" fn kvminit() {
    // SAFETY: precondition
    let pt = unsafe { make_kernel_map() }.expect("failed to create kernel map");

    let mut kpt = KERNEL_PAGETABLE.write();
    if let Some(_old_pt) = kpt.replace(pt) {
        panic!("KERNEL_PAGETABLE overwrite")
    }
}

#[no_mangle]
extern "C" fn with_kernel_pagetable(op: extern "C" fn(&mut IPagetable)) {
    let mut kpt = KERNEL_PAGETABLE.write();
    let kpt = kpt
        .as_mut()
        .expect("kernel pagetable was expected to be present");
    op(kpt
        .inner_mut()
        .expect("kernel pagetable is the Pagetable::Ref variant"));
}

//int mappages(pagetable_t pagetable, u64 va, u64 size, u64 pa, int perm)
#[no_mangle]
extern "C" fn mappages(
    pt: &mut IPagetable,
    virtual_address: usize,
    size: usize,
    physical_address: usize,
    perm: usize,
) -> ErrNo {
    /*warn!(
        "rwarn {:#x}, {:#x}",
        pt as *const _ as usize, virtual_address
    );*/
    let perm = perm.try_into().expect("invalid access mode");
    let mut pt = Pagetable::from_mut(pt);
    let result = pt.map_pages(virtual_address, physical_address, size, perm);
    if let Err(err) = result {
        return match err {
            PTError::AllocFail(_err) => ErrNo::ENOMEM,
            _ => panic!("ffi::mappages: {:?}", err),
        };
    }
    ErrNo::SUCCESS
}

// Look up a virtual address, return the physical address,
// or 0 if not mapped.
// Can only be used to look up user pages.
// u64 walkaddr(pagetable_t pagetable, u64 va)
#[no_mangle]
extern "C" fn walkaddr(pagetable: &IPagetable, virtual_address: usize) -> usize {
    Pagetable::from_ref(pagetable)
        .walk(virtual_address)
        .ok()
        .as_ref()
        .and_then(PtEntry::get)
        .filter(|(_addr, mode)| mode.get_u())
        .map_or(0, |(addr, _mode)| addr)
}
