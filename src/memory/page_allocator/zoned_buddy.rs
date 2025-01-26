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

    pub fn add_zone(&self, start: usize, size: usize) -> Result<(), ()> {
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
                buddy: lock_free_buddy_allocator::buddy_alloc::BuddyAlloc::new(
                    start,
                    size / BLOCK_SIZE,
                    &alloc::alloc::Global,
                )
                .ok_or(())?,
            });
        }
        Ok(())
    }

    fn alloc(&self, size: usize) -> Option<PhysAddr> {
        let blocks = size / BLOCK_SIZE;
        for zone in self.zones.read().iter() {
            if let Some(addr) = zone.buddy.alloc(blocks) {
                return Some(PhysAddr::from_usize(addr));
            }
        }
        None
    }

    fn free(&self, allocation: PhysAddr, size: usize) {
        let start = allocation.as_usize();
        let blocks = size / BLOCK_SIZE;
        for zone in self.zones.read().iter() {
            if start > zone.start && start + size < zone.start + zone.size {
                zone.buddy.free(allocation.as_usize(), blocks);
            }
        }
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
