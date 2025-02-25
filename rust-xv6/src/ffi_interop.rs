use crate::kalloc::thin_box::ThinBox;
use crate::kalloc::Page;
use alloc::sync::Arc;
use core::ffi::{c_char, CStr};
use core::ptr::NonNull;

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

pub trait FFISafe: Copy {}

impl<T: FFISafe> ToFFI for T {
    type Target = T;
    fn into_ffi(self) -> Self::Target {
        self
    }
}

impl<T: FFISafe> FromFFI for T {
    type Source = T;
    unsafe fn from_ffi(value: Self::Source) -> Self {
        value
    }
}

impl<T> ToFFI for Arc<T> {
    type Target = *const T;

    fn into_ffi(self) -> Self::Target {
        Self::into_raw(self)
    }
}

impl<T> FromFFI for Arc<T> {
    type Source = *const T;

    unsafe fn from_ffi(value: Self::Source) -> Self {
        // SAFETY: precondition
        unsafe { Self::from_raw(value) }
    }
}

impl FromFFI for &CStr {
    type Source = *const c_char;

    unsafe fn from_ffi(value: Self::Source) -> Self {
        // SAFETY: precondition
        unsafe { CStr::from_ptr(value) }
    }
}

impl ToFFI for Option<ThinBox<Page>> {
    type Target = Option<NonNull<Page>>;

    fn into_ffi(self) -> Self::Target {
        self.map(ThinBox::leak)
    }
}
impl FromFFI for Option<ThinBox<Page>> {
    type Source = Option<NonNull<Page>>;

    unsafe fn from_ffi(value: Self::Source) -> Self {
        // SAFETY: precondition
        value.map(|ptr| unsafe { ThinBox::new(ptr) })
    }
}

impl FFISafe for () {}
// impl FFISafe for crate::fs::types::CStat {}
impl FFISafe for usize {}
impl FFISafe for i64 {}
// impl FFISafe for Whence {}

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
        #[macro_rules_attribute::apply($crate::time_me)]
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
