use super::types::Result;
use crate::errno::ErrNo;
use crate::fs::types::{CStat, Whence};
use crate::fs::CFile;
use alloc::sync::Arc;
use core::ptr::NonNull;

crate::export_c_fn! {
    fn fileclose:rs_file_close(file: Arc<CFile>) {
        drop(file);
    }

    fn filedup:rs_file_dup(file: ref Arc<CFile>) -> Arc<CFile> {
        Arc::clone(file)
    }

    fn filestat:rs_file_stat(file: ref Arc<CFile>) -> CStat {
        file.lock().cstat()
    }

    /// # Safety
    /// `file` must point to valid `DirEntry`
    /// `out` must be valid to reads and writes for `len` bytes
    /// # Panics
    /// `out` is `None`
    unsafe fn __fileread:rs_file_read(file: ref Arc<CFile>, out: Option<NonNull<u8>>, len: usize) -> Result<usize> {
        let mut slice = NonNull::slice_from_raw_parts(out.expect("fileread's buffer is null"), len);
        // SAFETY: precondition
        let buf = unsafe { slice.as_mut() };

        file.lock().read_exact(buf)?;
        Ok(len)
    }

    /// # Safety
    /// `file` must point to valid `DirEntry`
    /// `out` must be valid to reads for `len` bytes
    /// # Panics
    /// `out` is `None`
    unsafe fn __filewrite:rs_file_write(file: ref Arc<CFile>, out: Option<NonNull<u8>>, len: usize) -> Result<usize> {
        let slice = NonNull::slice_from_raw_parts(out.expect("filewrite's buffer is null"), len);
        // SAFETY: precondition
        let buf = unsafe { slice.as_ref() };

        file.lock().write_exact(buf)?;
        Ok(len)
    }

    /// # Errors
    /// `file` type is not Type::Regular
    /// filesystem errors
    pub fn fileseek:rs_file_seek(file: ref Arc<CFile>, offset: u64, whence: Option<Whence>) -> Result<u64> {
        let mut file = file.lock();
        let whence = whence.ok_or_else(|| {
            log::error!("attempt to seek with invalid `Whence`");
            ErrNo::EINVAL
        })?;
        let pos = file.seek((whence, offset).into())?;
        Ok(pos)
    }
}
