#![no_std]
#![no_main]
#![cfg_attr(
    any(target_arch = "x86_64", target_arch = "x86"),
    feature(abi_x86_interrupt)
)]

#[cfg(debug_assertions)]
#[macro_use]
extern crate alloc;

pub mod arch;
pub mod panic;
