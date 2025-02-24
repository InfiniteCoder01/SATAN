use core::alloc::AllocError;
use core::sync::atomic::AtomicUsize;

use super::{PageAllocatorTrait, PageSizeTrait, PhysAddr};
use crate::sync::RwLock;

struct CpuId;
impl lock_free_buddy_allocator::cpuid::Cpu for CpuId {
    fn current_cpu() -> usize {
        crate::arch::instructions::cpu_id()
    }
}

struct Zone<const PAGE_SIZE: usize> {
    start: usize,
    size: usize,
    allocated: AtomicUsize,
    buddy: lock_free_buddy_allocator::buddy_alloc::BuddyAlloc<
        'static,
        PAGE_SIZE,
        CpuId,
        alloc::alloc::Global,
    >,
}

/// Zone-based buddy allocator. Manages zones,
/// each zone having a separate binary buddy,
/// similar to how linux does this
pub struct ZonedBuddy<const BLOCK_SIZE: usize> {
    zones: RwLock<alloc::vec::Vec<Zone<BLOCK_SIZE>>>,
}

impl<const BLOCK_SIZE: usize> ZonedBuddy<BLOCK_SIZE> {
    pub const fn new() -> Self {
        Self {
            zones: RwLock::new(alloc::vec::Vec::new()),
        }
    }

    pub fn add_zone(&self, start: usize, size: usize) -> Result<(), AllocError> {
        debug_assert!(
            start % BLOCK_SIZE == 0,
            "zone is not aligned ({:#x})",
            start
        );
        debug_assert!(size % BLOCK_SIZE == 0, "size is not aligned ({:#x})", size);

        if !size.is_power_of_two() {
            let mut start = start;
            for bit in 0..usize::BITS {
                let size_p2 = 1 << bit;
                if size & size_p2 != 0 {
                    self.add_zone(start, size_p2)?;
                    start += size_p2;
                }
            }
        } else {
            self.zones.write().push(Zone {
                start,
                size,
                allocated: AtomicUsize::new(0),
                buddy: lock_free_buddy_allocator::buddy_alloc::BuddyAlloc::new(
                    start,
                    size / BLOCK_SIZE,
                    &alloc::alloc::Global,
                )
                .ok_or(AllocError)?,
            });
        }
        Ok(())
    }

    pub fn alloc(&self, size: usize) -> Option<PhysAddr> {
        let blocks = size / BLOCK_SIZE;
        for zone in self.zones.read().iter() {
            if let Some(addr) = zone.buddy.alloc(blocks) {
                zone.allocated
                    .fetch_add(size, core::sync::atomic::Ordering::SeqCst);
                return Some(PhysAddr::from_usize(addr));
            }
        }
        None
    }

    pub fn free(&self, allocation: PhysAddr, size: usize) {
        let start = allocation.as_usize();
        let blocks = size / BLOCK_SIZE;
        for zone in self.zones.read().iter() {
            if start >= zone.start && start + size <= zone.start + zone.size {
                zone.buddy.free(allocation.as_usize(), blocks);
                zone.allocated
                    .fetch_sub(size, core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    /// Returns total amount of memory managed by the allocator.
    /// To get free space, use [`Self::total_memory`] - [`Self::allocated_memory`]
    pub fn total_memory(&self) -> usize {
        self.zones
            .read()
            .iter()
            .fold(0, |acc, zone| acc + zone.size)
    }

    /// Returns the amount of allocated memory
    pub fn allocated_memory(&self) -> usize {
        self.zones.read().iter().fold(0, |acc, zone| {
            acc + zone.allocated.load(core::sync::atomic::Ordering::SeqCst)
        })
    }
}

impl<const BLOCK_SIZE: usize> Default for ZonedBuddy<BLOCK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BLOCK_SIZE: usize, PageSize: PageSizeTrait> PageAllocatorTrait<PageSize>
    for ZonedBuddy<BLOCK_SIZE>
{
    fn alloc(&self, size: PageSize) -> Option<PhysAddr> {
        self.alloc(size.into())
    }

    fn free(&self, allocation: PhysAddr, size: PageSize) {
        self.free(allocation, size.into())
    }
}
