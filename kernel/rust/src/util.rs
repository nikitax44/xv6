use alloc::vec::Vec;
use core::fmt::{self, Write as _};
use core::ops::Deref;

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

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct String(Vec<u8>);

impl fmt::Write for String {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl String {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }
}

impl Deref for String {
    type Target = str;
    fn deref(&self) -> &str {
        core::str::from_utf8(&self.0).expect("String invariant was somehow broken")
    }
}
