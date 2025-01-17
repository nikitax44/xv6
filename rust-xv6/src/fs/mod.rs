mod fat;
mod file;
#[allow(clippy::module_inception)]
mod fs;
mod path;
pub(super) mod special;
pub(crate) mod types;

use crate::fs::fat::FatFS;
use crate::fs::types::{File, Fs};
use crate::hw::disk::MAIN_DISK;
use crate::util::lazy::Lazy;
use crate::util::mutex::Mutex;
use alloc::boxed::Box;
use log::info;

// must be Box<dyn File> because `Arc<CFile>` should be thin pointer
type CFile = Mutex<Box<dyn File>>;

#[must_use]
pub fn get_fs() -> &'static impl Fs {
    static MAIN_FS: Lazy<FatFS> = Lazy::new(|| {
        info!("MAIN_FS init on disk {:?}", MAIN_DISK.get_name());
        FatFS::from_disk(*MAIN_DISK).expect("failed to init disk")
    });
    &*MAIN_FS
}
