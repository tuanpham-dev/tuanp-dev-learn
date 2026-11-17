struct Cpu {
    a: u8,
    x: u8,
    pc: usize,
}

impl Cpu {
    fn new() -> Cpu {
        Cpu { a: 0, x: 0, pc: 0 }
    }

    fn run(&mut self, program: &[u8]) {
        loop {
            let opcode = program[self.pc];
            self.pc += 1;

            match opcode {
                0xA9 => {
                    // LDA: load the next byte into A
                    self.a = program[self.pc];
                    self.pc += 1;
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
    let program: [u8; 6] = [0xA9, 0xFE, 0xAA, 0xE8, 0xE8, 0x00];

    let mut cpu = Cpu::new();
    cpu.run(&program);

    println!("A = ${:02X}  X = ${:02X}", cpu.a, cpu.x);
}
