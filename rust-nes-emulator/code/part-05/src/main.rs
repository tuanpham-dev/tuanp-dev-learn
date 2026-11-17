struct Cpu {
    a: u8,
    x: u8,
    pc: u16,
    zero: bool,
    memory: [u8; 0x10000],
}

impl Cpu {
    fn new() -> Cpu {
        Cpu { a: 0, x: 0, pc: 0x8000, zero: false, memory: [0; 0x10000] }
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
            println!(
                "{:04X}  {:02X}  A={:02X} X={:02X} Z={}",
                self.pc, self.read(self.pc), self.a, self.x, self.zero as u8
            );
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
                0x00 => break, // BRK: stop
                _ => panic!("unknown opcode ${:02X}", opcode),
            }
        }
    }
}

fn main() {
    let program: [u8; 17] = [
        0xA2, 0x00,       // 8000  LDX #$00
        0xBD, 0x0E, 0x80, // 8002  LDA $800E,X
        0x9D, 0x00, 0x02, // 8005  STA $0200,X
        0xE8,             // 8008  INX
        0xE0, 0x03,       // 8009  CPX #$03
        0xD0, 0xF5,       // 800B  BNE $8002
        0x00,             // 800D  BRK
        0x4E, 0x45, 0x53, // 800E  the data: "NES"
    ];

    let mut cpu = Cpu::new();
    cpu.load(&program);
    cpu.run();

    print!("$0200: ");
    for addr in 0x0200..0x0203 {
        print!("{}", cpu.read(addr) as char);
    }
    println!();
}
