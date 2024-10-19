use crate::errno::ErrNo;
use crate::vm::kernel_map::make_kernel_map;
use crate::vm::pagetable::Pagetable;
use crate::vm::pte::PtEntry;
use crate::vm::PTError;

/// # Safety
/// no one owns memory outside of kernel and bios regions,
/// or it is declared in dtb's reserved regions
#[no_mangle]
unsafe extern "C" fn kvmmake() -> Pagetable<'static> {
    // SAFETY: precondition
    unsafe { make_kernel_map() }.expect("failed to create kernel map")
}

//int mappages(pagetable_t pagetable, u64 va, u64 size, u64 pa, int perm)
#[no_mangle]
extern "C" fn mappages(
    mut pt: Pagetable,
    virtual_address: usize,
    size: usize,
    physical_address: usize,
    perm: usize,
) -> ErrNo {
    let perm = perm.try_into().expect("invalid access mode");
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
extern "C" fn walkaddr(pagetable: Pagetable, virtual_address: usize) -> usize {
    pagetable
        .walk(virtual_address)
        .ok()
        .as_ref()
        .and_then(PtEntry::get)
        .filter(|(_addr, mode)| mode.get_u())
        .map_or(0, |(addr, _mode)| addr)
}
