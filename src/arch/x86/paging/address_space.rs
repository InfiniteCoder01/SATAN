use super::*;

/// Address space struct
pub struct AddressSpace(pub PhysAddr);

impl AddressSpace {
    /// Map the page table layer and get the page table entry associated with this address
    fn get_entry(
        layer: &<Self as AddressSpaceTrait>::Layer,
        vaddr: VirtAddr,
    ) -> &'static mut PTEntry {
        let page_table = tmp_page::map(layer.0);
        let page_table = page_table.as_mut_ptr_of::<PageTable>();

        let mask = PAGE_TABLE_ENTRIES - 1;

        let index = vaddr.as_usize() >> layer.1 & mask;
        unsafe { &mut (*page_table)[index] }
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
        let entry = Self::get_entry(&layer, vaddr);
        if entry.flags().contains(PTEFlags::P) {
            return Err(MappingError::MappingOver(entry.address()));
        }
        crate::println!(
            "Setting an entry in PT {:?} w/ page size {:#x}",
            layer.0,
            page_size as usize,
        );
        *entry = PTEntry::new_page(paddr, page_size, flags.into());
        Ok(())
    }

    fn next_layer(layer: Self::Layer, vaddr: VirtAddr) -> MappingResult<Self::Layer> {
        let entry = Self::get_entry(&layer, vaddr);

        if entry.flags().contains(PTEFlags::P | PTEFlags::PS) {
            return Err(MappingError::MappingOver(entry.address()));
        }

        let entry = if !entry.flags().contains(PTEFlags::P) {
            // Create a new page table
            let page_table_addr = page_alloc::alloc_page(PageSize::Size4K as _);
            let page_table = tmp_page::map(page_table_addr);
            let page_table = page_table.as_mut_ptr_of::<PageTable>();

            // Clear the page table
            for index in 0..PAGE_TABLE_ENTRIES {
                unsafe { (*page_table)[index] = PTEntry::NULL };
            }

            // Set the entry to this page table
            crate::println!("Setting an entry in PT {:?} to a page table", layer.0);
            let entry = Self::get_entry(&layer, vaddr);
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
}
