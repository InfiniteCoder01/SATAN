use super::*;

static mut EARLY_PAGE_ALLOC_ADDRESS: PhysAddr = PhysAddr::from_usize(0);

linker_symbol! {
    kernel_end(KERNEL_END) => "kernel_end";
}

pub(super) fn alloc_page(page_size: usize) -> PhysAddr {
    unsafe {
        let addr = EARLY_PAGE_ALLOC_ADDRESS.align_up(page_size);
        EARLY_PAGE_ALLOC_ADDRESS = addr + page_size;
        addr
    }
}

pub(super) fn free_page(address: PhysAddr, page_size: usize) {
    todo!()
}

pub(super) fn setup_page_info_table(boot_info: &multiboot2::BootInformation) {
    unsafe {
        EARLY_PAGE_ALLOC_ADDRESS = kernel_virt2phys(kernel_end());
    }

    let kernel_address_space = VirtAddr::from_usize(&raw const KERNEL_TOP_LEVEL_PAGE_TABLE as _);
    let kernel_address_space = AddressSpace(kernel_virt2phys(kernel_address_space));

    // let test = 0x800000 as *mut u32;
    let test = 0xC0400000 as *mut u32;
    kernel_address_space
        .map_page(
            kernel_address_space.top_layer(),
            VirtAddr::from_mut_ptr_of(test),
            PhysAddr::from_usize(0x800000),
            PageSize::Size4K,
            MappingFlags::PRESENT | MappingFlags::READ | MappingFlags::WRITE,
        )
        .unwrap();
    crate::println!("Hey?");
    unsafe {
        *test = 42;
    };
    crate::println!("Hey?");
    crate::println!("Testing page mapping: {}", unsafe { *test });
}
