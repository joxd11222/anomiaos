#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod file_system;
mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut w = vga_buffer::Writer {
        row_position: 0,
        column_position: 0,
        color_code: vga_buffer::ColorCode::new(vga_buffer::Color::Red, vga_buffer::Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut vga_buffer::Buffer) },
    };
    w.clear_screen();
    w.write_string("!!! PANIC !!!\n");
    if let Some(location) = info.location() {
        w.write_string("panic at ");
        w.write_string(location.file());
        w.write_string(":");
        let mut buf = [0u8; 20];
        w.write_string(&vga_buffer::int_to_string(location.line() as usize, &mut buf));
        w.write_string("\n");
    } else {
        w.write_string("panic location unknown.\n");
    }
    let msg = info.message();
    w.write_string("panic message: ");
    w.write_string(msg.as_str().unwrap_or("no known message"));
    w.write_string("\n");
    loop {}
}

fn read_scancode() -> u8 {
    loop {
        let mut status: u8;
        unsafe { core::arch::asm!("in al, 0x64", out("al") status, options(nomem, nostack, preserves_flags)); }
        if status & 1 != 0 {
            let mut sc: u8;
            unsafe { core::arch::asm!("in al, 0x60", out("al") sc, options(nomem, nostack, preserves_flags)); }
            return sc;
        }
    }
}

fn read_key() -> u8 {
    loop {
        let sc = read_scancode();
        if sc > 0 && sc < 0x80 { 

            loop {

                if read_scancode() == sc | 0x80 {
                    break;
                }
            }
            return sc;
        }
    }
}

fn scancode_to_char(sc: u8) -> Option<char> {
    match sc {
        0x02..=0x0B => Some("1234567890".as_bytes()[sc as usize - 0x02] as char),
        0x10..=0x19 => Some("qwertyuiop".as_bytes()[sc as usize - 0x10] as char),
        0x1E..=0x26 => Some("asdfghjkl".as_bytes()[sc as usize - 0x1E] as char),
        0x2C..=0x32 => Some("zxcvbnm".as_bytes()[sc as usize - 0x2C] as char),
        0x39 => Some(' '), 0x34 => Some('.'),
        _ => None,
    }
}

fn read_line<'a>(writer: &mut vga_buffer::Writer, buffer: &'a mut [u8]) -> &'a str {
    let mut i = 0;
    loop {
        let sc = read_key();
        match sc {
            0x1C => { 
                writer.write_byte(b'\n');
                break;
            }
            0x0E => { 
                if i > 0 {
                    i -= 1;

                    if writer.column_position > 0 {
                        writer.column_position -= 1;
                        writer.write_byte(b' ');
                        writer.column_position -= 1;
                    }
                }
            }
            _ => {
                if i < buffer.len() - 1 {
                    if let Some(c) = scancode_to_char(sc) {
                        buffer[i] = c as u8;
                        writer.write_byte(c as u8);
                        i += 1;
                    }
                }
            }
        }
    }
    buffer[i] = 0; 
    unsafe { core::str::from_utf8_unchecked(&buffer[0..i]) }
}

fn parse_command<'a>(input: &'a str) -> (&'a str, Option<&'a str>) {
    let mut parts = input.trim().splitn(2, ' ');
    let command = parts.next().unwrap_or("");
    let arg = parts.next();
    (command, arg)
}

fn cmd_help(writer: &mut vga_buffer::Writer) {
    writer.write_string("Anomia OS Commands:\n");
    writer.write_string("  ls, dir         - List files\n");
    writer.write_string("  cat <file>      - Display file content\n");
    writer.write_string("  nano <file>     - Simple text editor\n");
    writer.write_string("  write <file>    - Create/overwrite a file with one line of text\n");
    writer.write_string("  rm, del <file>  - Delete a file\n");
    writer.write_string("  tests           - Run system diagnostics\n");
    writer.write_string("  date            - Shows the current date and time\n");
    writer.write_string("  clear           - Clear the screen\n");
    writer.write_string("  exit, reboot    - Halts the CPU\n");
}

fn cmd_ls(writer: &mut vga_buffer::Writer, fs: &file_system::OsFileSystem) {
    writer.write_string("Directory listing:\n");

    match fs.list_files() {
        Ok(Some(file_bytes)) => {
            if let Ok(file_str) = core::str::from_utf8(file_bytes) {
                writer.write_string("  - ");
                writer.write_string(file_str);
                writer.write_byte(b'\n');
            } else {
                writer.write_string("  - [Invalid UTF-8 filename]\n");
            }
        }
        Ok(None) => writer.write_string("  (No files found)\n"),
        Err(_) => writer.write_string("  (Error reading directory)\n"),
    }
}

fn cmd_cat(writer: &mut vga_buffer::Writer, fs: &file_system::OsFileSystem, filename: Option<&str>) {
    if let Some(name) = filename {
        match fs.read_file(name) {
            Ok(data) => {
                for &byte in data {
                    writer.write_byte(byte);
                }
                writer.write_byte(b'\n');
            },
            Err(_) => writer.write_string("Error: File not found.\n"),
        }
    } else {
        writer.write_string("Usage: cat <filename>\n");
    }
}

