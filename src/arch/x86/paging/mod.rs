use crate::memory::*;

/// Temproary page, space for it is allocated after the kernel in the kernel address space.
/// Used to map page tables and manipulate their entries
mod tmp_page;

/// Page table entry and it's flags
mod page_table_entry;
use page_table_entry::{PTEFlags, PTEntry};

/// Page sizes possible to map
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(usize)]
pub enum PageSize {
    #[default]
    Size4K = 0x1000,
    #[cfg(target_arch = "x86")]
    Size4M = 0x400000,
    #[cfg(target_arch = "x86_64")]
    Size2M = 0x200000,
    #[cfg(target_arch = "x86_64")]
    Size1G = 0x40000000,
}

extern "C" {
    #[link_name = "kernel_top_level_page_table"]
    static mut KERNEL_TOP_LEVEL_PAGE_TABLE: PageTable;
}

linker_symbol! {
    kernel_offset(KERNEL_OFFSET_SYMBOL) => "KERNEL_OFFSET";
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
/// Number of page table entries in 32-bit x86 page table
pub(super) const PAGE_TABLE_ENTRIES: usize = 1024;
#[cfg(target_arch = "x86_64")]
/// Number of page table entries in x86_64 page table
pub(super) const PAGE_TABLE_ENTRIES: usize = 512;

/// Page table type
pub(super) type PageTable = [PTEntry; PAGE_TABLE_ENTRIES];

// -------------------------------- Memory mapping
/// Address space struct
pub struct AddressSpace(PhysAddr);

impl AddressSpace {
    /// Map the page table layer and get the page table entry associated with this address
    fn get_entry(
        layer: <Self as AddressSpaceTrait>::Layer,
        vaddr: VirtAddr,
    ) -> &'static mut PTEntry {
        let page_table = tmp_page::map(layer.0, MappingFlags::PRESENT | MappingFlags::WRITE);
        let page_table = page_table.as_mut_ptr_of::<PageTable>();

        let mask = PAGE_TABLE_ENTRIES - 1;

        let index = vaddr.as_usize() >> layer.1 & mask;
        unsafe { &mut (*page_table)[index] }
    }
}

impl AddressSpaceTrait for AddressSpace {
    type Layer = (PhysAddr, usize);

    fn page_size(layer: &Self::Layer) -> usize {
        1 << layer.1
    }

    fn set_entry(
        layer: Self::Layer,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        page_size: crate::arch::paging::PageSize,
        flags: MappingFlags,
    ) {
        assert!(vaddr.is_aligned(page_size as usize));
        assert!(paddr.is_aligned(page_size as usize));
        let entry = Self::get_entry(layer, vaddr);
        // TODO: Freeing entry
        *entry = PTEntry::new_page(paddr, page_size, flags);
    }

    fn next_layer(layer: Self::Layer, vaddr: VirtAddr) -> Self::Layer {
        let entry = Self::get_entry(layer, vaddr);
        if !entry.flags().contains(PTEFlags::P)
            || entry.flags().contains(PTEFlags::P | PTEFlags::PS)
        {
            // TODO: Freing the huge page and creating a new one
        }
        #[cfg(target_arch = "x86")]
        let bits = 10;
        #[cfg(target_arch = "x86_64")]
        let bits = 9;
        crate::println!("Selecting next layer with address of {:?}", entry.address());
        (entry.address(), layer.1 - bits)
    }

    fn top_layer(&self) -> Self::Layer {
        #[cfg(target_arch = "x86")]
        return (self.0, 22);
        #[cfg(target_arch = "x86_64")]
        return (self.0, 39);
    }
}

/// Setup page info table
pub(super) fn setup_info_table() {
    // Enable PSE
    unsafe {
        core::arch::asm!(
            "mov %cr4, {tmp}",
            "or $0x10, {tmp}",
            "mov {tmp}, %cr4",
            tmp = out(reg) _,
            options(att_syntax)
        );
    }

    let kernel_address_space = VirtAddr::from_usize(&raw const KERNEL_TOP_LEVEL_PAGE_TABLE as _);
    let kernel_address_space = AddressSpace(kernel_virt2phys(kernel_address_space));

    let test = 0x800000 as *mut u32;
    // let test = 0xC012d000 as *mut u32;
    kernel_address_space.map_page(
        kernel_address_space.top_layer(),
        VirtAddr::from_mut_ptr_of(test),
        PhysAddr::from_usize(0x800000),
        PageSize::Size4M,
        MappingFlags::PRESENT | MappingFlags::READ | MappingFlags::WRITE,
    );
    crate::println!("Hey?");
    unsafe {
        *test = 42;
    };
    crate::println!("Hey?");
    crate::println!("Testing page mapping: {}", unsafe { *test });
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
