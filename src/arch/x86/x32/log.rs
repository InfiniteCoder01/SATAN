const VGA_WIDTH: u8 = 80;
const VGA_HEIGHT: u8 = 20;
const VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16;
static mut LINE: u8 = 0;
static mut COL: u8 = 0;

#[inline]
fn write(ch: u8, color: u8) {
    match ch {
        b'\n' => unsafe {
            COL = 0;
            LINE += 1;
        },
        b'\r' => unsafe {
            COL = 0;
        },
        ch => unsafe {
            *VGA_BUFFER.offset(LINE as isize * VGA_WIDTH as isize + COL as isize) =
                ch as u16 | (color as u16) << 8;
            COL += 1;
            if COL >= VGA_WIDTH {
                COL = 0;
                LINE += 1;
            }
        },
    }
}

#[inline]
pub fn early_panic(msg: &[u8]) -> ! {
    for ch in msg.into_iter().copied() {
        write(ch, 0x04);
    }
    loop {}
}

pub fn early_print(msg: &str) {
    for ch in msg.chars() {
        write(ch as _, 0x0f);
    }
}
