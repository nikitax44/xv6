use crate::errno::ErrNo;
use crate::fs::types::Whence;
use crate::kalloc::pages::page::{Page, PageHandle};
use crate::kalloc::thin_box::ThinBox;
use alloc::sync::Arc;
use core::ffi::{c_char, CStr};
use core::mem::MaybeUninit;
use core::panic::Location;
use core::ptr::NonNull;

pub trait FFICast {
    type Safe: FFISafe;

    fn into_ffi_cast(self) -> Self::Safe;
    /// # Safety
    /// implementation-specific
    unsafe fn from_ffi_cast(value: Self::Safe) -> Self;
}

pub trait ToFFI {
    type Target: FFISafe;
    fn into_ffi(self) -> Self::Target;
}

pub trait FromFFI {
    type Source: FFISafe;

    /// # Safety
    /// implementation-specific
    unsafe fn from_ffi(value: Self::Source) -> Self;
}

impl<T: FFICast> ToFFI for T {
    type Target = T::Safe;

    fn into_ffi(self) -> Self::Target {
        self.into_ffi_cast()
    }
}

impl<T: FFICast> FromFFI for T {
    type Source = T::Safe;

    unsafe fn from_ffi(value: Self::Source) -> Self {
        // SAFETY: precondition
        unsafe { T::from_ffi_cast(value) }
    }
}

pub trait FFISafe: Copy + Sized {}

impl<T: FFISafe> FFICast for T {
    type Safe = T;
    fn into_ffi_cast(self) -> Self::Safe {
        self
    }

    /// # Safety
    /// safe
    unsafe fn from_ffi_cast(value: Self::Safe) -> Self {
        value
    }
}

impl<T> FFICast for Arc<T> {
    type Safe = *const T;

    fn into_ffi_cast(self) -> Self::Safe {
        Self::into_raw(self)
    }

    unsafe fn from_ffi_cast(value: Self::Safe) -> Self {
        assert!(!value.is_null(), "attempt to convert null to Arc");
        assert!(value.is_aligned(), "attempt to convert {value:?} to Arc");
        // SAFETY: precondition
        unsafe { Self::from_raw(value) }
    }
}

impl<T> FFICast for Option<Arc<T>> {
    type Safe = *const T;

    fn into_ffi_cast(self) -> Self::Safe {
        self.map_or(core::ptr::null(), Arc::into_raw)
    }

    unsafe fn from_ffi_cast(value: Self::Safe) -> Self {
        if value.is_null() {
            return None;
        }
        // SAFETY: precondition
        unsafe { Some(Arc::from_raw(value)) }
    }
}

impl FFICast for &CStr {
    type Safe = *const c_char;

    fn into_ffi_cast(self) -> Self::Safe {
        self.as_ptr()
    }

    unsafe fn from_ffi_cast(value: Self::Safe) -> Self {
        // SAFETY: precondition
        unsafe { CStr::from_ptr(value) }
    }
}

impl FFICast for Option<PageHandle> {
    type Safe = Option<NonNull<Page>>;

    fn into_ffi_cast(self) -> Self::Safe {
        self.map(|page| page.into_box().leak())
    }

    unsafe fn from_ffi_cast(value: Self::Safe) -> Self {
        value
            // SAFETY: precondition
            .map(|ptr| unsafe { ThinBox::new(ptr) })
            .map(|ptr| PageHandle::from_box(ptr, Location::caller(), "unknown C source"))
    }
}

impl FFISafe for () {}
impl FFISafe for crate::fs::types::CStat {}
impl FFISafe for isize {}
impl FFISafe for usize {}
impl FFISafe for i32 {}
impl FFISafe for u32 {}
impl FFISafe for i64 {}
impl FFISafe for u64 {}
impl FFICast for Option<Whence> {
    type Safe = u32;

    fn into_ffi_cast(self) -> Self::Safe {
        match self {
            None | Some(Whence::Set) => 0,
            Some(Whence::Head) => 1,
            Some(Whence::End) => 2,
        }
    }

