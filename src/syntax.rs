use crate::vga_buffer::{ColorCode, Color};
use crate::settings::{get_settings, EditorTheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Instruction,    
    Register,       
    Number,         
    Comment,        
    Label,          
    String,         
    Operator,       
    Normal,         
}

impl TokenType {
    pub fn get_color(&self, theme: EditorTheme) -> ColorCode {
        match theme {
            EditorTheme::Default => match self {
                TokenType::Instruction => ColorCode::new(Color::LightBlue, Color::Black),
                TokenType::Register => ColorCode::new(Color::LightGreen, Color::Black),
                TokenType::Number => ColorCode::new(Color::Yellow, Color::Black),
                TokenType::Comment => ColorCode::new(Color::LightGray, Color::Black),
                TokenType::Label => ColorCode::new(Color::LightCyan, Color::Black),
                TokenType::String => ColorCode::new(Color::Pink, Color::Black),
                TokenType::Operator => ColorCode::new(Color::White, Color::Black),
                TokenType::Normal => ColorCode::new(Color::White, Color::Black),
            },
            EditorTheme::Dark => match self {
                TokenType::Instruction => ColorCode::new(Color::Cyan, Color::Black),
                TokenType::Register => ColorCode::new(Color::Green, Color::Black),
                TokenType::Number => ColorCode::new(Color::Brown, Color::Black),
                TokenType::Comment => ColorCode::new(Color::DarkGray, Color::Black),
                TokenType::Label => ColorCode::new(Color::Blue, Color::Black),
                TokenType::String => ColorCode::new(Color::Magenta, Color::Black),
                TokenType::Operator => ColorCode::new(Color::LightGray, Color::Black),
                TokenType::Normal => ColorCode::new(Color::LightGray, Color::Black),
            },
            EditorTheme::Retro => match self {
                TokenType::Instruction => ColorCode::new(Color::LightGreen, Color::Black),
                TokenType::Register => ColorCode::new(Color::Green, Color::Black),
                TokenType::Number => ColorCode::new(Color::Yellow, Color::Black),
                TokenType::Comment => ColorCode::new(Color::DarkGray, Color::Black),
                TokenType::Label => ColorCode::new(Color::LightCyan, Color::Black),
                TokenType::String => ColorCode::new(Color::Pink, Color::Black),
                TokenType::Operator => ColorCode::new(Color::White, Color::Black),
                TokenType::Normal => ColorCode::new(Color::LightGreen, Color::Black),
            },
        }
    }
}

pub struct SyntaxHighlighter {
    instructions: [&'static str; 21],
    registers: [&'static str; 8],
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            instructions: [
                "mov", "add", "sub", "cmp", "je", "jz", "jmp", "call",
                "ret", "push", "pop", "nop", "halt", "stop", "int", "hlt", "print", "while", "loop", "call",
                "input",
            ],
            registers: [
                "eax", "ebx", "ecx", "edx", "esi", "edi", "esp", "ebp"
            ],
        }
    }

    pub fn classify_token(&self, token: &str) -> TokenType {

        if token.starts_with(';') {
            return TokenType::Comment;
        }

        if token.starts_with('"') && token.ends_with('"') {
            return TokenType::String;
        }

        if token.ends_with(':') {
            return TokenType::Label;
        }

        if self.is_number_str(token) {
            return TokenType::Number;
        }

        let mut lowercase_buf = [0u8; 32];
        let lowercase_len = self.str_to_lowercase(token, &mut lowercase_buf);
        let lowercase_token = unsafe { core::str::from_utf8_unchecked(&lowercase_buf[..lowercase_len]) };

        for &instruction in &self.instructions {
            if lowercase_token == instruction {
                return TokenType::Instruction;
            }
        }

        let token_without_comma = lowercase_token.trim_end_matches(',');
        for &register in &self.registers {
            if token_without_comma == register {
                return TokenType::Register;
            }
        }

        if token.len() == 1 {
            match token.chars().next().unwrap() {
                '+' | '-' | '*' | '/' | '=' | '<' | '>' | '&' | '|' | '^' => return TokenType::Operator,
                _ => {}
            }
        }

        TokenType::Normal
    }

