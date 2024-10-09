use LazyCell::{Data, Init, Invalid};

pub enum LazyCell<T, F: FnOnce() -> T = fn() -> T> {
    Data(T),
    Init(F),
    Invalid,
}

impl<T, F: FnOnce() -> T> LazyCell<T, F> {
    pub const fn new(init: F) -> Self {
        Init(init)
    }
    /// # Panics
    /// never
    pub fn get_mut(&mut self) -> &mut T {
        let mut buf = Invalid;
        core::mem::swap(&mut buf, self);
        *self = Data(match buf {
            Data(inner) => inner,
            Init(init) => init(),
            Invalid => panic!("LazyCell in Invalid state"),
        });
        match self {
            Data(inner) => inner,
            _ => unreachable!(),
        }
    }
}
