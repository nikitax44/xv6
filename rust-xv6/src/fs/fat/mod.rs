mod dir;
mod file;

use crate::fs::types::Fs;
use crate::hw::disk::Disk;
use fat32::volume::Volume;
use thiserror::Error;

pub type FatFS = Volume<Disk>;
pub type FatDir = fat32::dir::Dir<'static, Disk>;
use crate::errno::ErrNo;
pub use file::FatFile;

#[derive(Error, Debug)]
#[allow(clippy::empty_enum)]
pub enum FatFsError {}
impl From<FatFsError> for ErrNo {
    fn from(value: FatFsError) -> Self {
        match value {}
    }
}

impl Fs for FatFS {
    type Dir = FatDir;
    type Error = FatFsError;

    fn from_disk(disk: Disk) -> Result<Self, Self::Error> {
        Ok(Self::new(disk))
    }

    fn get_root(&'static self) -> Result<Self::Dir, Self::Error> {
        Ok(self.root_dir())
    }
}
