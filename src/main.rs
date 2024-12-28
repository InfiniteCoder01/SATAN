#![no_std]
#![no_main]
#![cfg_attr(
    any(target_arch = "x86_64", target_arch = "x86"),
    feature(abi_x86_interrupt)
)]

/// Arch-specific things
pub mod arch;

/// Basic logging facilities, calling arch-specific early print and panic functions
pub mod log;

/// Memory interfaces
pub mod memory;
