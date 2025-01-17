use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::{CriticalSection, RestoreState};

#[derive(Debug, Default)]
#[must_use]
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    locked: AtomicBool,
}

#[must_use]
pub struct MutexGuard<'cs, T> {
    mutex: &'cs Mutex<T>,
    cs: CriticalSection<'cs>,
    state: RestoreState,
}

impl<'cs, T> MutexGuard<'cs, T> {
    #[must_use]
    pub const fn get_cs(&self) -> CriticalSection<'cs> {
        self.cs
    }
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self
    where
        T: Sized,
    {
        Self {
            inner: UnsafeCell::new(value),
            locked: AtomicBool::new(false),
        }
    }

    #[allow(clippy::same_name_method)]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            }
            core::hint::spin_loop();
        }
    }

    #[allow(clippy::same_name_method)]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.locked.swap(true, Ordering::Acquire) {
            return None;
        }

        // SAFETY: TL;DW: it is correct
        let state = unsafe { critical_section::acquire() };
        // SAFETY: we're in a critical section
        let cs = unsafe { CriticalSection::new() };
        Some(MutexGuard {
            mutex: self,
            cs,
            state,
        })
    }

    /// # Safety
    /// lock must be held by caller (on the same HART)
    /// must be paired with `force_lock`
    /// no accesses to underlying data should be made until `force_lock` is called
    /// # Panics
    /// if mutex is not locked
    pub unsafe fn force_unlock(&self) {
        let was_locked = self.locked.swap(false, Ordering::Release);
        assert!(was_locked, "mutex is not locked");
        // SAFETY: precondition
        unsafe {
            critical_section::release(RestoreState::invalid());
        }
    }

    /// # Safety
    /// see `force_unlock`
    /// # Panics
    /// if mutex is not locked
    pub unsafe fn force_lock(&self) {
        let was_locked = self.locked.swap(true, Ordering::Release);
        assert!(!was_locked, "mutex is already locked");
        // SAFETY: precondition
        unsafe {
            critical_section::acquire();
        }
    }
}

// SAFETY: `&Mutex<T>` is `Send`
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: there is only one instance of MutexGuard corresponding to the mutex at a time
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: there is only one instance of MutexGuard corresponding to the mutex at a time
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
        // SAFETY: TL;DW: it is correct
        unsafe { critical_section::release(self.state) };
    }
}

mod _anon_ {
    use crate::util::mutex::Mutex;
    use core::sync::atomic::Ordering;
    use critical_section::RestoreState;
    use lock_api::GuardNoSend;

    // SAFETY: it is valid
    unsafe impl lock_api::RawMutex for Mutex<RestoreState> {
        #[allow(clippy::declare_interior_mutable_const)]
        const INIT: Self = Self::new(RestoreState::invalid());
        type GuardMarker = GuardNoSend;

        fn lock(&self) {
            let mut guard = Self::lock(self);
            *guard = guard.state;
            core::mem::forget(guard);
        }

        fn try_lock(&self) -> bool {
            let Some(mut guard) = Self::try_lock(self) else {
                return false;
            };
            *guard = guard.state;
            core::mem::forget(guard);
            true
        }

        unsafe fn unlock(&self) {
            // SAFETY: mutex is locked by us
            let state = unsafe { *self.inner.get() };
            self.locked.store(false, Ordering::Release);
            // SAFETY: mutex is locked by us
            unsafe {
                critical_section::release(state);
            }
        }
    }
}
