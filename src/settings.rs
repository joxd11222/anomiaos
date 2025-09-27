use crate::vga_buffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardLayout {
    Qwerty,
    Azerty,
    Qwertz,
    Dvorak,
}

impl KeyboardLayout {
    pub fn name(&self) -> &'static str {
        match self {
            KeyboardLayout::Qwerty => "QWERTY (US/UK)",
            KeyboardLayout::Azerty => "AZERTY (French)",
            KeyboardLayout::Qwertz => "QWERTZ (German)",
            KeyboardLayout::Dvorak => "Dvorak",
        }
    }

    pub fn next(&self) -> KeyboardLayout {
        match self {
            KeyboardLayout::Qwerty => KeyboardLayout::Azerty,
            KeyboardLayout::Azerty => KeyboardLayout::Qwertz,
            KeyboardLayout::Qwertz => KeyboardLayout::Dvorak,
            KeyboardLayout::Dvorak => KeyboardLayout::Qwerty,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub keyboard_layout: KeyboardLayout,
    pub caps_lock_enabled: bool,
    pub syntax_highlighting: bool,
    pub editor_theme: EditorTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTheme {
    Default,
    Dark,
    Retro,
}

impl EditorTheme {
    pub fn name(&self) -> &'static str {
        match self {
            EditorTheme::Default => "Default",
            EditorTheme::Dark => "Dark",
            EditorTheme::Retro => "Retro Green",
        }
    }

    pub fn next(&self) -> EditorTheme {
        match self {
            EditorTheme::Default => EditorTheme::Dark,
            EditorTheme::Dark => EditorTheme::Retro,
            EditorTheme::Retro => EditorTheme::Default,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            keyboard_layout: KeyboardLayout::Qwerty,
            caps_lock_enabled: false,
            syntax_highlighting: true,
            editor_theme: EditorTheme::Default,
        }
    }
}

static mut GLOBAL_SETTINGS: Settings = Settings {
    keyboard_layout: KeyboardLayout::Qwerty,
    caps_lock_enabled: false,
    syntax_highlighting: true,
    editor_theme: EditorTheme::Default,
};

pub fn get_settings() -> Settings {
    unsafe { GLOBAL_SETTINGS }
}

pub fn set_settings(settings: Settings) {
    unsafe { GLOBAL_SETTINGS = settings; }
}

pub fn scancode_to_char(sc: u8, shift_pressed: bool) -> Option<char> {
    let settings = get_settings();
    let caps = settings.caps_lock_enabled ^ shift_pressed; 

    match settings.keyboard_layout {
        KeyboardLayout::Qwerty => qwerty_scancode_to_char(sc, caps, shift_pressed),
        KeyboardLayout::Azerty => azerty_scancode_to_char(sc, caps, shift_pressed),
        KeyboardLayout::Qwertz => qwertz_scancode_to_char(sc, caps, shift_pressed),
        KeyboardLayout::Dvorak => dvorak_scancode_to_char(sc, caps, shift_pressed),
    }
}

fn qwerty_scancode_to_char(sc: u8, caps: bool, shift: bool) -> Option<char> {
    match sc {

        0x02 => Some(if shift { '!' } else { '1' }),
        0x03 => Some(if shift { '@' } else { '2' }),
        0x04 => Some(if shift { '#' } else { '3' }),
        0x05 => Some(if shift { '$' } else { '4' }),
        0x06 => Some(if shift { '%' } else { '5' }),
        0x07 => Some(if shift { '^' } else { '6' }),
        0x08 => Some(if shift { '&' } else { '7' }),
        0x09 => Some(if shift { '*' } else { '8' }),
        0x0A => Some(if shift { '(' } else { '9' }),
        0x0B => Some(if shift { ')' } else { '0' }),
        0x0C => Some(if shift { '_' } else { '-' }),
        0x0D => Some(if shift { '+' } else { '=' }),

        0x10 => Some(if caps { 'Q' } else { 'q' }),
        0x11 => Some(if caps { 'W' } else { 'w' }),
        0x12 => Some(if caps { 'E' } else { 'e' }),
        0x13 => Some(if caps { 'R' } else { 'r' }),
        0x14 => Some(if caps { 'T' } else { 't' }),
        0x15 => Some(if caps { 'Y' } else { 'y' }),
        0x16 => Some(if caps { 'U' } else { 'u' }),
        0x17 => Some(if caps { 'I' } else { 'i' }),
        0x18 => Some(if caps { 'O' } else { 'o' }),
        0x19 => Some(if caps { 'P' } else { 'p' }),
        0x1A => Some(if shift { '{' } else { '[' }),
        0x1B => Some(if shift { '}' } else { ']' }),

        0x1E => Some(if caps { 'A' } else { 'a' }),
        0x1F => Some(if caps { 'S' } else { 's' }),
        0x20 => Some(if caps { 'D' } else { 'd' }),
        0x21 => Some(if caps { 'F' } else { 'f' }),
        0x22 => Some(if caps { 'G' } else { 'g' }),
        0x23 => Some(if caps { 'H' } else { 'h' }),
        0x24 => Some(if caps { 'J' } else { 'j' }),
        0x25 => Some(if caps { 'K' } else { 'k' }),
        0x26 => Some(if caps { 'L' } else { 'l' }),
        0x27 => Some(if shift { ':' } else { ';' }),
        0x28 => Some(if shift { '"' } else { '\'' }),
        0x29 => Some(if shift { '~' } else { '`' }),

        0x2C => Some(if caps { 'Z' } else { 'z' }),
        0x2D => Some(if caps { 'X' } else { 'x' }),
        0x2E => Some(if caps { 'C' } else { 'c' }),
        0x2F => Some(if caps { 'V' } else { 'v' }),
        0x30 => Some(if caps { 'B' } else { 'b' }),
        0x31 => Some(if caps { 'N' } else { 'n' }),
        0x32 => Some(if caps { 'M' } else { 'm' }),
        0x33 => Some(if shift { '<' } else { ',' }),
        0x34 => Some(if shift { '>' } else { '.' }),
        0x35 => Some(if shift { '?' } else { '/' }),

        0x39 => Some(' '), 
        0x2B => Some(if shift { '|' } else { '\\' }),

        _ => None,
    }
}

fn azerty_scancode_to_char(sc: u8, caps: bool, shift: bool) -> Option<char> {
    match sc {

        0x02 => Some(if shift { '1' } else { '&' }),
        0x03 => Some(if shift { '2' } else { 'é' }),
        0x04 => Some(if shift { '3' } else { '"' }),
        0x05 => Some(if shift { '4' } else { '\'' }),
        0x06 => Some(if shift { '5' } else { '(' }),
        0x07 => Some(if shift { '6' } else { '-' }),
        0x08 => Some(if shift { '7' } else { 'è' }),
        0x09 => Some(if shift { '8' } else { '_' }),
        0x0A => Some(if shift { '9' } else { 'ç' }),
        0x0B => Some(if shift { '0' } else { 'à' }),
        0x0C => Some(if shift { '°' } else { ')' }),
        0x0D => Some(if shift { '+' } else { '=' }),

        0x10 => Some(if caps { 'A' } else { 'a' }),
        0x11 => Some(if caps { 'Z' } else { 'z' }),
        0x12 => Some(if caps { 'E' } else { 'e' }),
        0x13 => Some(if caps { 'R' } else { 'r' }),
        0x14 => Some(if caps { 'T' } else { 't' }),
        0x15 => Some(if caps { 'Y' } else { 'y' }),
        0x16 => Some(if caps { 'U' } else { 'u' }),
        0x17 => Some(if caps { 'I' } else { 'i' }),
        0x18 => Some(if caps { 'O' } else { 'o' }),
        0x19 => Some(if caps { 'P' } else { 'p' }),

        0x1E => Some(if caps { 'Q' } else { 'q' }),
        0x1F => Some(if caps { 'S' } else { 's' }),
        0x20 => Some(if caps { 'D' } else { 'd' }),
        0x21 => Some(if caps { 'F' } else { 'f' }),
        0x22 => Some(if caps { 'G' } else { 'g' }),
        0x23 => Some(if caps { 'H' } else { 'h' }),
        0x24 => Some(if caps { 'J' } else { 'j' }),
        0x25 => Some(if caps { 'K' } else { 'k' }),
        0x26 => Some(if caps { 'L' } else { 'l' }),
        0x27 => Some(if caps { 'M' } else { 'm' }),

        0x2C => Some(if caps { 'W' } else { 'w' }),
        0x2D => Some(if caps { 'X' } else { 'x' }),
        0x2E => Some(if caps { 'C' } else { 'c' }),
        0x2F => Some(if caps { 'V' } else { 'v' }),
        0x30 => Some(if caps { 'B' } else { 'b' }),
        0x31 => Some(if caps { 'N' } else { 'n' }),
        0x33 => Some(if shift { '?' } else { ',' }),
        0x34 => Some(if shift { '.' } else { ';' }),
        0x35 => Some(if shift { '/' } else { ':' }),

        0x39 => Some(' '),
        _ => None,
    }
}

fn qwertz_scancode_to_char(sc: u8, caps: bool, shift: bool) -> Option<char> {
    match sc {

        0x02..=0x0B => qwerty_scancode_to_char(sc, caps, shift), 

        0x10 => Some(if caps { 'Q' } else { 'q' }),
        0x11 => Some(if caps { 'W' } else { 'w' }),
        0x12 => Some(if caps { 'E' } else { 'e' }),
        0x13 => Some(if caps { 'R' } else { 'r' }),
        0x14 => Some(if caps { 'T' } else { 't' }),
        0x15 => Some(if caps { 'Z' } else { 'z' }), 
        0x16 => Some(if caps { 'U' } else { 'u' }),
        0x17 => Some(if caps { 'I' } else { 'i' }),
        0x18 => Some(if caps { 'O' } else { 'o' }),
        0x19 => Some(if caps { 'P' } else { 'p' }),

        0x1E => Some(if caps { 'A' } else { 'a' }),
        0x1F => Some(if caps { 'S' } else { 's' }),
        0x20 => Some(if caps { 'D' } else { 'd' }),
        0x21 => Some(if caps { 'F' } else { 'f' }),
        0x22 => Some(if caps { 'G' } else { 'g' }),
        0x23 => Some(if caps { 'H' } else { 'h' }),
        0x24 => Some(if caps { 'J' } else { 'j' }),
        0x25 => Some(if caps { 'K' } else { 'k' }),
        0x26 => Some(if caps { 'L' } else { 'l' }),

        0x2C => Some(if caps { 'Y' } else { 'y' }), 
        0x2D => Some(if caps { 'X' } else { 'x' }),
        0x2E => Some(if caps { 'C' } else { 'c' }),
        0x2F => Some(if caps { 'V' } else { 'v' }),
        0x30 => Some(if caps { 'B' } else { 'b' }),
        0x31 => Some(if caps { 'N' } else { 'n' }),
        0x32 => Some(if caps { 'M' } else { 'm' }),
        0x33 => Some(if shift { '<' } else { ',' }),
        0x34 => Some(if shift { '>' } else { '.' }),
        0x35 => Some(if shift { '?' } else { '/' }),

        0x39 => Some(' '),
        _ => qwerty_scancode_to_char(sc, caps, shift), 
    }
}

fn dvorak_scancode_to_char(sc: u8, caps: bool, shift: bool) -> Option<char> {
    match sc {

        0x02..=0x0D => qwerty_scancode_to_char(sc, caps, shift),

        0x10 => Some(if shift { '"' } else { '\'' }),
        0x11 => Some(if shift { '<' } else { ',' }),
        0x12 => Some(if shift { '>' } else { '.' }),
        0x13 => Some(if caps { 'P' } else { 'p' }),
        0x14 => Some(if caps { 'Y' } else { 'y' }),
        0x15 => Some(if caps { 'F' } else { 'f' }),
        0x16 => Some(if caps { 'G' } else { 'g' }),
        0x17 => Some(if caps { 'C' } else { 'c' }),
        0x18 => Some(if caps { 'R' } else { 'r' }),
        0x19 => Some(if caps { 'L' } else { 'l' }),

        0x1E => Some(if caps { 'A' } else { 'a' }),
        0x1F => Some(if caps { 'O' } else { 'o' }),
        0x20 => Some(if caps { 'E' } else { 'e' }),
        0x21 => Some(if caps { 'U' } else { 'u' }),
        0x22 => Some(if caps { 'I' } else { 'i' }),
        0x23 => Some(if caps { 'D' } else { 'd' }),
        0x24 => Some(if caps { 'H' } else { 'h' }),
        0x25 => Some(if caps { 'T' } else { 't' }),
        0x26 => Some(if caps { 'N' } else { 'n' }),
        0x27 => Some(if caps { 'S' } else { 's' }),

        0x2C => Some(if shift { ':' } else { ';' }),
        0x2D => Some(if caps { 'Q' } else { 'q' }),
        0x2E => Some(if caps { 'J' } else { 'j' }),
        0x2F => Some(if caps { 'K' } else { 'k' }),
        0x30 => Some(if caps { 'X' } else { 'x' }),
        0x31 => Some(if caps { 'B' } else { 'b' }),
        0x32 => Some(if caps { 'M' } else { 'm' }),
        0x33 => Some(if caps { 'W' } else { 'w' }),
        0x34 => Some(if caps { 'V' } else { 'v' }),
        0x35 => Some(if caps { 'Z' } else { 'z' }),

        0x39 => Some(' '),
        _ => None,
    }
}

pub fn get_caps_lock_state() -> bool {
    get_settings().caps_lock_enabled
}

pub fn toggle_caps_lock() {
    let mut s = get_settings();
    s.caps_lock_enabled = !s.caps_lock_enabled;
    set_settings(s);
}

pub fn show_settings_menu(writer: &mut vga_buffer::Writer) {
    let mut settings = get_settings();
    let mut selected = 0;
    let menu_items = 3; 

    loop {
        writer.clear_screen();
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
        writer.write_string("==== ANOMIA OS SETTINGS ====\n\n");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);

        if selected == 0 {
            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Black, vga_buffer::Color::White);
        }
        writer.write_string("1. Keyboard Layout: ");
        writer.write_string(settings.keyboard_layout.name());
        writer.write_string("\n");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightGray, vga_buffer::Color::Black);
        writer.write_string("   Caps Lock Status: ");
        writer.write_string(if get_caps_lock_state() { "ON (hardware)" } else { "OFF (hardware)" });
        writer.write_string("\n");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);

        if selected == 1 {
            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Black, vga_buffer::Color::White);
        }
        writer.write_string("2. CODE Syntax Highlighting: ");
        writer.write_string(if settings.syntax_highlighting { "ON" } else { "OFF" });
        writer.write_string("\n");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);

        if selected == 2 {
            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Black, vga_buffer::Color::White);
        }
        writer.write_string("3. Editor Theme: ");
        writer.write_string(settings.editor_theme.name());
        writer.write_string("\n");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);

        writer.write_string("\nUse Arrow Keys to navigate, Enter to change, ESC to exit\n");
        writer.write_string("Current layout test: ");

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
        writer.write_string("Try Shift+8 = ");
        if let Some(c) = scancode_to_char(0x09, true) { 
            writer.write_byte(c as u8);
        }
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        writer.write_string(", normal 8 = ");
        if let Some(c) = scancode_to_char(0x09, false) {
            writer.write_byte(c as u8);
        }
        writer.write_string("\nPress Caps Lock key to toggle caps state\n");

        let mut shift_pressed = false;
        let key = crate::read_scancode();

        match key {
            0x2A | 0x36 => { shift_pressed = true; }
            0xAA | 0xB6 => { shift_pressed = false; }
            _ => {}
        }

        if key >= 0x80 { continue; }

        match key {
            0x01 => break, 
            0x1C => { 
                match selected {
                    0 => settings.keyboard_layout = settings.keyboard_layout.next(),
                    1 => settings.syntax_highlighting = !settings.syntax_highlighting,
                    2 => settings.editor_theme = settings.editor_theme.next(),
                    _ => {}
                }
                set_settings(settings);
            }
            0x48 => { 
                selected = if selected == 0 { menu_items - 1 } else { selected - 1 };
            }
            0x50 => { 
                selected = (selected + 1) % menu_items;
            }
            0x3A => { 
                toggle_caps_lock();

                writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
                writer.write_string(" CAPS TOGGLED! ");
                writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
            }
            _ => {

                if let Some(c) = scancode_to_char(key, shift_pressed) {
                    writer.write_byte(c as u8);
                }
            }
        }
    }
}
