use memory_addr::{MemoryAddr, PhysAddr, VirtAddr};

use super::tmp_page;
use super::PageSize;
/// Physical page table entry types
mod entry {
    pub(super) use super::super::{PTEFlags, PTEntry};
}

use crate::memory::address_space::nested_page_table::{NestedPageTable, NestedPageTableLevel};
use crate::memory::address_space::AddressSpaceTrait;
use crate::memory::{MappingError, MappingResult, PageAllocatorTrait};

/// Interface page table entry types
mod if_entry {
    pub(super) use crate::memory::address_space::nested_page_table::PageTableEntry;
    pub(super) use crate::memory::MappingFlags;
}

/// Address space struct
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressSpace(PageTableLevel);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageTableLevel(PhysAddr, usize);

impl AddressSpace {
    pub(super) fn from_paddr(addr: PhysAddr) -> Self {
        #[cfg(target_arch = "x86")]
        let top_level_bits = 22;
        #[cfg(target_arch = "x86_64")]
        let top_level_bits = 39;
        Self(PageTableLevel(addr, top_level_bits))
    }
}

impl PageTableLevel {
    /// Map the page table level to tmp page
    /// and get the page table entry associated with this address
    fn lock_entry(&self, vaddr: VirtAddr) -> crate::sync::MappedLockGuard<entry::PTEntry> {
        let page_table = tmp_page::map::<super::PageTable>(self.0);

        let mask = super::PAGE_TABLE_ENTRIES - 1;
        let index = vaddr.as_usize() >> self.1 & mask;
        crate::sync::MappedLockGuard::map(page_table, |page_table| &mut page_table[index])
    }
}

impl NestedPageTable for AddressSpace {
    type PageSize = PageSize;
    type Level = PageTableLevel;

    fn top_level(&self) -> Self::Level {
        self.0.clone()
    }
}

impl NestedPageTableLevel for PageTableLevel {
    type PageSize = PageSize;

    fn region_size(&self) -> usize {
        1 << self.1
    }

    fn new_sublevel(&self, alloc: &impl PageAllocatorTrait<Self::PageSize>) -> Option<Self> {
        let addr = alloc.alloc(PageSize::Size4K)?;
        let mut page_table = tmp_page::map::<super::PageTable>(addr);

        // Clear the page table
        for index in 0..super::PAGE_TABLE_ENTRIES {
            page_table[index] = entry::PTEntry::NULL;
        }

        Some(PageTableLevel(addr, self.1 - super::PAGE_LEVEL_BITS))
    }

    fn free_sublevel(
        &self,
        sublevel: Self,
        alloc: &impl PageAllocatorTrait<Self::PageSize>,
    ) -> MappingResult<()> {
        alloc.free(sublevel.0, PageSize::Size4K);
        Ok(())
    }

    fn set_entry(
        &self,
        vaddr: VirtAddr,
        new_entry: crate::memory::address_space::nested_page_table::PageTableEntry<Self>,
    ) -> MappingResult<()> {
        if matches!(new_entry, if_entry::PageTableEntry::Page(_, _)) {
            debug_assert!(vaddr.is_aligned(1usize << self.1));
        }

        let mut entry = self.lock_entry(vaddr);
        if self.1 > 12 && entry.flags().contains(entry::PTEFlags::PS) {
            return Err(MappingError::MappingOver(entry.address()));
        }

        *entry = match new_entry {
            if_entry::PageTableEntry::Level(level) => entry::PTEntry::new_page_table(level.0),
            if_entry::PageTableEntry::Page(paddr, flags) => {
                entry::PTEntry::new_page(paddr, self.page_size().unwrap(), flags.into())
            }
        };

        // TODO: Check if this page table is currently active
        super::flush_tlb(vaddr);
        Ok(())
    }

    fn get_entry(&self, vaddr: VirtAddr) -> MappingResult<if_entry::PageTableEntry<Self>> {
        let entry = self.lock_entry(vaddr);
        if entry.flags().contains(entry::PTEFlags::P)
            && self.1 > 12
            && !entry.flags().contains(entry::PTEFlags::PS)
        {
            Ok(if_entry::PageTableEntry::Level(PageTableLevel(
                entry.address(),
                self.1 - super::PAGE_LEVEL_BITS,
            )))
        } else {
            Ok(if_entry::PageTableEntry::Page(
                entry.address(),
                entry.flags().into(),
            ))
        }
    }
}

impl AddressSpaceTrait<PageSize> for AddressSpace {
    fn map_alloc(
        &self,
        vaddr: VirtAddr,
        size: usize,
        flags: if_entry::MappingFlags,
        alloc: &impl PageAllocatorTrait<PageSize>,
    ) -> MappingResult<VirtAddr> {
        <Self as NestedPageTable>::map_alloc(self, vaddr, size, flags, alloc)
    }

    fn unmap_free(
        &self,
        vaddr: VirtAddr,
        size: usize,
        alloc: &impl PageAllocatorTrait<PageSize>,
    ) -> MappingResult<()> {
        <Self as NestedPageTable>::unmap_free(self, vaddr, size, alloc)
    }
}
