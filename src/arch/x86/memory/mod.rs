use memory_addr::{MemoryAddr, PhysAddr, VirtAddr};

/// Temproary page, space for it is allocated after the kernel in the kernel address space.
/// Used to map page tables and manipulate their entries
mod tmp_page;

mod page_size;
pub use page_size::PageSize;

/// Page table entry and it's flags
mod page_table_entry;
use page_table_entry::{PTEFlags, PTEntry};

/// Address space implementation
mod address_space;
pub use address_space::AddressSpace;

/// Use standard zone-based page allocator
pub type PageAllocator = crate::memory::page_allocator::ZonedBuddy<0x1000>;

extern "C" {
    #[link_name = "kernel_top_level_page_table"]
    static mut KERNEL_TOP_LEVEL_PAGE_TABLE: PageTable;
}

linker_symbol! {
    kernel_offset(KERNEL_OFFSET_SYMBOL) => "KERNEL_OFFSET";
    kernel_end(KERNEL_END) => "kernel_end";
    kernel_reserved_end(KERNEL_RESERVED_END) => "kernel_reserved_end";
}

/// Convert a physical address in the kernel address space to virtual by adding the offset
fn kernel_phys2virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr::from_usize(paddr.as_usize() + kernel_offset().as_usize())
}

/// Convert a virtual address in the kernel address space to physical by subtracting the offset
fn kernel_virt2phys(vaddr: VirtAddr) -> PhysAddr {
    PhysAddr::from_usize(vaddr.as_usize() - kernel_offset().as_usize())
}

#[cfg(target_arch = "x86")]
/// Number of bits each table takes off the vitual address
const PAGE_LEVEL_BITS: usize = 10;
#[cfg(target_arch = "x86_64")]
/// Number of bits each table takes off the vitual address
const PAGE_LEVEL_BITS: usize = 9;

/// Number of page table entries in a page table
const PAGE_TABLE_ENTRIES: usize = 1 << PAGE_LEVEL_BITS;

/// Page table type
type PageTable = [PTEntry; PAGE_TABLE_ENTRIES];

static PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

pub struct Memory;
impl crate::arch::MemoryTrait for Memory {
    type PageSize = PageSize;
    type PageAllocator = PageAllocator;
    type AddressSpace = AddressSpace;

    fn page_allocator() -> &'static Self::PageAllocator {
        &PAGE_ALLOCATOR
    }

    fn kernel_address_space() -> Self::AddressSpace {
        let kernel_address_space =
            VirtAddr::from_usize(&raw const KERNEL_TOP_LEVEL_PAGE_TABLE as _);
        AddressSpace::from_paddr(kernel_virt2phys(kernel_address_space))
    }
}

/// Setup paging
pub(super) fn setup_paging(boot_info: &multiboot2::BootInformation) {
    // Add zones to the page allocator
    let memory_map_tag = boot_info
        .memory_map_tag()
        .expect("Memory map not available");
    for region in memory_map_tag.memory_areas() {
        use multiboot2::MemoryAreaType;
        let typ = MemoryAreaType::from(region.typ());
        if typ == MemoryAreaType::Available {
            let kernel_physical_end = kernel_virt2phys(kernel_end());
            let start = PhysAddr::from_usize(region.start_address() as _);
            let start = start.max(kernel_physical_end).align_up_4k();
            let end = PhysAddr::from_usize(region.end_address() as _);
            if end <= start {
                continue;
            }

            if PAGE_ALLOCATOR
                .add_zone(start.as_usize(), memory_addr::align_down_4k(end - start))
                .is_err()
            {
                crate::println!("Failed to add memory zone {:#x} to {:#x}", start, end);
            }
        }
    }
}

macro_rules! linker_symbol {
    ($($name: ident ($symbol_name: ident) => $link_name: literal;)*) => {
        $(
            extern "C" {
                #[link_name = $link_name]
                static $symbol_name: u8;
            }

            fn $name() -> VirtAddr {
                VirtAddr::from_usize(unsafe { &$symbol_name } as *const _ as _)
            }
        )*
    };
}

use linker_symbol;
