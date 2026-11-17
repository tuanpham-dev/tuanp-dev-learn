// nesasm: a very small 6502 assembler that writes .nes cartridge files.
// It exists so that the cartridges used in this course can be read and
// changed as text. Usage:
//
//     cargo run -- ../roms/ticker.s ../roms/ticker.nes
//
// What it understands:
//
//     ; comment
//     NAME = $10              a named number
//     label:                  the address of whatever comes next
//         LDA #$20            an instruction, in the usual 6502 spelling
//         STA table,X
//     .byte 1, $02, "TEXT"    raw bytes; text becomes one byte per character
//     .font 3                 put a built-in font in the tiles (tile number =
//                             character code), drawn in colour 3;
//                             ".font 3, 1" puts it in the second set of 256 tiles
//     .tile $80               start drawing tile $80; the next 8 lines are its
//         ..3333..            rows, using . 1 2 3 for the four colours
//     .mirror vertical        cartridge wiring for the name tables
//     .vectors nmi, reset, irq    the three addresses stored at $FFFA
//
// The program always starts at $8000 and the cartridge always has 32K of
// PRG ROM and 8K of CHR ROM.

use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode { Imp, Acc, Imm, Zp, ZpX, ZpY, Abs, AbsX, AbsY, Ind, IndX, IndY, Rel }
use Mode::*;

