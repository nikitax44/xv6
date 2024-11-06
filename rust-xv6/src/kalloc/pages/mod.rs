mod ffi;
mod kmem;
pub mod page;

pub use kmem::{KMem, KMEM};

#[derive(Debug)]
pub enum KMEMError {
    NoFreePages(usize),
    AllocFail(core::alloc::AllocError),
}
