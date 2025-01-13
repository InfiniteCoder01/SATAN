use super::*;
use crate::memory::{AddressSpaceTrait, MappingFlags};

linker_symbol! {
    kernel_reserved_end(KERNEL_RESERVED_END) => "kernel_reserved_end";
}

// * Allocation
struct EarlyPageAllocator {
    alloc_start: PhysAddr,
    boot_info: &'static multiboot2::BootInformation<'static>,
}

impl EarlyPageAllocator {
    /// Get next possibly free region of memory to try to fit page. Returns none if page fits
    fn next_possibly_free(&self, addr: PhysAddr, page_size: PageSize) -> Option<PhysAddr> {
        let start = addr.as_usize();
        let end = addr.as_usize() + page_size as usize;

        // * Check for overlap
        // With kernel
        if addr < kernel_virt2phys(kernel_reserved_end()) {
            return Some(kernel_virt2phys(kernel_reserved_end()));
        }
        // With boot info structure
        if start < self.boot_info.end_address() && end > self.boot_info.start_address() {
            return Some(self.boot_info.end_address().into());
        }

        // * Check for memory region validity
        for region in self.boot_info.memory_map_tag().unwrap().memory_areas() {
            if start < region.end_address() as usize && end > region.start_address() as usize {
                use multiboot2::MemoryAreaType;
                match MemoryAreaType::from(region.typ()) {
                    MemoryAreaType::Available => (),
                    _ => return Some((region.end_address() as usize).into()),
                }
            }
        }
        None
    }
}

static EARLY_ALLOCATOR: spin::Mutex<Option<EarlyPageAllocator>> = spin::Mutex::new(None);

pub fn early_alloc_page(page_size: PageSize) -> PhysAddr {
    let mut allocator = crate::sync::lock_nb(&EARLY_ALLOCATOR);
    let Some(allocator) = allocator.as_mut() else {
        panic!("Early allocator not available")
    };
    let mut addr = allocator.alloc_start.align_up(page_size as usize);
    while let Some(next) = allocator.next_possibly_free(addr, page_size) {
        addr = next.align_up(page_size as usize);
    }
    allocator.alloc_start = addr + page_size as usize;
    addr
}

/// Setup page info table, responsible for page allocation
pub(super) fn setup_page_info_table(boot_info: &multiboot2::BootInformation) {
    let kernel_address_space = VirtAddr::from_usize(&raw const KERNEL_TOP_LEVEL_PAGE_TABLE as _);
    let kernel_address_space = AddressSpace(kernel_virt2phys(kernel_address_space));
    *crate::sync::lock_nb(&EARLY_ALLOCATOR) = Some(EarlyPageAllocator {
        alloc_start: kernel_virt2phys(kernel_reserved_end()),
        boot_info: unsafe {
            core::mem::transmute::<&multiboot2::BootInformation, &multiboot2::BootInformation>(
                boot_info,
            )
        },
    });

    // Get the address limit (last usable physical address)
    let mut address_limit = 0;
    let memory_map_tag = boot_info
        .memory_map_tag()
        .expect("Memory map not available");
    for region in memory_map_tag.memory_areas() {
        use multiboot2::MemoryAreaType;
        let typ = MemoryAreaType::from(region.typ());
        if typ == MemoryAreaType::Available {
            address_limit = address_limit.max(region.end_address());
        }
    }

    if address_limit > (usize::MAX as u64) {
        panic!(
            "Kernel address size can't handle {} of memory",
            crate::memory::FormatSize(address_limit)
        );
    }

    let page_info_table_entries = (address_limit / memory_addr::PAGE_SIZE_4K as u64) as usize;
    let page_info_table_size =
        page_info_table_entries * core::mem::size_of::<crate::memory::PageInfo>();

    // Allocate and map page info table
    let page_info_table_address = kernel_reserved_end().align_up_4k();
    let page_info_table_address = kernel_address_space
        .map_alloc(
            page_info_table_address,
            memory_addr::align_up_4k(page_info_table_size),
            MappingFlags::PRESENT | MappingFlags::READ | MappingFlags::WRITE | MappingFlags::GLOBAL,
        )
        .unwrap();

    let page_info_table: &[crate::memory::PageInfo] = unsafe {
        core::slice::from_raw_parts(page_info_table_address.as_ptr_of(), page_info_table_entries)
    };

    // Initialize page info table
    for entry in page_info_table {
        entry.reset();
    }

    *crate::memory::PAGE_INFO_TABLE.try_write().unwrap() = page_info_table;
    crate::sync::lock_nb(&EARLY_ALLOCATOR).take();

    // TEST
    let test_r = 0xc0801000 as *mut u32;
    let test_w = 0xc0800000 as *mut u32;
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
