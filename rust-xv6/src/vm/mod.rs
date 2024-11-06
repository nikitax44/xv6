use crate::kalloc::pages::KMEMError;

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

/// # Errors
/// None. assuming active pagetable is the kernel one which is currently always the case.
#[allow(
    clippy::missing_const_for_fn,
    reason = "the result is meaningless in const context"
)]
pub fn get_physical_address(virtual_address: *const ()) -> Result<usize, !> {
    Ok(virtual_address as usize)
}
