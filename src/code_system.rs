use crate::vga_buffer;
use crate::file_system::OsFileSystem;

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Nop = 0x90,
    MovEaxImm32 = 0xB8,
    MovEbxImm32 = 0xBB,
    MovEcxImm32 = 0xB9,
    MovEdxImm32 = 0xBA,
    AddEaxEbx = 0x01,
    SubEaxEbx = 0x29,
    CmpEaxImm32 = 0x3D,
    JeRel8 = 0x74,
    JmpRel8 = 0xEB,
    Int3 = 0xCC, 
    Ret = 0xC3,
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualCpu {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub eip: usize, 
    pub flags: u32,
}

impl VirtualCpu {
    pub fn new() -> Self {
        Self {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            eip: 0,
            flags: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

pub struct CodeExecutor {
    cpu: VirtualCpu,
    memory: [u8; 4096], 
    max_instructions: usize,
}

impl CodeExecutor {
    pub fn new() -> Self {
        Self {
            cpu: VirtualCpu::new(),
            memory: [0; 4096],
            max_instructions: 10000, 
        }
    }

    pub fn compile_code(&mut self, source: &str) -> Result<usize, &'static str> {
        let mut bytecode_len = 0;
        let mut line_num = 1;

        for line in source.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with(';') {
                line_num += 1;
                continue;
            }

            let mut parts = [("", 0usize); 8]; 
            let mut part_count = 0;

            for part in line.split_whitespace() {
                if part_count < parts.len() {
                    parts[part_count] = (part, part.len());
                    part_count += 1;
                }
            }

            if part_count == 0 {
                line_num += 1;
                continue;
            }

            let instruction = parts[0].0;

            let mut instr_buf = [0u8; 16];
            let mut instr_len = 0;
            for (i, &byte) in instruction.as_bytes().iter().enumerate() {
                if i < instr_buf.len() {
                    instr_buf[i] = if byte >= b'A' && byte <= b'Z' {
                        byte + 32 
                    } else {
                        byte
                    };
                    instr_len += 1;
                } else {
                    break;
                }
            }
            let instruction_lower = unsafe { core::str::from_utf8_unchecked(&instr_buf[..instr_len]) };

            match instruction_lower {
                "nop" => {
                    if bytecode_len >= self.memory.len() { return Err("Program too large"); }
                    self.memory[bytecode_len] = Opcode::Nop as u8;
                    bytecode_len += 1;
                }
                "mov" => {
                    if part_count < 3 { return Err("MOV requires 2 operands"); }
                    let dest = parts[1].0.trim_end_matches(',');
                    let src = parts[2].0;

                    if bytecode_len + 5 >= self.memory.len() { return Err("Program too large"); }

                    match dest {
                        "eax" => {
                            self.memory[bytecode_len] = Opcode::MovEaxImm32 as u8;
                            bytecode_len += 1;
                            if let Ok(val) = self.parse_immediate(src) {
                                let bytes = val.to_le_bytes();
                                self.memory[bytecode_len..bytecode_len + 4].copy_from_slice(&bytes);
                                bytecode_len += 4;
                            } else {
                                return Err("Invalid immediate value");
                            }
                        }
                        "ebx" => {
                            self.memory[bytecode_len] = Opcode::MovEbxImm32 as u8;
                            bytecode_len += 1;
                            if let Ok(val) = self.parse_immediate(src) {
                                let bytes = val.to_le_bytes();
                                self.memory[bytecode_len..bytecode_len + 4].copy_from_slice(&bytes);
                                bytecode_len += 4;
                            } else {
                                return Err("Invalid immediate value");
                            }
                        }
                        "ecx" => {
                            self.memory[bytecode_len] = Opcode::MovEcxImm32 as u8;
                            bytecode_len += 1;
                            if let Ok(val) = self.parse_immediate(src) {
                                let bytes = val.to_le_bytes();
                                self.memory[bytecode_len..bytecode_len + 4].copy_from_slice(&bytes);
                                bytecode_len += 4;
                            } else {
                                return Err("Invalid immediate value");
                            }
                        }
                        "edx" => {
                            self.memory[bytecode_len] = Opcode::MovEdxImm32 as u8;
                            bytecode_len += 1;
                            if let Ok(val) = self.parse_immediate(src) {
                                let bytes = val.to_le_bytes();
                                self.memory[bytecode_len..bytecode_len + 4].copy_from_slice(&bytes);
                                bytecode_len += 4;
                            } else {
                                return Err("Invalid immediate value");
                            }
                        }
                        _ => return Err("Unsupported MOV destination register"),
                    }
                }
                "add" => {
                    if part_count < 3 { return Err("ADD requires 2 operands"); }
                    if parts[1].0.trim_end_matches(',') == "eax" && parts[2].0 == "ebx" {
                        if bytecode_len >= self.memory.len() { return Err("Program too large"); }
                        self.memory[bytecode_len] = 0x01; 
                        self.memory[bytecode_len + 1] = 0xD8; 
                        bytecode_len += 2;
                    } else {
                        return Err("Unsupported ADD operands");
                    }
                }
                "sub" => {
                    if part_count < 3 { return Err("SUB requires 2 operands"); }
                    if parts[1].0.trim_end_matches(',') == "eax" && parts[2].0 == "ebx" {
                        if bytecode_len >= self.memory.len() { return Err("Program too large"); }
                        self.memory[bytecode_len] = 0x29; 
                        self.memory[bytecode_len + 1] = 0xD8; 
                        bytecode_len += 2;
                    } else {
                        return Err("Unsupported SUB operands");
                    }
                }
                "cmp" => {
                    if part_count < 3 { return Err("CMP requires 2 operands"); }
                    if parts[1].0.trim_end_matches(',') == "eax" {
                        if bytecode_len + 5 >= self.memory.len() { return Err("Program too large"); }
                        self.memory[bytecode_len] = Opcode::CmpEaxImm32 as u8;
                        bytecode_len += 1;
                        if let Ok(val) = self.parse_immediate(parts[2].0) {
                            let bytes = val.to_le_bytes();
                            self.memory[bytecode_len..bytecode_len + 4].copy_from_slice(&bytes);
                            bytecode_len += 4;
                        } else {
                            return Err("Invalid immediate value");
                        }
                    } else {
                        return Err("Unsupported CMP operands");
                    }
                }
                "je" | "jz" => {
                    if part_count < 2 { return Err("JE requires 1 operand"); }
                    if bytecode_len + 2 >= self.memory.len() { return Err("Program too large"); }
                    self.memory[bytecode_len] = Opcode::JeRel8 as u8;

                    if let Ok(offset) = self.parse_immediate(parts[1].0) {
                        self.memory[bytecode_len + 1] = offset as u8;
                        bytecode_len += 2;
                    } else {
                        return Err("Invalid jump offset");
                    }
                }
                "jmp" => {
                    if part_count < 2 { return Err("JMP requires 1 operand"); }
                    if bytecode_len + 2 >= self.memory.len() { return Err("Program too large"); }
                    self.memory[bytecode_len] = Opcode::JmpRel8 as u8;
                    if let Ok(offset) = self.parse_immediate(parts[1].0) {
                        self.memory[bytecode_len + 1] = offset as u8;
                        bytecode_len += 2;
                    } else {
                        return Err("Invalid jump offset");
                    }
                }
                "halt" | "stop" => {
                    if bytecode_len >= self.memory.len() { return Err("Program too large"); }
                    self.memory[bytecode_len] = Opcode::Int3 as u8;
                    bytecode_len += 1;
                }
                "ret" => {
                    if bytecode_len >= self.memory.len() { return Err("Program too large"); }
                    self.memory[bytecode_len] = Opcode::Ret as u8;
                    bytecode_len += 1;
                }
                _ => {
                    return Err("Unknown instruction");
                }
            }

            line_num += 1;
        }

        Ok(bytecode_len)
    }

    fn parse_immediate(&self, s: &str) -> Result<u32, &'static str> {
        if s.starts_with("0x") || s.starts_with("0X") {

            let hex_str = &s[2..];
            let mut result = 0u32;
            for c in hex_str.chars() {
                result = result << 4;
                match c {
                    '0'..='9' => result |= (c as u32) - ('0' as u32),
                    'a'..='f' => result |= (c as u32) - ('a' as u32) + 10,
                    'A'..='F' => result |= (c as u32) - ('A' as u32) + 10,
                    _ => return Err("Invalid hex digit"),
                }
            }
            Ok(result)
        } else {

            let mut result = 0u32;
            for c in s.chars() {
                if c.is_ascii_digit() {
                    result = result * 10 + (c as u32 - '0' as u32);
                } else {
                    return Err("Invalid decimal digit");
                }
            }
            Ok(result)
        }
    }