// Every official instruction: nickname, addressing mode, opcode byte.
const OPCODES: [(&str, Mode, u8); 151] = [
    ("ADC", Imm, 0x69), ("ADC", Zp, 0x65), ("ADC", ZpX, 0x75), ("ADC", Abs, 0x6D),
    ("ADC", AbsX, 0x7D), ("ADC", AbsY, 0x79), ("ADC", IndX, 0x61), ("ADC", IndY, 0x71),
    ("AND", Imm, 0x29), ("AND", Zp, 0x25), ("AND", ZpX, 0x35), ("AND", Abs, 0x2D),
    ("AND", AbsX, 0x3D), ("AND", AbsY, 0x39), ("AND", IndX, 0x21), ("AND", IndY, 0x31),
    ("ASL", Acc, 0x0A), ("ASL", Zp, 0x06), ("ASL", ZpX, 0x16), ("ASL", Abs, 0x0E), ("ASL", AbsX, 0x1E),
    ("BCC", Rel, 0x90), ("BCS", Rel, 0xB0), ("BEQ", Rel, 0xF0), ("BMI", Rel, 0x30),
    ("BNE", Rel, 0xD0), ("BPL", Rel, 0x10), ("BVC", Rel, 0x50), ("BVS", Rel, 0x70),
    ("BIT", Zp, 0x24), ("BIT", Abs, 0x2C), ("BRK", Imp, 0x00),
    ("CLC", Imp, 0x18), ("CLD", Imp, 0xD8), ("CLI", Imp, 0x58), ("CLV", Imp, 0xB8),
    ("CMP", Imm, 0xC9), ("CMP", Zp, 0xC5), ("CMP", ZpX, 0xD5), ("CMP", Abs, 0xCD),
    ("CMP", AbsX, 0xDD), ("CMP", AbsY, 0xD9), ("CMP", IndX, 0xC1), ("CMP", IndY, 0xD1),
    ("CPX", Imm, 0xE0), ("CPX", Zp, 0xE4), ("CPX", Abs, 0xEC),
    ("CPY", Imm, 0xC0), ("CPY", Zp, 0xC4), ("CPY", Abs, 0xCC),
    ("DEC", Zp, 0xC6), ("DEC", ZpX, 0xD6), ("DEC", Abs, 0xCE), ("DEC", AbsX, 0xDE),
    ("DEX", Imp, 0xCA), ("DEY", Imp, 0x88),
    ("EOR", Imm, 0x49), ("EOR", Zp, 0x45), ("EOR", ZpX, 0x55), ("EOR", Abs, 0x4D),
    ("EOR", AbsX, 0x5D), ("EOR", AbsY, 0x59), ("EOR", IndX, 0x41), ("EOR", IndY, 0x51),
    ("INC", Zp, 0xE6), ("INC", ZpX, 0xF6), ("INC", Abs, 0xEE), ("INC", AbsX, 0xFE),
    ("INX", Imp, 0xE8), ("INY", Imp, 0xC8),
    ("JMP", Abs, 0x4C), ("JMP", Ind, 0x6C), ("JSR", Abs, 0x20),
    ("LDA", Imm, 0xA9), ("LDA", Zp, 0xA5), ("LDA", ZpX, 0xB5), ("LDA", Abs, 0xAD),
    ("LDA", AbsX, 0xBD), ("LDA", AbsY, 0xB9), ("LDA", IndX, 0xA1), ("LDA", IndY, 0xB1),
    ("LDX", Imm, 0xA2), ("LDX", Zp, 0xA6), ("LDX", ZpY, 0xB6), ("LDX", Abs, 0xAE), ("LDX", AbsY, 0xBE),
    ("LDY", Imm, 0xA0), ("LDY", Zp, 0xA4), ("LDY", ZpX, 0xB4), ("LDY", Abs, 0xAC), ("LDY", AbsX, 0xBC),
    ("LSR", Acc, 0x4A), ("LSR", Zp, 0x46), ("LSR", ZpX, 0x56), ("LSR", Abs, 0x4E), ("LSR", AbsX, 0x5E),
    ("NOP", Imp, 0xEA),
    ("ORA", Imm, 0x09), ("ORA", Zp, 0x05), ("ORA", ZpX, 0x15), ("ORA", Abs, 0x0D),
    ("ORA", AbsX, 0x1D), ("ORA", AbsY, 0x19), ("ORA", IndX, 0x01), ("ORA", IndY, 0x11),
    ("PHA", Imp, 0x48), ("PHP", Imp, 0x08), ("PLA", Imp, 0x68), ("PLP", Imp, 0x28),
    ("ROL", Acc, 0x2A), ("ROL", Zp, 0x26), ("ROL", ZpX, 0x36), ("ROL", Abs, 0x2E), ("ROL", AbsX, 0x3E),
    ("ROR", Acc, 0x6A), ("ROR", Zp, 0x66), ("ROR", ZpX, 0x76), ("ROR", Abs, 0x6E), ("ROR", AbsX, 0x7E),
    ("RTI", Imp, 0x40), ("RTS", Imp, 0x60),
    ("SBC", Imm, 0xE9), ("SBC", Zp, 0xE5), ("SBC", ZpX, 0xF5), ("SBC", Abs, 0xED),
    ("SBC", AbsX, 0xFD), ("SBC", AbsY, 0xF9), ("SBC", IndX, 0xE1), ("SBC", IndY, 0xF1),
    ("SEC", Imp, 0x38), ("SED", Imp, 0xF8), ("SEI", Imp, 0x78),
    ("STA", Zp, 0x85), ("STA", ZpX, 0x95), ("STA", Abs, 0x8D), ("STA", AbsX, 0x9D),
    ("STA", AbsY, 0x99), ("STA", IndX, 0x81), ("STA", IndY, 0x91),
    ("STX", Zp, 0x86), ("STX", ZpY, 0x96), ("STX", Abs, 0x8E),
    ("STY", Zp, 0x84), ("STY", ZpX, 0x94), ("STY", Abs, 0x8C),
    ("TAX", Imp, 0xAA), ("TAY", Imp, 0xA8), ("TSX", Imp, 0xBA),
    ("TXA", Imp, 0x8A), ("TXS", Imp, 0x9A), ("TYA", Imp, 0x98),
];

fn opcode(name: &str, mode: Mode) -> Option<u8> {
    for (n, m, byte) in OPCODES {
        if n == name && m == mode {
            return Some(byte);
        }
    }
    None
}

