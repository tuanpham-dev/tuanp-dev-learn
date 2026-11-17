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

    control: u8,
    status: u8,
    addr: u16,
    second_write: bool,

    pub nmi_pending: bool,
    pub frame: Vec<u32>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            vram: [0; 0x4000],
            control: 0,
            status: 0,
            addr: 0,
            second_write: false,
            nmi_pending: false,
            frame: vec![0; WIDTH * HEIGHT],
        }
    }

    // The tiles live at the start of the PPU's memory.
    pub fn load_chr(&mut self, chr: &[u8]) {
        for i in 0..chr.len() {
            self.vram[i] = chr[i];
        }
    }

    // ---- the registers the processor can see ----

    pub fn write_register(&mut self, register: u16, value: u8) {
        match register {
            0 => {
                let was_on = self.control & 0x80 != 0;
                self.control = value;
                // Switching the interrupt on in the middle of vblank fires it at once.
                if !was_on && value & 0x80 != 0 && self.status & 0x80 != 0 {
                    self.nmi_pending = true;
                }
            }
            6 => {
                if !self.second_write {
                    self.addr = (self.addr & 0x00FF) | ((value as u16) << 8);
                } else {
                    self.addr = (self.addr & 0xFF00) | value as u16;
                }
                self.second_write = !self.second_write;
            }
            7 => {
                self.vram[self.addr as usize % 0x4000] = value;
                self.step_addr();
            }
            _ => {}
        }
    }

    pub fn read_register(&mut self, register: u16) -> u8 {
        match register {
            2 => {
                let value = self.status;
                self.status &= 0x7F; // reading clears the vblank flag
                self.second_write = false;
                value
            }
            _ => 0,
        }
    }

    fn step_addr(&mut self) {
        let step = if self.control & 0x04 != 0 { 32 } else { 1 };
        self.addr = self.addr.wrapping_add(step);
    }

    // ---- timing ----

    pub fn start_vblank(&mut self) {
        self.status |= 0x80;
        if self.control & 0x80 != 0 {
            self.nmi_pending = true;
        }
    }

    pub fn end_vblank(&mut self) {
        self.status &= 0x7F;
    }

    // ---- drawing ----

    // Draw one horizontal line of the picture into `frame`.
    pub fn render_scanline(&mut self, y: usize) {
        for x in 0..WIDTH {
            let tile = self.vram[0x2000 + (y / 8) * 32 + x / 8] as usize;
            let colour = tile_pixel(&self.vram, tile, x % 8, y % 8);
            self.frame[y * WIDTH + x] = GREYS[colour as usize];
        }
    }
}
