use super::types::{File, Result};
use crate::errno::ErrNo;
use crate::ffi_interop::ToFFI;
use crate::fs::path::{InvalidPath, OwnedPath, PathExt};
use crate::fs::types::Dir;
use crate::fs::types::Fs;
use crate::fs::{get_fs, CFile};
use crate::util::mutex::Mutex;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::mem::MaybeUninit;

pub static SPECIALS: Mutex<BTreeMap<String, Arc<CFile>>> = Mutex::new(BTreeMap::new());

crate::export_c_fn! {
    #[allow(clippy::large_stack_frames)]
    pub fn fileopen:rs_fs_open(path: Result<OwnedPath<'_>, InvalidPath>) -> Result<Arc<CFile>> {
        let path = path?;

        if let Some(spec)= SPECIALS.lock().get(&path.to_string()) {
            return Ok(Arc::clone(spec));
        }

        let mut dir = get_fs().get_root().map_err(Into::into)?
                                              .traverse(path.parent()).map_err(Into::into)?;
        let file = dir.open_file(path.basename()?).map_err(|err| {
            log::info!("failed to open: {path}");
            err.into()
        })?;

        let bx: Box<dyn File> = Box::try_new(file)?;
        let arc = Arc::new(Mutex::new(bx));
        Ok(arc)
    }


    #[allow(clippy::large_stack_frames)]
    fn filecreate:rs_fs_create(path: Result<OwnedPath<'_>, InvalidPath>) -> Result<()> {
        let path=path?;
        let fs = get_fs().get_root().map_err(Into::into)?;
        fs.traverse(path.parent()).map_err(Into::into)?.create_file(path.basename()?).map_err(Into::into)?;
        Ok(())
    }

    pub fn mkdir:rs_fs_mkdir(path: Result<OwnedPath<'_>, InvalidPath>) -> Result<()> {
        let path=path?;
        let fs = get_fs().get_root().map_err(Into::into)?;
        fs.traverse(path.parent()).map_err(Into::into)?.create_dir(path.basename()?).map_err(Into::into)?;
        Ok(())
    }

    pub fn mknod:rs_fs_mknod(path: Result<OwnedPath<'_>, InvalidPath>, major: u32, minor: u32) -> Result<()> {
        let path=path?;
        if major != 1 || minor != 0 {
            log::error!("attempt to call mknod with invalid `minor` and `major`");
            return Err(ErrNo::EINVAL);
        }

        SPECIALS.lock().insert(path.to_string(), Arc::new(Mutex::new(Box::new(crate::fs::special::Stdio))));
        Ok(())
    }

   pub fn hardlink:rs_fs_hard_link(old: Result<OwnedPath<'_>, InvalidPath>, new: Result<OwnedPath<'_>, InvalidPath>) -> Result<()> {
        let _old=old?;
        let _new=new?;

        todo!()
    }

    pub fn unlink:rs_fs_unlink(path: Result<OwnedPath<'_>, InvalidPath>) -> Result<()> {
        let path=path?;

        let fs=get_fs().get_root().map_err(Into::into)?;
        fs.traverse(path.parent()).map_err(Into::into)?.unlink(path.basename()?).map_err(Into::into)?;
        Ok(())
    }

    pub fn rmdir:rs_fs_rmdir(path: Result<OwnedPath<'_>, InvalidPath>) -> Result<()> {
        let path=path?;

        let fs=get_fs().get_root().map_err(Into::into)?;
        fs.traverse(path.parent()).map_err(Into::into)?.rmdir(path.basename()?).map_err(Into::into)?;
        Ok(())
    }

    fn __pipealloc:rs_pipe_alloc(rp: Option<&mut MaybeUninit<*const CFile>>, wp: Option<&mut MaybeUninit<*const CFile>>) -> Result<()> {
        let Some((rp, wp)) = rp.zip(wp) else {
            return Err(ErrNo::EFAULT);
        };

        let (r,w) = pipealloc()?;
        rp.write(r.into_ffi());
        wp.write(w.into_ffi());
        Ok(())
    }
}

pub fn pipealloc() -> Result<(Arc<CFile>, Arc<CFile>), ErrNo> {
    todo!()
}
