#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
use core::panic::PanicInfo;
mod file_system;
mod vga_buffer;
mod code_system;
mod syntax;
mod settings;

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
    settings::scancode_to_char(sc, false) 
}

fn read_line<'a>(writer: &mut vga_buffer::Writer, buffer: &'a mut [u8]) -> &'a str {
    let mut i = 0;
    let mut shift_pressed = false;

    loop {
        let sc = read_scancode();

        match sc {
            0x2A | 0x36 => { shift_pressed = true; continue; } 
            0xAA | 0xB6 => { shift_pressed = false; continue; } 
            _ => {}
        }

        if sc >= 0x80 { continue; }

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
            0x3A => { 
                let mut settings = settings::get_settings();
                settings.caps_lock_enabled = !settings.caps_lock_enabled;
                settings::set_settings(settings);
            }
            _ => {
                if i < buffer.len() - 1 {
                    if let Some(c) = settings::scancode_to_char(sc, shift_pressed) {
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

fn display_highlighted_content(writer: &mut vga_buffer::Writer, content: &[u8], highlighter: &syntax::SyntaxHighlighter) {
    let content_str = unsafe { core::str::from_utf8_unchecked(content) };

    for line in content_str.lines() {
        syntax::highlight_line(line, writer, highlighter);
        writer.write_byte(b'\n');
    }
}

fn cmd_help(writer: &mut vga_buffer::Writer) {
    writer.write_string("Anomia OS Commands:\n");
    writer.write_string("  ls, dir         - List files and directories\n");
    writer.write_string("  cd <dir>        - Change current directory\n");
    writer.write_string("  cat <file>      - Display file content\n");
    writer.write_string("  nano <file>     - Text editor (syntax highlighting for .code files)\n");
    writer.write_string("  write <file>    - Create/overwrite a file with one line of text\n");
    writer.write_string("  rm, del <file>  - Delete a file\n");
    writer.write_string("  run <file>      - Execute a CODE assembly program\n");
    writer.write_string("  sample          - Create a sample CODE program (demo.code)\n");
    writer.write_string("  settings        - Configure keyboard, editor, and display options\n");
    writer.write_string("  tests           - Run system diagnostics\n");
    writer.write_string("  date            - Shows the current date and time\n");
    writer.write_string("  clear           - Clear the screen\n");
    writer.write_string("  exit, reboot    - Halts the CPU\n");
    writer.write_string("\nCODE Language Instructions:\n");
    writer.write_string("  mov reg, value  - Load immediate value into register (eax,ebx,ecx,edx)\n");
    writer.write_string("  add eax, ebx    - Add EBX to EAX\n");
    writer.write_string("  sub eax, ebx    - Subtract EBX from EAX\n");
    writer.write_string("  cmp eax, value  - Compare EAX with immediate value\n");
    writer.write_string("  je offset       - Jump if equal (after CMP)\n");
    writer.write_string("  jmp offset      - Unconditional jump\n");
    writer.write_string("  halt            - Stop program execution\n");
    writer.write_string("  nop             - No operation\n");
    writer.write_string("  ; comment       - Comment line\n");
    writer.write_string("\nKeyboard Features:\n");
    writer.write_string("  Hardware Caps Lock - Press Caps Lock key to toggle uppercase\n");
    writer.write_string("  Shift Support   - Hold Shift for symbols (Shift+8 = *, etc.)\n");
    writer.write_string("  5 Layouts       - QWERTY, AZERTY, QWERTZ, Spanish, Dvorak\n");
    writer.write_string("  Full Symbols    - All punctuation and special characters\n");
    writer.write_string("  Spanish chars   - ñ, ´, ¡, ¿, ç and more\n");
    writer.write_string("\nEditor Features:\n");
    writer.write_string("  Syntax Colors   - Instructions (blue), registers (green), numbers (yellow)\n");
    writer.write_string("  3 Themes        - Default, Dark, Retro Green\n");
    writer.write_string("  Real-time       - Colors appear as you type in .code files\n");
}

fn cmd_ls(writer: &mut vga_buffer::Writer) {
    writer.write_string("Directory listing:\n");

    file_system::with_fs(|fs| {
        let (folders, files) = fs.list_current_directory();
        let mut total_count = 0;

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
        for folder_name_opt in folders.iter() {
            if let Some(folder_name) = folder_name_opt {
                if let Ok(folder_str) = core::str::from_utf8(folder_name) {
                    writer.write_string("  [DIR] ");
                    writer.write_string(folder_str);
                    writer.write_byte(b'\n');
                    total_count += 1;
                }
            }
        }

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        let all_files = fs.list_all_files();
        for file_name_option in &all_files {
            if let Some(file_name_bytes) = file_name_option {
                if let Ok(file_str) = core::str::from_utf8(file_name_bytes) {
                    writer.write_string("  - ");
                    writer.write_string(file_str);
                    writer.write_byte(b'\n');
                    total_count += 1;
                }
            }
        }

        if total_count == 0 {
            writer.write_string("  (Empty directory)\n");
        } else {
            writer.write_string("\nTotal items: ");
            let mut buf = [0u8; 20];
            writer.write_string(&vga_buffer::int_to_string(total_count, &mut buf));
            writer.write_string("\n");
        }
    });
}

fn cmd_cat(writer: &mut vga_buffer::Writer, filename: Option<&str>) {
    if let Some(name) = filename {
        file_system::with_fs(|fs| {
            match fs.read_file(name) {
                Ok(data) => {
                    for &byte in data {
                        writer.write_byte(byte);
                    }
                    writer.write_byte(b'\n');
                },
                Err(_) => writer.write_string("Error: File not found.\n"),
            }
        });
    } else {
        writer.write_string("Usage: cat <filename>\n");
    }
}

fn cmd_rm(writer: &mut vga_buffer::Writer, filename: Option<&str>) {
    if let Some(name) = filename {
        file_system::with_fs_mut(|fs| {
            match fs.delete_file(name) {
                Ok(_) => {
                    writer.write_string("File '");
                    writer.write_string(name);
                    writer.write_string("' deleted.\n");
                },
                Err(_) => writer.write_string("Error: File could not be deleted.\n"),
            }
        });
    } else {
        writer.write_string("Usage: rm <filename>\n");
    }
}

fn cmd_write(writer: &mut vga_buffer::Writer, filename: Option<&str>) {
    if let Some(name) = filename {
        writer.write_string("Enter text to write and press Enter:\n> ");
        let mut buffer = [0u8; 1024];
        let input = read_line(writer, &mut buffer);

        file_system::with_fs_mut(|fs| {
            match fs.write_file(name, input.as_bytes()) {
                 Ok(_) => writer.write_string("File written successfully.\n"),
                 Err(_) => writer.write_string("Error: Could not write file.\n"),
            }
        });
    } else {
        writer.write_string("Usage: write <filename>\n");
    }
}

fn cmd_mkdir(writer: &mut vga_buffer::Writer, foldername: Option<&str>) {
    if let Some(name) = foldername {
        file_system::with_fs_mut(|fs| {
            match fs.create_folder(name) {
                Ok(_) => {
                    writer.write_string("Folder '");
                    writer.write_string(name);
                    writer.write_string("' created successfully.\n");
                },
                Err(_) => writer.write_string("Error: Could not create folder.\n"),
            }
        });
    } else {
        writer.write_string("Usage: mkdir <foldername>\n");
    }
}

fn cmd_rmdir(writer: &mut vga_buffer::Writer, foldername: Option<&str>) {
    if let Some(name) = foldername {
        file_system::with_fs_mut(|fs| {
            match fs.delete_folder(name) {
                Ok(_) => {
                    writer.write_string("Folder '");
                    writer.write_string(name);
                    writer.write_string("' deleted successfully.\n");
                },
                Err(_) => writer.write_string("Error: Could not delete folder.\n"),
            }
        });
    } else {
        writer.write_string("Usage: rmdir <foldername>\n");
    }
}

fn cmd_cd(writer: &mut vga_buffer::Writer, path: Option<&str>) {
    if let Some(dir_path) = path {
        file_system::with_fs_mut(|fs| {
            match fs.change_directory(dir_path) {
                Ok(_) => {},
                Err(_) => {
                    writer.write_string("Error: Directory not found: ");
                    writer.write_string(dir_path);
                    writer.write_string("\n");
                }
            }
        });
    } else {
        writer.write_string("Usage: cd <directory>\n");
    }
}

fn cmd_nano(writer: &mut vga_buffer::Writer, filename: Option<&str>) {
    let filename_str = if let Some(name) = filename {
        name
    } else {
        writer.write_string("Usage: nano <filename>\n");
        return;
    };

    let is_code_file = filename_str.ends_with(".code");
    let settings = settings::get_settings();
    let theme = settings.editor_theme;
    let highlighter = syntax::SyntaxHighlighter::new();

    writer.clear_screen();

    writer.color_code = syntax::get_editor_status_color(theme);
    writer.write_string(" Anomia Editor - ");
    writer.write_string(filename_str);
    if is_code_file && settings.syntax_highlighting {
        writer.write_string(" [CODE+Syntax] ");
    }
    writer.write_string(" (ESC=Save&Exit) ");

    for _ in writer.column_position..vga_buffer::BUFFER_WIDTH {
        writer.write_byte(b' ');
    }
    writer.write_byte(b'\n');

    writer.color_code = syntax::get_editor_border_color(theme);
    for _ in 0..vga_buffer::BUFFER_WIDTH {
        writer.write_byte(b'-');
    }
    writer.write_byte(b'\n');

    let mut content_buf = [0u8; 4096];
    let mut content_len = 0;

    crate::file_system::with_fs(|fs| {
        if let Ok(data) = fs.read_file(filename_str) {
            let len = data.len().min(content_buf.len());
            content_buf[..len].copy_from_slice(&data[..len]);
            content_len = len;

            if is_code_file && settings.syntax_highlighting {
                display_highlighted_content(writer, &content_buf[..content_len], &highlighter);
            } else {
                writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
                for &byte in &content_buf[..content_len] { 
                    writer.write_byte(byte); 
                }
            }
        }
    });

    let mut shift_pressed = false;

    loop {
        let sc = read_scancode();

        match sc {
            0x2A | 0x36 => { shift_pressed = true; continue; }
            0xAA | 0xB6 => { shift_pressed = false; continue; }
            _ => {}
        }

        if sc >= 0x80 { continue; }

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
            0x3A => { 
                let mut settings = settings::get_settings();
                settings.caps_lock_enabled = !settings.caps_lock_enabled;
                settings::set_settings(settings);
            }
            _ => {
                if content_len < content_buf.len() {
                    if let Some(c) = settings::scancode_to_char(sc, shift_pressed) {
                        content_buf[content_len] = c as u8;
                        content_len += 1;

                        if is_code_file && settings.syntax_highlighting {
                            let mut tmp = [0u8; 4];                    
                            let token_str = c.encode_utf8(&mut tmp);   
                            let token_type = highlighter.classify_token(token_str);
                            writer.color_code = token_type.get_color(theme);
                        } else {
                            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
                        }
                        writer.write_byte(c as u8);
                    }
                }
            }
        }
    }

    writer.color_code = syntax::get_editor_status_color(theme);
    writer.write_string("\n");
    for _ in 0..vga_buffer::BUFFER_WIDTH {
        writer.write_byte(b'-');
    }
    writer.write_string(" Saving... ");

    crate::file_system::with_fs_mut(|fs| {
        match fs.write_file(filename_str, &content_buf[..content_len]) {
            Ok(_) => writer.write_string("Done! "),
            Err(_) => writer.write_string("Failed! "),
        }
    });

    writer.write_string("Press any key to continue ");
    read_key();
    writer.clear_screen();
    writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
}

fn cmd_run(writer: &mut vga_buffer::Writer, fs: &file_system::OsFileSystem, filename: Option<&str>) {
    if let Some(name) = filename {
        writer.write_string("Executing CODE file: ");
        writer.write_string(name);
        writer.write_string("\n");

        match code_system::execute_code_file(name, fs, writer) {
            Ok(_) => {}, 
            Err(e) => {
                writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Red, vga_buffer::Color::Black);
                writer.write_string("Execution error: ");
                writer.write_string(e);
                writer.write_string("\n");
                writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
            }
        }
    } else {
        writer.write_string("Usage: run <filename.code>\n");
    }
}

fn cmd_sample(writer: &mut vga_buffer::Writer, fs: &mut file_system::OsFileSystem) {
    writer.write_string("Creating sample CODE program 'demo.code'...\n");
    let sample_code = code_system::create_sample_program();

    match fs.write_file("demo.code", sample_code.as_bytes()) {
        Ok(_) => {
            writer.write_string("Sample program created successfully.\n");
            writer.write_string("Run it with: run demo.code\n");
        },
        Err(_) => {
            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Red, vga_buffer::Color::Black);
            writer.write_string("Error: Could not create sample file.\n");
            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        }
    }
}

fn cmd_settings(writer: &mut vga_buffer::Writer) {
    settings::show_settings_menu(writer);
    writer.clear_screen();
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
            "ls" | "dir" => cmd_ls(&mut writer),
            "cd" => cmd_cd(&mut writer, arg),
            "cat" => cmd_cat(&mut writer, arg),
            "nano" => cmd_nano(&mut writer, arg),
            "write" => cmd_write(&mut writer, arg),
            "rm" | "del" => cmd_rm(&mut writer, arg),
            "mkdir" => cmd_mkdir(&mut writer, arg),
            "rmdir" => cmd_rmdir(&mut writer, arg),
            "run" => {
                let fs_ref: &file_system::OsFileSystem = unsafe { &*fs };
                cmd_run(&mut writer, fs_ref, arg);
            },
            "sample" => {
                let fs_mut: &mut file_system::OsFileSystem = unsafe { &mut *fs };
                cmd_sample(&mut writer, fs_mut);
            },
            "settings" | "config" => cmd_settings(&mut writer),
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
                writer.color_code =
                    vga_buffer::ColorCode::new(vga_buffer::Color::Red, vga_buffer::Color::Black);
                writer.write_string("Unknown command: '");
                writer.write_string(command);
                writer.write_string("'\n");
                writer.color_code = vga_buffer::ColorCode::new(
                    vga_buffer::Color::White,
                    vga_buffer::Color::Black,
                );
            }
        }
    }

    writer.clear_screen();
    writer.write_string("Shutting down Anomia OS. Goodbye!");

    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags)); }
    }
}
