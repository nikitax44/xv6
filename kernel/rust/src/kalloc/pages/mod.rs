#[cfg(not(feature = "rust_kalloc"))]
mod c_kmem;
#[cfg(feature = "rust_kalloc")]
mod ffi;
pub mod page;
#[cfg(feature = "rust_kalloc")]
mod rust_kmem;

#[cfg(not(feature = "rust_kalloc"))]
pub use c_kmem::{KMem, KMEM};
#[cfg(feature = "rust_kalloc")]
pub use rust_kmem::{KMem, KMEM};

#[derive(Debug)]
pub enum KMEMError {
    NoFreePages(usize),
}
