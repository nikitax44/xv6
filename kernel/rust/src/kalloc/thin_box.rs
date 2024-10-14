use crate::println;
use core::any::type_name;
use core::fmt::{Debug, Formatter};
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

/// # Invariant
/// inner is valid and properly aligned.
/// `ThinBox` owns `inner`'s content
#[repr(transparent)]
#[must_use]
pub struct ThinBox<T: ?Sized> {
    inner: NonNull<T>,
}

/// # SAFETY:
/// by invariant we own the value, so no aliased accesses are allowed
unsafe impl<T: ?Sized> Send for ThinBox<T> {}

impl<T: ?Sized> ThinBox<T> {
    /// # Safety
    /// `inner` is valid, properly aligned, and you have ownership over its contents
    /// you transfer the ownership over the `inner` contents to `ThinBox`
    pub const unsafe fn new(inner: NonNull<T>) -> Self {
        Self { inner }
    }

    /// # Safety
    /// reinterpret cast from T to U must be valid
    pub unsafe fn cast<U>(self) -> ThinBox<U> {
        // SAFETY: precondition
        unsafe { ThinBox::new(ManuallyDrop::new(self).inner.cast()) }
    }

    #[must_use]
    pub fn leak(self) -> NonNull<T> {
        core::mem::ManuallyDrop::new(self).inner
    }

    #[must_use]
    pub fn leak_ref(self) -> &'static mut T {
        // SAFETY: we owned it.
        unsafe { self.leak().as_mut() }
    }

    /// creates an aliased pointer for use in `ManuallyDrop` context
    pub fn inner(&mut self) -> NonNull<T> {
        self.inner
    }
}

impl<T: ?Sized> From<&'static mut T> for ThinBox<T> {
    fn from(value: &'static mut T) -> Self {
        // SAFETY: inner is valid and properly aligned. we now have the ownership for &'static mut
        unsafe { Self::new(value.into()) }
    }
}

impl<T: ?Sized> Deref for ThinBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        // by invariant we own the value for &'self
        unsafe { self.inner.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for ThinBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:
        // by invariant we own the value for &'self mut
        unsafe { self.inner.as_mut() }
    }
}

impl<T: ?Sized> Drop for ThinBox<T> {
    fn drop(&mut self) {
        println!(
            "ThinBox<{}>@{:0x?} was dropped",
            type_name::<T>(),
            self.inner
        );
    }
}

impl<T: ?Sized> Debug for ThinBox<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}
