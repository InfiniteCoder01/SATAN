use super::{PageAllocatorTrait, PageSizeTrait};
use memory_addr::{PhysAddr, VirtAddr};

bitflags::bitflags! {
    /// Generic page table entry flags that indicate the corresponding mapped
    /// memory region permissions and attributes.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MappingFlags: usize {
        /// Memory is present. If not, generate a page fault
        const PRESENT       = 1 << 0;
        /// The memory is readable.
        const READ          = 1 << 1;
        /// The memory is writable.
        const WRITE         = 1 << 2;
        /// The memory is executable.
        const EXECUTE       = 1 << 3;
        /// The memory is user accessible.
        const USER          = 1 << 4;
        /// The memory is uncached.
        const UNCACHED      = 1 << 5;
        /// The memory globally accessible, doesn't invalidate TLB.
        const GLOBAL        = 1 << 6;
    }
}

/// Kinds of errors if mapping failed
#[derive(Clone, Debug, thiserror::Error)]
pub enum MappingError {
    /// Mapping over an already existing page
    #[error("mapping over existing page at address {0:#x}")]
    MappingOver(PhysAddr),
    /// Page allocation failed
    #[error("page allocation failed")]
    PageAllocationFailed,

    /// Mapping an unaligned address
    #[error("mapping an unaligned address {0:#x}")]
    UnalignedPhysicalAddress(PhysAddr),
    /// Mapping to an unaligned address
    #[error("mapping to an unaligned address {0:#x}")]
    UnalignedVirtualAddress(VirtAddr),
    /// Unmapping a page that wasn't mapped
    #[error("unmapping a page that wasn't mapped (address {0:#x})")]
    UnmappingNotMapped(VirtAddr),
    /// Unmapping part of a large page
    #[error("unmapping part of a large page at {0:#x}")]
    UnmappingPartOfLargePage(PhysAddr),
}

/// Result type for memory mapping operations
pub type MappingResult<T> = Result<T, MappingError>;

pub mod nested_page_table;

/// Address space allows for control over accessible memory
pub trait AddressSpaceTrait<PageSize: PageSizeTrait> {
    // pub fn map(&mut self, vaddr: VirtAddr, paddr: PhysAddr, size: usize) -> MappingResult<VirtAddr>;
    // pub fn unmap(&mut self, addr: VirtAddr, size: usize) -> MappingResult<()>;

    /// Allocate and map a region of memory into
    /// the address space. On success returns
    /// actual address region has been mapped to.
    /// vaddr must be a valid hint
    fn map_alloc(
        &self,
        vaddr: VirtAddr,
        size: usize,
        flags: MappingFlags,
        alloc: &impl PageAllocatorTrait<PageSize>,
    ) -> MappingResult<VirtAddr>;

    /// Unmap a region of memory from the address space and mark it as free
    fn unmap_free(
        &self,
        vaddr: VirtAddr,
        size: usize,
        alloc: &impl PageAllocatorTrait<PageSize>,
    ) -> MappingResult<()>;
}
