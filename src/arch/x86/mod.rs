#[cfg(target_arch = "x86")]
core::arch::global_asm!(include_str!("x32/bootstrap.S"), options(att_syntax));

/// Early logging facilities
mod early_logger;

/// CPU Interface
mod cpu;

/// Interrupts and IDT
mod interrupts;

/// Paging implementation
/// I spent a lot of time here.
/// And I hate every single second of it.
mod memory;

#[cfg(feature = "kernel-tests")]
mod tests;

/// Arch implementation
pub struct Arch;
impl crate::arch::ArchTrait for Arch {
    type EarlyLogger = early_logger::EarlyLogger;
    type Cpu = cpu::Cpu;
    type Memory = memory::Memory;
}

/// Allocator
mod allocator {
    const SIZE: usize = 0x1000;
    static mut ARENA: [u8; SIZE] = [0; SIZE];

    // TODO: Use system allocator on OOM
    #[global_allocator]
    static ALLOCATOR: talc::Talck<spin::Mutex<()>, talc::ClaimOnOom> = talc::Talc::new(unsafe {
        // if we're in a hosted environment, the Rust runtime may allocate before
        // main() is called, so we need to initialize the arena automatically
        talc::ClaimOnOom::new(talc::Span::from_slice(core::ptr::addr_of_mut!(ARENA)))
    })
    .lock();
}

/// Kernel setup function. First thing that is called
/// after assembly bootstrap setus up GDT and higher-half address space
#[no_mangle]
pub extern "cdecl" fn ksetup(mb_magic: u32, mbi_ptr: u32) -> ! {
    crate::println!("Hello, SATAN!");
    interrupts::setup();

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

    memory::setup_paging(&boot_info);

    #[cfg(feature = "kernel-tests")]
    tests::run();

    loop {}
}
