core::arch::global_asm!(include_str!("bootstrap.S"), options(att_syntax));

pub mod log;
pub mod paging;

#[link_section = ".bootstrap"]
#[no_mangle]
pub fn ksetup(mb_magic: u32, mbi_ptr: u32) -> ! {
    let boot_info = if mb_magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            multiboot2::BootInformation::load(mbi_ptr as *const multiboot2::BootInformationHeader)
        };
        match boot_info {
            Ok(boot_info) => boot_info,
            Err(_err) => {
                log::early_panic(b"Failed to parse multiboot2 info! How did you end up here?")
            }
        }
    } else {
        log::early_panic(b"Multiboot2 magic is invalid. How did you even get there?!");
    };

    log::early_print("Hello, SATAN!");

    loop {}
}
