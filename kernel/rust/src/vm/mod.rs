use crate::kalloc::pages::KMEMError;

mod ffi;
mod kernel_map;
pub mod mode;
pub mod pagetable;
mod pt_inner;
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
}