    pub fn execute(&mut self, bytecode_len: usize, writer: &mut vga_buffer::Writer) -> Result<(), &'static str> {
        self.cpu.reset();
        let mut instruction_count = 0;

        while self.cpu.eip < bytecode_len && instruction_count < self.max_instructions {
            let opcode = self.memory[self.cpu.eip];

            match opcode {
                0x90 => { 
                    self.cpu.eip += 1;
                }
                0xB8 => { 
                    if self.cpu.eip + 5 > bytecode_len { return Err("Unexpected end of program"); }
                    let imm = u32::from_le_bytes([
                        self.memory[self.cpu.eip + 1],
                        self.memory[self.cpu.eip + 2],
                        self.memory[self.cpu.eip + 3],
                        self.memory[self.cpu.eip + 4],
                    ]);
                    self.cpu.eax = imm;
                    self.cpu.eip += 5;
                }
                0xBB => { 
                    if self.cpu.eip + 5 > bytecode_len { return Err("Unexpected end of program"); }
                    let imm = u32::from_le_bytes([
                        self.memory[self.cpu.eip + 1],
                        self.memory[self.cpu.eip + 2],
                        self.memory[self.cpu.eip + 3],
                        self.memory[self.cpu.eip + 4],
                    ]);
                    self.cpu.ebx = imm;
                    self.cpu.eip += 5;
                }
                0xB9 => { 
                    if self.cpu.eip + 5 > bytecode_len { return Err("Unexpected end of program"); }
                    let imm = u32::from_le_bytes([
                        self.memory[self.cpu.eip + 1],
                        self.memory[self.cpu.eip + 2],
                        self.memory[self.cpu.eip + 3],
                        self.memory[self.cpu.eip + 4],
                    ]);
                    self.cpu.ecx = imm;
                    self.cpu.eip += 5;
                }
                0xBA => { 
                    if self.cpu.eip + 5 > bytecode_len { return Err("Unexpected end of program"); }
                    let imm = u32::from_le_bytes([
                        self.memory[self.cpu.eip + 1],
                        self.memory[self.cpu.eip + 2],
                        self.memory[self.cpu.eip + 3],
                        self.memory[self.cpu.eip + 4],
                    ]);
                    self.cpu.edx = imm;
                    self.cpu.eip += 5;
                }
                0x01 => { 
                    if self.cpu.eip + 2 > bytecode_len { return Err("Unexpected end of program"); }
                    self.cpu.eax = self.cpu.eax.wrapping_add(self.cpu.ebx);
                    self.cpu.eip += 2;
                }
                0x29 => { 
                    if self.cpu.eip + 2 > bytecode_len { return Err("Unexpected end of program"); }
                    self.cpu.eax = self.cpu.eax.wrapping_sub(self.cpu.ebx);
                    self.cpu.eip += 2;
                }
                0x3D => { 
                    if self.cpu.eip + 5 > bytecode_len { return Err("Unexpected end of program"); }
                    let imm = u32::from_le_bytes([
                        self.memory[self.cpu.eip + 1],
                        self.memory[self.cpu.eip + 2],
                        self.memory[self.cpu.eip + 3],
                        self.memory[self.cpu.eip + 4],
                    ]);

                    if self.cpu.eax == imm {
                        self.cpu.flags |= 0x40; 
                    } else {
                        self.cpu.flags &= !0x40;
                    }
                    self.cpu.eip += 5;
                }
                0x74 => { 
                    if self.cpu.eip + 2 > bytecode_len { return Err("Unexpected end of program"); }
                    let offset = self.memory[self.cpu.eip + 1] as i8;
                    if (self.cpu.flags & 0x40) != 0 { 
                        self.cpu.eip = (self.cpu.eip as i32 + 2 + offset as i32) as usize;
                    } else {
                        self.cpu.eip += 2;
                    }
                }
                0xEB => { 
                    if self.cpu.eip + 2 > bytecode_len { return Err("Unexpected end of program"); }
                    let offset = self.memory[self.cpu.eip + 1] as i8;
                    self.cpu.eip = (self.cpu.eip as i32 + 2 + offset as i32) as usize;
                }
                0xCC => { 
                    break;
                }
                0xC3 => { 
                    break;
                }
                _ => {
                    return Err("Unknown opcode");
                }
            }

            instruction_count += 1;
        }

