pub use memory_addr::{pa, va, va_range, MemoryAddr, PhysAddr, VirtAddr};

/// Temproary page, space for it is allocated after the kernel in the kernel address space.
/// Used to map page tables and manipulate their entries
mod tmp_page;

/// Page table entry and it's flags
mod page_table_entry;
use page_table_entry::{PTEFlags, PTEntry};

/// Address space implementation
mod address_space;
pub use address_space::AddressSpace;

/// Page allocator manages free pages
mod early_page_alloc;
pub use early_page_alloc::early_alloc_page;

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

impl PageSize {
    pub fn from_usize(size: usize) -> Option<Self> {
        match size {
            0x1000 => Some(Self::Size4K),
            #[cfg(target_arch = "x86")]
            0x400000 => Some(Self::Size4M),
            #[cfg(target_arch = "x86_64")]
            0x200000 => Some(Self::Size2M),
            #[cfg(target_arch = "x86_64")]
            0x40000000 => Some(Self::Size1G),
            _ => None,
        }
    }

    pub fn min() -> Self {
        Self::Size4K
    }
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
/// Number of bits each table takes off the vitual address
const PAGE_LEVEL_BITS: usize = 10;
#[cfg(target_arch = "x86_64")]
/// Number of bits each table takes off the vitual address
const PAGE_LEVEL_BITS: usize = 9;

/// Number of page table entries in a page table
const PAGE_TABLE_ENTRIES: usize = 1 << PAGE_LEVEL_BITS;

/// Page table type
type PageTable = [PTEntry; PAGE_TABLE_ENTRIES];

/// Flush transition lookaside buffer, required when an entry for virtual address was changed
fn flush_tlb(address: VirtAddr) {
    unsafe {
        core::arch::asm!(
            "invlpg ({address})",
            address = in(reg) address.as_usize(),
            options(att_syntax)
        );
    }
}

/// Setup paging
pub(super) fn setup_paging(boot_info: &multiboot2::BootInformation) {
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

    early_page_alloc::setup_page_info_table(boot_info);
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
