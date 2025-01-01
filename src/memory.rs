use super::*;
pub use memory_addr::{pa, va, va_range, MemoryAddr, PhysAddr, VirtAddr};

bitflags::bitflags! {
    /// Generic page table entry flags that indicate the corresponding mapped
    /// memory region permissions and attributes.
    #[derive(Clone, Copy, PartialEq)]
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

/// Trait to be implemented by an address space
pub trait AddressSpaceTrait {
    /// Single page table
    type Layer;

    /// Page size of one page
    fn page_size(layer: &Self::Layer) -> usize;

    /// Set an entry in the page table layer to map vaddr to paddr with size and flags
    fn set_entry(
        layer: Self::Layer,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        page_size: arch::paging::PageSize,
        flags: MappingFlags,
    ) -> MappingResult<()>;

    /// Unset an entry in the page table layer
    fn unset_entry(
        layer: Self::Layer,
        vaddr: VirtAddr,
        page_size: arch::paging::PageSize,
    ) -> MappingResult<()>;

    /// Get or create (only if map is true) a page table layer in this layer
    /// that is associated with this virtual address. map parameter indicates
    /// if this call corresponds to mapping/unmapping operation
    fn next_layer(layer: Self::Layer, vaddr: VirtAddr, map: bool) -> MappingResult<Self::Layer>;

    /// Get top level page table layer for this address space
    fn top_layer(&self) -> Self::Layer;

    /// Map a single (possibly large/huge) page.
    /// As a layer should take [`AddressSpaceTrait::top_layer`]
    fn map_page(
        &self,
        layer: Self::Layer,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        page_size: arch::paging::PageSize,
        flags: MappingFlags,
    ) -> MappingResult<()> {
        if !vaddr.is_aligned(page_size as usize) {
            return Err(MappingError::UnalignedVirtualAddress(vaddr));
        }
        if !paddr.is_aligned(page_size as usize) {
            return Err(MappingError::UnalignedPhysicalAddress(paddr));
        }

        if Self::page_size(&layer) == page_size as usize {
            Self::set_entry(layer, vaddr, paddr, page_size, flags)
        } else {
            self.map_page(
                Self::next_layer(layer, vaddr, true)?,
                vaddr,
                paddr,
                page_size,
                flags,
            )
        }
    }

    /// Unmap a single (possibly large/huge) page or a whole page table of the same size.
    /// As a layer should take [`AddressSpaceTrait::top_layer`]
    fn unmap_page(
        &self,
        layer: Self::Layer,
        vaddr: VirtAddr,
        page_size: arch::paging::PageSize,
    ) -> MappingResult<()> {
        if !vaddr.is_aligned(page_size as usize) {
            return Err(MappingError::UnalignedVirtualAddress(vaddr));
        }

        if Self::page_size(&layer) == page_size as usize {
            Self::unset_entry(layer, vaddr, page_size)
        } else {
            self.unmap_page(Self::next_layer(layer, vaddr, false)?, vaddr, page_size)
        }
    }
}
