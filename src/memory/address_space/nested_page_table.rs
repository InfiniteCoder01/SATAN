use memory_addr::MemoryAddr;

use super::{MappingError, MappingFlags, MappingResult};
use super::{PageAllocatorTrait, PageSizeTrait};
use super::{PhysAddr, VirtAddr};

/// Page table entry returned
pub enum PageTableEntry<Level: NestedPageTableLevel> {
    /// Page table entry maps to the next level page table
    Level(Level),
    /// Page table entry identity maps (regular or large/huge pages)
    Page(PhysAddr, MappingFlags),
}

impl<Level: NestedPageTableLevel> PageTableEntry<Level> {
    const NULL: Self = Self::Page(PhysAddr::from_usize(0), MappingFlags::empty());
}

/// A single level of a nested page table
/// (underlying type should be something like a pointer that's freely cloneable)
pub trait NestedPageTableLevel: Clone + Sized {
    type PageSize: PageSizeTrait;

    /// Get the size of a page/page table of this layer, similar to page_size, but
    /// returns the memory region that a sub-level page table manages if page can't
    /// be mapped here
    fn region_size(&self) -> usize;

    /// Get page size of this layer, if a page can be mapped here
    fn page_size(&self) -> Option<Self::PageSize> {
        self.region_size().try_into().ok()
    }

    /// Allocate a new page table level, that's gonna come after this one
    fn new_sublevel(&self, alloc: &impl PageAllocatorTrait<Self::PageSize>) -> Option<Self>;

    /// Set an entry in this level. vaddr might not be aligned if entry
    /// is [`PageTableEntry::Level`]
    fn set_entry(&self, vaddr: VirtAddr, entry: PageTableEntry<Self>) -> MappingResult<()>;

    /// Get an entry in this page table. vaddr might not be aligned
    fn get_entry(&self, vaddr: VirtAddr) -> MappingResult<PageTableEntry<Self>>;

    /// Map a single (possibly large/huge) page.
    fn map_page(
        &self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        page_size: Self::PageSize,
        flags: MappingFlags,
        alloc: &impl PageAllocatorTrait<Self::PageSize>,
    ) -> MappingResult<()> {
        if self.page_size() == Some(page_size) {
            self.set_entry(vaddr, PageTableEntry::Page(paddr, flags))
        } else {
            let entry = self.get_entry(vaddr)?;
            let next_level = match entry {
                PageTableEntry::Page(addr, flags) => {
                    if flags.contains(MappingFlags::PRESENT) {
                        return Err(MappingError::MappingOver(addr));
                    }
                    let level = self
                        .new_sublevel(alloc)
                        .ok_or(MappingError::PageAllocationFailed)?;
                    self.set_entry(vaddr, PageTableEntry::Level(level.clone()))?;
                    level
                }
                PageTableEntry::Level(level) => level,
            };
            next_level.map_page(vaddr, paddr, page_size, flags, alloc)
        }
    }

    fn unmap_free(&self, vaddr: VirtAddr, size: usize) -> MappingResult<()> {
        let region_size = self.region_size();
        let start = vaddr.align_down(region_size);
        let end = (vaddr + size).align_up(region_size);
        for page in (start.as_usize()..end.as_usize()).step_by(region_size) {
            let page = VirtAddr::from(page);
            let entry = self.get_entry(page)?;
            if page < vaddr || page + region_size > vaddr + size {
                match entry {
                    PageTableEntry::Level(level) => {
                        if page < vaddr {
                            level.unmap_free(vaddr, page + region_size - vaddr)?;
                        } else {
                            level.unmap_free(page, vaddr + size - page)?;
                        }
                    }
                    PageTableEntry::Page(paddr, flags) => {
                        if flags.contains(MappingFlags::PRESENT) {
                            return Err(MappingError::UnmappingPartOfLargePage(paddr));
                        }
                    }
                }
            } else {
                match entry {
                    PageTableEntry::Level(level) => {
                        level.unmap_free(page, region_size)?;
                    }
                    PageTableEntry::Page(_, flags) => {
                        if flags.contains(MappingFlags::PRESENT) {
                            self.set_entry(page, PageTableEntry::NULL)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Implementation of [`super::AddressSpaceTrait`] for a nested page table
/// structure (x86 for example)
pub trait NestedPageTable {
    /// Page size
    type PageSize: PageSizeTrait;

    /// Single level of paging
    type Level: NestedPageTableLevel<PageSize = Self::PageSize>;

    /// Get top level page table for this address space
    fn top_level(&self) -> Self::Level;

    // /// Unmap a single (possibly large/huge) page or a whole page table of the same size.
    // /// As a layer should take [`AddressSpaceTrait::top_layer`]
    // /// DOES NOT FREE
    // fn unmap_page(
    //     &self,
    //     layer: Self::Layer,
    //     vaddr: VirtAddr,
    //     page_size: PageSize,
    //     alloc: &impl PageAllocatorTrait<PageSize>,
    // ) -> MappingResult<()> {
    //     if !vaddr.is_aligned(page_size.clone()) {
    //         return Err(MappingError::UnalignedVirtualAddress(vaddr));
    //     }

    //     if Self::page_size(&layer) == page_size.clone().into() {
    //         Self::set_entry(layer, vaddr, page_size)
    //     } else {
    //         self.unmap_page(Self::next_layer(layer, vaddr, None)?, vaddr, page_size)
    //     }
    // }

    /// Implementation of [`super::AddressSpaceTrait::map_alloc`]
    fn map_alloc(
        &self,
        vaddr: VirtAddr,
        size: usize,
        flags: MappingFlags,
        alloc: &impl PageAllocatorTrait<Self::PageSize>,
    ) -> MappingResult<VirtAddr> {
        // TODO: Possibly bigger pages
        for page in 0..size / Self::PageSize::MIN.into() {
            self.top_level().map_page(
                vaddr + page * Self::PageSize::MIN.into(),
                alloc.alloc(Self::PageSize::MIN).unwrap(),
                Self::PageSize::MIN,
                flags,
                alloc,
            )?;
        }
        Ok(vaddr)
    }

    /// Implementation of [`super::AddressSpaceTrait::unmap_free`]
    fn unmap_free(&self, vaddr: VirtAddr, size: usize) -> MappingResult<()> {
        self.top_level().unmap_free(vaddr, size)
    }
}
