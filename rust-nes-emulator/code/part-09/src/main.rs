use minifb::{Key, Scale, Window, WindowOptions};
use std::fs;

// The NES screen, in pixels.
const WIDTH: usize = 256;
const HEIGHT: usize = 240;

// Four shades of grey, as 0xRRGGBB, for the four colours of a tile.
const GREYS: [u32; 4] = [0x000000, 0x555555, 0xAAAAAA, 0xFFFFFF];

// The colour (0 to 3) of one pixel of one tile.
fn tile_pixel(chr: &[u8], tile: usize, x: usize, y: usize) -> u8 {
    let lo = chr[tile * 16 + y];
    let hi = chr[tile * 16 + y + 8];
    let shift = 7 - x;
    let lo_bit = (lo >> shift) & 1;
    let hi_bit = (hi >> shift) & 1;
    hi_bit * 2 + lo_bit
}

struct Ppu {
    vram: [u8; 0x4000],
    addr: u16,
    frame: Vec<u32>,
}

impl Ppu {
    fn new() -> Ppu {
        Ppu { vram: [0; 0x4000], addr: 0, frame: vec![0; WIDTH * HEIGHT] }
    }

    // The tiles live at the start of the PPU's memory.
    fn load_chr(&mut self, chr: &[u8]) {
        for i in 0..chr.len() {
            self.vram[i] = chr[i];
        }
    }

    // A write to $2006: one more byte of the address, high byte first.
    fn write_addr(&mut self, value: u8) {
        self.addr = (self.addr << 8) | value as u16;
    }

    // A write to $2007: store the byte, then step to the next address.
    fn write_data(&mut self, value: u8) {
        self.vram[self.addr as usize % 0x4000] = value;
        self.addr = self.addr.wrapping_add(1);
    }

    // Paint the picture into `frame`: for every pixel, which tile is it in,
    // and which pixel of that tile?
    fn draw(&mut self) {
        for py in 0..HEIGHT {
            for px in 0..WIDTH {
                let tile = self.vram[0x2000 + (py / 8) * 32 + px / 8] as usize;
                let colour = tile_pixel(&self.vram, tile, px % 8, py % 8);
                self.frame[py * WIDTH + px] = GREYS[colour as usize];
            }
        }
    }
}

struct Cpu {
    a: u8,
    x: u8,
    pc: u16,
    zero: bool,
    trace: bool,
    memory: [u8; 0x10000],
    ppu: Ppu,
}

impl Cpu {
    fn new() -> Cpu {
        Cpu { a: 0, x: 0, pc: 0x8000, zero: false, trace: false, memory: [0; 0x10000], ppu: Ppu::new() }
    }

    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x2006 => self.ppu.write_addr(value),
            0x2007 => self.ppu.write_data(value),
            _ => self.memory[addr as usize] = value,
        }
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

fn main() {
    let rom: Vec<u8> = fs::read("hello.nes").expect("could not read hello.nes");

    if rom[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
        panic!("this is not a NES cartridge file");
    }

    let prg_size = rom[4] as usize * 16384;
    let chr_size = rom[5] as usize * 8192;
    let prg = &rom[16..16 + prg_size];
    let chr = &rom[16 + prg_size..16 + prg_size + chr_size];

    let mut cpu = Cpu::new();
    cpu.load(prg);
    cpu.ppu.load_chr(chr);
    cpu.reset();
    cpu.run();

    cpu.ppu.draw();

    let mut options = WindowOptions::default();
    options.scale = Scale::X2;
    let mut window = Window::new("NES", WIDTH, HEIGHT, options).expect("could not open a window");
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&cpu.ppu.frame, WIDTH, HEIGHT)
            .expect("could not draw to the window");
    }
}
