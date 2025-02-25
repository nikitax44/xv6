#![no_std]
#![no_main]
#![deny(
    // keep-sorted start
    clippy::as_underscore,
    clippy::clone_on_ref_ptr,
    clippy::complexity,
    clippy::correctness,
    clippy::default_union_representation,
    clippy::missing_safety_doc,
    clippy::mixed_read_write_in_expression,
    clippy::nursery,
    clippy::self_named_module_files,
    clippy::suspicious,
    clippy::undocumented_unsafe_blocks,
    ffi_unwind_calls,
    unsafe_op_in_unsafe_fn
    // keep-sorted end
)]
#![warn(
    // keep-sorted start
    clippy::alloc_instead_of_core,
    clippy::cargo,
    clippy::decimal_literal_representation,
    clippy::else_if_without_else,
    clippy::empty_drop,
    clippy::float_arithmetic,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::infinite_loop,
    clippy::map_err_ignore,
    clippy::missing_assert_message,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::pedantic,
    clippy::perf,
    clippy::pub_without_shorthand,
    clippy::ref_patterns,
    clippy::renamed_function_params,
    clippy::same_name_method,
    clippy::style,
    clippy::suspicious_xor_used_as_pow,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc
    // keep-sorted end
)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::ptr_as_ptr, clippy::module_name_repetitions, reason = "Useful")]
#![allow(clippy::cargo_common_metadata, reason = "TODO")]
#![allow(refining_impl_trait, reason = "more informative")]
#![allow(internal_features, reason = "greatly simplifies debugging")]
#![allow(
    clippy::uninlined_format_args,
    reason = "inlined ones are harder to see"
)]
#![allow(
    clippy::multiple_crate_versions,
    reason = "I need FromZeros 0.8 but virtio_drivers uses 0.7"
)]
#![warn(clippy::large_stack_frames)]
#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(box_vec_non_null)]
#![feature(step_trait)]
#![feature(new_range_api)]
#![feature(maybe_uninit_uninit_array)]
#![feature(negative_impls)]
#![feature(maybe_uninit_as_bytes)]
#![feature(never_type)]
#![feature(try_with_capacity)]
#![feature(maybe_uninit_slice)]
#![feature(vec_push_within_capacity)]
#![feature(c_variadic)]
#![feature(ptr_as_uninit)]
#![feature(custom_test_frameworks)]
#![feature(slice_as_chunks)]
#![feature(int_roundings)]
#![feature(rustc_attrs)]
#![feature(duration_constants)]
#![feature(round_char_boundary)]
#![feature(slice_from_ptr_range)]
#![feature(const_format_args)]
#![test_runner(test_runner)]

extern crate alloc;

#[allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    unreachable_pub,
    dead_code,
    clippy::unreadable_literal,
    clippy::decimal_literal_representation
)]
#[path = "../xv6_binds.rs"]
pub mod bindings;
pub mod dtb;
pub mod errno;
mod exports;
pub mod ffi_interop;
pub mod hw;
pub mod kalloc;
mod log;
pub mod memlayout;
pub mod panic;
mod printf;
pub mod proc;
mod uapi;
pub mod util;
pub mod vm;

pub use crate::util::time::Instant;

#[macro_export]
macro_rules! static_assert {
    ($e:expr) => (
        const _: [(); { const ASSERT: bool = $e; ASSERT } as usize - 1] = [];
    );
}

#[macro_export]
macro_rules! time_me {
    (
        $(#[$($attrss:tt)*])*
        $vs:vis $(unsafe $(@ $uf:tt)?)?
        fn $name:ident($($arg:ident: $tp:ty),* $(,)?) $(-> $ret:ty)? $body:block
    ) => {
        $(#[$($attrss)*])*
        $vs $(unsafe $($uf)?)?
        fn $name($($arg: $tp),*) $(-> $ret)? {
            struct PerfGuard($crate::Instant);
            impl ::core::ops::Drop for PerfGuard {
                fn drop(&mut self) {
                    let now = $crate::Instant::now();
                    let elapsed = now - self.0;
                    if (elapsed >= ::core::time::Duration::from_millis(50)) {
                        ::log::debug!("perf: {} took {:?} to finish", stringify!(#function_identifier), elapsed);
                    }
                }
            }
            let _guard = PerfGuard($crate::Instant::now());

            $body
        }
    };
}

#[cfg(test)]
fn test_runner(_tests: &[&dyn Fn()]) {}

mod ffi {
    use crate::log::XV6Logger;
    use crate::memlayout::addrof_kernel;
    use core::fmt::{Debug, Display, Formatter};
    use lazy_static::lazy_static;
    use log::{error, info};

    static LOGGER: XV6Logger = XV6Logger {
        max_level: log::LevelFilter::Trace,
    };

    #[derive(Debug)]
    struct Features {
        start: usize,
    }

    lazy_static! {
        static ref FEATURES: Features = Features {
            start: addrof_kernel(),
        };
    }

    impl Display for Features {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            writeln!(f)?;
            writeln!(f, "  start: {:#x}", self.start)
        }
    }

    #[no_mangle]
    extern "C" fn init_logger() {
        log::set_max_level(LOGGER.max_level);
        if let Err(err) = log::set_logger(&LOGGER) {
            error!("failed to set logger: {err}");
        }
    }

    #[no_mangle]
    extern "C" fn dumpconf() {
        info!("rust features: {}", &*FEATURES);
    }

    #[no_mangle]
    extern "C" fn testpanic() {
        panic!("test panic");
    }
}