// A 5x7 font: for each character, seven rows of five pixels.
const FONT: [(char, [u8; 7]); 41] = [
    ('0', [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E]), ('1', [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E]),
    ('2', [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F]), ('3', [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E]),
    ('4', [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02]), ('5', [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E]),
    ('6', [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E]), ('7', [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08]),
    ('8', [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E]), ('9', [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C]),
    ('A', [0x0E, 0x11, 0x11, 0x11, 0x1F, 0x11, 0x11]), ('B', [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E]),
    ('C', [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E]), ('D', [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C]),
    ('E', [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F]), ('F', [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10]),
    ('G', [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F]), ('H', [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11]),
    ('I', [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E]), ('J', [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C]),
    ('K', [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11]), ('L', [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F]),
    ('M', [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11]), ('N', [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11]),
    ('O', [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E]), ('P', [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10]),
    ('Q', [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D]), ('R', [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11]),
    ('S', [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E]), ('T', [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04]),
    ('U', [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E]), ('V', [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04]),
    ('W', [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A]), ('X', [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11]),
    ('Y', [0x11, 0x11, 0x11, 0x0A, 0x04, 0x04, 0x04]), ('Z', [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F]),
    ('!', [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04]), ('-', [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00]),
    (':', [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00]), ('.', [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C]),
    ('?', [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04]),
];

struct Assembler {
    names: HashMap<String, i64>,
    prg: Vec<u8>,
    chr: Vec<u8>,
    pc: usize,
    last_pass: bool,
    vertical: bool,
    line_number: usize,
}

impl Assembler {
    fn fail(&self, message: &str) -> ! {
        eprintln!("line {}: {}", self.line_number, message);
        std::process::exit(1);
    }

    fn emit(&mut self, byte: u8) {
        if self.pc < 0x8000 || self.pc > 0xFFFF {
            self.fail("the program does not fit in $8000-$FFFF");
        }
        self.prg[self.pc - 0x8000] = byte;
        self.pc += 1;
    }

    // One number: $hex, %binary, decimal, 'c' or a name.
    // Returns None for a name that is not known yet (first pass only).
    fn term(&self, text: &str) -> Option<i64> {
        let text = text.trim();
        if let Some(hex) = text.strip_prefix('$') {
            return Some(i64::from_str_radix(hex, 16).unwrap_or_else(|_| self.fail("bad hex number")));
        }
        if let Some(bin) = text.strip_prefix('%') {
            return Some(i64::from_str_radix(bin, 2).unwrap_or_else(|_| self.fail("bad binary number")));
        }
        if text.len() == 3 && text.starts_with('\'') && text.ends_with('\'') {
            return Some(text.as_bytes()[1] as i64);
        }
        if let Ok(number) = text.parse::<i64>() {
            return Some(number);
        }
        match self.names.get(text) {
            Some(value) => Some(*value),
            None if self.last_pass => self.fail(&format!("unknown name '{}'", text)),
            None => None,
        }
    }

    // terms joined by + and -, optionally with < (low byte) or > (high byte) in front.
    fn value(&self, text: &str) -> Option<i64> {
        let text = text.trim();
        if let Some(rest) = text.strip_prefix('<') {
            return self.value(rest).map(|v| v & 0xFF);
        }
        if let Some(rest) = text.strip_prefix('>') {
            return self.value(rest).map(|v| (v >> 8) & 0xFF);
        }
        let mut total = 0;
        let mut sign = 1;
        let mut start = 0;
        let bytes = text.as_bytes();
        for i in 0..=bytes.len() {
            if i == bytes.len() || ((bytes[i] == b'+' || bytes[i] == b'-') && i > start) {
                total += sign * self.term(&text[start..i])?;
                if i < bytes.len() {
                    sign = if bytes[i] == b'+' { 1 } else { -1 };
                }
                start = i + 1;
            }
        }
        Some(total)
    }

