#![no_std]
#![deny(
    // keep-sorted start
    clippy::allow_attributes_without_reason,
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
#![allow(clippy::ptr_as_ptr, clippy::module_name_repetitions, reason = "Useful")]
#![allow(clippy::cargo_common_metadata, reason = "TODO")]
#![allow(
    clippy::multiple_crate_versions,
    reason = "I need FromZeros 0.8 but virtio_drivers uses 0.7"
)]
#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(box_vec_non_null)]
extern crate alloc;

pub mod asm;
pub mod dtb;
pub mod errno;
pub mod hw;
pub mod kalloc;
pub mod memlayout;
pub mod panic;
pub mod printf;
pub mod util;
pub mod vm;

mod ffi {
    use crate::println;

    #[cfg(feature = "rust_kalloc")]
    const KALLOC: &str = "rust";
    #[cfg(not(feature = "rust_kalloc"))]
    const KALLOC: &str = "C";

    #[no_mangle]
    extern "C" fn dumpconf() {
        println!("rust features:");
        println!("  kalloc: {}", KALLOC);
        println!();
    }

    #[no_mangle]
    extern "C" fn testpanic() {
        panic!("test panic");
    }
}
