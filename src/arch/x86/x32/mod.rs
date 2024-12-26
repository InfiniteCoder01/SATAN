core::arch::global_asm!(include_str!("bootstrap.S"), options(att_syntax));

pub mod early_logger;

// mod paging;
// pub(super) use paging::ks_phys_addr;
// pub use paging::mmap;
