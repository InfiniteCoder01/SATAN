#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("x32/bootstrap.S"), options(att_syntax));

/// Early logging facilities
pub mod early_logger;

/// Interrupts and IDT
pub mod interrupts;

/// Paging implementation
/// I spent a lot of time here
pub mod paging;

/// Kernel setup function. First thing that is called
/// after assembly bootstrap setus up GDT and higher-half address space
#[no_mangle]
pub extern "cdecl" fn ksetup(mb_magic: u32, mbi_ptr: u32) -> ! {
    // loop {}
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
    paging::setup_paging(&boot_info);

    loop {}
}
