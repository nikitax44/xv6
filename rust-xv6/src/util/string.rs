use core::ffi::CStr;
use core::fmt;
use core::fmt::{Display, Error, Formatter};
use core::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq)]
#[must_use]
pub struct StackString<const N: usize>([u8; N], usize);

impl<const N: usize> Default for StackString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> StackString<N> {
    pub const fn new() -> Self {
        Self([0; N], 0)
    }
}

impl<const N: usize> fmt::Write for StackString<{ N }> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let buf = &mut self.0[self.1..];
        let bytes = s.as_bytes();
        if bytes.len() > buf.len() {
            return Err(Error);
        }
        buf[..bytes.len()].copy_from_slice(bytes);
        self.1 += bytes.len();
        Ok(())
    }
}

impl<const N: usize> Display for StackString<{ N }> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes = &self.0[..self.1];
        let str = core::str::from_utf8(bytes).ok().ok_or(Error)?;
        f.write_str(str)
    }
}

#[macro_export]
macro_rules! sized_format {
    ($sz:literal, $($tts:tt)*) => {{
        use core::fmt::Write;
        let mut buf = $crate::util::string::StackString::<$sz>::new();
        write!(buf, $($tts)*).ok();
        buf
    }};
}

pub struct InlineCString<const N: usize> {
    buffer: [core::ffi::c_char; N],
    size_with_null: usize,
}

impl<const N: usize> Deref for InlineCString<N> {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        CStr::from_bytes_with_nul(&self.buffer[..self.size_with_null])
            .expect("InlineCString invariant was broken")
    }
}
