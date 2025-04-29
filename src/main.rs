#![no_std]
#![no_main]
#![cfg_attr(
    any(target_arch = "x86_64", target_arch = "x86"),
    feature(abi_x86_interrupt)
)]
#![feature(allocator_api)]

extern crate alloc;

/// Synchronization primitives
pub mod sync;

/// Architecture implementaitons
pub mod arch;
pub use arch::Arch;

/// Basic logging facilities, calling arch-specific early print and panic functions
pub mod log;

/// Memory interfaces
pub mod memory;
