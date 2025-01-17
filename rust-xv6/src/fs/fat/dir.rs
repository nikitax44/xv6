use crate::errno::ErrNo;
use crate::fs::fat::{FatDir, FatFile};
use crate::fs::path::PathSegment;
use crate::fs::types::{CStat, CType, Dir, File, FileError, SeekFrom};
use alloc::boxed::Box;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("dir error: {0:?}")]
pub struct DirError(fat32::dir::DirError);

impl From<fat32::dir::DirError> for DirError {
    fn from(value: fat32::dir::DirError) -> Self {
        Self(value)
    }
}

impl From<DirError> for ErrNo {
    fn from(value: DirError) -> Self {
        use fat32::dir::DirError;
        match value.0 {
            DirError::NoMatchDir | DirError::NoMatchFile => Self::ENOENT,
            DirError::IllegalChar => Self::EINVAL,
            DirError::DirHasExist | DirError::FileHasExist => Self::EEXIST,
        }
    }
}

impl Dir for FatDir {
    type File = Box<dyn File>;
    type Error = DirError;

    fn open_file(&mut self, name: PathSegment) -> Result<Self::File, Self::Error> {
        if name.0 == "." {
            return Ok(Box::new(DirFile::new(self)));
        }

        let file = FatDir::open_file(self, name.0)?;
        Ok(Box::new(FatFile::from_inner(&file)))
    }

    fn open_dir(&self, name: PathSegment) -> Result<Self, Self::Error> {
        Ok(FatDir::cd(self, name.0)?)
    }

    fn create_file(&mut self, name: PathSegment) -> Result<(), Self::Error> {
        Ok(FatDir::create_file(self, name.0)?)
    }

    fn create_dir(&mut self, name: PathSegment) -> Result<(), Self::Error> {
        Ok(FatDir::create_dir(self, name.0)?)
    }

    fn unlink(&mut self, name: PathSegment) -> Result<(), Self::Error> {
        Ok(FatDir::delete_file(self, name.0)?)
    }

    fn rmdir(&mut self, name: PathSegment) -> Result<(), Self::Error> {
        Ok(FatDir::delete_dir(self, name.0)?)
    }
}

pub struct DirFile {
    #[allow(dead_code)]
    dir: FatDir,
    offset: usize,
}

impl DirFile {
    const fn new(dir: &FatDir) -> Self {
        Self {
            dir: *dir,
            offset: 0,
        }
    }
}

impl File for DirFile {
    fn truncate(&mut self, _pos: SeekFrom) -> crate::fs::types::Result<(), FileError> {
        Err(FileError::NotImplemented)
    }

    fn size(&self) -> u64 {
        0
    }

    fn cstat(&self) -> CStat {
        CStat {
            dev: 1,
            ino: 0,
            r#type: CType::Directory,
            nlink: 0,
            size: 0,
        }
    }

    fn read(&mut self, _buf: &mut [u8]) -> crate::fs::types::Result<usize, FileError> {
        Ok(0)
    }

    fn write(&mut self, _buf: &[u8]) -> crate::fs::types::Result<usize, FileError> {
        Err(FileError::NotImplemented)
    }

    fn seek(&mut self, pos: SeekFrom) -> crate::fs::types::Result<u64, FileError> {
        if pos != SeekFrom::Start(0) {
            return Err(FileError::NotImplemented);
        }
        self.offset = 0;
        Ok(0)
    }
}
