use super::*;
use crate::memory::*;

/// Address space struct
pub struct AddressSpace(pub PhysAddr);

impl AddressSpace {
    /// Map the page table layer and get the page table entry associated with this address
    fn get_entry(
        layer: &<Self as AddressSpaceTrait>::Layer,
        vaddr: VirtAddr,
    ) -> crate::sync::MappedLockGuard<PTEntry> {
        let page_table = tmp_page::map::<PageTable>(layer.0);

        let mask = PAGE_TABLE_ENTRIES - 1;
        let index = vaddr.as_usize() >> layer.1 & mask;
        crate::sync::MappedLockGuard::map(page_table, |page_table| &mut page_table[index])
    }
}

impl AddressSpaceTrait for AddressSpace {
    type Layer = (PhysAddr, usize);

    fn page_size(layer: &Self::Layer) -> usize {
        1 << layer.1
    }

    fn set_entry(
        layer: Self::Layer,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        page_size: crate::arch::paging::PageSize,
        flags: MappingFlags,
    ) -> MappingResult<()> {
        debug_assert_eq!(1usize << layer.1, page_size as usize);
        let mut entry = Self::get_entry(&layer, vaddr);
        if entry.flags().contains(PTEFlags::P) {
            return Err(MappingError::MappingOver(entry.address()));
        }
        *entry = PTEntry::new_page(paddr, page_size, flags.into());
        flush_tlb(vaddr);
        Ok(())
    }

    fn unset_entry(
        layer: Self::Layer,
        vaddr: VirtAddr,
        page_size: crate::arch::paging::PageSize,
    ) -> MappingResult<()> {
        debug_assert_eq!(1usize << layer.1, page_size as usize);
        let mut entry = Self::get_entry(&layer, vaddr);
        if !entry.flags().contains(PTEFlags::P) {
            return Err(MappingError::UnmappingNotMapped(vaddr));
        }
        *entry = PTEntry::NULL;
        flush_tlb(vaddr);
        Ok(())
    }

    fn next_layer(layer: Self::Layer, vaddr: VirtAddr, map: bool) -> MappingResult<Self::Layer> {
        let entry = Self::get_entry(&layer, vaddr);

        if entry.flags().contains(PTEFlags::P | PTEFlags::PS) {
            if map {
                return Err(MappingError::MappingOver(entry.address()));
            } else {
                return Err(MappingError::UnmappingPartOfLargePage(entry.address()));
            }
        }

        let entry = if !entry.flags().contains(PTEFlags::P) {
            drop(entry);
            if !map {
                return Err(MappingError::UnmappingNotMapped(vaddr));
            }

            // Create a new page table
            let page_table_addr = crate::memory::alloc_page(PageSize::Size4K as _);
            let mut page_table = tmp_page::map::<PageTable>(page_table_addr);

            // Clear the page table
            for index in 0..PAGE_TABLE_ENTRIES {
                page_table[index] = PTEntry::NULL;
            }

            drop(page_table);

            // Set the entry to this page table
            let mut entry = Self::get_entry(&layer, vaddr);
            *entry = PTEntry::new_page_table(page_table_addr);
            entry
        } else {
            entry
        };

        Ok((entry.address(), layer.1 - PAGE_LEVEL_BITS))
    }

    fn top_layer(&self) -> Self::Layer {
        #[cfg(target_arch = "x86")]
        return (self.0, 22);
        #[cfg(target_arch = "x86_64")]
        return (self.0, 39);
    }

    /// Decrement reference count of all pages related to this one
    fn free_page(&self, layer: &Self::Layer, vaddr: VirtAddr) -> MappingResult<()> {
        let mut entry = Self::get_entry(&layer, vaddr);
        if !entry.flags().contains(PTEFlags::P) {
            return Ok(());
        }

        if !entry.flags().contains(PTEFlags::PS) && page_info(entry.address()).uses() {
            for page in 0..Self::page_size(layer) / PageSize::min() as usize {
                //
            }
        }
        free_page(
            entry.address(),
            PageSize::from_usize(Self::page_size(layer)).unwrap(),
        );
        *entry = PTEntry::NULL;
        flush_tlb(vaddr);
        Ok(())
    }
}
