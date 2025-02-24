use super::PageSizeTrait;
use memory_addr::PhysAddr;

pub mod zoned_buddy;
pub use zoned_buddy::ZonedBuddy;

pub trait PageAllocatorTrait<PageSize: PageSizeTrait> {
    fn alloc(&self, size: PageSize) -> Option<PhysAddr>;
    fn free(&self, allocation: PhysAddr, size: PageSize);
}
