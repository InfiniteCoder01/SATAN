use super::*;

linker_symbol! {
    address(TMP_PAGE_SYMBOL) => "kernel_tmp_page_address";
}

extern "C" {
    #[link_name = "kernel_tmp_page_entry_address"]
    static TMP_PAGE_ENTRY: *mut PTEntry;
}

/// Map a physical address to the TMP page. Returns virtual address of the TMP page
pub(super) fn map(addr: PhysAddr, flags: MappingFlags) -> VirtAddr {
    assert!(
        addr.is_aligned_4k(),
        "mapping an unaligned address {:?} to tmp page",
        addr
    );
    unsafe {
        *TMP_PAGE_ENTRY = PTEntry::new_page(addr, PageSize::Size4K, flags);
    }
    address()
}
