pub mod lazy;
pub mod spinlock;
pub mod string;
pub mod time;

pub type Mutex<T> = lock_api::Mutex<spinlock::Xv6Spinlock, T>;
