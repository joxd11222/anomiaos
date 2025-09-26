use core::str;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0, Blue = 1, Green = 2, Cyan = 3, Red = 4, Magenta = 5,
    Brown = 6, LightGray = 7, DarkGray = 8, LightBlue = 9,
    LightGreen = 10, LightCyan = 11, LightRed = 12, Pink = 13,
    Yellow = 14, White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);
impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {

    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub row_position: usize,
    pub column_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

pub fn int_to_string<'a>(mut n: usize, buf: &'a mut [u8]) -> &'a str {
    if n == 0 {
        buf[0] = b'0';
        return unsafe { str::from_utf8_unchecked(&buf[..1]) };
    }
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    buf[..i].reverse();
    unsafe { str::from_utf8_unchecked(&buf[..i]) }
}

pub fn wait_for_enter() {
    loop {
        let mut scancode: u8 = 0;
        unsafe {
            core::arch::asm!(
                "in al, 0x60",
                out("al") scancode,
                options(nomem, nostack, preserves_flags),
            );
        }
        if scancode == 0x1C {
            break;
        }
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = self.row_position;
                let col = self.column_position;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: b,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
    fn new_line(&mut self) {
        if self.row_position + 1 < BUFFER_HEIGHT {
            self.row_position += 1;
            self.column_position = 0;
            return;
        }
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let ch = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(ch);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
        self.row_position = BUFFER_HEIGHT - 1;
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.row_position = 0;
        self.column_position = 0;
    }
}

pub fn color_test() {
    let mut writer = Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    for bg in 0..=15 {
        for fg in 0..=15 {
            writer.color_code = ColorCode::new(
                unsafe { core::mem::transmute(fg as u8) },
                unsafe { core::mem::transmute(bg as u8) },
            );
            writer.write_string("X");
        }
        writer.write_string("\n");
    }
}

pub fn ascii_test() {
    let mut writer = Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    for c in 0u8..=255 {
        writer.write_byte(c);
    }
}

pub fn keyboard_test() {
    let mut writer = Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_string("Press keys to see their scancodes:\n");

    loop {
        let mut scancode: u8 = 0;
        unsafe {
            core::arch::asm!(
                "in al, 0x60",
                out("al") scancode,
                options(nomem, nostack, preserves_flags),
            );
        }
        writer.write_string("Scancode: ");
        let mut num_buf = [0u8; 20];
        let s = int_to_string(scancode as usize, &mut num_buf);
        writer.write_string(s);
        writer.write_string("\n");
        if scancode == 0x1C {
            break;
        }
    }
}

pub fn math_test() {
    let mut writer = Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::Cyan, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_string("Basic Math Tests:\n");

    let a = 5;
    let b = 3;

    writer.write_string("Addition: ");
    let sum = a + b;
    let mut num_buf = [0u8; 20];
    let s = int_to_string(sum, &mut num_buf);
    writer.write_string(s);
    writer.write_string("\n");

    writer.write_string("Subtraction: ");
    let diff = a - b;
    let s = int_to_string(diff, &mut num_buf);
    writer.write_string(s);
    writer.write_string("\n");

    writer.write_string("Multiplication: ");
    let prod = a * b;
    let s = int_to_string(prod, &mut num_buf);
    writer.write_string(s);
    writer.write_string("\n");

    writer.write_string("Division: ");
    let quot = a / b;
    let s = int_to_string(quot, &mut num_buf);
    writer.write_string(s);
    writer.write_string("\n");
}

pub fn panic_test() {
    panic!("This is a test panic!");
}
