mod ffi;
mod kmem;
pub mod page;

pub use ffi::kfree;
pub use kmem::{KMem, KMEM};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KMEMError {
    #[error("failed to allocate")]
    AllocFail(#[from] core::alloc::AllocError),
}
