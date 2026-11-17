use std::fs;

// Everything we need from a .nes file.
pub struct Cartridge {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub vertical_mirroring: bool,
}

impl Cartridge {
    pub fn load(path: &str) -> Cartridge {
        let rom: Vec<u8> = fs::read(path).expect("could not read the cartridge file");

        if rom.len() < 16 || rom[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
            panic!("this is not a NES cartridge file");
        }

        let mapper = (rom[6] >> 4) | (rom[7] & 0xF0);
        if mapper != 0 {
            panic!("this cartridge needs mapper {}, and we only have mapper 0", mapper);
        }

        let prg_size = rom[4] as usize * 16384;
        let chr_size = rom[5] as usize * 8192;
        if chr_size == 0 {
            panic!("this cartridge has no CHR ROM, which we do not support");
        }
        let prg = rom[16..16 + prg_size].to_vec();
        let chr = rom[16 + prg_size..16 + prg_size + chr_size].to_vec();

        Cartridge { prg, chr, vertical_mirroring: rom[6] & 1 != 0 }
    }
}
