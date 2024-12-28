use super::*;

bitflags::bitflags! {
    /// Page table entry flags (first byte from the right)
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

impl From<MappingFlags> for PTEFlags {
    fn from(value: MappingFlags) -> Self {
        let mut flags = Self::empty();
        if value.contains(MappingFlags::PRESENT) {
            flags |= Self::P;
        }
        if value.contains(MappingFlags::WRITE) {
            flags |= Self::RW;
        }
        #[cfg(target_arch = "x86_64")]
        if value.contains(MappingFlags::EXECUTE) {
            todo!()
        }
        if value.contains(MappingFlags::USER) {
            flags |= Self::US;
        }
        if value.contains(MappingFlags::UNCACHED) {
            flags |= Self::PCD;
        }
        flags
    }
}

/// Page table entry
#[repr(C, packed)]
pub(super) struct PTEntry(usize);

impl PTEntry {
    /// Create a new entry associated with a page
    pub(super) fn new_page(addr: PhysAddr, page_size: PageSize, flags: MappingFlags) -> Self {
        assert!(addr.is_aligned(page_size as usize));
        let mut flags = PTEFlags::from(flags);
        match page_size {
            #[cfg(target_arch = "x86")]
            PageSize::Size4M => flags |= PTEFlags::PS,
            #[cfg(target_arch = "x86_64")]
            PageSize::Size2M => flags |= PTEFlags::PS,
            #[cfg(target_arch = "x86_64")]
            PageSize::Size1G => flags |= PTEFlags::PS,
            _ => (),
        }
        Self(addr.as_usize() | flags.bits())
    }

    /// Create a new entry associated with a page table
    pub(super) fn new_page_table(addr: PhysAddr, flags: MappingFlags) {}

    /// Get flags of this page table entry
    pub(super) fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    /// Get the address this page table entry holds
    pub(super) fn address(&self) -> PhysAddr {
        PhysAddr::from_usize(self.0 & !0x000000ff)
    }
}
