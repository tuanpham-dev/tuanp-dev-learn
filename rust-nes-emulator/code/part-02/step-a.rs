// Part 2, first attempt: counting with loose variables.
// This version crashes on purpose. Copy it over src/main.rs to try it.
fn main() {
    let program: [u8; 6] = [0xA9, 0xFE, 0xAA, 0xE8, 0xE8, 0x00];

    let mut a: u8 = 0;
    let mut x: u8 = 0;
    let mut pc = 0;

    loop {
        let opcode = program[pc];
        pc += 1;

        match opcode {
            0xA9 => {
                // LDA: load the next byte into A
                a = program[pc];
                pc += 1;
            }
            0xAA => x = a,     // TAX: copy A into X
            0xE8 => x = x + 1, // INX: add one to X
            0x00 => break,     // BRK: stop
            _ => panic!("unknown opcode ${:02X}", opcode),
        }
    }

    println!("A = ${:02X}  X = ${:02X}", a, x);
}
