mod cursor;

extern crate spin;
extern crate x86;

use spin::Mutex;
use core::fmt;
use core;

const CONSOLE_COLS: isize = 80;
const CONSOLE_ROWS: isize = 25;

pub fn initialize() {
    clear_console();
    cursor::initialize();
}

pub fn clear_console() {
    let mut b = BUFFER.lock();
    b.clear();
}

#[allow(dead_code)]
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
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}


#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColorCode(u8);

const DEFAULT_COLOR: ColorCode = ColorCode::new(Color::LightGreen, Color::Black);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Copy,Clone)]
#[repr(C)]
struct VgaCell {
    character: u8,
    color: ColorCode,
}

pub static BUFFER: Mutex<VgaBuffer> = Mutex::new(VgaBuffer {
    buffer: [VgaCell {
        character: b' ',
        color: DEFAULT_COLOR,
    }; (CONSOLE_ROWS * CONSOLE_COLS) as usize],
    position: 0,
    color_code: DEFAULT_COLOR,
});

pub struct VgaBuffer {
    buffer: [VgaCell; (CONSOLE_ROWS * CONSOLE_COLS) as usize],
    position: usize,
    color_code: ColorCode,
}

impl VgaBuffer {
    pub fn flush(&self) {
        unsafe {
            let vga = 0xb8000 as *mut u8;
            let length = self.buffer.len() * 2;
            let buffer = self.buffer.as_ptr() as *const u8;
            core::ptr::copy_nonoverlapping(buffer, vga, length);
        }
    }

    pub fn change_color(&mut self, color: ColorCode) {
        self.color_code = color;
    }

    fn write_byte(&mut self, byte: u8) {
        if byte == ('\n' as u8) {
            // to get the current line, we divide by the length of a line
            let current_line = (self.position as isize) / CONSOLE_COLS;
            self.position = ((current_line + 1) * CONSOLE_COLS) as usize;
        } else {
            let cell = &mut self.buffer[self.position];

            *cell = VgaCell {
                character: byte,
                color: self.color_code,
            };
            self.position += 1;
        }

        if self.position >= self.buffer.len() {
            self.scroll_up();
        }

        cursor::set(self.position as u16);
    }


    fn scroll_up(&mut self) {
        let end = CONSOLE_ROWS * CONSOLE_COLS;

        for i in (CONSOLE_COLS+80)..(end) { // Added 80 to preserve top header
            let prev = i - CONSOLE_COLS;
            self.buffer[prev as usize] = self.buffer[i as usize];
        }

        // blank out the last row
        for i in (end - CONSOLE_COLS)..(end) {
            let cell = &mut self.buffer[i as usize];
            *cell = VgaCell {
                character: ' ' as u8,
                color: DEFAULT_COLOR,
            };
        }

        self.position = (end - CONSOLE_COLS) as usize;
    }

    fn reset_position(&mut self) {
        self.position = 0;
        cursor::set(0);
    }

    fn clear(&mut self) {
        for i in 0..(CONSOLE_ROWS * CONSOLE_COLS) {
            let cell = &mut self.buffer[i as usize];
            *cell = VgaCell {
                character: b' ',
                color: DEFAULT_COLOR,
            };
        }

        self.reset_position();
        self.flush();
    }
}

impl fmt::Write for VgaBuffer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! kprintln {
	($fmt:expr) => (kprint!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! kprint {
	($($arg:tt)*) => ({
		use core::fmt::Write;
		let mut b = $crate::vga::BUFFER.lock();
		b.write_fmt(format_args!($($arg)*)).unwrap();
		b.flush();
	});
}


#[allow(unused_must_use)]
pub unsafe fn print_error(fmt: fmt::Arguments) {
    use core::fmt::Write;

    let mut writer = VgaBuffer {
        buffer: [VgaCell {
            character: ' ' as u8,
            color: ColorCode::new(Color::LightGreen, Color::Red),
        }; (CONSOLE_ROWS * CONSOLE_COLS) as usize],
        position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Red),
    };
    writer.write_fmt(fmt);
    writer.flush();
}