core::arch::global_asm!(include_str!("bootstrap.S"));

#[inline]
fn early_panic(msg: &[u8]) -> ! {
    let vga_buffer = 0xb8000 as *mut u8;
    for (index, ch) in msg.into_iter().copied().enumerate() {
        unsafe {
            *vga_buffer.offset(index as isize * 2) = ch;
            *vga_buffer.offset(index as isize * 2 + 1) = 0x04;
        }
    }
    loop {}
}

#[link_section = ".bootstrap"]
#[no_mangle]
pub fn ksetup(mb_magic: u32, mbi_ptr: u32) -> ! {
    let boot_info = if mb_magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            multiboot2::BootInformation::load(mbi_ptr as *const multiboot2::BootInformationHeader)
        };
        match boot_info {
            Ok(boot_info) => boot_info,
            Err(_err) => early_panic(b"Failed to parse multiboot2 info! How did you end up here?"),
        }
    } else {
        early_panic(b"Multiboot2 magic is invalid. How did you even get there?!");
    };

    let vga_buffer = 0xb8000 as *mut u8;
    let mut i = 0;
    for (index, ch) in b"Hello, SATAN!".into_iter().copied().enumerate() {
        unsafe {
            *vga_buffer.offset(index as isize * 2) = ch;
            *vga_buffer.offset(index as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
