#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("x32/bootstrap.S"), options(att_syntax));

pub mod early_logger;
pub mod interrupts;
pub mod paging;

#[no_mangle]
pub extern "cdecl" fn ksetup(mb_magic: u32, mbi_ptr: u32) -> ! {
    let boot_info = if mb_magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            multiboot2::BootInformation::load(mbi_ptr as *const multiboot2::BootInformationHeader)
        };
        match boot_info {
            Ok(boot_info) => boot_info,
            Err(err) => panic!(
                "Failed to parse multiboot2 info! How did you end up here?\n{}",
                err
            ),
        }
    } else {
        panic!(
            "Multiboot2 magic is invalid ({:#x}). How did you even get there?!",
            mb_magic
        );
    };

    crate::println!("Hello, SATAN!");
    interrupts::setup();
    paging::setup_info_table();
    // paging::setup_page_info_table(&boot_info);

    loop {}
}
