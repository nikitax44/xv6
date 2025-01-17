use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicIsize, Ordering};
use critical_section::{CriticalSection, RestoreState};

#[derive(Debug, Default)]
#[must_use]
pub struct RwLock<T> {
    inner: UnsafeCell<T>,
    locked: AtomicIsize, // -1 is locked for write
}

#[must_use]
pub struct WriteGuard<'cs, T> {
    mutex: &'cs RwLock<T>,
    cs: CriticalSection<'cs>,
    state: RestoreState,
}

impl<'cs, T> WriteGuard<'cs, T> {
    #[must_use]
    pub const fn get_cs(&self) -> CriticalSection<'cs> {
        self.cs
    }
}

#[must_use]
pub struct ReadGuard<'cs, T> {
    mutex: &'cs RwLock<T>,
    cs: CriticalSection<'cs>,
    state: RestoreState,
}

impl<'cs, T> ReadGuard<'cs, T> {
    #[must_use]
    pub const fn get_cs(&self) -> CriticalSection<'cs> {
        self.cs
    }
}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self
    where
        T: Sized,
    {
        Self {
            inner: UnsafeCell::new(value),
            locked: AtomicIsize::new(0),
        }
    }

    pub fn upgradeable_read(&self) -> ReadGuard<'_, T> {
        self.read()
    }
    pub fn read(&self) -> ReadGuard<'_, T> {
        loop {
            if let Some(guard) = self.try_read() {
                return guard;
            }
            core::hint::spin_loop();
        }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        loop {
            if let Some(guard) = self.try_write() {
                return guard;
            }
            core::hint::spin_loop();
        }
    }

    pub fn try_read(&self) -> Option<ReadGuard<'_, T>> {
        let curr = self.locked.load(Ordering::Relaxed);
        if curr < 0 {
            return None;
        }
        if self
            .locked
            .compare_exchange(curr, curr + 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return None;
        }

        // SAFETY: TL;DW: it is correct
        let state = unsafe { critical_section::acquire() };
        // SAFETY: we're in a critical section
        let cs = unsafe { CriticalSection::new() };
        Some(ReadGuard {
            mutex: self,
            cs,
            state,
        })
    }

    pub fn try_write(&self) -> Option<WriteGuard<'_, T>> {
        if self.locked.swap(-1, Ordering::Acquire) != 0 {
            return None;
        }

        // SAFETY: TL;DW: it is correct
        let state = unsafe { critical_section::acquire() };
        // SAFETY: we're in a critical section
        let cs = unsafe { CriticalSection::new() };
        Some(WriteGuard {
            mutex: self,
            cs,
            state,
        })
    }
}
// SAFETY: `&RwLock<T>` is `Send`
unsafe impl<T: Send> Sync for RwLock<T> {}

impl<'cs, T> ReadGuard<'cs, T> {
    pub fn upgrade(self) -> WriteGuard<'cs, T> {
        while self
            .mutex
            .locked
            .compare_exchange(1, -1, Ordering::SeqCst, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        let guard = WriteGuard {
            mutex: self.mutex,
            cs: self.cs,
            state: self.state,
        };
        core::mem::forget(self);
        guard
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: there is only one instance of MutexGuard corresponding to the mutex at a time
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: there is only one instance of MutexGuard corresponding to the mutex at a time
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: there is only one instance of MutexGuard corresponding to the mutex at a time
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(0, Ordering::Release);
        // SAFETY: TL;DW: it is correct
        unsafe { critical_section::release(self.state) };
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        let old = self.mutex.locked.fetch_sub(1, Ordering::Release);
        assert!(
            old > 0,
            "`ReadGuard::drop`: parent `RwLock` is held by `WriteGuard`"
        );
        // SAFETY: TL;DW: it is correct
        unsafe { critical_section::release(self.state) };
    }
}