        if instruction_count >= self.max_instructions {
            return Err("Program execution limit exceeded");
        }

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightGreen, vga_buffer::Color::Black);
        writer.write_string("Program execution completed successfully!\n");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        writer.write_string("\nFinal register values:\n");
        writer.write_string("======================\n");

        let mut buf = [0u8; 20];

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
        writer.write_string("EAX");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        writer.write_string(" (main result): ");
        writer.write_string(&vga_buffer::int_to_string(self.cpu.eax as usize, &mut buf));
        writer.write_string(" (decimal) = 0x");
        writer.write_string(&vga_buffer::hex_to_string(self.cpu.eax, &mut buf));
        writer.write_string(" (hex)\n");

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
        writer.write_string("EBX");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        writer.write_string(" (secondary):   ");
        writer.write_string(&vga_buffer::int_to_string(self.cpu.ebx as usize, &mut buf));
        writer.write_string(" (decimal) = 0x");
        writer.write_string(&vga_buffer::hex_to_string(self.cpu.ebx, &mut buf));
        writer.write_string(" (hex)\n");

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
        writer.write_string("ECX");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        writer.write_string(" (counter):     ");
        writer.write_string(&vga_buffer::int_to_string(self.cpu.ecx as usize, &mut buf));
        writer.write_string(" (decimal) = 0x");
        writer.write_string(&vga_buffer::hex_to_string(self.cpu.ecx, &mut buf));
        writer.write_string(" (hex)\n");

        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
        writer.write_string("EDX");
        writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
        writer.write_string(" (data):        ");
        writer.write_string(&vga_buffer::int_to_string(self.cpu.edx as usize, &mut buf));
        writer.write_string(" (decimal) = 0x");
        writer.write_string(&vga_buffer::hex_to_string(self.cpu.edx, &mut buf));
        writer.write_string(" (hex)\n");

        if self.cpu.eax == 15 && self.cpu.ebx == 5 {
            writer.write_string("\n");
            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::LightCyan, vga_buffer::Color::Black);
            writer.write_string("Sample program explanation:\n");
            writer.color_code = vga_buffer::ColorCode::new(vga_buffer::Color::White, vga_buffer::Color::Black);
            writer.write_string("- Loaded 10 into EAX register\n");
            writer.write_string("- Loaded 5 into EBX register\n");
            writer.write_string("- Added EBX (5) to EAX (10)\n");
            writer.write_string("- Final result: EAX = 15 (which is 10 + 5)\n");
        }

        writer.write_string("\n");

        Ok(())
    }
}

pub fn execute_code_file(
    filename: &str,
    fs: &OsFileSystem,
    writer: &mut vga_buffer::Writer,
) -> Result<(), &'static str> {

    let file_data = fs.read_file(filename).map_err(|_| "File not found")?;
    let source_code = core::str::from_utf8(file_data).map_err(|_| "Invalid UTF-8 in file")?;

    let mut executor = CodeExecutor::new();
    let bytecode_len = executor.compile_code(source_code)?;

    if bytecode_len == 0 {
        return Err("No executable code found");
    }

    writer.write_string("Compiling and executing CODE program...\n");

    executor.execute(bytecode_len, writer)
}

pub fn create_sample_program() -> &'static str {
    "; sample CODE program - simple math boy 10 + 5 = 15
; This program demonstrates basic arithmetic
; EAX is the main register for results

mov eax, 10     ; Put the number 10 into register EAX
mov ebx, 5      ; Put the number 5 into register EBX
add eax, ebx    ; Add EBX (5) to EAX (10), result stored in EAX
halt            ; Stop the program

; After running:
; EAX will contain 15 (10 + 5)
; EBX will contain 5 (unchanged)
; The result 15 in hex is 0xF
"
}
