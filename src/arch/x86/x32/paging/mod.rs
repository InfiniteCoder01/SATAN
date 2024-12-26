use x86::current::paging as x86_paging;
use x86_paging::{PAddr, VAddr};

const KERNEL_OFFSET: usize = 0xC0000000;
pub fn ks_phys_addr(vaddr: VAddr) -> PAddr {
    PAddr(vaddr.0 - KERNEL_OFFSET as u32)
}

macro_rules! linker_symbol {
    ($($name: ident ($symbol_name: ident) => $link_name: literal;)*) => {
        $(
            extern "C" {
                #[link_name = $link_name]
                static $symbol_name: ();
            }

            fn $name() -> VAddr {
                VAddr(unsafe { &$symbol_name } as *const () as _)
            }
        )*
    };
}

linker_symbol! {
    kernel_start(KERNEL_START_SYMBOL) => "kernel_start";
    kernel_end(KERNEL_END_SYMBOL) => "kernel_end";
}

mod mmap;
pub use mmap::mmap;

mod palloc;
pub use palloc::alloc_page;

#[link_section = ".bootstrap"]
pub(super) fn setup_page_info_table(boot_info: &multiboot2::BootInformation) {
    let Some(mmap_tag) = boot_info.memory_map_tag() else {
        super::early_logger::early_panic("No mmap tag provided by multiboot2!");
    };

    // Compute total pages
    let mut address_limit = 0;
    for area in mmap_tag.memory_areas() {
        address_limit = address_limit.max(area.end_address());
    }
    unsafe {
        palloc::TOTAL_PAGES = (address_limit >> x86_paging::BASE_PAGE_SHIFT as u64) as _;
    }
    let page_info_table_size =
        unsafe { palloc::TOTAL_PAGES } * core::mem::size_of::<palloc::PageInfo>();

    // Initialize page addresses
    unsafe {
        palloc::PAGE_INFO_TABLE = kernel_end().align_up_to_base_page().0 as _;
        palloc::EARLY_PAGE_ALLOC_START = (ks_phys_addr(kernel_end().align_up_to_base_page())
            + page_info_table_size)
            .align_up_to_base_page();
    }
    super::early_logger::early_print("early\n");
    mmap(
        VAddr(unsafe { palloc::PAGE_INFO_TABLE } as _),
        ks_phys_addr(kernel_end().align_up_to_base_page()),
        page_info_table_size,
        x86_paging::PTFlags::P | x86_paging::PTFlags::RW,
    );
    super::early_logger::early_print("mmapped\n");
    unsafe {
        (*palloc::PAGE_INFO_TABLE.offset(15))
            .uses
            .store(42, core::sync::atomic::Ordering::Relaxed);
    }
    super::early_logger::early_print("stored\n");
    unsafe {
        super::early_logger::early_print(&format!(
            "{}",
            (*palloc::PAGE_INFO_TABLE.offset(15))
                .uses
                .load(core::sync::atomic::Ordering::Relaxed)
        ));
    }
    super::early_logger::early_print("loaded\n");
    loop {}

    // super::early_logger::early_print(&format!("{}\n", page_info_table_size));
    // // Map info table
    // mmap(kernel_end, physical_kernel_end, page_info_table_size);
    // // Map enough pages for the page info table
    // let page_info_table_end = kernel_end + page_info_table_size;

    // let mut kernel_mapped_pages_end = kernel_end.align_up_to_base_page();
    // let mut kernel_mapped_page_tables_end = kernel_end.align_up_to_large_page();
    // let mut kernel_physical_pages_end = PAddr(kernel_end.0 - KERNEL_OFFSET as u32);
    // while kernel_mapped_pages_end < page_info_table_end {
    //     if kernel_mapped_page_tables_end <= kernel_mapped_pages_end {
    //         PD.
    //     }
    // }
    // unsafe { kernel_end }.align_up_to_base_page().0 as usize - KERNEL_OFFSET;
    // Memory map more pages
    // while TOTAL_PAGES kernel_address_space_end

    // PAGE_INFO_TABLE = kernel_end.0 as _;
    // let total_pages = P
    // PAGE_INFO_TABLE.offset();
}
