use crate::arch::traits::*;

pub(super) fn run() -> ! {
    test_paging();
    panic!("Testing finished");
}

fn test_paging() {
    use crate::memory::*;
    let page_allocator = crate::arch::Memory::page_allocator();
    let kernel_address_space = crate::arch::Memory::kernel_address_space();

    crate::println!("Total memory: {}", page_allocator.total_memory());

    use crate::memory::MappingFlags;
    use crate::memory::{AddressSpaceTrait, PageSizeTrait};
    let test = 0xc0801000 as *mut u32;
    let test = kernel_address_space
        .map_alloc(
            VirtAddr::from_mut_ptr_of(test),
            crate::arch::x86::memory::PageSize::MIN as _,
            MappingFlags::PRESENT | MappingFlags::READ | MappingFlags::WRITE,
            page_allocator,
        )
        .unwrap()
        .as_mut_ptr_of();
    crate::println!("Mapped!");
    crate::println!("Allocated memory: {}", page_allocator.allocated_memory());
    unsafe {
        *test = 42;
    };
    crate::println!("Wrote!");
    crate::println!("Testing page mapping: {}", unsafe { *test });
    kernel_address_space
        .unmap_free(VirtAddr::from_mut_ptr_of(test), 4096, page_allocator)
        .unwrap();
    crate::println!(
        "Allocated memory after freeing: {}",
        page_allocator.allocated_memory()
    );
    crate::println!("Testing page unmapping (You should see a page fault):");
    crate::println!("Huh? {}", unsafe { *test });
}
