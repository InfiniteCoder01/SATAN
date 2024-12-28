use super::*;
pub use memory_addr::{pa, va, va_range, MemoryAddr, PhysAddr, VirtAddr};

bitflags::bitflags! {
    /// Generic page table entry flags that indicate the corresponding mapped
    /// memory region permissions and attributes.
    #[derive(Clone, Copy, PartialEq)]
    pub struct MappingFlags: usize {
        /// Memory is present. If not, generate a page fault interrupt
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
    );

    /// Get or create a page table layer in this layer that is associated with this virtual address
    fn next_layer(layer: Self::Layer, vaddr: VirtAddr) -> Self::Layer;

    /// Get top level page table layer for this address space
    fn top_layer(&self) -> Self::Layer;

    /// Map a single (possibly large/huge) page. As a layer should take [`AddressSpaceTrait::top_layer`]
    fn map_page(
        &self,
        layer: Self::Layer,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        page_size: arch::paging::PageSize,
        flags: MappingFlags,
    ) {
        if Self::page_size(&layer) == page_size as usize {
            Self::set_entry(layer, vaddr, paddr, page_size, flags);
        } else {
            self.map_page(
                Self::next_layer(layer, vaddr),
                vaddr,
                paddr,
                page_size,
                flags,
            );
        }
    }

    // TODO!!!
    fn unmap_page(&self, vaddr: VirtAddr, page_size: arch::paging::PageSize) {}
}
