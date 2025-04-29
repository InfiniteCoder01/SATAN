use super::*;

linker_symbol! {
    address(TMP_PAGE_SYMBOL) => "kernel_tmp_page_address";
}

extern "C" {
    #[link_name = "kernel_tmp_page_entry_address"]
    static TMP_PAGE_ENTRY: *mut PTEntry;
}

static TMP_PAGE_MUTEX: crate::sync::Lock = crate::sync::Lock::new(());

/// Map a physical address to the TMP page. Returns virtual address of the TMP page
pub(super) fn map<T>(addr: PhysAddr) -> crate::sync::MappedLockGuard<T> {
    // TODO: Make it lock-free for multiple CPUs
    debug_assert!(
        core::mem::size_of::<T>() <= memory_addr::PAGE_SIZE_4K,
        "TMP page is mapped with a type bigger than one page"
    );
    debug_assert!(
        addr.is_aligned_4k(),
        "mapping an unaligned address {:?} to tmp page",
        addr
    );

    crate::sync::LockGuard::map(TMP_PAGE_MUTEX.lock(), |_| {
        let entry = PTEntry::new_page(addr, PageSize::Size4K, PTEFlags::P | PTEFlags::RW);
        unsafe {
            if *TMP_PAGE_ENTRY != entry {
                *TMP_PAGE_ENTRY = entry;
                flush_tlb(address());
            }
        }
        unsafe { &mut *address().as_mut_ptr_of() }
    })
}
