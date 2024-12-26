use crate::errno::ErrNo;
use crate::kalloc::pages::{KMEMError, KMEM};
use crate::memlayout::{KSTACK, PGSIZE};
use crate::vm::kernel_map::make_kernel_map;
use crate::vm::mode::Mode;
use crate::vm::pagetable::Pagetable;
use crate::vm::pt_inner::IPagetable;
use crate::vm::pte::PtEntry;
use crate::vm::PTError;
use spin::rwlock::RwLock;

static KERNEL_PAGETABLE: RwLock<Option<Pagetable>> = RwLock::new(None);

#[allow(clippy::large_stack_frames, reason = "it is fine")]
#[no_mangle]
extern "C" fn map_stack(pos: usize) -> ErrNo {
    let va = KSTACK(pos);
    for i in 0..2 {
        let page = KMEM.lock().alloc("proc stack");
        if let Err(err) = page {
            return match err {
                KMEMError::AllocFail(_err) => ErrNo::ENOMEM,
            };
        }
        let mres = KERNEL_PAGETABLE
            .write()
            .as_mut()
            .expect("map_page on None")
            .map_page(
                va + PGSIZE * i,
                page.expect("something went wrong")
                    .into_box()
                    .leak()
                    .as_ptr() as usize,
                Mode::_RW_,
            );
        if let Err(err) = mres {
            return match err {
                PTError::AllocFail(_err) => ErrNo::ENOMEM,
                _ => panic!("ffi::proc_map_one_stack: {:?}", err),
            };
        };
    }
    ErrNo::SUCCESS
}

#[allow(clippy::large_stack_frames, reason = "it is fine")]
#[no_mangle]
extern "C" fn unmap_stack(pos: usize) {
    let va = KSTACK(pos);
    for i in 0..2 {
        // Safety: address is allocated so it's safe
        unsafe {
            KERNEL_PAGETABLE
                .write()
                .as_mut()
                .expect("unmap_page on None")
                .unmap_page_and_free(va + i * PGSIZE)
                .expect("no errors are expected");
        }
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
