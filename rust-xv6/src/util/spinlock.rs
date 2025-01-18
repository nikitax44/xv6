use core::ffi::{c_char, CStr};
use core::sync::atomic::AtomicU32;
use lock_api::{GuardNoSend, RawMutex};

#[repr(C)]
#[must_use]
pub struct Xv6Spinlock {
    locked: AtomicU32, // Is the lock held?

    // For debugging:
    name: *const c_char, // Name of lock.
    cpu: *const (),      // The cpu holding the lock.
}

// SAFETY: `name` and `cpu` are never dereferenced
unsafe impl Send for Xv6Spinlock {}
// SAFETY: `name` and `cpu` are never dereferenced
unsafe impl Sync for Xv6Spinlock {}

impl Xv6Spinlock {
    pub const fn new(name: &'static CStr) -> Self {
        Self {
            locked: AtomicU32::new(0),
            name: name.as_ptr(),
            cpu: core::ptr::null_mut(),
        }
    }
}

// SAFETY: contract is upheld
unsafe impl RawMutex for Xv6Spinlock {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self::new(c"RawMutex");
    type GuardMarker = GuardNoSend;

    fn lock(&self) {
        // SAFETY: ptr is valid
        unsafe { acquire(core::ptr::from_ref(self)) }
    }

    fn try_lock(&self) -> bool {
        // SAFETY: ptr is valid
        unsafe { try_acquire(core::ptr::from_ref(self)) }
    }

    unsafe fn unlock(&self) {
        // SAFETY: ptr is valid
        unsafe { release(core::ptr::from_ref(self)) }
    }
}

extern "C" {
    fn acquire(spin: *const Xv6Spinlock);
    fn try_acquire(spin: *const Xv6Spinlock) -> bool;
    fn release(spin: *const Xv6Spinlock);
}
