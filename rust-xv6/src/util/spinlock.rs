use crate::bindings::{acquire, release, spinlock, try_acquire};
use core::ffi::CStr;
use lock_api::{GuardNoSend, RawMutex};

pub type Xv6Spinlock = spinlock;

// SAFETY: `name` and `cpu` are never dereferenced
unsafe impl Send for Xv6Spinlock {}
// SAFETY: `name` and `cpu` are never dereferenced
unsafe impl Sync for Xv6Spinlock {}

impl Xv6Spinlock {
    #[must_use]
    pub const fn new(name: &'static CStr) -> Self {
        Self {
            waiters: crate::bindings::AtomicU32 { value: 0 },
            released: crate::bindings::AtomicU32 { value: 0 },
            name: name.as_ptr(),
            cpu: core::ptr::null(),
        }
    }

    const fn as_mut(&self) -> *mut Self {
        core::ptr::from_ref(self).cast_mut()
    }

    pub fn with<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.lock();
        let res = f();
        // SAFETY: we acquired the lock
        unsafe {
            self.unlock();
        }
        res
    }
}

// SAFETY: contract is upheld
unsafe impl RawMutex for Xv6Spinlock {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self::new(c"RawMutex");
    type GuardMarker = GuardNoSend;

    fn lock(&self) {
        // SAFETY: ptr is valid
        unsafe { acquire(self.as_mut()) }
    }

    fn try_lock(&self) -> bool {
        // SAFETY: ptr is valid
        unsafe { try_acquire(self.as_mut()) != 0 }
    }

    unsafe fn unlock(&self) {
        // SAFETY: ptr is valid
        unsafe { release(self.as_mut()) }
    }
}

#[macro_export]
macro_rules! with_lock {
    ($lock:path, $f:expr) => {{
        #[allow(static_mut_refs)]
        // SAFETY: we do not observe the state
        let lock = unsafe { &$lock };
        lock.with($f)
    }};
}
