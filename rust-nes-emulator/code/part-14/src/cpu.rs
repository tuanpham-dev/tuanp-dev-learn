use crate::cartridge::Cartridge;
use crate::ppu::Ppu;

// The ways an instruction can find the value it works on.
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

// The 56 instructions of the 6502, by nickname.
#[derive(Clone, Copy)]
pub enum Op {
    Adc, And, Asl, Bcc, Bcs, Beq, Bit, Bmi, Bne, Bpl, Brk, Bvc, Bvs, Clc,
    Cld, Cli, Clv, Cmp, Cpx, Cpy, Dec, Dex, Dey, Eor, Inc, Inx, Iny, Jmp,
    Jsr, Lda, Ldx, Ldy, Lsr, Nop, Ora, Pha, Php, Pla, Plp, Rol, Ror, Rti,
    Rts, Sbc, Sec, Sed, Sei, Sta, Stx, Sty, Tax, Tay, Tsx, Txa, Txs, Tya,
}

// What each opcode byte means: which instruction, which addressing mode,
// and how many clock cycles it takes.
fn decode(opcode: u8) -> (Op, Mode, u64) {
    use Mode::*;
    use Op::*;
    match opcode {
        0x69 => (Adc, Immediate, 2),
        0x65 => (Adc, ZeroPage, 3),
        0x75 => (Adc, ZeroPageX, 4),
        0x6D => (Adc, Absolute, 4),
        0x7D => (Adc, AbsoluteX, 4),
        0x79 => (Adc, AbsoluteY, 4),
        0x61 => (Adc, IndirectX, 6),
        0x71 => (Adc, IndirectY, 5),

        0x29 => (And, Immediate, 2),
        0x25 => (And, ZeroPage, 3),
        0x35 => (And, ZeroPageX, 4),
        0x2D => (And, Absolute, 4),
        0x3D => (And, AbsoluteX, 4),
        0x39 => (And, AbsoluteY, 4),
        0x21 => (And, IndirectX, 6),
        0x31 => (And, IndirectY, 5),

        0x0A => (Asl, Accumulator, 2),
        0x06 => (Asl, ZeroPage, 5),
        0x16 => (Asl, ZeroPageX, 6),
        0x0E => (Asl, Absolute, 6),
        0x1E => (Asl, AbsoluteX, 7),

        0x90 => (Bcc, Relative, 2),
        0xB0 => (Bcs, Relative, 2),
        0xF0 => (Beq, Relative, 2),
        0x30 => (Bmi, Relative, 2),
        0xD0 => (Bne, Relative, 2),
        0x10 => (Bpl, Relative, 2),
        0x50 => (Bvc, Relative, 2),
        0x70 => (Bvs, Relative, 2),

        0x24 => (Bit, ZeroPage, 3),
        0x2C => (Bit, Absolute, 4),

        0x00 => (Brk, Implied, 7),

        0x18 => (Clc, Implied, 2),
        0xD8 => (Cld, Implied, 2),
        0x58 => (Cli, Implied, 2),
        0xB8 => (Clv, Implied, 2),

        0xC9 => (Cmp, Immediate, 2),
        0xC5 => (Cmp, ZeroPage, 3),
        0xD5 => (Cmp, ZeroPageX, 4),
        0xCD => (Cmp, Absolute, 4),
        0xDD => (Cmp, AbsoluteX, 4),
        0xD9 => (Cmp, AbsoluteY, 4),
        0xC1 => (Cmp, IndirectX, 6),
        0xD1 => (Cmp, IndirectY, 5),

        0xE0 => (Cpx, Immediate, 2),
        0xE4 => (Cpx, ZeroPage, 3),
        0xEC => (Cpx, Absolute, 4),

        0xC0 => (Cpy, Immediate, 2),
        0xC4 => (Cpy, ZeroPage, 3),
        0xCC => (Cpy, Absolute, 4),

        0xC6 => (Dec, ZeroPage, 5),
        0xD6 => (Dec, ZeroPageX, 6),
        0xCE => (Dec, Absolute, 6),
        0xDE => (Dec, AbsoluteX, 7),

        0xCA => (Dex, Implied, 2),
        0x88 => (Dey, Implied, 2),

        0x49 => (Eor, Immediate, 2),
        0x45 => (Eor, ZeroPage, 3),
        0x55 => (Eor, ZeroPageX, 4),
        0x4D => (Eor, Absolute, 4),
        0x5D => (Eor, AbsoluteX, 4),
        0x59 => (Eor, AbsoluteY, 4),
        0x41 => (Eor, IndirectX, 6),
        0x51 => (Eor, IndirectY, 5),

        0xE6 => (Inc, ZeroPage, 5),
        0xF6 => (Inc, ZeroPageX, 6),
        0xEE => (Inc, Absolute, 6),
        0xFE => (Inc, AbsoluteX, 7),

        0xE8 => (Inx, Implied, 2),
        0xC8 => (Iny, Implied, 2),

        0x4C => (Jmp, Absolute, 3),
        0x6C => (Jmp, Indirect, 5),
        0x20 => (Jsr, Absolute, 6),

        0xA9 => (Lda, Immediate, 2),
        0xA5 => (Lda, ZeroPage, 3),
        0xB5 => (Lda, ZeroPageX, 4),
        0xAD => (Lda, Absolute, 4),
        0xBD => (Lda, AbsoluteX, 4),
        0xB9 => (Lda, AbsoluteY, 4),
        0xA1 => (Lda, IndirectX, 6),
        0xB1 => (Lda, IndirectY, 5),

        0xA2 => (Ldx, Immediate, 2),
        0xA6 => (Ldx, ZeroPage, 3),
        0xB6 => (Ldx, ZeroPageY, 4),
        0xAE => (Ldx, Absolute, 4),
        0xBE => (Ldx, AbsoluteY, 4),

        0xA0 => (Ldy, Immediate, 2),
        0xA4 => (Ldy, ZeroPage, 3),
        0xB4 => (Ldy, ZeroPageX, 4),
        0xAC => (Ldy, Absolute, 4),
        0xBC => (Ldy, AbsoluteX, 4),

        0x4A => (Lsr, Accumulator, 2),
        0x46 => (Lsr, ZeroPage, 5),
        0x56 => (Lsr, ZeroPageX, 6),
        0x4E => (Lsr, Absolute, 6),
        0x5E => (Lsr, AbsoluteX, 7),

        0xEA => (Nop, Implied, 2),

        0x09 => (Ora, Immediate, 2),
        0x05 => (Ora, ZeroPage, 3),
        0x15 => (Ora, ZeroPageX, 4),
        0x0D => (Ora, Absolute, 4),
        0x1D => (Ora, AbsoluteX, 4),
        0x19 => (Ora, AbsoluteY, 4),
        0x01 => (Ora, IndirectX, 6),
        0x11 => (Ora, IndirectY, 5),

        0x48 => (Pha, Implied, 3),
        0x08 => (Php, Implied, 3),
        0x68 => (Pla, Implied, 4),
        0x28 => (Plp, Implied, 4),

        0x2A => (Rol, Accumulator, 2),
        0x26 => (Rol, ZeroPage, 5),
        0x36 => (Rol, ZeroPageX, 6),
        0x2E => (Rol, Absolute, 6),
        0x3E => (Rol, AbsoluteX, 7),

        0x6A => (Ror, Accumulator, 2),
        0x66 => (Ror, ZeroPage, 5),
        0x76 => (Ror, ZeroPageX, 6),
        0x6E => (Ror, Absolute, 6),
        0x7E => (Ror, AbsoluteX, 7),

        0x40 => (Rti, Implied, 6),
        0x60 => (Rts, Implied, 6),

        0xE9 => (Sbc, Immediate, 2),
        0xE5 => (Sbc, ZeroPage, 3),
        0xF5 => (Sbc, ZeroPageX, 4),
        0xED => (Sbc, Absolute, 4),
        0xFD => (Sbc, AbsoluteX, 4),
        0xF9 => (Sbc, AbsoluteY, 4),
        0xE1 => (Sbc, IndirectX, 6),
        0xF1 => (Sbc, IndirectY, 5),

        0x38 => (Sec, Implied, 2),
        0xF8 => (Sed, Implied, 2),
        0x78 => (Sei, Implied, 2),

        0x85 => (Sta, ZeroPage, 3),
        0x95 => (Sta, ZeroPageX, 4),
        0x8D => (Sta, Absolute, 4),
        0x9D => (Sta, AbsoluteX, 5),
        0x99 => (Sta, AbsoluteY, 5),
        0x81 => (Sta, IndirectX, 6),
        0x91 => (Sta, IndirectY, 6),

        0x86 => (Stx, ZeroPage, 3),
        0x96 => (Stx, ZeroPageY, 4),
        0x8E => (Stx, Absolute, 4),

        0x84 => (Sty, ZeroPage, 3),
        0x94 => (Sty, ZeroPageX, 4),
        0x8C => (Sty, Absolute, 4),

        0xAA => (Tax, Implied, 2),
        0xA8 => (Tay, Implied, 2),
        0xBA => (Tsx, Implied, 2),
        0x8A => (Txa, Implied, 2),
        0x9A => (Txs, Implied, 2),
        0x98 => (Tya, Implied, 2),

        _ => panic!("unknown opcode ${:02X}", opcode),
    }
}

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,

    pub carry: bool,
    pub zero: bool,
    pub interrupt_off: bool,
    pub decimal: bool,
    pub overflow: bool,
    pub negative: bool,

    pub cycles: u64,
    pub stopped: bool,

    ram: [u8; 0x800],
    prg: Vec<u8>,
    pub ppu: Ppu,
}

