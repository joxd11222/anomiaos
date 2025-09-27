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

pub fn hex_to_string(mut num: u32, buffer: &mut [u8]) -> &str {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let mut i = 0;
    if num == 0 {
        buffer[0] = b'0';
        return unsafe { core::str::from_utf8_unchecked(&buffer[0..1]) };
    }
    let mut temp_buf = [0u8; 8];
    let mut temp_len = 0;
    while num > 0 && temp_len < temp_buf.len() {
        temp_buf[temp_len] = HEX_CHARS[(num & 0xF) as usize];
        num >>= 4;
        temp_len += 1;
    }
    for j in 0..temp_len {
        if i < buffer.len() {
            buffer[i] = temp_buf[temp_len - 1 - j];
            i += 1;
        }
    }
    unsafe { core::str::from_utf8_unchecked(&buffer[0..i]) }
}

pub fn read_line(writer: &mut Writer, buffer: &mut [u8]) -> Result<usize, &'static str> {
    let mut i = 0usize;
    loop {
        let mut scancode: u8 = 0;
        unsafe {
            core::arch::asm!(
                "in al, 0x60",
                out("al") scancode,
                options(nomem, nostack, preserves_flags),
            );
        }
        if (scancode & 0x80) != 0 {
            continue;
        }
        if scancode == 0x1C {
            writer.write_string("\n");
            break;
        }
        if scancode == 0x0E {
            if i > 0 {
                i -= 1;

                writer.write_string("\u{8} \u{8}");
            }
            continue;
        }
        let c = match scancode {
            0x02 => b'1', 0x03 => b'2', 0x04 => b'3', 0x05 => b'4',
            0x06 => b'5', 0x07 => b'6', 0x08 => b'7', 0x09 => b'8',
            0x0A => b'9', 0x0B => b'0', 0x0C => b'-', 0x0D => b'=',
            0x10 => b'q', 0x11 => b'w', 0x12 => b'e', 0x13 => b'r',
            0x14 => b't', 0x15 => b'y', 0x16 => b'u', 0x17 => b'i',
            0x18 => b'o', 0x19 => b'p', 0x1A => b'[', 0x1B => b']',
            0x1E => b'a', 0x1F => b's', 0x20 => b'd', 0x21 => b'f',
            0x22 => b'g', 0x23 => b'h', 0x24 => b'j', 0x25 => b'k',
            0x26 => b'l', 0x27 => b';', 0x28 => b'\'', 0x29 => b'`',
            0x2C => b'z', 0x2D => b'x', 0x2E => b'c', 0x2F => b'v',
            0x30 => b'b', 0x31 => b'n', 0x32 => b'm', 0x33 => b',',
            0x34 => b'.', 0x35 => b'/', 0x39 => b' ', 
            _ => 0,
        };
        if c != 0 {
            if i < buffer.len() {
                buffer[i] = c;
                i += 1;
                writer.write_byte(c);
            } else {
            }
        }
    }
    Ok(i)
}

pub fn file_system_test() {
    let mut writer = Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_string("File System Test:\n");

    crate::file_system::with_fs_mut(|fs| {
        match fs.write_file("test.txt", b"Hello, World!") {
            Ok(_) => writer.write_string("✓ File write successful\n"),
            Err(_) => writer.write_string("✗ File write failed\n"),
        }
    });

    crate::file_system::with_fs(|fs| {
        match fs.read_file("test.txt") {
            Ok(data) => {
                writer.write_string("✓ File read successful: ");
                for &byte in data {
                    writer.write_byte(byte);
                }
                writer.write_string("\n");
            },
            Err(_) => writer.write_string("✗ File read failed\n"),
        }
    });

    crate::file_system::with_fs(|fs| {
        let files = fs.list_all_files();
        let mut count = 0;
        writer.write_string("✓ Files in system: ");
        for file_option in &files {
            if let Some(file_name) = file_option {
                if let Ok(name_str) = core::str::from_utf8(file_name) {
                    if count > 0 { writer.write_string(", "); }
                    writer.write_string(name_str);
                    count += 1;
                }
            }
        }
        if count == 0 {
            writer.write_string("(none)");
        }
        writer.write_string("\n");
    });

    crate::file_system::with_fs_mut(|fs| {
        match fs.delete_file("test.txt") {
            Ok(_) => writer.write_string("✓ File deletion successful\n"),
            Err(_) => writer.write_string("✗ File deletion failed\n"),
        }
    });

    writer.write_string("File system test completed.\n\n");
}
