use crate::fs::Dir;
use efs::file::{File, Type, TypeWithFile};
use efs::fs::ext2::error::Ext2Error;

#[derive(Copy, Clone)]
pub enum Whence {
    Set = 0,
    Head = 1,
    End = 2,
}

#[derive(Copy, Clone)]
#[repr(C)]
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
    dev: u32,      // File system's disk device
    ino: usize,    // Inode number
    r#type: CType, // Type of file
    nlink: u32,    // Number of links to file
    size: u64,     // Size of file in bytes
}

pub trait AsFile<FSE: core::error::Error> {
    fn as_file(&self) -> &dyn File<FsError = FSE>;
}

impl AsFile<Ext2Error> for TypeWithFile<Dir> {
    fn as_file(&self) -> &dyn File<FsError = Ext2Error> {
        match self {
            Self::Regular(file) => file,
            Self::Directory(dir) => dir,
            Self::SymbolicLink(link) => link,
            Self::Fifo(fifo) => fifo,
            Self::CharacterDevice(chardev) => chardev,
            Self::BlockDevice(blkdev) => blkdev,
            Self::Socket(socket) => socket,
        }
    }
}

pub trait FileExt: File<FsError = Ext2Error> {
    fn cstat(&self) -> CStat {
        let ty = match self.get_type() {
            Type::Regular => CType::Regular,
            Type::Directory => CType::Directory,
            Type::SymbolicLink => CType::SymbolicLink,
            Type::Fifo => CType::Fifo,
            Type::CharacterDevice => CType::CharacterDevice,
            Type::BlockDevice => CType::BlockDevice,
            Type::Socket => CType::Socket,
            _ => unimplemented!(),
        };
        let stat = self.stat();
        CStat {
            dev: stat.dev.0,
            ino: stat.ino.0,
            r#type: ty,
            nlink: stat.nlink.0,
            size: stat.size.0.try_into().unwrap(),
        }
    }
}

impl<T: ?Sized + File<FsError = Ext2Error>> FileExt for T {}