impl Cpu {
    pub fn new(cart: Cartridge) -> Cpu {
        let mut ppu = Ppu::new();
        ppu.load_chr(&cart.chr);

        Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            carry: false,
            zero: false,
            interrupt_off: true,
            decimal: false,
            overflow: false,
            negative: false,
            cycles: 0,
            stopped: false,
            ram: [0; 0x800],
            prg: cart.prg,
            ppu,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 0x800],
            0x8000..=0xFFFF => self.prg[(addr as usize - 0x8000) % self.prg.len()],
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 0x800] = value,
            0x2006 => self.ppu.write_addr(value),
            0x2007 => self.ppu.write_data(value),
            _ => {}
        }
    }

    fn read_addr(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        hi * 256 + lo
    }

    // Start where the cartridge says: the address stored at $FFFC.
    pub fn reset(&mut self) {
        self.pc = self.read_addr(0xFFFC);
        self.sp = 0xFD;
        self.interrupt_off = true;
    }

    // Read the byte at pc, then move pc past it.
    fn fetch(&mut self) -> u8 {
        let byte = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    // Read a two-byte address: low byte first, then high byte.
    fn fetch_addr(&mut self) -> u16 {
        let lo = self.fetch() as u16;
        let hi = self.fetch() as u16;
        hi * 256 + lo
    }

    // Work out the address an instruction will use, from its addressing mode.
    fn operand_address(&mut self, mode: Mode) -> u16 {
        match mode {
            Mode::Implied | Mode::Accumulator => 0,
            Mode::Immediate => {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                addr
            }
            Mode::ZeroPage => self.fetch() as u16,
            Mode::ZeroPageX => self.fetch().wrapping_add(self.x) as u16,
            Mode::ZeroPageY => self.fetch().wrapping_add(self.y) as u16,
            Mode::Absolute => self.fetch_addr(),
            Mode::AbsoluteX => {
                let base = self.fetch_addr();
                base.wrapping_add(self.x as u16)
            }
            Mode::AbsoluteY => {
                let base = self.fetch_addr();
                base.wrapping_add(self.y as u16)
            }
            Mode::Indirect => {
                // The address of the address. A famous 6502 bug: the second
                // byte is fetched from the same page as the first.
                let ptr = self.fetch_addr();
                let lo = self.read(ptr) as u16;
                let hi = self.read((ptr & 0xFF00) | (ptr.wrapping_add(1) & 0x00FF)) as u16;
                hi * 256 + lo
            }
            Mode::IndirectX => {
                let ptr = self.fetch().wrapping_add(self.x);
                let lo = self.read(ptr as u16) as u16;
                let hi = self.read(ptr.wrapping_add(1) as u16) as u16;
                hi * 256 + lo
            }
            Mode::IndirectY => {
                let ptr = self.fetch();
                let lo = self.read(ptr as u16) as u16;
                let hi = self.read(ptr.wrapping_add(1) as u16) as u16;
                (hi * 256 + lo).wrapping_add(self.y as u16)
            }
            Mode::Relative => {
                let offset = self.fetch() as i8;
                self.pc.wrapping_add_signed(offset as i16)
            }
        }
    }

    fn set_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = value & 0x80 != 0;
    }

    // ---- the stack: page $01, growing downwards ----

    fn push(&mut self, value: u8) {
        self.write(0x0100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(0x0100 + self.sp as u16)
    }

    fn push_addr(&mut self, addr: u16) {
        self.push((addr >> 8) as u8);
        self.push(addr as u8);
    }

    fn pop_addr(&mut self) -> u16 {
        let lo = self.pop() as u16;
        let hi = self.pop() as u16;
        hi * 256 + lo
    }

    // ---- the flags, packed into one byte and back ----

    pub fn status(&self) -> u8 {
        (self.carry as u8)
            | (self.zero as u8) << 1
            | (self.interrupt_off as u8) << 2
            | (self.decimal as u8) << 3
            | 1 << 5
            | (self.overflow as u8) << 6
            | (self.negative as u8) << 7
    }

    fn set_status(&mut self, value: u8) {
        self.carry = value & 0x01 != 0;
        self.zero = value & 0x02 != 0;
        self.interrupt_off = value & 0x04 != 0;
        self.decimal = value & 0x08 != 0;
        self.overflow = value & 0x40 != 0;
        self.negative = value & 0x80 != 0;
    }

    // ---- helpers shared by several instructions ----

    fn add(&mut self, value: u8) {
        let sum = self.a as u16 + value as u16 + self.carry as u16;
        let result = sum as u8;
        self.carry = sum > 0xFF;
        self.overflow = (self.a ^ result) & (value ^ result) & 0x80 != 0;
        self.a = result;
        self.set_flags(result);
    }

    fn compare(&mut self, register: u8, value: u8) {
        self.carry = register >= value;
        self.set_flags(register.wrapping_sub(value));
    }

    fn branch(&mut self, condition: bool, target: u16) {
        if condition {
            self.pc = target;
        }
    }

    // Shifts and rotates work on A or on a memory slot, depending on the mode.
    fn shift(&mut self, op: Op, mode: Mode, addr: u16) {
        let value = if mode == Mode::Accumulator { self.a } else { self.read(addr) };
        let carry_in = self.carry as u8;
        let result = match op {
            Op::Asl => {
                self.carry = value & 0x80 != 0;
                value << 1
            }
            Op::Lsr => {
                self.carry = value & 0x01 != 0;
                value >> 1
            }
            Op::Rol => {
                self.carry = value & 0x80 != 0;
                (value << 1) | carry_in
            }
            _ => {
                self.carry = value & 0x01 != 0;
                (value >> 1) | (carry_in << 7)
            }
        };
        self.set_flags(result);
        if mode == Mode::Accumulator {
            self.a = result;
        } else {
            self.write(addr, result);
        }
    }

    // Run one instruction.
    pub fn step(&mut self) {
        let start = self.pc;
        let opcode = self.fetch();
        let (op, mode, cycles) = decode(opcode);
        self.cycles += cycles;
        let addr = self.operand_address(mode);

        match op {
            // loading and storing
            Op::Lda => {
                self.a = self.read(addr);
                self.set_flags(self.a);
            }
            Op::Ldx => {
                self.x = self.read(addr);
                self.set_flags(self.x);
            }
            Op::Ldy => {
                self.y = self.read(addr);
                self.set_flags(self.y);
            }
            Op::Sta => self.write(addr, self.a),
            Op::Stx => self.write(addr, self.x),
            Op::Sty => self.write(addr, self.y),

            // copying between registers
            Op::Tax => {
                self.x = self.a;
                self.set_flags(self.x);
            }
            Op::Tay => {
                self.y = self.a;
                self.set_flags(self.y);
            }
            Op::Txa => {
                self.a = self.x;
                self.set_flags(self.a);
            }
            Op::Tya => {
                self.a = self.y;
                self.set_flags(self.a);
            }
            Op::Tsx => {
                self.x = self.sp;
                self.set_flags(self.x);
            }
            Op::Txs => self.sp = self.x,

            // counting up and down
            Op::Inx => {
                self.x = self.x.wrapping_add(1);
                self.set_flags(self.x);
            }
            Op::Iny => {
                self.y = self.y.wrapping_add(1);
                self.set_flags(self.y);
            }
            Op::Dex => {
                self.x = self.x.wrapping_sub(1);
                self.set_flags(self.x);
            }
            Op::Dey => {
                self.y = self.y.wrapping_sub(1);
                self.set_flags(self.y);
            }
            Op::Inc => {
                let value = self.read(addr).wrapping_add(1);
                self.write(addr, value);
                self.set_flags(value);
            }
            Op::Dec => {
                let value = self.read(addr).wrapping_sub(1);
                self.write(addr, value);
                self.set_flags(value);
            }

            // arithmetic
            Op::Adc => {
                let value = self.read(addr);
                self.add(value);
            }
            Op::Sbc => {
                let value = self.read(addr);
                self.add(!value);
            }
            Op::Cmp => {
                let value = self.read(addr);
                self.compare(self.a, value);
            }
            Op::Cpx => {
                let value = self.read(addr);
                self.compare(self.x, value);
            }
            Op::Cpy => {
                let value = self.read(addr);
                self.compare(self.y, value);
            }

            // bit by bit
            Op::And => {
                self.a &= self.read(addr);
                self.set_flags(self.a);
            }
            Op::Ora => {
                self.a |= self.read(addr);
                self.set_flags(self.a);
            }
            Op::Eor => {
                self.a ^= self.read(addr);
                self.set_flags(self.a);
            }
            Op::Bit => {
                let value = self.read(addr);
                self.zero = self.a & value == 0;
                self.overflow = value & 0x40 != 0;
                self.negative = value & 0x80 != 0;
            }
            Op::Asl | Op::Lsr | Op::Rol | Op::Ror => self.shift(op, mode, addr),

            // branches
            Op::Bcc => self.branch(!self.carry, addr),
            Op::Bcs => self.branch(self.carry, addr),
            Op::Bne => self.branch(!self.zero, addr),
            Op::Beq => self.branch(self.zero, addr),
            Op::Bpl => self.branch(!self.negative, addr),
            Op::Bmi => self.branch(self.negative, addr),
            Op::Bvc => self.branch(!self.overflow, addr),
            Op::Bvs => self.branch(self.overflow, addr),

            // jumps and subroutines
            Op::Jmp => {
                if addr == start {
                    self.stopped = true; // jumping to itself: the program is done
                }
                self.pc = addr;
            }
            Op::Jsr => {
                self.push_addr(self.pc.wrapping_sub(1));
                self.pc = addr;
            }
            Op::Rts => self.pc = self.pop_addr().wrapping_add(1),
            Op::Rti => {
                let status = self.pop();
                self.set_status(status);
                self.pc = self.pop_addr();
            }

            // the stack
            Op::Pha => self.push(self.a),
            Op::Php => self.push(self.status() | 0x10),
            Op::Pla => {
                self.a = self.pop();
                self.set_flags(self.a);
            }
            Op::Plp => {
                let status = self.pop();
                self.set_status(status);
            }

            // setting and clearing flags
            Op::Clc => self.carry = false,
            Op::Sec => self.carry = true,
            Op::Cli => self.interrupt_off = false,
            Op::Sei => self.interrupt_off = true,
            Op::Cld => self.decimal = false,
            Op::Sed => self.decimal = true,
            Op::Clv => self.overflow = false,

            Op::Nop => {}
            Op::Brk => self.stopped = true,
        }
    }
}
