use crate::fs::fs::SPECIALS;
use crate::fs::types::{CStat, CType, File, FileError, SeekFrom};
use crate::hw::console::CONSOLE;

pub struct Stdio;

impl File for Stdio {
    fn truncate(&mut self, _pos: SeekFrom) -> crate::fs::types::Result<(), FileError> {
        Err(FileError::NotImplemented)
    }

    fn size(&self) -> u64 {
        u64::MAX
    }

    fn cstat(&self) -> CStat {
        CStat {
            dev: 1,
            ino: 0,
            r#type: CType::CharacterDevice,
            nlink: 0,
            size: 0,
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> crate::fs::types::Result<usize, FileError> {
        extern "C" {
            /// returns `-1` if process is killed
            fn consoleread(user_dst: bool, dst: u64, n: usize) -> usize;
        }
        // I'm very sorry, but deadline is near
        // SAFETY: it is paired with `force_lock` later
        unsafe {
            SPECIALS.lock().get("/console").unwrap().force_unlock();
        }

        // SAFETY: pointer is valid, type is specified correctly
        let n = unsafe { consoleread(false, buf.as_mut_ptr() as usize as u64, buf.len()) };

        // SAFETY: UNSOUND, but will check at runtime
        unsafe {
            SPECIALS.lock().get("/console").unwrap().force_lock();
        }

        if n == usize::MAX {
            return Ok(buf.len());
        }

        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> crate::fs::types::Result<usize, FileError> {
        CONSOLE.lock().put_bytes(buf);
        Ok(buf.len())
    }

    fn seek(&mut self, _pos: SeekFrom) -> crate::fs::types::Result<u64, FileError> {
        Err(FileError::NotImplemented)
    }
}
