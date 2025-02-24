#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
/// x86 and x86_64 architectures
pub mod x86;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86 as current;

// * Stub imports to make it easier to see required functions and types

pub use current::{_panic, _print};

/// Instructions like cpuid
pub mod instructions {
    pub use super::current::instructions::cpu_id;
}

/// Interrupt handling
pub mod interrupts {
    pub use super::current::interrupts::{disable, enable};
}
