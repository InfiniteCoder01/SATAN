#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
/// x86 and x86_64 architectures
pub mod x86;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86 as current;

pub use current::{_panic, _print};

pub mod interrupts {
    pub use super::current::interrupts::{disable, enable};
}

pub mod paging {
    pub use super::current::paging::{early_alloc_page, AddressSpace, PageSize};
}
