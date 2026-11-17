use crate::cartridge::Cartridge;
use crate::ppu::Ppu;

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub pc: u16,
    pub zero: bool,
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
            pc: 0,
            zero: false,
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

    // Start where the cartridge says: the address stored at $FFFC.
    pub fn reset(&mut self) {
        let lo = self.read(0xFFFC) as u16;
        let hi = self.read(0xFFFD) as u16;
        self.pc = hi * 256 + lo;
    }

    // Read the byte at pc, then move pc past it.
    fn fetch(&mut self) -> u8 {
        let byte = self.read(self.pc);
        self.pc += 1;
        byte
    }

    // Read a two-byte address: low byte first, then high byte.
    fn fetch_addr(&mut self) -> u16 {
        let lo = self.fetch() as u16;
        let hi = self.fetch() as u16;
        hi * 256 + lo
    }

    // Read a two-byte address, then add X to it.
    fn fetch_addr_x(&mut self) -> u16 {
        let base = self.fetch_addr();
        base.wrapping_add(self.x as u16)
    }

    // Run one instruction.
    pub fn step(&mut self) {
        let start = self.pc;
        let opcode = self.fetch();

        match opcode {
            0xA9 => {
                // LDA: load the next byte into A
                self.a = self.fetch();
                self.zero = self.a == 0;
            }
            0xA2 => {
                // LDX: load the next byte into X
                self.x = self.fetch();
                self.zero = self.x == 0;
            }
            0x8D => {
                // STA: store A at the address that follows
                let addr = self.fetch_addr();
                self.write(addr, self.a);
            }
            0xBD => {
                // LDA indexed: load A from (address + X)
                let addr = self.fetch_addr_x();
                self.a = self.read(addr);
                self.zero = self.a == 0;
            }
            0x9D => {
                // STA indexed: store A at (address + X)
                let addr = self.fetch_addr_x();
                self.write(addr, self.a);
            }
            0xAA => {
                // TAX: copy A into X
                self.x = self.a;
                self.zero = self.x == 0;
            }
            0xE8 => {
                // INX: add one to X
                self.x = self.x.wrapping_add(1);
                self.zero = self.x == 0;
            }
            0xE0 => {
                // CPX: compare X with the next byte
                let value = self.fetch();
                self.zero = self.x == value;
            }
            0xD0 => {
                // BNE: branch if the last result was not zero
                let offset = self.fetch() as i8;
                if !self.zero {
                    self.pc = self.pc.wrapping_add_signed(offset as i16);
                }
            }
            0x4C => {
                // JMP: jump to the address that follows
                let target = self.fetch_addr();
                if target == start {
                    self.stopped = true; // jumping to itself: the program is done
                }
                self.pc = target;
            }
            0x00 => self.stopped = true, // BRK: stop
            _ => {
                println!("unknown opcode ${:02X} at ${:04X}", opcode, start);
                self.stopped = true;
            }
        }
    }
}
