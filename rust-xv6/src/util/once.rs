use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

#[must_use]
pub struct Once<T = ()> {
    inner: UnsafeCell<MaybeUninit<T>>,
    is_init: AtomicU8,
}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Once<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
            is_init: AtomicU8::new(0),
        }
    }

    pub fn call_once(&self, f: impl FnOnce() -> T) -> &T {
        match self
            .is_init
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(0) => {
                // SAFETY: self.inner is valid and no concurrent accesses are performed
                unsafe {
                    core::ptr::write(self.inner.get(), MaybeUninit::new(f()));
                }
                self.is_init.store(2, Ordering::Release);
            }
            Err(1) => while self.is_init.load(Ordering::Acquire) == 1 {},
            Err(2) => {}
            _ => unreachable!(),
        };

        // SAFETY: `self.is_init` contains 2
        unsafe { &*self.inner.get().cast() }
    }

    pub fn get(&self) -> Option<&T> {
        // SAFETY: `self.is_init` contains 2
        (self.is_init.load(Ordering::Acquire) == 2).then(|| unsafe { &*self.inner.get().cast() })
    }

    /// # Errors
    /// `f` returned `Err`
    pub fn try_call_once<E>(&self, f: impl FnOnce() -> Result<T, E>) -> Result<&T, E> {
        match self
            .is_init
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(0) => {
                // SAFETY: self.inner is valid and no concurrent accesses are performed
                unsafe {
                    core::ptr::write(self.inner.get(), MaybeUninit::new(f()?));
                }
                self.is_init.store(2, Ordering::Release);
            }
            Err(1) => while self.is_init.load(Ordering::Acquire) == 1 {},
            Err(2) => {}
            _ => unreachable!(),
        };

        // SAFETY: `self.is_init` contains 2
        Ok(unsafe { &*self.inner.get().cast() })
    }
}

// SAFETY: `&Once<T>` is `Send`
unsafe impl<T: Send> Sync for Once<T> {}
