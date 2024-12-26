use super::*;

pub(super) static mut PAGE_INFO_TABLE: *mut PageInfo = core::ptr::null_mut();
pub(super) static mut TOTAL_PAGES: usize = 0;
pub(super) struct PageInfo {
    pub(super) uses: core::sync::atomic::AtomicU32,
}

pub(super) static mut EARLY_PAGE_ALLOC_START: PAddr = PAddr::zero();
pub fn alloc_page() -> PAddr {
    unsafe {
        let page = EARLY_PAGE_ALLOC_START;
        EARLY_PAGE_ALLOC_START += x86_paging::BASE_PAGE_SIZE as u32;
        page
    }
}

pub fn free_page(page: PAddr) {
    todo!();
}
