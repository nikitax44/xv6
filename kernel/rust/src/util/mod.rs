pub mod once;
pub mod string;
use core::fmt::Write as _;
use string::String;

pub const PGSIZE: usize = 4096;
pub const PGSHIFT: usize = 12;

#[derive(Debug)]
pub struct NotPageAligned;

/// # Errors
/// if page is not aligned
pub fn is_page_aligned(val: usize) -> Result<(), NotPageAligned> {
    if val % PGSIZE == 0 {
        Ok(())
    } else {
        Err(NotPageAligned)
    }
}

/// # Panics
/// if page is not aligned
#[track_caller]
pub fn assert_page_aligned(val: usize) {
    let mut st = String::new();
    write!(
        st,
        "page alignment assertion failed: from {}",
        core::panic::Location::caller()
    )
    .ok();
    is_page_aligned(val).expect(&st);
}
