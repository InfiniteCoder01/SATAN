const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER: usize = 0xb8000;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
struct Character {
    character: u8,
    color: u8,
}

impl Character {
    fn new(character: u8, color: u8) -> Self {
        Self { character, color }
    }
}

type Buffer = [[Character; VGA_WIDTH]; VGA_HEIGHT];
pub struct Writer {
    col: usize,
    color: u8,
    buffer: *mut Buffer,
}

impl Writer {
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = (bg as u8) << 4 | (fg as u8);
    }

    pub fn write(&mut self, ch: u8) {
        match ch {
            b'\n' => self.new_line(),
            b'\r' => self.col = 0,
            ch => {
                if self.col >= VGA_WIDTH {
                    self.new_line();
                }
                unsafe {
                    (*self.buffer)[VGA_HEIGHT - 1][self.col] = Character::new(ch, self.color);
                }
                self.col += 1;
            }
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                unsafe {
                    (*self.buffer)[row - 1][col] = (*self.buffer)[row][col];
                }
            }
        }
        self.clear_row(VGA_HEIGHT - 1);
        self.col = 0;
    }

    pub fn clear_row(&mut self, row: usize) {
        let blank = Character::new(b' ', self.color);
        for col in 0..VGA_WIDTH {
            unsafe {
                (*self.buffer)[row][col] = blank;
            }
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.chars() {
            self.write(ch as _);
        }
        Ok(())
    }
}

unsafe impl Send for Writer {}

pub static WRITER: spin::Mutex<Writer> = {
    spin::Mutex::new(Writer {
        col: 0,
        color: 0x0f,
        buffer: VGA_BUFFER as _,
    })
};

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write as _;
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn _panic(args: core::fmt::Arguments) -> ! {
    use core::fmt::Write as _;
    let mut writer = WRITER.lock();
    writer.set_color(Color::Red, Color::Black);
    writer.write_fmt(args).unwrap();
    crate::arch::interrupts::disable();
    #[allow(clippy::empty_loop)]
    loop {}
}
