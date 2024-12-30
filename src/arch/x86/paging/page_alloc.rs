use super::*;

static mut EARLY_PAGE_ALLOC_ADDRESS: PhysAddr = PhysAddr::from_usize(0);

linker_symbol! {
    kernel_early_alloc_start(KERNEL_EARLY_ALLOC_START) => "kernel_tmp_page_address";
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
        EARLY_PAGE_ALLOC_ADDRESS = kernel_virt2phys(kernel_early_alloc_start());
    }

    let kernel_address_space = VirtAddr::from_usize(&raw const KERNEL_TOP_LEVEL_PAGE_TABLE as _);
    let kernel_address_space = AddressSpace(kernel_virt2phys(kernel_address_space));

    let test_r = 0xc03ff000 as *mut u32;
    let test_w = 0xc03fe000 as *mut u32;
    kernel_address_space
        .map_page(
            kernel_address_space.top_layer(),
            VirtAddr::from_mut_ptr_of(test_r),
            PhysAddr::from_usize(0x800000),
            PageSize::Size4K,
            MappingFlags::PRESENT | MappingFlags::READ,
        )
        .unwrap();
    kernel_address_space
        .map_page(
            kernel_address_space.top_layer(),
            VirtAddr::from_mut_ptr_of(test_w),
            PhysAddr::from_usize(0x800000),
            PageSize::Size4K,
            MappingFlags::PRESENT | MappingFlags::WRITE,
        )
        .unwrap();
    crate::println!("Mapped!");
    unsafe {
        *test_w = 42;
    };
    crate::println!("Wrote!");
    crate::println!("Testing page mapping: {}", unsafe { *test_r });
    kernel_address_space
        .unmap_page(
            kernel_address_space.top_layer(),
            VirtAddr::from_mut_ptr_of(test_r),
            PageSize::Size4K,
        )
        .unwrap();
    crate::println!("Testing page unmapping (you should see a page fault)...");
    crate::println!("If you see this everything broke: {}", unsafe { *test_r });
}
