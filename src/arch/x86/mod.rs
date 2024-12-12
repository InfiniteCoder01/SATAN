#[cfg(target_arch = "x86")]
pub mod x32;
#[cfg(target_arch = "x86")]
pub use x32::*;

#[cfg(target_arch = "x86_64")]
pub mod x64;
#[cfg(target_arch = "x86_64")]
pub use x64::*;
