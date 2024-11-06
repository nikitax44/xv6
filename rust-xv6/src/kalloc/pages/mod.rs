mod ffi;
pub mod page;
mod rust_kmem;

pub use rust_kmem::{KMem, KMEM};

#[derive(Debug)]
pub enum KMEMError {
    NoFreePages(usize),
}
