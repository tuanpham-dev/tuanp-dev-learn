// Part 7, first stage: decode and print a single tile.
// Copy it over src/main.rs to try it.
use std::fs;

struct Cpu {
    a: u8,
    x: u8,
    pc: u16,
    zero: bool,
    trace: bool,
    memory: [u8; 0x10000],
}

impl Cpu {
    fn new() -> Cpu {
        Cpu { a: 0, x: 0, pc: 0x8000, zero: false, trace: false, memory: [0; 0x10000] }
    }

    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    // Copy a program into memory, starting at $8000.
    fn load(&mut self, program: &[u8]) {
        for i in 0..program.len() {
            self.memory[0x8000 + i] = program[i];
        }
    }

    // Start where the cartridge says: the address stored at $FFFC.
    fn reset(&mut self) {
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

    fn run(&mut self) {
        loop {
            if self.trace {
                println!(
                    "{:04X}  {:02X}  A={:02X} X={:02X} Z={}",
                    self.pc, self.read(self.pc), self.a, self.x, self.zero as u8
                );
            }
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
                    let start = self.pc - 1;
                    let target = self.fetch_addr();
                    if target == start {
                        break; // jumping to itself: the program is done
                    }
                    self.pc = target;
                }
                0x00 => break, // BRK: stop
                _ => panic!("unknown opcode ${:02X}", opcode),
            }
        }
    }
}

const SHADES: [char; 4] = ['.', '+', 'o', '#'];

// The colour (0 to 3) of one pixel of one tile.
fn tile_pixel(chr: &[u8], tile: usize, x: usize, y: usize) -> u8 {
    let lo = chr[tile * 16 + y];
    let hi = chr[tile * 16 + y + 8];
    let shift = 7 - x;
    let lo_bit = (lo >> shift) & 1;
    let hi_bit = (hi >> shift) & 1;
    hi_bit * 2 + lo_bit
}

fn main() {
    let rom: Vec<u8> = fs::read("hello.nes").expect("could not read hello.nes");

    if rom[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
        panic!("this is not a NES cartridge file");
    }

    let prg_size = rom[4] as usize * 16384;
    let chr_size = rom[5] as usize * 8192;
    let prg = &rom[16..16 + prg_size];
    let chr = &rom[16 + prg_size..16 + prg_size + chr_size];
    println!("PRG ROM: {} bytes, CHR ROM: {} bytes", prg.len(), chr.len());

    let mut cpu = Cpu::new();
    cpu.load(prg);
    cpu.reset();
    cpu.run();

    for y in 0..8 {
        for x in 0..8 {
            let colour = tile_pixel(chr, 7, x, y);
            print!("{}", SHADES[colour as usize]);
        }
        println!();
    }
}
