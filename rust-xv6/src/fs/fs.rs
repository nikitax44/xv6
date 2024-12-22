use crate::errno::ErrNo;
use crate::ffi_interop::ToFFI;
use crate::fs::{CFile, MAIN_FS};
use alloc::sync::Arc;
use core::mem::MaybeUninit;
use core::str::FromStr;
use efs::file::Type;
use efs::fs::FileSystem;
use efs::path::Path;
use efs::permissions::Permissions;
use efs::types::{Gid, Uid};
use log::{error, trace};

crate::export_c_fn! {
    #[allow(clippy::large_stack_frames)]
    fn filecreate:rs_fs_create(path: Option<Path<'_>>) -> ErrNo {
        let Some(path) = path else {
            return ErrNo::EINVAL;
        };
        let mut fs = MAIN_FS.write();
        fs.create_file(&path, Type::Regular, Permissions::all(), Uid(0), Gid(0))
          .map(|_| ())
          .into()
    }

    pub fn mkdir:rs_fs_mkdir(path: Option<Path<'_>>) -> ErrNo {
        let Some(path) = path else {
            return ErrNo::EINVAL;
        };
        let mut fs = MAIN_FS.write();
        fs.create_file(&path, Type::Directory, Permissions::all(), Uid(0), Gid(0))
          .map(|_| ())
          .into()
    }

    pub fn mknod:rs_fs_mknod(path: Option<Path<'_>>, major: u32, minor: u32) -> ErrNo {
        let Some(path) = path else {
            return ErrNo::EINVAL;
        };
        if major != 1 || minor != 0 {
            return ErrNo::EINVAL;
        }
        let path = Path::from_str("/").unwrap().join(&path);
        trace!("mknod {}:{}:{}", path, major, minor);
        let mut fs = MAIN_FS.write();
        let ret = fs.create_file(&path, Type::CharacterDevice, Permissions::all(), Uid(0), Gid(0))
          .inspect_err(|err| error!("mknod error: {}", err))
          .map(|_| ())
          .into();
        trace!("mknod: {:?}", ret);
        ret
    }

   pub fn hardlink:rs_fs_hard_link(old: Option<Path<'_>>, new: Option<Path<'_>>) -> ErrNo {
        let Some((_old, _new)) = old.zip(new) else {
            return ErrNo::EINVAL
        };

        todo!()
    }

    pub fn unlink:rs_fs_unlink(path: Option<Path<'_>>) -> ErrNo {
        let Some(path) = path else {
            return ErrNo::EINVAL
        };

        let mut fs=MAIN_FS.write();
        fs.remove_file(path).into()
    }

    fn __pipealloc:rs_pipe_alloc(rp: Option<&mut MaybeUninit<*const CFile>>, wp: Option<&mut MaybeUninit<*const CFile>>) -> ErrNo {
        let Some((rp, wp)) = rp.zip(wp) else {
            return ErrNo::EFAULT;
        };

        match pipealloc() {
            Ok((r, w)) => {
                rp.write(r.into_ffi());
                wp.write(w.into_ffi());

                ErrNo::SUCCESS
            },
            Err(err) => err
        }
    }
}

pub fn pipealloc() -> Result<(Arc<CFile>, Arc<CFile>), ErrNo> {
    todo!()
}
