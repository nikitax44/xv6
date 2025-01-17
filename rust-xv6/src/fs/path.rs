use crate::errno::ErrNo;
use crate::ffi_interop::FromFFI;
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};
use core::fmt::{Display, Formatter};
use core::ops::Deref;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidPath {
    #[error("Invalid path: empty path")]
    EmptyPath,
    #[error("Invalid path: Unexpected `/`")]
    UnexpectedSlash,
    #[error("not utf8 path")]
    Utf8Error,
}
impl From<InvalidPath> for ErrNo {
    fn from(_value: InvalidPath) -> Self {
        Self::EINVAL
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PathSegment<'a>(pub &'a str);
#[derive(Debug, Clone)]
pub struct OwnedPath<'a>(Vec<PathSegment<'a>>);

pub type Path<'a> = [PathSegment<'a>];

impl<'a> PathSegment<'a> {
    pub fn new(path: &'a str) -> Result<Self, InvalidPath> {
        if path.contains('/') {
            Err(InvalidPath::UnexpectedSlash)?;
        }
        if path.is_empty() {
            Err(InvalidPath::EmptyPath)?;
        }
        Ok(Self(path))
    }
}
impl<'a> OwnedPath<'a> {
    pub fn new(path: &'a str) -> Result<Self, InvalidPath> {
        if path.is_empty() {
            Err(InvalidPath::EmptyPath)?;
        }

        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(PathSegment::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(path))
    }
}

impl Display for OwnedPath<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for segment in &self.0 {
            write!(f, "/{}", segment.0)?;
        }
        Ok(())
    }
}

pub trait PathExt {
    fn parent(&self) -> &Self;
    fn basename(&self) -> Result<PathSegment, InvalidPath>;
}
impl PathExt for Path<'_> {
    fn parent(&self) -> &Self {
        if self.is_empty() {
            &[]
        } else {
            &self[..(self.len() - 1)]
        }
    }

    fn basename(&self) -> Result<PathSegment, InvalidPath> {
        self.last().copied().ok_or(InvalidPath::EmptyPath)
    }
}

impl<'a> Deref for OwnedPath<'a> {
    type Target = Path<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromFFI for Result<OwnedPath<'_>, InvalidPath> {
    type Source = *const c_char;

    unsafe fn from_ffi(value: Self::Source) -> Self {
        // SAFETY: precondition
        let str = unsafe { CStr::from_ptr(value as *const c_char) };
        let str = str.to_str().ok().ok_or(InvalidPath::Utf8Error)?;
        OwnedPath::new(str)
    }
}
