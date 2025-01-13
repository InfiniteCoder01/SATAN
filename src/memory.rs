use core::sync::atomic::Ordering;
pub use memory_addr::{pa, va, va_range, MemoryAddr, PhysAddr, VirtAddr};

pub mod address_space;
pub use address_space::{AddressSpaceTrait, MappingError, MappingFlags, MappingResult};

/// Kernel page info entry
pub struct PageInfo {
    pub uses: core::sync::atomic::AtomicU32,
}

impl PageInfo {
    /// Reset page info to an unused page
    pub fn reset(&self) {
        self.uses.store(0, Ordering::SeqCst);
    }

    // /// Add one to uses count
    // pub fn r#use(&self) {
    //     self.uses.fetch_add(1, Ordering::Relaxed);
    // }

    // /// Unuse this page, pass in it's address. Will free if rc goes to zero
    // pub fn r#unuse(&self, addr: PhysAddr) {
    //     self.uses.fetch_sub(1, Ordering::Relaxed);
    // }

    pub fn acquire(&self) -> bool {
        self.uses
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }
}

pub static PAGE_INFO_TABLE: spin::RwLock<&[PageInfo]> = spin::RwLock::new(&[]);

/// Access the page info table
pub fn page_info_table() -> spin::RwLockReadGuard<'static, &'static [PageInfo]> {
    match PAGE_INFO_TABLE.try_read() {
        Some(guard) => guard,
        None => panic!("Tried to lock a locked mutex!"),
    }
}

/// Allocate page of size page_size aligned to page_size
pub fn alloc_page(page_size: crate::arch::paging::PageSize) -> PhysAddr {
    let page_info_table = page_info_table();
    if page_info_table.is_empty() {
        crate::arch::paging::early_alloc_page(page_size)
    } else {
        for (index, page_info) in page_info_table.iter().enumerate() {
            if page_info.acquire() {
                return PhysAddr::from_usize(index * crate::arch::paging::PageSize::min() as usize);
            }
        }
        todo!()
    }
}

/// Free page allocated with [alloc_page]
pub fn free_page(addr: PhysAddr, page_size: crate::arch::paging::PageSize) {
    let page_info_table = page_info_table();
    if page_info_table.is_empty() {
        panic!("Can't free page without page info table");
    }
    todo!()
}

/// Wrap a u64 in this struct to display it with size postfix (KiB, MiB, GiB, etc.)
pub struct FormatSize(pub u64);

impl core::ops::Deref for FormatSize {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for FormatSize {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl core::fmt::Display for FormatSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut value = self.0;
        let mut order = 0;
        let orders = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
        while value >= 1 << 10 && order + 1 < orders.len() {
            value >>= 10;
            order += 1;
        }

        if value >= 10 {
            write!(f, "{} {}", value, orders[order])
        } else {
            write!(
                f,
                "{}.{} {}",
                value,
                ((self.0 * 10) >> (order * 10)) % 10,
                orders[order]
            )
        }
    }
}
