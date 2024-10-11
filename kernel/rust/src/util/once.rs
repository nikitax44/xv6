use crate::util::once::OnceError::{AlreadyCalled, InProgress};
use core::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug)]
pub enum OnceError {
    InProgress,
    AlreadyCalled,
}

#[derive(Default)]
pub struct Once {
    state: AtomicU8,
}

impl Once {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
        }
    }

    /// # Errors
    /// `Once` was initialized or in progress
    pub fn init(&self, func: impl FnOnce()) -> Result<(), OnceError> {
        if let Err(old) = self
            .state
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        {
            return match old {
                1 => Err(InProgress),
                2 => Err(AlreadyCalled),
                _ => unreachable!(),
            };
        }
        func();
        self.state.store(2, Ordering::SeqCst);
        Ok(())
    }
    pub fn wait(&self) {
        while self.state.load(Ordering::SeqCst) != 2 {
            core::hint::spin_loop();
        }
    }
}
