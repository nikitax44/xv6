#![no_std]
#![deny(
    unsafe_op_in_unsafe_fn,
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::as_underscore,
    clippy::clone_on_ref_ptr,
    clippy::default_union_representation,
    clippy::mixed_read_write_in_expression,
    clippy::self_named_module_files,
    clippy::undocumented_unsafe_blocks,
    clippy::nursery
)]
#![warn(
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    clippy::cargo,
    clippy::alloc_instead_of_core,
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
    clippy::pub_without_shorthand,
    clippy::ref_patterns,
    clippy::renamed_function_params,
    clippy::same_name_method,
    clippy::suspicious_xor_used_as_pow
)]
#![allow(clippy::ptr_as_ptr, clippy::module_name_repetitions, reason = "Useful")]

extern crate alloc;

pub mod asm;
pub mod errno;
pub mod kalloc;
pub mod memlayout;
pub mod panic;
pub mod printf;
pub mod spinlock;
pub mod util;
pub mod vm;
