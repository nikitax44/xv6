use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Default, Debug)]
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinlockGuard<'l, T: 'l + Send> {
    lock: &'l Spinlock<T>,
    _no_send: PhantomData<*mut T>,
}

impl<T: Send> Spinlock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        SpinlockGuard {
            lock: self,
            _no_send: PhantomData,
        }
    }
}

impl<T: Clone + Send> Clone for Spinlock<T> {
    fn clone(&self) -> Self {
        Self::new(self.lock().clone())
    }
}

impl<T: Send> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

impl<T: Send> Deref for SpinlockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        // we have acquired the lock, nobody else can use it.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: Send> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:
        // we have acquired the lock, nobody else can use it.
        unsafe { &mut *self.lock.data.get() }
    }
}

// # SAFETY:
// if T can be sent, so can be `Spinlock<T>`
unsafe impl<T: Send> Send for Spinlock<T> {}
// # SAFETY:
// `Spinlock` guarantees that undelying value can be accessed only from one thread at a time
unsafe impl<T: Send> Sync for Spinlock<T> {}
