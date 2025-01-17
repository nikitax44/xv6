use core::cell::UnsafeCell;
use core::ops::Deref;
use core::sync::atomic::{AtomicU8, Ordering};

#[must_use]
pub struct Lazy<T, F: FnOnce() -> T = fn() -> T> {
    inner: UnsafeCell<State<T, F>>,
    is_init: AtomicU8,
}

enum State<T, F: FnOnce() -> T> {
    Data(T),
    Init(F),
    Invalid,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    pub const fn new(initializer: F) -> Self {
        Self {
            inner: UnsafeCell::new(State::Init(initializer)),
            is_init: AtomicU8::new(0),
        }
    }

    pub fn init(&self) -> &T {
        match self
            .is_init
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(0) => {
                let State::Init(f) =
                    // SAFETY: self.inner is valid and no concurrent accesses are performed
                    (unsafe { core::ptr::replace(self.inner.get(), State::Invalid) })
                else {
                    unreachable!()
                };
                // SAFETY: same
                unsafe {
                    core::ptr::write(self.inner.get(), State::Data(f()));
                }

                self.is_init.store(2, Ordering::Release);
            }
            Err(1) => while self.is_init.load(Ordering::Acquire) == 1 {},
            Err(2) => {}
            _ => unreachable!(),
        };

        // SAFETY: `self.is_init` contains 2
        let State::Data(value) = (unsafe { &*self.inner.get() }) else {
            unreachable!()
        };
        value
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.init()
    }
}

// SAFETY: `&Lazy<T>` is `Send`
unsafe impl<T: Send> Sync for Lazy<T> {}
