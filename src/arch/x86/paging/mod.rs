use crate::memory::AddressSpace as AddressSpaceTrait;
use crate::memory::*;

mod page_table;
mod tmp_page;

pub use page_table::PageSize;
use page_table::{PTEFlags, PTEntry, PageTable};

extern "C" {
    #[link_name = "kernel_top_level_page_table"]
    static mut KERNEL_TOP_LEVEL_PAGE_TABLE: PageTable;
}

linker_symbol! {
    kernel_offset(KERNEL_OFFSET_SYMBOL) => "KERNEL_OFFSET";
}

fn kernel_phys2virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr::from_usize(paddr.as_usize() + kernel_offset().as_usize())
}

fn kernel_virt2phys(vaddr: VirtAddr) -> PhysAddr {
    PhysAddr::from_usize(vaddr.as_usize() - kernel_offset().as_usize())
}

// -------------------------------- Memory mapping
fn map_page(
    page_table: PhysAddr,
    level_page_size: usize,
    vaddr: VirtAddr,
    paddr: PhysAddr,
    page_size: usize,
    flags: MappingFlags,
) {
    let page_table = tmp_page::map(page_table, PTEFlags::P | PTEFlags::RW);
    let page_table = page_table.as_mut_ptr_of::<PageTable>();
    // page_table
}

struct AddressSpace(PhysAddr);
impl AddressSpaceTrait for AddressSpace {
    #[inline]
    fn map_page(&self, vaddr: VirtAddr, paddr: PhysAddr, page_size: PageSize, flags: MappingFlags) {
        map_page(
            self.0,
            PageSize::top_level(),
            vaddr,
            paddr,
            page_size as _,
            flags,
        )
    }
}

pub(super) fn setup_info_table() {
    let kernel_address_space = VirtAddr::from_usize(&raw const KERNEL_TOP_LEVEL_PAGE_TABLE as _);
    let kernel_address_space = AddressSpace(kernel_virt2phys(kernel_address_space));

    let test = 0x800000 as *mut u32;
    kernel_address_space.map_page(
        VirtAddr::from_mut_ptr_of(test),
        PhysAddr::from_usize(0x800000),
        PageSize::Size4K,
        MappingFlags::PRESENT | MappingFlags::READ | MappingFlags::WRITE,
    );
    // unsafe {
    //     *test = 42;
    // };
    // crate::arch::early_logger::early_print(&format!("{}\n", unsafe { *test }));
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
