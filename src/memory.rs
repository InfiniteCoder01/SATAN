pub use memory_addr::{pa, va, va_range, MemoryAddr, PhysAddr, VirtAddr};

/// Address space implementations
pub mod address_space;
pub use address_space::{AddressSpaceTrait, MappingError, MappingFlags, MappingResult};

/// Different page allocator implementaitons
pub mod page_allocator;
pub use page_allocator::PageAllocatorTrait;

/// Page size trait, implement for an enum (or a struct) that could hold valid page sizes
pub trait PageSizeTrait: Copy + PartialEq + Eq + TryFrom<usize> + Into<usize> {
    const MIN: Self;
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
