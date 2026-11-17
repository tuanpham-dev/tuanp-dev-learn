// The NES screen, in pixels.
pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

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

pub struct Ppu {
    vram: [u8; 0x4000],
    addr: u16,
    pub frame: Vec<u32>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu { vram: [0; 0x4000], addr: 0, frame: vec![0; WIDTH * HEIGHT] }
    }

    // The tiles live at the start of the PPU's memory.
    pub fn load_chr(&mut self, chr: &[u8]) {
        for i in 0..chr.len() {
            self.vram[i] = chr[i];
        }
    }

    // A write to $2006: one more byte of the address, high byte first.
    pub fn write_addr(&mut self, value: u8) {
        self.addr = (self.addr << 8) | value as u16;
    }

    // A write to $2007: store the byte, then step to the next address.
    pub fn write_data(&mut self, value: u8) {
        self.vram[self.addr as usize % 0x4000] = value;
        self.addr = self.addr.wrapping_add(1);
    }

    // Paint the picture into `frame`: for every pixel, which tile is it in,
    // and which pixel of that tile?
    pub fn draw(&mut self) {
        for py in 0..HEIGHT {
            for px in 0..WIDTH {
                let tile = self.vram[0x2000 + (py / 8) * 32 + px / 8] as usize;
                let colour = tile_pixel(&self.vram, tile, px % 8, py % 8);
                self.frame[py * WIDTH + px] = GREYS[colour as usize];
            }
        }
    }
}