    unsafe fn from_ffi_cast(value: Self::Safe) -> Self {
        match value {
            0 => Some(Whence::Set),
            1 => Some(Whence::Head),
            2 => Some(Whence::End),
            _ => None,
        }
    }
}

impl<T> FromFFI for Option<&mut MaybeUninit<T>> {
    type Source = Option<NonNull<T>>;

    unsafe fn from_ffi(value: Self::Source) -> Self {
        value.map(|ptr| {
            assert!(ptr.is_aligned(), "ffi passed unaligned pointer");
            // SAFETY: precondition
            unsafe { ptr.as_uninit_mut() }
        })
    }
}

impl ToFFI for Result<u64, ErrNo> {
    type Target = i64;

    fn into_ffi(self) -> Self::Target {
        match self {
            Ok(val) => val.try_into().ok().unwrap_or_else(|| {
                log::error!("ffi cast overflow");
                -(ErrNo::EOVERFLOW as i64)
            }),
            Err(err) => -(err as i64),
        }
    }
}

impl ToFFI for Result<usize, ErrNo> {
    type Target = isize;

    fn into_ffi(self) -> Self::Target {
        match self {
            Ok(val) => val.try_into().ok().unwrap_or_else(|| {
                log::error!("ffi cast overflow");
                -(ErrNo::EOVERFLOW as isize)
            }),
            Err(err) => -(err as isize),
        }
    }
}

impl ToFFI for Result<(), ErrNo> {
    type Target = i32;

    fn into_ffi(self) -> Self::Target {
        match self {
            Ok(()) => 0,
            Err(err) => -(err as i32),
        }
    }
}

impl<U: Sized, T: ToFFI<Target = *const U>> ToFFI for Result<T, ErrNo> {
    type Target = *const U;

    fn into_ffi(self) -> Self::Target {
        match self {
            Ok(val) => val.into_ffi(),
            Err(err) => {
                log::error!("passing error to C: {:?}", err);
                (-(err as i64)) as *const U
            }
        }
    }
}

impl<T> FFISafe for Option<NonNull<T>> {}
impl<T> FFISafe for *const T {}

#[macro_export]
macro_rules! export_c_fn {
    (
        $(#[$($attrss:tt)*])*
        $(pub $(@ $pub_:tt)? )? $(unsafe $(@ $unsafe_:tt)? )? fn
            $name:ident
            : $cname:ident
            ($($arg:ident: $(ref $(@ $ref_:tt)?)? $val:ty),* $(,)?)
            $(-> $ret:ty)?
            $body:block
        $($tts:tt)*
    ) => {
        $(#[$($attrss)*])*
        // #[attr_wrapper::time_me]
        $(pub $($pub_)? )? $(unsafe $($unsafe_)? )? fn $name($($arg: $(& $($ref_)?)? $val),*)
            $(-> $ret)?
            $body

        #[doc=concat!(" generated wrapper for `", stringify!($name), "`")]
        #[no_mangle]
        unsafe extern "C" fn $cname($($arg: <$val as $crate::ffi_interop::FromFFI>::Source),*) $(-> <$ret as $crate::ffi_interop::ToFFI>::Target)? {
            $($crate::export_c_fn!(@incast $arg: $val);)*

            let value = $(
            // SAFETY: we're private. caller always checks
            unsafe
            $($unsafe_)?
            )? {
                $name($($(& $($ref_)?)? $arg),*)
            };

            $($($($ref_)?
            $crate::ffi_interop::ToFFI::into_ffi($arg);
            )?)*

            $crate::ffi_interop::ToFFI::into_ffi(value)
        }


        $crate::export_c_fn!($($tts)*);
    };
    () => {};

    (@incast $name:ident: $tp:ty) => {
        // SAFETY: precondition
        let $name: $tp = unsafe { <$tp as $crate::ffi_interop::FromFFI>::from_ffi($name) };
    }
}
