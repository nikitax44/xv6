use core::alloc::AllocError;
use core::fmt::{Debug, Formatter};
use thiserror::Error;

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
    Remap(usize),
    #[error("failed to allocate memory")]
    AllocFail(#[from] AllocError),
    #[error("this va is not mapped")]
    NotMapped,
    #[error("this PTE contains memory reference and not inner PT")]
    NotTable,
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
pub fn get_physical_address(virtual_address: *const ()) -> Result<usize, !> {
    Ok(virtual_address as usize)
}
