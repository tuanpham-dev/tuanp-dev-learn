// mkrom: builds hello.nes, the tiny cartridge used from Part 6 onwards.
//
// You do not need to understand this file to follow the course, but after
// Part 8 every line of it should make sense. Run it with:
//
//     cargo run -- hello.nes
//
// The cartridge file has three sections:
//   16 bytes      iNES header
//   32768 bytes   PRG ROM: the 6502 program, mapped at $8000-$FFFF
//   8192 bytes    CHR ROM: the tiles (8x8 pixel pictures)

use std::env;
use std::fs;

// The message, as tile numbers: H E L L O (blank) N E S (heart)
const MESSAGE: [u8; 10] = [1, 2, 3, 3, 4, 0, 5, 2, 6, 7];

// The program, assembled by hand. It lives right after the message,
// so it starts at $800A.
const CODE: [u8; 26] = [
    0xA9, 0x20,       // 800A  LDA #$20
    0x8D, 0x06, 0x20, // 800C  STA $2006     PPU address, high byte
    0xA9, 0x20,       // 800F  LDA #$20
    0x8D, 0x06, 0x20, // 8011  STA $2006     PPU address, low byte: $2020
    0xA2, 0x00,       // 8014  LDX #$00
    0xBD, 0x00, 0x80, // 8016  LDA $8000,X   next tile number of the message
    0x8D, 0x07, 0x20, // 8019  STA $2007     hand it to the PPU
    0xE8,             // 801C  INX
    0xE0, 0x0A,       // 801D  CPX #$0A      all ten done?
    0xD0, 0xF5,       // 801F  BNE $8016     no: go round again
    0x4C, 0x21, 0x80, // 8021  JMP $8021     yes: spin here forever
];

// The tiles. '.' is colour 0, '1' '2' '3' are colours 1 to 3.
const TILES: [[&str; 8]; 8] = [
    [   // tile 0: blank
        "........", "........", "........", "........",
        "........", "........", "........", "........",
    ],
    [   // tile 1: H
        "33..33..", "33..33..", "33..33..", "333333..",
        "33..33..", "33..33..", "33..33..", "........",
    ],
    [   // tile 2: E
        "333333..", "33......", "33......", "33333...",
        "33......", "33......", "333333..", "........",
    ],
    [   // tile 3: L
        "33......", "33......", "33......", "33......",
        "33......", "33......", "333333..", "........",
    ],
    [   // tile 4: O
        ".3333...", "33..33..", "33..33..", "33..33..",
        "33..33..", "33..33..", ".3333...", "........",
    ],
    [   // tile 5: N
        "33..33..", "333.33..", "333333..", "333333..",
        "33.333..", "33..33..", "33..33..", "........",
    ],
    [   // tile 6: S
        ".33333..", "33......", "33......", ".3333...",
        "....33..", "....33..", "33333...", "........",
    ],
    [   // tile 7: heart, using all four colours
        ".33.33..", "3223223.", "3122223.", "3222223.",
        ".32223..", "..323...", "...3....", "........",
    ],
];

fn main() {
    let path = env::args().nth(1).unwrap_or(String::from("hello.nes"));

    // Header: "NES" + $1A, 2 x 16K of PRG, 1 x 8K of CHR, the rest zero.
    let mut rom: Vec<u8> = vec![0x4E, 0x45, 0x53, 0x1A, 2, 1];
    rom.resize(16, 0);

    // PRG ROM: message at $8000, code at $800A, reset vector at $FFFC.
    let mut prg = vec![0u8; 32768];
    prg[0..10].copy_from_slice(&MESSAGE);
    prg[10..10 + CODE.len()].copy_from_slice(&CODE);
    prg[0x7FFC] = 0x0A; // reset vector, low byte
    prg[0x7FFD] = 0x80; // reset vector, high byte: $800A
    rom.extend(&prg);

    // CHR ROM: each tile is 8 bytes of "low bits" then 8 bytes of "high bits".
    let mut chr = vec![0u8; 8192];
    for (n, tile) in TILES.iter().enumerate() {
        for (y, row) in tile.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let colour = match ch {
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    _ => 0,
                };
                let bit = 7 - x;
                chr[n * 16 + y] |= (colour & 1) << bit;
                chr[n * 16 + y + 8] |= (colour >> 1) << bit;
            }
        }
    }
    rom.extend(&chr);

    fs::write(&path, &rom).expect("could not write the ROM file");
    println!("wrote {} ({} bytes)", path, rom.len());
}
