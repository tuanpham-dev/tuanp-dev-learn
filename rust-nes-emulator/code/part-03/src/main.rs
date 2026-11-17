struct Cpu {
    a: u8,
    x: u8,
    pc: u16,
    memory: [u8; 0x10000],
}

impl Cpu {
    fn new() -> Cpu {
        Cpu { a: 0, x: 0, pc: 0x8000, memory: [0; 0x10000] }
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

    fn run(&mut self) {
        loop {
            let opcode = self.fetch();

            match opcode {
                0xA9 => self.a = self.fetch(), // LDA: load the next byte into A
                0x8D => {
                    // STA: store A at the address that follows
                    let addr = self.fetch_addr();
                    self.write(addr, self.a);
                }
                0xAA => self.x = self.a,                 // TAX: copy A into X
                0xE8 => self.x = self.x.wrapping_add(1), // INX: add one to X
                0x00 => break,                           // BRK: stop
                _ => panic!("unknown opcode ${:02X}", opcode),
            }
        }
    }
}

fn main() {
    let program: [u8; 11] = [
        0xA9, 0x2A,       // LDA #$2A
        0x8D, 0x00, 0x02, // STA $0200
        0xA9, 0x07,       // LDA #$07
        0x8D, 0x01, 0x02, // STA $0201
        0x00,             // BRK
    ];

    let mut cpu = Cpu::new();
    cpu.load(&program);
    cpu.run();

    println!("A = ${:02X}  X = ${:02X}", cpu.a, cpu.x);
    println!("$0200: {:02X} {:02X} {:02X}", cpu.read(0x0200), cpu.read(0x0201), cpu.read(0x0202));
}