    fn instruction(&mut self, name: &str, operand: &str) {
        let operand = operand.trim();
        let upper = operand.to_uppercase();

        // Work out the addressing mode from the spelling of the operand.
        let (mut mode, expr) = if operand.is_empty() {
            (Imp, "")
        } else if upper == "A" {
            (Acc, "")
        } else if let Some(rest) = operand.strip_prefix('#') {
            (Imm, rest)
        } else if upper.starts_with('(') && upper.ends_with(",X)") {
            (IndX, &operand[1..operand.len() - 3])
        } else if upper.starts_with('(') && upper.ends_with("),Y") {
            (IndY, &operand[1..operand.len() - 3])
        } else if upper.starts_with('(') && upper.ends_with(')') {
            (Ind, &operand[1..operand.len() - 1])
        } else if upper.ends_with(",X") {
            (AbsX, &operand[..operand.len() - 2])
        } else if upper.ends_with(",Y") {
            (AbsY, &operand[..operand.len() - 2])
        } else {
            (Abs, operand)
        };
        if mode == Imp && opcode(name, Imp).is_none() && opcode(name, Acc).is_some() {
            mode = Acc;
        }
        if mode == Abs && opcode(name, Rel).is_some() {
            mode = Rel;
        }

        let value = if expr.is_empty() { Some(0) } else { self.value(expr) };

        // Use the short zero-page form when the address is known to fit in one byte.
        if let Some(v) = value {
            let short = match mode {
                Abs => Zp,
                AbsX => ZpX,
                AbsY => ZpY,
                other => other,
            };
            if (0..256).contains(&v) && opcode(name, short).is_some() {
                mode = short;
            }
        }

        let byte = match opcode(name, mode) {
            Some(byte) => byte,
            None => self.fail(&format!("{} cannot be used with this kind of operand", name)),
        };
        let v = value.unwrap_or(0);
        self.emit(byte);
        match mode {
            Imp | Acc => {}
            Imm | Zp | ZpX | ZpY | IndX | IndY => {
                if self.last_pass && !(-128..256).contains(&v) {
                    self.fail("value does not fit in one byte");
                }
                self.emit(v as u8);
            }
            Rel => {
                let offset = v - (self.pc as i64 + 1);
                if self.last_pass && !(-128..128).contains(&offset) {
                    self.fail("branch target is too far away");
                }
                self.emit(offset as u8);
            }
            Abs | AbsX | AbsY | Ind => {
                self.emit(v as u8);
                self.emit((v >> 8) as u8);
            }
        }
    }

    fn directive(&mut self, name: &str, rest: &str, lines: &[&str], index: &mut usize) {
        match name {
            ".byte" => {
                for part in split_commas(rest) {
                    let part = part.trim();
                    if part.starts_with('"') && part.ends_with('"') && part.len() >= 2 {
                        for ch in part[1..part.len() - 1].bytes() {
                            self.emit(ch);
                        }
                    } else {
                        let v = self.value(part).unwrap_or(0);
                        self.emit(v as u8);
                    }
                }
            }
            ".font" => {
                let parts = split_commas(rest);
                let colour = self.value(parts[0]).unwrap_or(3) as u8;
                let table = if parts.len() > 1 { self.value(parts[1]).unwrap_or(0) as usize } else { 0 };
                for (ch, rows) in FONT {
                    for y in 0..7 {
                        let bits = rows[y] << 2;
                        let at = (table * 256 + ch as usize) * 16 + y;
                        self.chr[at] = if colour & 1 != 0 { bits } else { 0 };
                        self.chr[at + 8] = if colour & 2 != 0 { bits } else { 0 };
                    }
                }
            }
            ".tile" => {
                let number = self.value(rest).unwrap_or(0) as usize;
                for y in 0..8 {
                    *index += 1;
                    if *index >= lines.len() {
                        self.fail("a tile needs 8 rows");
                    }
                    self.line_number = *index + 1;
                    let row = strip_comment(lines[*index]).trim();
                    if row.len() != 8 {
                        self.fail("a tile row must be 8 characters");
                    }
                    let mut lo = 0u8;
                    let mut hi = 0u8;
                    for (x, ch) in row.chars().enumerate() {
                        let colour = match ch {
                            '.' => 0,
                            '1' => 1,
                            '2' => 2,
                            '3' => 3,
                            _ => self.fail("tile rows use only . 1 2 3"),
                        };
                        lo |= (colour & 1) << (7 - x);
                        hi |= (colour >> 1) << (7 - x);
                    }
                    self.chr[number * 16 + y] = lo;
                    self.chr[number * 16 + y + 8] = hi;
                }
            }
            ".mirror" => self.vertical = rest.trim() == "vertical",
            ".vectors" => {
                let parts = split_commas(rest);
                if parts.len() != 3 {
                    self.fail(".vectors needs three addresses: nmi, reset, irq");
                }
                for (i, part) in parts.iter().enumerate() {
                    let v = self.value(part).unwrap_or(0);
                    self.prg[0x7FFA + i * 2] = v as u8;
                    self.prg[0x7FFB + i * 2] = (v >> 8) as u8;
                }
            }
            _ => self.fail(&format!("unknown directive {}", name)),
        }
    }

