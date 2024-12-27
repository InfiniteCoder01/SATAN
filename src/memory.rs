use super::*;
pub use memory_addr::{pa, va, va_range, MemoryAddr, PhysAddr, VirtAddr};

bitflags::bitflags! {
    /// Generic page table entry flags that indicate the corresponding mapped
    /// memory region permissions and attributes.
    #[derive(Clone, Copy, PartialEq)]
    pub struct MappingFlags: usize {
        const PRESENT       = 1 << 0;
        /// The memory is readable.
        const READ          = 1 << 1;
        /// The memory is writable.
        const WRITE         = 1 << 2;
        /// The memory is executable.
        const EXECUTE       = 1 << 3;
        /// The memory is user accessible.
        const USER          = 1 << 4;
        /// The memory is device memory.
        const DEVICE        = 1 << 5;
        /// The memory is uncached.
        const UNCACHED      = 1 << 6;
    }
}

pub trait AddressSpace {
    fn map_page(
        &self,
        vaddr: VirtAddr,
        target: PhysAddr,
        page_size: arch::paging::PageSize,
        flags: MappingFlags,
    );
}
