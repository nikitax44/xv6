use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

/// # Invariant
/// inner is valid and properly aligned.
/// `ThinBox` owns `inner`'s content
#[repr(transparent)]
pub struct ThinBox<T: ?Sized> {
    inner: NonNull<T>,
}

impl<T: ?Sized> ThinBox<T> {
    /// # Safety
    /// `inner` is valid, properly aligned, and you have ownership over its contents
    /// you transfer the ownership over the `inner` contents to `ThinBox`
    #[must_use]
    pub unsafe fn new(inner: NonNull<T>) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn leak(self) -> NonNull<T> {
        core::mem::ManuallyDrop::new(self).inner
    }
}

impl<T: ?Sized> From<&'static mut T> for ThinBox<T> {
    fn from(value: &'static mut T) -> Self {
        // SAFETY: inner is valid and properly aligned. we were given the ownership for 'static
        unsafe { Self::new(value.into()) }
    }
}

impl<T: ?Sized> Deref for ThinBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        // invariant
        unsafe { self.inner.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for ThinBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:
        // invariant
        unsafe { self.inner.as_mut() }
    }
}

impl<T: ?Sized> Drop for ThinBox<T> {
    fn drop(&mut self) {
        panic!("ThinBox was dropped");
    }
}
