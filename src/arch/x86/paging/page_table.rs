use super::*;

#[repr(usize)]
pub enum PageSize {
    Size4K = 12,
    #[cfg(target_arch = "x86")]
    Size4M = 22,
    #[cfg(target_arch = "x86_64")]
    Size2M = 21,
    #[cfg(target_arch = "x86_64")]
    Size1G = 30,
}

impl PageSize {
    pub(super) fn top_level() -> usize {
        #[cfg(target_arch = "x86")]
        return 32;
        #[cfg(target_arch = "x86_64")]
        return 64;
    }
}

#[cfg(target_arch = "x86")]
pub(super) const PAGE_TABLE_ENTRIES: usize = 1024;
#[cfg(target_arch = "x86_64")]
pub(super) const PAGE_TABLE_ENTRIES: usize = 512;

bitflags::bitflags! {
    /// PD configuration bits description.
    #[repr(transparent)]
    pub(super) struct PTEFlags: usize {
        /// Present
        const P       = 1 << 0;
        /// Read/write; if 0, writes may not be allowed
        const RW      = 1 << 1;
        /// User/supervisor; if 0, user-mode accesses are not allowed
        const US      = 1 << 2;
        /// Write-through
        const PWT     = 1 << 3;
        /// Cache disable
        const PCD     = 1 << 4;
        /// Page size; if set this entry maps a large/huge page; otherwise, this entry references a normal page/page table.
        const PS      = 1 << 7;
    }
}

pub(super) type PageTable = [PTEntry; PAGE_TABLE_ENTRIES];
#[repr(C, packed)]
pub(super) struct PTEntry(usize);

impl PTEntry {
    pub(super) fn new(addr: PhysAddr, flags: PTEFlags) -> Self {
        let mask = 0xfffff000;
        Self(addr.as_usize() & mask | flags.bits())
    }
}