    fn pass(&mut self, source: &str) {
        self.pc = 0x8000;
        let lines: Vec<&str> = source.lines().collect();
        let mut index = 0;
        while index < lines.len() {
            self.line_number = index + 1;
            let mut line = strip_comment(lines[index]).trim();

            // NAME = value
            if let Some((name, value)) = line.split_once('=') {
                if !line.starts_with('.') && !name.contains('"') && !name.trim().contains(' ') {
                    if let Some(v) = self.value(value) {
                        self.names.insert(name.trim().to_string(), v);
                    }
                    index += 1;
                    continue;
                }
            }
            // label:
            if let Some((label, rest)) = line.split_once(':') {
                if !label.contains(' ') && !label.contains('"') && !label.contains('\'') && !label.starts_with('.') {
                    self.names.insert(label.to_string(), self.pc as i64);
                    line = rest.trim();
                }
            }
            if !line.is_empty() {
                let (word, rest) = match line.split_once(char::is_whitespace) {
                    Some((word, rest)) => (word, rest),
                    None => (line, ""),
                };
                if word.starts_with('.') {
                    self.directive(&word.to_lowercase(), rest, &lines, &mut index);
                } else {
                    self.instruction(&word.to_uppercase(), rest);
                }
            }
            index += 1;
        }
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_text = false;
    for (i, ch) in line.char_indices() {
        if ch == '"' {
            in_text = !in_text;
        }
        if ch == ';' && !in_text {
            return &line[..i];
        }
    }
    line
}

fn split_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut in_text = false;
    let mut start = 0;
    for (i, ch) in text.char_indices() {
        if ch == '"' {
            in_text = !in_text;
        }
        if ch == ',' && !in_text {
            parts.push(&text[start..i]);
            start = i + 1;
        }
    }
    if !text[start..].trim().is_empty() {
        parts.push(&text[start..]);
    }
    parts
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: nesasm <source.s> <output.nes>");
        std::process::exit(1);
    }
    let source = fs::read_to_string(&args[1]).expect("could not read the source file");

    let mut asm = Assembler {
        names: HashMap::new(),
        prg: vec![0; 32768],
        chr: vec![0; 8192],
        pc: 0x8000,
        last_pass: false,
        vertical: false,
        line_number: 0,
    };
    // Twice: the first pass learns where every label is, the second uses them.
    asm.pass(&source);
    asm.last_pass = true;
    asm.pass(&source);

    let mut rom: Vec<u8> = vec![0x4E, 0x45, 0x53, 0x1A, 2, 1, asm.vertical as u8];
    rom.resize(16, 0);
    rom.extend(&asm.prg);
    rom.extend(&asm.chr);
    fs::write(&args[2], &rom).expect("could not write the output file");
    println!("wrote {} ({} bytes of program used)", args[2], asm.pc - 0x8000);
}
