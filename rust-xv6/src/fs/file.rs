use crate::fs::types::{AsFile, CStat, FileExt, Whence};
use crate::fs::{CFile, DirEntry, Error, MAIN_FS};
use alloc::sync::Arc;
use core::ptr::NonNull;
use efs::file::{File, SymbolicLink, Type, TypeWithFile};
use efs::fs::error::FsError;
use efs::fs::FileSystem;
use efs::io::{Read, Seek, SeekFrom, Write};
use efs::path::{Path, UnixStr};
use log::{debug, error, warn};
use spin::Mutex;

crate::export_c_fn! {
    #[allow(clippy::large_stack_frames)]
    pub fn fileopen:rs_file_open(path: Option<Path<'_>>) -> Option<Arc<CFile>> {
        let path = path?;
        let fs = MAIN_FS.read();
        let root = fs
            .root()
            .map_err(|err| error!("failed to get the fs root: {err}"))
            .ok()?;

        let file = fs
            .get_file(&path, root, true)
            .map_err(|err| warn!("error while opening the file {path}: {err}"))
            .ok()?;

        let entry: DirEntry = DirEntry {
            filename: path.as_unix_str().to_owned(),
            file,
        };
        let arc = Arc::new(Mutex::new(entry));
        Some(arc)
    }

    fn fileclose:rs_file_close(file: Arc<CFile>) {
        drop(file);
    }

    fn filedup:rs_file_dup(file: ref Arc<CFile>) -> Arc<CFile> {
        Arc::clone(file)
    }

    fn filestat:rs_file_stat(file: ref Arc<CFile>) -> CStat {
        file.lock().file.as_file().cstat()
    }

    /// # Safety
    /// `file` must point to valid `DirEntry`
    /// `out` must be valid to reads and writes for `len` bytes
    /// # Panics
    /// `out` is `None`
    unsafe fn __fileread:rs_file_read(file: ref Arc<CFile>, out: Option<NonNull<u8>>, len: usize) -> usize {
        let mut slice = NonNull::slice_from_raw_parts(out.expect("fileread's buffer is null"), len);
        // SAFETY: precondition
        let buf = unsafe { slice.as_mut() };

        fileread(file, buf)
    }

    /// # Safety
    /// `file` must point to valid `DirEntry`
    /// `out` must be valid to reads for `len` bytes
    /// # Panics
    /// `out` is `None`
    unsafe fn __filewrite:rs_file_write(file: ref Arc<CFile>, out: Option<NonNull<u8>>, len: usize) -> usize {
        let slice = NonNull::slice_from_raw_parts(out.expect("filewrite's buffer is null"), len);
        // SAFETY: precondition
        let buf = unsafe { slice.as_ref() };

        filewrite(file, buf)
    }

    /// # Errors
    /// `file` type is not Type::Regular
    /// filesystem errors
    pub fn fileseek:rs_file_seek(file: ref Arc<CFile>, offset: i64, whence: Option<Whence>) -> Result<u64, Error> {
        let mut file = file.lock();

        let TypeWithFile::Regular(file) = &mut file.file else {
            error!("seek on {}({:?})", file.filename, file.file.as_file().get_type());
            Err(FsError::WrongFileType {
                expected: Type::Regular,
                given: file.file.as_file().get_type(),
            })?;
            unreachable!()
        };

        let seek = match whence.expect("invalid Whence variant") {
            Whence::Set => SeekFrom::Start(offset.cast_unsigned()),
            Whence::End => SeekFrom::End(offset),
            Whence::Head => SeekFrom::Current(offset),
        };

        file.seek(seek)
    }
}

pub fn fileread(file: &Arc<CFile>, buf: &mut [u8]) -> usize {
    #[allow(clippy::large_stack_frames)]
    let res = (|| {
        let mut file = file.lock();
        let DirEntry { file, filename } = &mut *file;
        let filename = &*filename;
        let inner = match file {
            TypeWithFile::Regular(file) => file,
            TypeWithFile::Directory(_dir) => {
                warn!("attempt to read dir {}", filename);
                todo!()
            }
            TypeWithFile::SymbolicLink(link) => {
                let path = link.get_pointed_file()?;
                let path = Path::new(UnixStr::new(path)?);
                let path = Path::new(filename.clone()).join(&path);

                debug!("read symlink: {} -> {}", filename, path);
                todo!()
            }
            TypeWithFile::Fifo(fifo) => {
                let stat = fifo.stat();
                debug!("read fifo {}: {:?}", filename, stat);
                todo!()
            }
            TypeWithFile::CharacterDevice(chardev) => {
                let stat = chardev.stat();
                debug!("read char dev {}: {:?}", filename, stat);
                todo!()
            }
            TypeWithFile::BlockDevice(blockdev) => {
                let stat = blockdev.stat();
                debug!("read block dev {}: {:?}", filename, stat);
                todo!()
            }
            TypeWithFile::Socket(socket) => {
                let stat = socket.stat();
                debug!("read socket {}: {:?}", filename, stat);
                todo!()
            }
        };
        let size = inner.read(buf)?;
        // let pos = inner.seek(SeekFrom::Current(0))? as usize;
        // trace!(
        //     "read from {} at pos {}..{}: {:02x?}",
        //     filename,
        //     pos,
        //     pos + size,
        //     &buf[..size]
        // );
        Ok::<_, Error>(size)
    })();
    res.unwrap_or_else(|err| {
        error!("fileread error: {:?}", err);
        0
    })
}

pub fn filewrite(file: &Arc<CFile>, buf: &[u8]) -> usize {
    #[allow(clippy::large_stack_frames)]
    let res = (|| {
        let mut file = file.lock();
        let DirEntry { file, filename } = &mut *file;
        let filename = &*filename;
        let inner = match file {
            TypeWithFile::Regular(file) => file,
            TypeWithFile::Directory(_dir) => {
                warn!("attempt to write to dir {}", filename);
                return Err(Error::Fs(FsError::WrongFileType {
                    expected: Type::Regular,
                    given: Type::Directory,
                }));
            }
            TypeWithFile::SymbolicLink(link) => {
                let path = link.get_pointed_file()?;
                let path = Path::new(UnixStr::new(path)?);
                let path = Path::new(filename.clone()).join(&path);

                debug!("write symlink: {} -> {}", filename, path);
                todo!()
            }
            TypeWithFile::Fifo(fifo) => {
                let stat = fifo.stat();
                debug!("write fifo {}: {:?}", filename, stat);
                todo!()
            }
            TypeWithFile::CharacterDevice(chardev) => {
                let stat = chardev.stat();
                debug!("write char dev {}: {:?}", filename, stat);
                todo!()
            }
            TypeWithFile::BlockDevice(blockdev) => {
                let stat = blockdev.stat();
                debug!("write block dev {}: {:?}", filename, stat);
                todo!()
            }
            TypeWithFile::Socket(socket) => {
                let stat = socket.stat();
                debug!("write socket {}: {:?}", filename, stat);
                todo!()
            }
        };
        inner.write(buf)
    })();
    res.unwrap_or_else(|err| {
        error!("filewrite error: {:?}", err);
        0
    })
}
