core::arch::global_asm!(include_str!("multiboot2.asm"));

pub static HELLO: &[u8] = b"Hello World!";

extern "C" {
    fn testing(i: u8) -> u8;
}

pub fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    // unsafe {
    //     *vga_buffer = testing(5);
    // }

    loop {}
}
