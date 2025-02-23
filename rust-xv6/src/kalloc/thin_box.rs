use crate::kalloc::Page;
use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use core::any::type_name;
use core::fmt::{Debug, Formatter};
use core::mem;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use log::error;
use zerocopy::FromZeros;

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
unsafe impl<T: Send + ?Sized> Send for ThinBox<T> {}
/// # SAFETY:
/// `&ThinBox<T>` is `Send`
unsafe impl<T: Send + Sync + ?Sized> Sync for ThinBox<T> {}

impl<T: ?Sized> ThinBox<T> {
    /// # Panics
    /// `inner` is not properly aligned
    /// # Safety
    /// `inner` is valid, and you have ownership over its contents
    /// you transfer the ownership over the `inner` contents to `ThinBox`
    pub unsafe fn new(inner: NonNull<T>) -> Self
    where
        T: Sized,
    {
        assert!(inner.is_aligned(), "ThinBox::new(unaligned)");
        Self { inner }
    }

    /// # Safety
    /// `inner` is valid, properly aligned, and you have ownership over its contents
    /// you transfer the ownership over the `inner` contents to `ThinBox`
    pub const unsafe fn new_unchecked(inner: NonNull<T>) -> Self {
        Self { inner }
    }

    /// # Safety
    /// reinterpret cast from T to U must be valid
    pub unsafe fn cast<U>(self) -> ThinBox<U> {
        // SAFETY: precondition
        unsafe { ThinBox::new(ManuallyDrop::new(self).inner.cast()) }
    }

    #[must_use]
    pub const fn const_deref(&self) -> &T {
        // SAFETY:
        // by invariant we own the value for &'self
        unsafe { self.inner.as_ref() }
    }

    #[must_use]
    pub const fn const_deref_mut(&mut self) -> &mut T {
        // SAFETY:
        // by invariant we own the value for &'self mut
        unsafe { self.inner.as_mut() }
    }

    #[must_use]
    pub const fn leak(self) -> NonNull<T> {
        let value = self.inner;
        mem::forget(self);
        value
    }

    #[must_use]
    pub const fn leak_ref(self) -> &'static mut T {
        // SAFETY: we owned it.
        unsafe { self.leak().as_mut() }
    }

    pub fn free(self) {
        let ptr = self.leak();
        // SAFETY: we owned it.
        unsafe {
            let _box = Box::from_non_null(ptr);
        }
    }
}

impl<T: 'static> ThinBox<T> {
    /// # Errors
    /// out of memory
    pub fn alloc() -> Result<ThinBox<MaybeUninit<T>>, core::alloc::AllocError> {
        Box::<T>::try_new_uninit().map(Box::leak).map(From::from)
    }

    /// # Errors
    /// failed to allocate memory
    pub fn alloc_array(len: usize) -> Result<ThinBox<[MaybeUninit<T>]>, TryReserveError> {
        Vec::try_with_capacity(len).map(Vec::leak).map(From::from)
    }
}

impl ThinBox<Page> {
    /// # Errors
    /// failed to allocate Page
    pub fn alloc_page() -> Result<Self, core::alloc::AllocError> {
        // SAFETY: page contents are already wrapped in MaybeUninit
        unsafe { Ok(Self::alloc()?.assume_init()) }
    }
}

impl<T> ThinBox<MaybeUninit<T>> {
    pub fn zero(mut self) -> Self {
        MaybeUninit::as_bytes_mut(&mut self).fill(MaybeUninit::zeroed());
        self
    }

    /// # Safety
    /// value must be initialized
    pub unsafe fn assume_init(self) -> ThinBox<T> {
        // SAFETY: precondition
        unsafe { self.cast() }
    }

    pub fn zeroed(self) -> ThinBox<T>
    where
        T: FromZeros,
    {
        // SAFETY: T is `FromZeros`
        unsafe { self.zero().assume_init() }
    }
}

// actually 'static is not a requirement, but for bow i'll leave it be
impl<T: 'static> ThinBox<[MaybeUninit<T>]> {
    /// # Safety
    /// every element must be initialized
    pub unsafe fn slice_assume_init_mut(self) -> ThinBox<[T]> {
        // SAFETY: precondition
        unsafe { self.leak_ref().assume_init_mut() }.into()
    }

    pub fn zeroed_array(mut self) -> ThinBox<[T]>
    where
        T: FromZeros,
    {
        self.deref_mut().fill_with(MaybeUninit::zeroed);
        // SAFETY: T is `FromZeros`
        unsafe { self.slice_assume_init_mut() }
    }
}

impl<T: ?Sized> From<&'static mut T> for ThinBox<T> {
    fn from(value: &'static mut T) -> Self {
        // SAFETY: inner is valid and properly aligned. we now have the ownership for &'static mut
        unsafe { Self::new_unchecked(value.into()) }
    }
}

impl<T: ?Sized> Deref for ThinBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.const_deref()
    }
}

impl<T: ?Sized> DerefMut for ThinBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.const_deref_mut()
    }
}

impl<T: ?Sized> Drop for ThinBox<T> {
    fn drop(&mut self) {
        error!(
            "memory leak: ThinBox<{}>@{:0x?} was dropped",
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
