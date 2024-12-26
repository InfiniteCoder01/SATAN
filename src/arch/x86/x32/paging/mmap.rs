use super::*;
use x86_paging::{PDEntry, PDFlags, PTEntry, PTFlags};

linker_symbol! {
    tmp_page(TMP_PAGE_SYMBOL) => "kernel_tmp_page_address";
}

extern "C" {
    #[link_name = "kernel_page_directory"]
    static mut PD: x86_paging::PD;
    #[link_name = "kernel_tmp_page_entry_address"]
    static TMP_PAGE_ENTRY: *mut PTEntry;
}

fn mmap_tmp_page(addr: PAddr) -> VAddr {
    assert!(addr.is_base_page_aligned());
    unsafe {
        *TMP_PAGE_ENTRY = PTEntry::new(addr, PTFlags::P | PTFlags::RW);
    }
    tmp_page()
}

pub fn mmap_page(vaddr: VAddr, paddr: PAddr, flags: PTFlags) {
    let pd_entry = &mut unsafe { PD[x86_paging::pd_index(vaddr)] };
    let mut clear = false;
    if !pd_entry.is_present() {
        *pd_entry = PDEntry::new(alloc_page(), PDFlags::P | PDFlags::RW);
        clear = true;
    }

    let pt: *mut x86_paging::PT = mmap_tmp_page(pd_entry.address()).0 as _;
    if clear {
        for page in unsafe { *pt }.iter_mut() {
            *page = PTEntry::new(PAddr::zero(), PTFlags::empty());
        }
    }

    let pt_entry = &mut unsafe { *pt }[x86_paging::pt_index(vaddr)];
    if pt_entry.is_present() {
        palloc::free_page(pt_entry.address());
    }

    *pt_entry = PTEntry::new(paddr, flags);
}

pub fn mmap(vaddr: VAddr, paddr: PAddr, size: usize, flags: PTFlags) {
    assert!(vaddr.is_base_page_aligned());
    assert!(paddr.is_base_page_aligned());

    let last_page = VAddr((size - 1) as _).align_up_to_base_page().0 >> x86_paging::BASE_PAGE_SHIFT;
    for page in 0..=last_page {
        let offset = page << x86_paging::BASE_PAGE_SHIFT;
        mmap_page(vaddr + offset, paddr + offset, flags);
    }
}
