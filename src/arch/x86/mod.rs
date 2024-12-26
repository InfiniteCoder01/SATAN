#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("x32/bootstrap.S"), options(att_syntax));

mod early_logger;
mod interrupts;

#[cfg(debug_assertions)]
mod allocator {
    const HEAP_SIZE: usize = 0x1000;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    use talc::*;
    #[global_allocator]
    static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
        // if we're in a hosted environment, the Rust runtime may allocate before
        // main() is called, so we need to initialize the arena automatically
        ClaimOnOom::new(Span::from_array(core::ptr::addr_of!(HEAP).cast_mut()))
    })
    .lock();
}

#[no_mangle]
pub extern "cdecl" fn ksetup(mb_magic: u32, mbi_ptr: u32) -> ! {
    let boot_info = if mb_magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            multiboot2::BootInformation::load(mbi_ptr as *const multiboot2::BootInformationHeader)
        };
        match boot_info {
            Ok(boot_info) => boot_info,
            Err(_err) => early_logger::early_panic(
                "Failed to parse multiboot2 info! How did you end up here?",
            ),
        }
    } else {
        early_logger::early_panic("Multiboot2 magic is invalid. How did you even get there?!");
    };

    early_logger::early_print("Hello, SATAN!\n");
    interrupts::setup();
    // paging::setup_page_info_table(&boot_info);

    loop {}
}
