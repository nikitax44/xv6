#![no_std]

extern crate alloc;

pub mod asm;
pub mod kalloc;
pub mod panic;
pub mod printf;
pub mod spinlock;

#[no_mangle]
pub extern "C" fn rustdiv(x: i32, y: i32) -> i32 {
    x / y
}

#[no_mangle]
pub extern "C" fn rustinit() {
    println!("hello world from rust!");
    println!("it even supports Ñ©и-αßсιι characters");
    println!("and {}: 3+8={}", "substitution", 3 + 8);
    println!();
}
