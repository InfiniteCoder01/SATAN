/// Traits that an architecture must implement
pub mod traits {
    /// Simple logger trait
    pub trait LoggerTrait {
        fn _print(args: core::fmt::Arguments);
        fn _panic(args: core::fmt::Arguments) -> !;
    }

    /// CPU instructions
    pub trait CpuTrait {
        /// Get CPU id, a unique number identifying a CPU core
        fn cpu_id() -> usize;
    }

    /// Memory abstraction layer
    pub trait MemoryTrait {
        type PageSize: crate::memory::PageSizeTrait;
        type PageAllocator: crate::memory::PageAllocatorTrait<Self::PageSize>;
        fn page_allocator() -> &'static Self::PageAllocator;
    }

    /// A trait that every architecture has to implement
    pub trait ArchTrait {
        /// Early Logger, must be available as soon as possible
        type EarlyLogger: LoggerTrait;
        /// See [Cpu]
        type Cpu: CpuTrait;
        /// See [MemoryTrait]
        type Memory: MemoryTrait;
    }
}

pub use traits::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
/// x86 and x86_64 architectures
pub mod x86;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86::Arch;

// Working around https://github.com/rust-lang/rust/issues/104119
pub type EarlyLogger = <Arch as ArchTrait>::EarlyLogger;
pub type Cpu = <Arch as ArchTrait>::Cpu;
pub type Memory = <Arch as ArchTrait>::Memory;
