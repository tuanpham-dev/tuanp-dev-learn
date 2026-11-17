use std::fs;

// Everything we need from a .nes file.
pub struct Cartridge {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
}

impl Cartridge {
    pub fn load(path: &str) -> Cartridge {
        let rom: Vec<u8> = fs::read(path).expect("could not read the cartridge file");

        if rom[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
            panic!("this is not a NES cartridge file");
        }

        let prg_size = rom[4] as usize * 16384;
        let chr_size = rom[5] as usize * 8192;
        let prg = rom[16..16 + prg_size].to_vec();
        let chr = rom[16 + prg_size..16 + prg_size + chr_size].to_vec();

        Cartridge { prg, chr }
    }
}
