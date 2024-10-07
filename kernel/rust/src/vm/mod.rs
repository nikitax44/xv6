mod kernel_map;
pub mod mode;
pub mod pagetable;
mod pt_inner;
pub mod pte;

#[derive(Debug)]
pub enum PTError {
    Remap,
    AllocFail,
    NotMapped,
    InvalidVirtualAddress,
    InvalidPhysicalAddress,
    InvalidMode,
    InvalidSize,
}
