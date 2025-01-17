use crate::fs::types::{CStat, CType, File, FileError, SeekFrom};
use crate::hw::disk::virtio_blk::SectorID;
use crate::hw::disk::Disk;
use crate::util::copy_data;
use alloc::boxed::Box;
use fat32::file::WriteType;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("fat32 file error: {0:?}")]
pub struct FatError(fat32::file::FileError);

pub struct FatFile {
    inner: fat32::file::File<'static, Disk>,
    offset: u64,
    size: u64,
}

impl FatFile {
    pub(super) fn from_inner(inner: &fat32::file::File<'static, Disk>) -> Self {
        let size = inner.read_per_sector().map(|(_, sz)| sz as u64).sum();
        Self {
            inner: *inner,
            offset: 0,
            size,
        }
    }
}

impl File for FatFile {
    fn truncate(&mut self, pos: SeekFrom) -> Result<(), FileError> {
        match pos {
            SeekFrom::Start(0) => {
                self.offset = 0;
                self.size = 0;
                self.inner
                    .write(&[], WriteType::OverWritten)
                    .map_err(FatError)
                    .map_err(|err| FileError::ImplError(Box::new(err)))
            }
            _ => Err(FileError::NotImplemented),
        }
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn cstat(&self) -> CStat {
        CStat {
            dev: 1,
            ino: 0,
            r#type: CType::Regular,
            nlink: 1,
            size: self.size,
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError> {
        let mut reader = self.inner.read_per_sector();
        let (sector, off) = SectorID::parts_from_bytes(self.offset);
        match reader.nth(sector.in_sectors().try_into().unwrap()) {
            None => Ok(0),
            Some((ref sector, sz)) => {
                let sector = &sector[off..sz];
                let n = copy_data(buf, sector);
                self.offset += n as u64;
                Ok(n)
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        if self.offset != 0 {
            Err(FileError::NotImplemented)?;
        }
        self.inner
            .write(buf, WriteType::OverWritten)
            .map_err(FatError)
            .map_err(|err| FileError::ImplError(Box::new(err)))?;
        self.offset += buf.len() as u64;
        self.size = buf.len() as u64;
        Ok(buf.len())
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileError> {
        self.offset = pos
            .apply(self.offset, self.size)
            .ok_or(FileError::InvalidSeek)?;
        Ok(self.offset)
    }
}
