use super::*;

linker_symbol! {
    address(TMP_PAGE_SYMBOL) => "kernel_tmp_page_address";
}

extern "C" {
    #[link_name = "kernel_tmp_page_entry_address"]
    static TMP_PAGE_ENTRY: *mut PTEntry;
}

pub(super) fn map(addr: PhysAddr, flags: PTEFlags) -> VirtAddr {
    assert!(addr.is_aligned_4k());
    unsafe {
        *TMP_PAGE_ENTRY = PTEntry::new(addr, flags);
    }
    address()
}
