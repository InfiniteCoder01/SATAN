#![no_std]
#![no_main]
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