    fn str_to_lowercase(&self, s: &str, buf: &mut [u8]) -> usize {
        let bytes = s.as_bytes();
        let len = bytes.len().min(buf.len());

        for i in 0..len {
            buf[i] = if bytes[i] >= b'A' && bytes[i] <= b'Z' {
                bytes[i] + 32
            } else {
                bytes[i]
            };
        }
        len
    }

    fn is_number_str(&self, token: &str) -> bool {
        if token.is_empty() { return false; }

        if token.starts_with("0x") || token.starts_with("0X") {
            if token.len() <= 2 { return false; }
            return token[2..].chars().all(|c| c.is_ascii_hexdigit());
        }

        if token.starts_with("0b") || token.starts_with("0B") {
            if token.len() <= 2 { return false; }
            return token[2..].chars().all(|c| c == '0' || c == '1');
        }

        token.chars().all(|c| c.is_ascii_digit())
    }
}

pub fn highlight_line(line: &str, writer: &mut crate::vga_buffer::Writer, highlighter: &SyntaxHighlighter) {
    let settings = get_settings();

    if !settings.syntax_highlighting {

        writer.color_code = ColorCode::new(Color::White, Color::Black);
        writer.write_string(line);
        return;
    }

    let mut current_pos = 0;
    let line_bytes = line.as_bytes();

    if line.trim_start().starts_with(';') {
        writer.color_code = TokenType::Comment.get_color(settings.editor_theme);
        writer.write_string(line);
        return;
    }

    while current_pos < line_bytes.len() {

        while current_pos < line_bytes.len() && line_bytes[current_pos].is_ascii_whitespace() {
            writer.color_code = TokenType::Normal.get_color(settings.editor_theme);
            writer.write_byte(line_bytes[current_pos]);
            current_pos += 1;
        }

        if current_pos >= line_bytes.len() {
            break;
        }

        if line_bytes[current_pos] == b';' {
            writer.color_code = TokenType::Comment.get_color(settings.editor_theme);
            while current_pos < line_bytes.len() {
                writer.write_byte(line_bytes[current_pos]);
                current_pos += 1;
            }
            break;
        }

        let token_start = current_pos;
        while current_pos < line_bytes.len() && 
              !line_bytes[current_pos].is_ascii_whitespace() &&
              line_bytes[current_pos] != b';' &&
              line_bytes[current_pos] != b',' {
            current_pos += 1;
        }

        let has_comma = current_pos < line_bytes.len() && line_bytes[current_pos] == b',';

        if token_start < current_pos {
            let token = unsafe { 
                core::str::from_utf8_unchecked(&line_bytes[token_start..current_pos])
            };

            let token_type = highlighter.classify_token(token);
            writer.color_code = token_type.get_color(settings.editor_theme);
            writer.write_string(token);

            if has_comma {
                writer.color_code = TokenType::Operator.get_color(settings.editor_theme);
                writer.write_byte(b',');
                current_pos += 1;
            }
        }
    }
}

pub fn get_editor_background_color(theme: EditorTheme) -> Color {
    match theme {
        EditorTheme::Default => Color::Black,
        EditorTheme::Dark => Color::Black,
        EditorTheme::Retro => Color::Black,
    }
}

pub fn get_editor_border_color(theme: EditorTheme) -> ColorCode {
    match theme {
        EditorTheme::Default => ColorCode::new(Color::LightCyan, Color::Black),
        EditorTheme::Dark => ColorCode::new(Color::DarkGray, Color::Black),
        EditorTheme::Retro => ColorCode::new(Color::LightGreen, Color::Black),
    }
}

pub fn get_editor_status_color(theme: EditorTheme) -> ColorCode {
    match theme {
        EditorTheme::Default => ColorCode::new(Color::White, Color::Blue),
        EditorTheme::Dark => ColorCode::new(Color::LightGray, Color::DarkGray),
        EditorTheme::Retro => ColorCode::new(Color::Black, Color::LightGreen),
    }
}
