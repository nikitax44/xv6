use crate::kalloc::pages::KMEMError;
use crate::memlayout::PGROUNDDOWN;
use crate::util::rw_lock::RwLock;
use crate::vm::pagetable::Pagetable;
use core::fmt::{Debug, Formatter};
use thiserror::Error;

pub static KERNEL_PAGETABLE: RwLock<Option<Pagetable>> = RwLock::new(None);

mod ffi;
mod kernel_map;
pub mod mode;
pub mod pagetable;
pub(crate) mod pt_inner;
pub mod pte;

#[derive(Error, PartialEq)]
#[non_exhaustive]
pub enum PTError {
    #[error("this va is already mapped")]
    Remap,
    #[error("failed to allocate memory")]
    AllocFail(#[from] KMEMError),
    #[error("this va is not mapped")]
    NotMapped,
    #[error("invalid va: {0:#x}")]
    InvalidVirtualAddress(usize),
    #[error("the va {0:#x} does not point to the start of the Page")]
    UnalignedVirtualAddress(usize),
    #[error("the pa {0:#x} does not point to the start of the Page")]
    UnalignedPhysicalAddress(usize),
    #[error("the mode was `Mode::Table` when it wasn't expected")]
    UnexpectedTable,
    #[error("the size {0:#x} is not evenly divisible by PGSIZE")]
    UnalignedSize(usize),
    #[error("`Pagetable` is borrowed as readonly")]
    ROPagetable,
}

impl Debug for PTError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

/// # Errors
/// None. assuming active pagetable is the kernel one which is currently always the case.
#[allow(
    clippy::missing_const_for_fn,
    reason = "the result is meaningless in const context"
)]
pub fn get_physical_address(virtual_address: *const ()) -> Result<usize, PTError> {
    let guard = KERNEL_PAGETABLE.read();
    let pt = guard.as_ref().ok_or(PTError::Remap)?;
    let base = PGROUNDDOWN(virtual_address as usize);
    let pte = pt.walk(base)?;
    pte.addr()
        .map(|addr| addr + (virtual_address as usize - base))
}