fn cmd_rm(writer: &mut vga_buffer::Writer, fs: &mut file_system::OsFileSystem, filename: Option<&str>) {
    if let Some(name) = filename {
        match fs.delete_file(name) {
            Ok(_) => {
                writer.write_string("File '");
                writer.write_string(name);
                writer.write_string("' deleted.\n");
            },
            Err(_) => writer.write_string("Error: File could not be deleted.\n"),
        }
    } else {
        writer.write_string("Usage: rm <filename>\n");
    }
}

fn cmd_nano(writer: &mut vga_buffer::Writer, fs: &mut file_system::OsFileSystem, filename: Option<&str>) {
    let filename_str = if let Some(name) = filename {
        name
    } else {
        writer.write_string("Usage: nano <filename>\n");
        return;
    };

    writer.clear_screen();
    writer.write_string("Anomia Editor - ");
    writer.write_string(filename_str);
    writer.write_string(" (Press ESC to save and exit)\n");
    writer.write_string("--------------------------------------------------\n");

    let mut content_buf = [0u8; 4096];
    let mut content_len = 0;

    if let Ok(data) = fs.read_file(filename_str) {
        let len = data.len().min(content_buf.len());
        content_buf[..len].copy_from_slice(&data[..len]);
        content_len = len;
        for &byte in &content_buf[..content_len] { writer.write_byte(byte); }
    }

    loop {
        let sc = read_key();
        match sc {
            0x01 => break, 
            0x1C => { 
                if content_len < content_buf.len() {
                    content_buf[content_len] = b'\n';
                    content_len += 1;
                    writer.write_byte(b'\n');
                }
            }
            0x0E => { 
                if content_len > 0 {
                    content_len -= 1;
                    if writer.column_position > 0 {
                        writer.column_position -= 1;
                        writer.write_byte(b' ');
                        writer.column_position -= 1;
                    }
                }
            }
            _ => {
                if content_len < content_buf.len() {
                    if let Some(c) = scancode_to_char(sc) {
                        content_buf[content_len] = c as u8;
                        content_len += 1;
                        writer.write_byte(c as u8);
                    }
                }
            }
        }
    }

    writer.write_string("\n--------------------------------------------------\nSaving... ");
    match fs.write_file(filename_str, &content_buf[..content_len]) {
        Ok(_) => writer.write_string("Done.\n"),
        Err(_) => writer.write_string("Failed!\n"),
    }
}

fn cmd_write(writer: &mut vga_buffer::Writer, fs: &mut file_system::OsFileSystem, filename: Option<&str>) {
    if let Some(name) = filename {
        writer.write_string("Enter text to write and press Enter:\n> ");
        let mut buffer = [0u8; 1024];
        let input = read_line(writer, &mut buffer);
        match fs.write_file(name, input.as_bytes()) {
             Ok(_) => writer.write_string("File written successfully.\n"),
             Err(_) => writer.write_string("Error: Could not write file.\n"),
        }
    } else {
        writer.write_string("Usage: write <filename>\n");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    let mut writer = vga_buffer::Writer {
        row_position: 0,
        column_position: 0,
        color_code: vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut vga_buffer::Buffer) },
    };

    let mut fs = file_system::new_os_file_system();
    writer.clear_screen();
    writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
    writer.write_string("==== WELCOME TO ANOMIA OS ====\n");
    writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
    writer.write_string("Type 'help' for a list of commands.\n\n");

    let mut command_buffer = [0u8; 256];

    loop {
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightGreen, vga_buffer::Color::Black);
        writer.write_string("anomia> ");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);

        let input = read_line(&mut writer, &mut command_buffer);
        let (command, arg) = parse_command(input);

        match command {
            "help" => cmd_help(&mut writer),
            "ls" | "dir" => cmd_ls(&mut writer, &fs),
            "cat" => cmd_cat(&mut writer, &fs, arg),
            "nano" => cmd_nano(&mut writer, &mut fs, arg),
            "write" => cmd_write(&mut writer, &mut fs, arg),
            "rm" | "del" => cmd_rm(&mut writer, &mut fs, arg),
            "clear" => writer.clear_screen(),
            "tests" => {
                 vga_buffer::color_test();
                 vga_buffer::ascii_test();
                 vga_buffer::math_test();
                 vga_buffer::file_system_test();
                 writer.write_string("System tests complete.\n");
            },
            "date" => writer.write_string("Current time: Sat, 27 Sep 2025 01:26 AM CEST\n"),
            "exit" | "reboot" => break,
            "" => {} 
            _ => {
                writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Red, vga_buffer::Color::Black);
                writer.write_string("Unknown command: '");
                writer.write_string(command);
                writer.write_string("'\n");
            }
        }
    }

    writer.clear_screen();
    writer.write_string("Shutting down Anomia OS. Goodbye!");

    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags)); }
    }
}
