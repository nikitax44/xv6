use alloc::vec::Vec;
use core::fmt;
use core::ops::Deref;

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
