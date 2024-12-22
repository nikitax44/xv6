mod file;
#[allow(clippy::module_inception)]
mod fs;
pub(crate) mod types;

use crate::hw::disk::{Disk, MAIN_DISK};
use efs::file::DirectoryEntry;
use efs::fs::ext2::file::Directory;
use efs::fs::ext2::Ext2Fs;
use log::info;
use spin::{Lazy, Mutex, RwLock};

// type Fs = Ext2Fs<Disk>;
type Dir = Directory<Disk>;
type DirEntry = DirectoryEntry<'static, Dir>; // owned
type CFile = Mutex<DirEntry>;
pub(crate) type Error = efs::error::Error<efs::fs::ext2::error::Ext2Error>;

// TODO: support multiple disks
pub static MAIN_FS: Lazy<RwLock<Ext2Fs<Disk>>> = Lazy::new(|| {
    info!("MAIN_FS init on disk {:?}", MAIN_DISK.lock().get_name());
    RwLock::new(Ext2Fs::new_celled(MAIN_DISK.clone(), 1, true).expect("failed to init ext2 fs"))
});
