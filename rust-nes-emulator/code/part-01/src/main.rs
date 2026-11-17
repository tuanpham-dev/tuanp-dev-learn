fn main() {
    let program: [u8; 3] = [0xA9, 0x05, 0x00];

    let mut a: u8 = 0;
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
            0x00 => break, // BRK: stop
            _ => panic!("unknown opcode ${:02X}", opcode),
        }
    }

    println!("A = ${:02X}", a);
}
