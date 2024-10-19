use crate::asm::get_current_pt;
use crate::kalloc::pages::KMEMError;
use crate::vm::pagetable::Pagetable;

mod ffi;
mod kernel_map;
pub mod mode;
pub mod pagetable;
pub(crate) mod pt_inner;
pub mod pte;

#[derive(Debug)]
pub enum PTError {
    Remap,
    AllocFail(KMEMError),
    NotMapped,
    InvalidVirtualAddress,
    InvalidPhysicalAddress,
    InvalidMode,
    InvalidSize,
    NotPage,
}

/// # Panics
/// satp must be in SV39 mode
pub fn with_current_pt<R>(op: impl FnOnce(&Pagetable) -> R) -> R {
    riscv::interrupt::free(|| {
        let mut pt = get_current_pt().unwrap();
        // SAFETY: pt is valid for duration of interrupt::free
        // it is read-only for all threads since it's creation
        let mref = unsafe { pt.as_mut() };

        op(&Pagetable::new(mref))
    })
}

/// # Errors
/// see `Pagetable::translate`
pub fn get_physical_address(virtual_address: usize) -> Result<usize, PTError> {
    with_current_pt(|pt| pt.translate(virtual_address))
}
