use crate::errno::ErrNo;
use crate::fs::path::{Path, PathSegment};
use crate::hw::disk::Disk;
use alloc::boxed::Box;
use thiserror::Error;

#[derive(Copy, Clone)]
pub enum Whence {
    Set = 0,
    Head = 1,
    End = 2,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

impl SeekFrom {
    pub fn apply(self, pos: u64, len: u64) -> Option<u64> {
        match self {
            Self::Start(off) => Some(off),
            Self::End(diff) => {
                let buf = len.checked_add_signed(diff)?;
                (buf <= len).then_some(buf)
            }
            Self::Current(diff) => pos.checked_add_signed(diff),
        }
    }
}

impl From<(Whence, u64)> for SeekFrom {
    fn from((whence, offset): (Whence, u64)) -> Self {
        match whence {
            Whence::Set => Self::Start(offset),
            Whence::Head => Self::Current(offset.cast_signed()),
            Whence::End => Self::End(offset.cast_signed()),
        }
    }
}

impl From<SeekFrom> for (Whence, u64) {
    fn from(value: SeekFrom) -> Self {
        match value {
            SeekFrom::Start(offset) => (Whence::Set, offset),
            SeekFrom::Current(offset) => (Whence::Head, offset.cast_unsigned()),
            SeekFrom::End(offset) => (Whence::End, offset.cast_unsigned()),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
#[allow(dead_code)]
pub enum CType {
    Regular = 1,
    Directory,
    SymbolicLink,
    Fifo,
    CharacterDevice,
    BlockDevice,
    Socket,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CStat {
    pub(crate) dev: u32,      // File system's disk device
    pub(crate) ino: usize,    // Inode number
    pub(crate) r#type: CType, // Type of file
    pub(crate) nlink: u32,    // Number of links to file
    pub(crate) size: u64,     // Size of file in bytes
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("file impl error: {0:?}")]
    ImplError(#[from] Box<dyn core::error::Error>),
    #[error("seek out of bounds")]
    InvalidSeek,
    #[error("not implemented")]
    NotImplemented,
    #[error("Unexpected EOF in file")]
    UnexpectedEof,
}

impl From<FileError> for ErrNo {
    fn from(value: FileError) -> Self {
        match value {
            FileError::ImplError(_) => Self::EIO,
            FileError::InvalidSeek => Self::EINVAL,
            FileError::NotImplemented => Self::EOPNOTSUPP,
            FileError::UnexpectedEof => Self::ENODATA,
        }
    }
}

pub trait File: 'static + Send + Sync {
    fn truncate(&mut self, pos: SeekFrom) -> Result<(), FileError>;
    fn size(&self) -> u64;
    fn cstat(&self) -> CStat;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError>;
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileError>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), FileError> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => return Err(FileError::UnexpectedEof),
                Ok(sz) => buf = &mut buf[sz..],
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    fn write_exact(&mut self, mut buf: &[u8]) -> Result<(), FileError> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(FileError::UnexpectedEof),
                Ok(sz) => buf = &buf[sz..],
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
}

pub trait Dir: 'static + Sized + Send + Sync + Clone {
    type File: File + Sized;
    type Error: core::error::Error + Into<ErrNo>;
    fn open_file(&mut self, name: PathSegment) -> Result<Self::File, Self::Error>;
    fn open_dir(&self, name: PathSegment) -> Result<Self, Self::Error>;

    fn create_file(&mut self, name: PathSegment) -> Result<(), Self::Error>;
    fn create_dir(&mut self, name: PathSegment) -> Result<(), Self::Error>;

    fn unlink(&mut self, name: PathSegment) -> Result<(), Self::Error>;
    fn rmdir(&mut self, name: PathSegment) -> Result<(), Self::Error>;

    fn traverse(&self, path: &Path) -> Result<Self, Self::Error> {
        let mut this = self.clone();
        for part in path.iter().copied() {
            this = this.open_dir(part)?;
        }
        Ok(this)
    }
}

impl<F: File + ?Sized> File for Box<F> {
    fn truncate(&mut self, pos: SeekFrom) -> Result<(), FileError> {
        (**self).truncate(pos)
    }

    fn size(&self) -> u64 {
        (**self).size()
    }

    fn cstat(&self) -> CStat {
        (**self).cstat()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError> {
        (**self).read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        (**self).write(buf)
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileError> {
        (**self).seek(pos)
    }
}

pub trait Fs: 'static + Sized + Send + Sync {
    type Dir: Dir;
    type Error: core::error::Error + Into<ErrNo>;

    fn from_disk(disk: Disk) -> Result<Self, Self::Error>;

    fn get_root(&'static self) -> Result<Self::Dir, Self::Error>;
}

pub type Result<T, E = ErrNo> = core::result::Result<T, E>;
